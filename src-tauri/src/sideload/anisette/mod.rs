use anisette_data::AnisetteData;
use base64::{engine::general_purpose, Engine};
use keyring::Entry;
use rand::RngCore;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tungstenite::connect;
use uuid::Uuid;

pub mod anisette_data;

const ANISETTE_BASE_URL: &str = "ani.sidestore.io";

fn get_identifier() -> String {
    let entry = Entry::new("ycode", "identifier").expect("Failed to create keyring entry");
    if let Ok(id) = entry.get_password() {
        if !id.trim().is_empty() {
            return id;
        }
    }
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    let id = general_purpose::STANDARD.encode(&bytes);
    let _ = entry.set_password(&id);
    id
}

#[derive(Clone)]
pub struct ClientInfo {
    pub client_info: String,
    pub user_agent: String,
    pub md_lu: String,
    pub device_id: String,
    pub identifier: String,
}

fn get_adi_pb() -> Result<String, String> {
    let entry = Entry::new("ycode", "adi_pb").expect("Failed to create keyring entry");
    match entry.get_password() {
        Ok(adi_pb) => {
            if adi_pb.trim().is_empty() {
                Err("adi_pb is empty".to_string())
            } else {
                Ok(adi_pb)
            }
        }
        Err(e) => Err(format!("Failed to read adi_pb: {}", e)),
    }
}

pub async fn get_anisette_data(
    log: Arc<dyn Fn(String) + Send + Sync + 'static>,
) -> Result<AnisetteData, String> {
    let adi_pb = match get_adi_pb() {
        Ok(adi_pb) => adi_pb,
        Err(_) => {
            provision(log.clone()).await;
            match get_adi_pb() {
                Ok(adi_pb) => adi_pb,
                Err(e) => {
                    log(format!("Provisioning Failed: {}", e));
                    return Err(e);
                }
            }
        }
    };

    let identifier = get_identifier();
    let client_info = fetch_client_info(log.clone()).await;
    if client_info.is_none() {
        log("Failed to fetch client info".to_string());
        return Err("Failed to fetch client info".to_string());
    }

    let client_info = client_info.unwrap();

    let url = format!("https://{}/v3/get_headers", ANISETTE_BASE_URL);
    let body = serde_json::json!({
        "identifier": identifier,
        "adi_pb": adi_pb,
    });

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client");

    let resp = match client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log(format!("Failed to POST to get_headers: {}", e));
            return Err(format!("Failed to POST to get_headers: {}", e));
        }
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log(format!("Failed to read get_headers response: {}", e));
            return Err(format!("Failed to read get_headers response: {}", e));
        }
    };

    let json: Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(e) => {
            log(format!("Failed to parse get_headers response JSON: {}", e));
            return Err(format!("Failed to parse get_headers response JSON: {}", e));
        }
    };

    if let Some(result) = json.get("result") {
        if let Some(result_str) = result.as_str() {
            if result_str == "GetHeadersError" {
                log(format!(
                    "GetHeadersError: {}",
                    json.get("message")
                        .unwrap_or(&Value::String("Unknown error".to_string()))
                ));
                return Err(format!(
                    "GetHeadersError: {}",
                    json.get("message")
                        .unwrap_or(&Value::String("Unknown error".to_string()))
                ));
            }
        }
    }

    let mut formatted_json = std::collections::BTreeMap::new();
    formatted_json.insert("deviceSerialNumber".to_string(), "0".to_string());

    if let Some(machine_id) = json.get("X-Apple-I-MD-M").and_then(|v| v.as_str()) {
        formatted_json.insert("machineID".to_string(), machine_id.to_string());
    }
    if let Some(one_time_password) = json.get("X-Apple-I-MD").and_then(|v| v.as_str()) {
        formatted_json.insert("oneTimePassword".to_string(), one_time_password.to_string());
    }
    if let Some(routing_info) = json.get("X-Apple-I-MD-RINFO").and_then(|v| v.as_str()) {
        formatted_json.insert("routingInfo".to_string(), routing_info.to_string());
    }

    formatted_json.insert(
        "deviceDescription".to_string(),
        client_info.client_info.clone(),
    );
    formatted_json.insert("localUserID".to_string(), client_info.md_lu.clone());
    formatted_json.insert(
        "deviceUniqueIdentifier".to_string(),
        client_info.device_id.clone(),
    );

    //let date_string = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    //formatted_json.insert("date".to_string(), date_string);

    let locale = locale_config::Locale::user_default().to_string();
    //formatted_json.insert("locale".to_string(), locale);

    let time_zone = chrono::Local::now().format("%Z").to_string();
    //formatted_json.insert("timeZone".to_string(), time_zone);

    let anisette_data = AnisetteData {
        machine_id: formatted_json
            .get("machineID")
            .unwrap_or(&"".to_string())
            .to_string(),
        one_time_password: formatted_json
            .get("oneTimePassword")
            .unwrap_or(&"".to_string())
            .to_string(),
        local_user_id: formatted_json
            .get("localUserID")
            .unwrap_or(&"".to_string())
            .to_string(),
        routing_info: formatted_json
            .get("routingInfo")
            .and_then(|v| Some(v.as_str()))
            .unwrap_or("0")
            .parse()
            .unwrap_or(0),
        device_unique_identifier: formatted_json
            .get("deviceUniqueIdentifier")
            .unwrap_or(&"".to_string())
            .to_string(),
        device_serial_number: formatted_json
            .get("deviceSerialNumber")
            .unwrap_or(&"".to_string())
            .to_string(),
        device_description: formatted_json
            .get("deviceDescription")
            .unwrap_or(&"".to_string())
            .to_string(),
        date: chrono::Utc::now(),
        locale,
        time_zone,
    };
    Ok(anisette_data)
}

fn build_apple_request(
    url: &str,
    info: &ClientInfo,
    method: reqwest::Method,
) -> reqwest::RequestBuilder {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client");

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Mme-Client-Info",
        HeaderValue::from_str(&info.client_info).unwrap(),
    );
    headers.insert(USER_AGENT, HeaderValue::from_str(&info.user_agent).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/x-xml-plist"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        "X-Apple-I-MD-LU",
        HeaderValue::from_str(&info.md_lu).unwrap(),
    );
    headers.insert(
        "X-Mme-Device-Id",
        HeaderValue::from_str(&info.device_id).unwrap(),
    );

    // Date/time headers
    let date_string = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    headers.insert(
        "X-Apple-I-Client-Time",
        HeaderValue::from_str(&date_string).unwrap(),
    );
    headers.insert(
        "X-Apple-Locale",
        HeaderValue::from_str(&locale_config::Locale::user_default().to_string()).unwrap(),
    );
    headers.insert(
        "X-Apple-I-TimeZone",
        HeaderValue::from_str(&chrono::Local::now().format("%Z").to_string()).unwrap(),
    );

    client.request(method, url).headers(headers)
}

async fn provision(log: Arc<dyn Fn(String) + Send + Sync + 'static>) {
    let client_info = fetch_client_info(log.clone()).await;
    if client_info.is_none() {
        log("Failed to fetch client info".to_string());
        return;
    }
    let client_info = client_info.unwrap();

    // Fetch Apple provisioning URLs (startProvisioningURL, endProvisioningURL)
    let lookup_url = "https://gsa.apple.com/grandslam/GsService2/lookup";
    let resp = match build_apple_request(lookup_url, &client_info, reqwest::Method::GET)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log(format!("Failed to fetch Apple lookup: {e}"));
            return;
        }
    };

    let data = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            log(format!("Failed to read Apple lookup response: {e}"));
            return;
        }
    };

    // Parse the property list (plist) response
    let plist: plist::Value = match plist::Value::from_reader_xml(data.as_ref()) {
        Ok(p) => p,
        Err(e) => {
            log(format!("Failed to parse Apple plist: {e}"));
            return;
        }
    };

    // Extract URLs
    let urls = match plist
        .as_dictionary()
        .and_then(|dict| dict.get("urls"))
        .and_then(|urls| urls.as_dictionary())
    {
        Some(urls) => urls,
        None => {
            log("Failed to find 'urls' dictionary in plist".to_string());
            return;
        }
    };

    let start_provisioning_url = match urls.get("midStartProvisioning").and_then(|v| v.as_string())
    {
        Some(url) => url,
        None => {
            log("Failed to find 'midStartProvisioning' in plist".to_string());
            return;
        }
    };

    let end_provisioning_url = match urls
        .get("midFinishProvisioning")
        .and_then(|v| v.as_string())
    {
        Some(url) => url,
        None => {
            log("Failed to find 'midFinishProvisioning' in plist".to_string());
            return;
        }
    };

    start_provisioning_session(
        &client_info,
        start_provisioning_url,
        end_provisioning_url,
        log,
    );
}

async fn fetch_client_info(log: Arc<dyn Fn(String) + Send + Sync + 'static>) -> Option<ClientInfo> {
    let client_info_url = format!("https://{}/v3/client_info", ANISETTE_BASE_URL);

    let resp = match reqwest::get(&client_info_url).await {
        Ok(r) => r,
        Err(e) => {
            log(format!("Failed to fetch client info: {e}"));
            return None;
        }
    };

    let json: Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            log(format!("Failed to parse client info JSON: {e}"));
            return None;
        }
    };

    let client_info = match json.get("client_info").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            log("No client_info in JSON".to_string());
            return None;
        }
    };

    let user_agent = match json.get("user_agent").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            log("No user_agent in JSON".to_string());
            return None;
        }
    };

    let identifier = get_identifier();

    // mdLu = sha256(identifier_base64_decoded).hex()
    let decoded = match general_purpose::STANDARD.decode(&identifier) {
        Ok(d) => d,
        Err(e) => {
            log(format!("Failed to decode identifier: {e}"));
            return None;
        }
    };
    let md_lu = hex::encode(Sha256::digest(&decoded));

    // deviceId = UUID from decoded identifier, uppercased
    let device_id = match Uuid::from_slice(&decoded) {
        Ok(uuid) => uuid.to_string().to_uppercase(),
        Err(e) => {
            log(format!("Failed to convert identifier to UUID: {e}"));
            return None;
        }
    };

    Some(ClientInfo {
        client_info,
        user_agent,
        md_lu,
        device_id,
        identifier,
    })
}

// returns the adi_pb
fn start_provisioning_session(
    client_info: &ClientInfo,
    start_provisioning_url: &str,
    end_provisioning_url: &str,
    log: Arc<dyn Fn(String) + Send + Sync + 'static>,
) {
    // Convert all borrowed data to owned types for thread safety and to avoid lifetime issues
    let client_info = client_info.clone();
    let start_provisioning_url = start_provisioning_url.to_string();
    let end_provisioning_url = end_provisioning_url.to_string();
    let log = log.clone();

    let provisioning_session_url = format!("wss://{}/v3/provisioning_session", ANISETTE_BASE_URL);
    log(format!(
        "Starting provisioning session at: {}",
        provisioning_session_url
    ));

    let (mut socket, _response) =
        connect(provisioning_session_url).expect("Can't connect to WebSocket");

    loop {
        match socket.read() {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.into_text().unwrap();
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            if let Some(result) = json.get("result") {
                                if let Some(result_str) = result.as_str() {
                                    match result_str {
                                        "GiveIdentifier" => {
                                            let identifier = get_identifier();
                                            let response =
                                                serde_json::json!({ "identifier": identifier })
                                                    .to_string();
                                            if let Err(e) = socket
                                                .send(tungstenite::Message::Text(response.into()))
                                            {
                                                log(format!("Failed to send identifier: {}", e));
                                            }
                                        }
                                        "GiveStartProvisioningData" => {
                                            let mut body = plist::Dictionary::new();
                                            body.insert(
                                                "Header".to_string(),
                                                plist::Value::Dictionary(plist::Dictionary::new()),
                                            );
                                            body.insert(
                                                "Request".to_string(),
                                                plist::Value::Dictionary(plist::Dictionary::new()),
                                            );
                                            let body_value = plist::Value::Dictionary(body);

                                            let mut body_bytes: Vec<u8> = Vec::new();
                                            if let Err(e) =
                                                body_value.to_writer_xml(&mut body_bytes)
                                            {
                                                log(format!(
                                                    "Failed to serialize plist body: {}",
                                                    e
                                                ));
                                                break;
                                            }

                                            // Use owned clones for thread move
                                            let client_info = client_info.clone();
                                            let start_provisioning_url =
                                                start_provisioning_url.clone();
                                            let log_thread = log.clone(); // <-- clone log for thread
                                            let (tx, rx) = std::sync::mpsc::channel();

                                            std::thread::spawn(move || {
                                                let rt = tokio::runtime::Runtime::new().unwrap();
                                                rt.block_on(async move {
                                                    let resp = build_apple_request(
                                                        &start_provisioning_url,
                                                        &client_info,
                                                        reqwest::Method::POST,
                                                    )
                                                    .body(body_bytes)
                                                    .send()
                                                    .await;

                                                    match resp {
                                                        Ok(resp) => {
                                                            let data = match resp.bytes().await {
                                                                Ok(b) => b,
                                                                Err(e) => {
                                                                    log_thread(format!("Failed to read Apple response: {}", e));
                                                                    let _ = tx.send(None);
                                                                    return;
                                                                }
                                                            };
                                                            let plist: Result<plist::Value, _> = plist::Value::from_reader_xml(data.as_ref());
                                                            match plist {
                                                                Ok(plist) => {
                                                                    if let Some(response_dict) = plist.as_dictionary()
                                                                        .and_then(|d| d.get("Response"))
                                                                        .and_then(|v| v.as_dictionary())
                                                                    {
                                                                        if let Some(spim) = response_dict.get("spim").and_then(|v| v.as_string()) {
                                                                            let response = serde_json::json!({ "spim": spim }).to_string();
                                                                            let _ = tx.send(Some(response));
                                                                            return;
                                                                        }
                                                                    }
                                                                    log_thread("Apple didn't give valid start provisioning data!".to_string());
                                                                    let _ = tx.send(None);
                                                                }
                                                                Err(e) => {
                                                                    log_thread(format!("Failed to parse Apple plist: {}", e));
                                                                    let _ = tx.send(None);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log_thread(format!("Failed to send POST to Apple: {}", e));
                                                            let _ = tx.send(None);
                                                        }
                                                    }
                                                });
                                            });

                                            // Wait for the response from the spawned thread and send it over the websocket
                                            if let Ok(Some(response)) = rx.recv() {
                                                if let Err(e) = socket.send(
                                                    tungstenite::Message::Text(response.into()),
                                                ) {
                                                    log(format!("Failed to send spim: {}", e));
                                                }
                                            }
                                            // Continue the loop to handle the next server message (e.g., GiveEndProvisioningData)
                                            continue;
                                        }
                                        "GiveEndProvisioningData" => {
                                            // Extract cpim from the JSON
                                            let cpim =
                                                match json.get("cpim").and_then(|v| v.as_str()) {
                                                    Some(s) => s.to_string(),
                                                    None => {
                                                        log("The server didn't give us a cpim"
                                                            .to_string());
                                                        // Optionally: socket.close(None); // or handle disconnect
                                                        break;
                                                    }
                                                };

                                            // Build plist body: {"Header": {}, "Request": {"cpim": cpim}}
                                            let mut body = plist::Dictionary::new();
                                            body.insert(
                                                "Header".to_string(),
                                                plist::Value::Dictionary(plist::Dictionary::new()),
                                            );
                                            let mut request_dict = plist::Dictionary::new();
                                            request_dict.insert(
                                                "cpim".to_string(),
                                                plist::Value::String(cpim),
                                            );
                                            body.insert(
                                                "Request".to_string(),
                                                plist::Value::Dictionary(request_dict),
                                            );
                                            let body_value = plist::Value::Dictionary(body);

                                            let mut body_bytes: Vec<u8> = Vec::new();
                                            if let Err(e) =
                                                body_value.to_writer_xml(&mut body_bytes)
                                            {
                                                log(format!(
                                                    "Failed to serialize plist body: {}",
                                                    e
                                                ));
                                                break;
                                            }

                                            // Use owned clones for thread move
                                            let client_info = client_info.clone();
                                            let end_provisioning_url = end_provisioning_url.clone();
                                            let log_thread = log.clone();
                                            let (tx, rx) = std::sync::mpsc::channel();

                                            std::thread::spawn(move || {
                                                let rt = tokio::runtime::Runtime::new().unwrap();
                                                rt.block_on(async move {
                                                    let resp = build_apple_request(
                                                        &end_provisioning_url,
                                                        &client_info,
                                                        reqwest::Method::POST,
                                                    )
                                                    .body(body_bytes)
                                                    .send()
                                                    .await;

                                                    match resp {
                                                        Ok(resp) => {
                                                            let data = match resp.bytes().await {
                                                                Ok(b) => b,
                                                                Err(e) => {
                                                                    log_thread(format!("Failed to read Apple response: {}", e));
                                                                    let _ = tx.send(None);
                                                                    return;
                                                                }
                                                            };
                                                            let plist: Result<plist::Value, _> = plist::Value::from_reader_xml(data.as_ref());
                                                            match plist {
                                                                Ok(plist) => {
                                                                    if let Some(response_dict) = plist.as_dictionary()
                                                                        .and_then(|d| d.get("Response"))
                                                                        .and_then(|v| v.as_dictionary())
                                                                    {
                                                                        let ptm = response_dict.get("ptm").and_then(|v| v.as_string());
                                                                        let tk = response_dict.get("tk").and_then(|v| v.as_string());
                                                                        if let (Some(ptm), Some(tk)) = (ptm, tk) {
                                                                            let response = serde_json::json!({ "ptm": ptm, "tk": tk }).to_string();
                                                                            let _ = tx.send(Some(response));
                                                                            return;
                                                                        }
                                                                    }
                                                                    log_thread("Apple didn't give valid end provisioning data!".to_string());
                                                                    let _ = tx.send(None);
                                                                }
                                                                Err(e) => {
                                                                    log_thread(format!("Failed to parse Apple plist: {}", e));
                                                                    let _ = tx.send(None);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log_thread(format!("Failed to send POST to Apple: {}", e));
                                                            let _ = tx.send(None);
                                                        }
                                                    }
                                                });
                                            });

                                            // Wait for the response from the spawned thread and send it over the websocket
                                            if let Ok(Some(response)) = rx.recv() {
                                                if let Err(e) = socket.send(
                                                    tungstenite::Message::Text(response.into()),
                                                ) {
                                                    log(format!("Failed to send ptm/tk: {}", e));
                                                }
                                            }
                                            // Continue the loop to handle the next server message (e.g., ProvisioningSuccess)
                                            continue;
                                        }
                                        "ProvisioningSuccess" => {
                                            log("Provisioning successful".to_string());
                                            // disconnect the socket
                                            if let Err(e) = socket.close(None) {
                                                log(format!("Failed to close socket: {}", e));
                                            }
                                            // read adi_pb from json
                                            let adi_pb =
                                                match json.get("adi_pb").and_then(|v| v.as_str()) {
                                                    Some(s) => s.to_string(),
                                                    None => {
                                                        log("The server didn't give us an adi_pb"
                                                            .to_string());
                                                        break;
                                                    }
                                                };
                                            let entry = Entry::new("ycode", "adi_pb")
                                                .expect("Failed to create keyring entry");
                                            if let Err(e) = entry.set_password(&adi_pb) {
                                                log(format!("Failed to store adi_pb: {}", e));
                                            } else {
                                                log(format!(
                                                    "Stored adi_pb in keyring: {}",
                                                    adi_pb
                                                ));
                                            }
                                        }
                                        "Timeout" => {
                                            log("Provisioning Failed (Timed Out)".to_string());
                                            break;
                                        }
                                        _ => {
                                            log(format!(
                                                "Unknown message recieved: {}",
                                                result_str
                                            ));
                                        }
                                    }
                                } else {
                                    log("Result is not a string".to_string());
                                }
                            } else {
                                log("No result found in JSON".to_string());
                            }
                        }
                        Err(e) => {
                            log(format!("Failed to parse JSON: {}", e));
                        }
                    }
                } else if msg.is_close() {
                    log("WebSocket closed by server".to_string());
                    break;
                }
            }
            Err(e) => {
                log(format!("WebSocket error: {}", e));
                break;
            }
        }
    }
}
