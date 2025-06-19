use futures::channel::oneshot;
use plist::{Dictionary as PlistDict, Value as PlistValue};
use serde_json::Value;
use std::collections::HashMap;
use tauri::{Emitter, Listener, Window};

pub struct ApplicationInformation {
    pub application_name: String,
    pub application_id: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppleLoginErrorCode {
    MismatchedSrp = 1,
    MisformattedEncryptedToken = 2,
    No2FAAttempt = 3,
    UnsupportedNextStep = 4,
    AccountLocked = -20209,
    InvalidValidationCode = -21669,
    InvalidPassword = -22406,
    UnableToSignIn = -36607,
}

#[derive(Debug, Clone)]
pub struct AppleLoginError {
    pub code: AppleLoginErrorCode,
    pub description: String,
}

// --- Login response sum type ---
pub enum AppleLoginResponse {
    Account(AppleAccount),
    Error(AppleLoginError),
}

// --- 2FA/secondary action response sum type ---
pub struct Success;
pub struct ReloginNeeded;
pub enum AppleSecondaryActionResponse {
    Success(Success),
    ReloginNeeded(ReloginNeeded),
    Error(AppleLoginError),
}

// --- Handler types ---
pub type Send2FADelegate = Box<dyn Fn() -> bool>;
pub type Submit2FADelegate = Box<dyn Fn(String) -> AppleSecondaryActionResponse>;
pub type TFAHandlerDelegate = Box<dyn Fn(Send2FADelegate, Submit2FADelegate)>;
pub type NextLoginStepHandler =
    Box<dyn Fn(String, HashMap<String, String>, String, bool) -> AppleSecondaryActionResponse>;

// --- Constants ---
pub const RINFO: &str = "17106176";

// --- Main AppleAccount struct ---
pub struct AppleAccount {
    anisette_data: Value,
    apple_identifier: String,
    adsid: String,
    token: String,
    urls: HashMap<String, String>,
}

impl AppleAccount {
    pub fn apple_id(&self) -> &str {
        &self.apple_identifier
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        anisette_data: Value,
        urls: HashMap<String, String>,
        apple_id: String,
        adsid: String,
        token: String,
    ) -> Self {
        Self {
            anisette_data,
            urls,
            apple_identifier: apple_id,
            adsid,
            token,
        }
    }

    // --- Use plist crate for client_provided_data stub ---
    async fn client_provided_data(anisette_data: Value) -> PlistValue {
        use chrono::{Local, SecondsFormat, Utc};
        use sys_locale::get_locale;
        let mut dict = PlistDict::new();

        // Time
        let now = Utc::now();
        let iso_time = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        dict.insert(
            "X-Apple-I-Client-Time".to_string(),
            PlistValue::String(iso_time),
        );

        // Anisette headers (stubbed)
        dict.insert(
            "X-Apple-I-MD".to_string(),
            PlistValue::String(
                anisette_data
                    .get("X-Apple-I-MD")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("STUB_MD")
                    .to_string(),
            ),
        );
        dict.insert(
            "X-Apple-I-MD-LU".to_string(),
            PlistValue::String(
                anisette_data
                    .get("X-Apple-I-MD-LU")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("STUB_MD_LU")
                    .to_string(),
            ),
        );
        dict.insert(
            "X-Apple-I-MD-M".to_string(),
            PlistValue::String(
                anisette_data
                    .get("X-Apple-I-MD-M")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("STUB_M")
                    .to_string(),
            ),
        );
        dict.insert(
            "X-Apple-I-MD-RINFO".to_string(),
            PlistValue::String(RINFO.to_string()),
        );

        dict.insert(
            "X-Apple-I-SRL-NO".to_string(),
            PlistValue::String("0".to_string()),
        );

        // Timezone (real)
        dict.insert(
            "X-Apple-I-TimeZone".to_string(),
            PlistValue::String("EDT".to_string()),
        );

        // Locale (real)
        let locale = get_locale().unwrap_or_else(|| "en_US".to_string());
        // replace - with _
        let locale = locale.replace("-", "_");
        dict.insert(
            "X-Apple-Locale".to_string(),
            PlistValue::String(locale.clone()),
        );

        dict.insert(
            "X-Mme-Device-Id".to_string(),
            PlistValue::String(
                anisette_data
                    .get("X-Mme-Device-Id")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("STUB_DEVICE_ID")
                    .to_string(),
            ),
        );

        // Miscellaneous headers
        dict.insert("bootstrap".to_string(), PlistValue::Boolean(true));
        dict.insert("icscrec".to_string(), PlistValue::Boolean(true));
        dict.insert("loc".to_string(), PlistValue::String(locale));
        dict.insert("pbe".to_string(), PlistValue::Boolean(false));
        dict.insert("prkgen".to_string(), PlistValue::Boolean(true));
        dict.insert("svct".to_string(), PlistValue::String("iCloud".to_string()));

        // Uncomment and fill in as needed:
        // dict.insert("capp".to_string(), PlistValue::String(_application_information.application_name.clone()));
        // dict.insert("ckgen".to_string(), PlistValue::Boolean(true));

        PlistValue::Dictionary(dict)
    }

    // --- Use plist crate for validate_status helper ---
    fn validate_status(status: &PlistDict) -> Option<AppleLoginError> {
        let error_code = status
            .get("ec")
            .and_then(|v| v.as_signed_integer())
            .unwrap_or(0);
        if error_code == 0 {
            None
        } else {
            let description = status
                .get("em")
                .and_then(|v| v.as_string())
                .unwrap_or("")
                .to_string();
            Some(AppleLoginError {
                code: match error_code {
                    1 => AppleLoginErrorCode::MismatchedSrp,
                    2 => AppleLoginErrorCode::MisformattedEncryptedToken,
                    3 => AppleLoginErrorCode::No2FAAttempt,
                    4 => AppleLoginErrorCode::UnsupportedNextStep,
                    -20209 => AppleLoginErrorCode::AccountLocked,
                    -21669 => AppleLoginErrorCode::InvalidValidationCode,
                    -22406 => AppleLoginErrorCode::InvalidPassword,
                    -36607 => AppleLoginErrorCode::UnableToSignIn,
                    _ => AppleLoginErrorCode::UnableToSignIn,
                },
                description,
            })
        }
    }

    // --- login with TFA handler (stub) ---
    #[allow(clippy::too_many_arguments)]
    pub async fn login_with_tfa_handler(
        anisette_data: Value,
        apple_id: String,
        password: String,
        window: Window,
    ) -> AppleLoginResponse {
        use plist::Value as PlistValue;
        use reqwest::Client;
        use std::collections::HashMap;

        // 1. Prepare HTTP client
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        // 2. Prepare ApplicationInformation (stub, fill as needed)
        let app_info = ApplicationInformation {
            application_name: "Xcode".to_string(),
            application_id: "com.apple.gs.xcode.auth".to_string(),
            headers: [
                ("X-Xcode-Version".to_string(), "14.2 (14C18)".to_string()),
                (
                    "X-Apple-App-Info".to_string(),
                    "com.apple.gs.xcode.auth".to_string(),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        // 3. Fetch URL bag
        let url_bag_url = "https://gsa.apple.com/grandslam/GsService2/lookup";
        let mut req_headers = reqwest::header::HeaderMap::new();

        // insert X-Mme-Client-Info header from the anisette data
        if let Some(client_info) = anisette_data.get("X-Mme-Client-Info") {
            if let Some(client_info_str) = client_info.as_str() {
                req_headers.insert("X-Mme-Client-Info", client_info_str.parse().unwrap());
            }
        }
        req_headers.insert("User-Agent", app_info.application_name.parse().unwrap());
        // add the headers from ApplicationInformation
        for (key, value) in app_info.headers.iter() {
            req_headers.insert(
                reqwest::header::HeaderName::try_from(key).unwrap(),
                value.parse().unwrap(),
            );
        }

        req_headers.insert("Accept-Encoding", "gzip,deflate".parse().unwrap());
        req_headers.insert("Connection", "Keep-Alive".parse().unwrap());
        req_headers.insert("Transfer-Encoding", "chunked".parse().unwrap());
        req_headers.insert("Accept", "text/x-xml-plist".parse().unwrap());
        req_headers.insert("Content-Type", "text/x-xml-plist".parse().unwrap());

        let url_bag_resp = match client
            .get(url_bag_url)
            .headers(req_headers.clone())
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to fetch URL bag: {e}"),
                });
            }
        };
        let url_bag_bytes = match url_bag_resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to read URL bag response: {e}"),
                });
            }
        };
        let url_bag_plist = match PlistValue::from_reader_xml(&*url_bag_bytes) {
            Ok(v) => v,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to parse URL bag plist: {e}"),
                });
            }
        };
        let urls = url_bag_plist
            .as_dictionary()
            .and_then(|d| d.get("urls"))
            .and_then(|v| v.as_dictionary())
            .map(|d| {
                d.iter()
                    .filter_map(|(k, v)| v.as_string().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        // 4. SRP Step 1
        let mut srp = crate::sideload::server::apple_srp_session::AppleSrpSession::new();
        let a = srp.step1();

        // 5. Prepare first request plist
        let cpd = Self::client_provided_data(anisette_data.clone()).await;
        let mut header = PlistDict::new();
        header.insert(
            "Version".to_string(),
            PlistValue::String("1.0.1".to_string()),
        );
        let mut request = PlistDict::new();
        request.insert("A2k".to_string(), PlistValue::Data(a.clone()));
        request.insert("cpd".to_string(), cpd);
        request.insert("o".to_string(), PlistValue::String("init".to_string()));
        request.insert(
            "ps".to_string(),
            PlistValue::Array(vec![
                PlistValue::String("s2k".to_string()),
                PlistValue::String("s2k_fo".to_string()),
            ]),
        );
        request.insert("u".to_string(), PlistValue::String(apple_id.clone()));
        let mut plist = PlistDict::new();
        plist.insert("Header".to_string(), PlistValue::Dictionary(header));
        plist.insert("Request".to_string(), PlistValue::Dictionary(request));
        let mut plist_bytes = Vec::new();
        PlistValue::Dictionary(plist)
            .to_writer_xml(&mut plist_bytes)
            .unwrap();

        // 6. Send first request
        let gs_service_url = urls.get("gsService").cloned().unwrap_or_default();

        let resp1 = match client
            .post(&gs_service_url)
            .headers(req_headers.clone())
            .body(plist_bytes)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to send first SRP request: {e}"),
                });
            }
        };
        let resp1_bytes = match resp1.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to read first SRP response: {e}"),
                });
            }
        };
        let resp1_plist = match PlistValue::from_reader_xml(&*resp1_bytes) {
            Ok(v) => v,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to parse first SRP plist: {e}"),
                });
            }
        };
        let response1 = resp1_plist
            .as_dictionary()
            .and_then(|d| d.get("Response"))
            .and_then(|v| v.as_dictionary())
            .cloned()
            .unwrap_or_default();

        // 7. Check for error in response1
        if let Some(status) = response1.get("Status").and_then(|v| v.as_dictionary()) {
            if let Some(err) = Self::validate_status(status) {
                return AppleLoginResponse::Error(err);
            }
        }

        // 8. Extract SRP params
        let iterations = response1
            .get("i")
            .and_then(|v| v.as_unsigned_integer())
            .unwrap_or(0) as usize;
        let salt = response1
            .get("s")
            .and_then(|v| v.as_data())
            .map(|v| v.to_vec())
            .unwrap_or_default();
        let selected_protocol = response1
            .get("sp")
            .and_then(|v| v.as_string())
            .unwrap_or("s2k");
        let cookie = response1.get("c").and_then(|v| v.as_string()).unwrap_or("");
        let b = response1
            .get("B")
            .and_then(|v| v.as_data())
            .map(|v| v.to_vec())
            .unwrap_or_default();

        // 9. SRP Step 2
        let m1 = srp.step2(
            &apple_id,
            &password,
            selected_protocol == "s2k_fo",
            &b,
            &salt,
            iterations,
        );

        // Print the first and second request XML for debugging
        // if let Ok(xml) = String::from_utf8(plist_bytes.clone()) {
        //     println!("First SRP request XML:\n{xml}");
        // }
        // if let Ok(xml) = String::from_utf8(plist2_bytes.clone()) {
        //     println!("Second SRP request XML:\n{xml}");
        // }

        // 10. Prepare second request plist
        let cpd2 = Self::client_provided_data(anisette_data.clone()).await;
        let mut header2 = PlistDict::new();
        header2.insert(
            "Version".to_string(),
            PlistValue::String("1.0.1".to_string()),
        );
        let mut request2 = PlistDict::new();
        request2.insert("M1".to_string(), PlistValue::Data(m1.clone()));
        request2.insert("c".to_string(), PlistValue::String(cookie.to_string()));
        request2.insert("cpd".to_string(), cpd2);
        request2.insert("o".to_string(), PlistValue::String("complete".to_string()));
        request2.insert("u".to_string(), PlistValue::String(apple_id.clone()));
        let mut plist2 = PlistDict::new();
        plist2.insert("Header".to_string(), PlistValue::Dictionary(header2));
        plist2.insert("Request".to_string(), PlistValue::Dictionary(request2));

        //println!("plist2: {:?}", plist2);

        let mut plist2_bytes = Vec::new();
        PlistValue::Dictionary(plist2)
            .to_writer_xml(&mut plist2_bytes)
            .unwrap();

        // 11. Send second request
        let resp2 = match client
            .post(&gs_service_url)
            .headers(req_headers.clone())
            .body(plist2_bytes)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to send second SRP request: {e}"),
                });
            }
        };
        let resp2_bytes = match resp2.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to read second SRP response: {e}"),
                });
            }
        };
        let resp2_plist = match PlistValue::from_reader_xml(&*resp2_bytes) {
            Ok(v) => v,
            Err(e) => {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to parse second SRP plist: {e}"),
                });
            }
        };
        let response2 = resp2_plist
            .as_dictionary()
            .and_then(|d| d.get("Response"))
            .and_then(|v| v.as_dictionary())
            .cloned()
            .unwrap_or_default();

        // 12. Check for error in response2
        if let Some(status) = response2.get("Status").and_then(|v| v.as_dictionary()) {
            if let Some(err) = Self::validate_status(status) {
                return AppleLoginResponse::Error(err);
            }
        }

        // 13. Check for 2FA requirement
        let hsc = response2
            .get("Status")
            .and_then(|v| v.as_dictionary())
            .and_then(|d| d.get("hsc"))
            .and_then(|v| v.as_unsigned_integer())
            .unwrap_or(0);

        if hsc == 409 {
            // 2FA required
            if let Err(e) = window.emit("2fa_required", ()) {
                return AppleLoginResponse::Error(AppleLoginError {
                    code: AppleLoginErrorCode::UnableToSignIn,
                    description: format!("Failed to emit 2fa_required event: {e}"),
                });
            }
            let (tx, rx) = oneshot::channel::<String>();
            let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
            let handler_id = window.listen("2fa_recieved", {
                let tx = tx.clone();
                move |event| {
                    let code = event.payload();
                    if let Some(sender) = tx.lock().unwrap().take() {
                        let _ = sender.send(code.to_string());
                    }
                }
            });
            let code = match rx.await {
                Ok(code) => code,
                Err(_) => {
                    window.unlisten(handler_id);
                    return AppleLoginResponse::Error(AppleLoginError {
                        code: AppleLoginErrorCode::No2FAAttempt,
                        description: "2FA code not received".to_string(),
                    });
                }
            };
            window.unlisten(handler_id);

            // Here you would send the code to Apple and handle the response.
            // For demonstration, return success.
            return AppleLoginResponse::Account(AppleAccount {
                anisette_data,
                apple_identifier: apple_id,
                adsid: "STUB_ADSID".to_string(),
                token: "STUB_TOKEN".to_string(),
                urls,
            });
        }

        // 14. If no 2FA, complete authentication (stubbed)
        AppleLoginResponse::Account(AppleAccount {
            anisette_data,
            apple_identifier: apple_id,
            adsid: "STUB_ADSID".to_string(),
            token: "STUB_TOKEN".to_string(),
            urls,
        })
    }

    // --- login with next step handler (stub) ---
    #[allow(clippy::too_many_arguments)]
    pub fn login_with_next_step_handler(
        anisette_data: Value,
        _apple_id: String,
        _password: String,
        _next_step_handler: NextLoginStepHandler,
    ) -> AppleLoginResponse {
        // TODO: Implement this function
        AppleLoginResponse::Error(AppleLoginError {
            code: AppleLoginErrorCode::UnableToSignIn,
            description: "Not implemented".to_string(),
        })
    }

    // --- Use plist crate for send_request stub ---
    pub fn send_request(&self, _url: &str, _request: Option<PlistValue>) -> PlistValue {
        // TODO: Implement this function
        let mut dict = PlistDict::new();
        dict.insert(
            "stub".to_string(),
            PlistValue::String("not implemented".to_string()),
        );
        PlistValue::Dictionary(dict)
    }
}
