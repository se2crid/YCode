use icloud_auth::{AnisetteConfiguration, AppleAccount, DeveloperDeviceType, DeveloperTeam};
use once_cell::sync::OnceCell;
use serde_json::Value;
use std::{
    sync::{mpsc::RecvTimeoutError, Arc, Mutex},
    time::Duration,
};
use tauri::{Emitter, Listener, Manager};

use crate::{
    device::DeviceInfo,
    emit_error_and_return,
    sideloader::apple_commands::{get_stored_credentials, store_credentials},
};

pub static APPLE_ACCOUNT: OnceCell<Mutex<Option<Arc<AppleAccount>>>> = OnceCell::new();

pub async fn get_account(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<Arc<AppleAccount>, String> {
    let cell = APPLE_ACCOUNT.get_or_init(|| Mutex::new(None));
    {
        let account_guard = cell.lock().unwrap();
        if let Some(account) = &*account_guard {
            return Ok(account.clone());
        }
    }

    let account = login(handle, window, anisette_server).await?;
    let mut account_guard = cell.lock().unwrap();
    *account_guard = Some(account.clone());
    Ok(account)
}

pub async fn login(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<Arc<AppleAccount>, String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let window_clone = window.clone();

    let appleid_closure = move || -> (String, String) {
        if let Some((email, password)) = get_stored_credentials() {
            window_clone
                .emit(
                    "build-output",
                    "Using stored Apple ID credentials".to_string(),
                )
                .ok();
            return (email, password);
        }

        window_clone
            .emit("apple-id-required", ())
            .expect("Failed to emit apple-id-required event");

        let tx1 = tx.clone();
        let handler_id_recieved = window_clone.listen("apple-id-recieved", move |event| {
            let json = event.payload();
            let _ = tx1.send(format!("apple-id-recieved:{}", json));
        });

        let tx2 = tx.clone();
        let handler_id_cancelled = window_clone.listen("login-cancelled", move |_event| {
            let _ = tx2.send("login-cancelled".to_string());
        });

        let result = rx.recv_timeout(Duration::from_secs(120));
        window_clone.unlisten(handler_id_recieved);
        window_clone.unlisten(handler_id_cancelled);

        match result {
            Ok(msg) if msg.starts_with("apple-id-recieved:") => {
                let json = &msg["apple-id-recieved:".len()..];
                let json: Value = serde_json::from_str(json).expect("Failed to parse json");
                let apple_id = json
                    .get("appleId")
                    .and_then(Value::as_str)
                    .expect("Failed to get apple_id from json")
                    .to_string();
                let password = json
                    .get("applePass")
                    .and_then(Value::as_str)
                    .expect("Failed to get password from json")
                    .to_string();
                let save_credentials = json
                    .get("saveCredentials")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                if save_credentials {
                    if let Err(e) = store_credentials(&apple_id, &password) {
                        window_clone
                            .emit(
                                "build-output",
                                format!("Failed to save credentials: {:?}", e),
                            )
                            .ok();
                    }
                }

                (apple_id, password)
            }
            Ok(msg) if msg == "login-cancelled" => {
                window_clone
                    .emit("build-output", "Login cancelled by user".to_string())
                    .ok();
                panic!("Login cancelled by user");
            }
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) | _ => {
                window_clone
                    .emit("build-output", "Login cancelled or timed out".to_string())
                    .ok();
                panic!("Login cancelled or timed out");
            }
        }
    };

    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let window_clone = window.clone();
    let tfa_closure = move || -> String {
        window_clone
            .emit("2fa-required", ())
            .expect("Failed to emit 2fa-required event");

        let tx = tx.clone();
        let handler_id = window_clone.listen("2fa-recieved", move |event| {
            let code = event.payload();
            let _ = tx.send(code.to_string());
        });

        let result = rx.recv_timeout(Duration::from_secs(120));
        window_clone.unlisten(handler_id);

        match result {
            Ok(code) => {
                let code = code.trim_matches('"').to_string();
                code
            }
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => {
                window_clone
                    .emit("build-output", "2FA cancelled or timed out".to_string())
                    .ok();
                panic!("2FA cancelled or timed out");
            }
        }
    };

    let config = AnisetteConfiguration::default();
    let config =
        config.set_configuration_path(handle.path().app_config_dir().map_err(|e| e.to_string())?);
    let config = config.set_anisette_url(format!("https://{}", anisette_server));
    window
        .emit("build-output", "Logging in...")
        .map_err(|e| e.to_string())?;

    let account = AppleAccount::login(appleid_closure, tfa_closure, config).await;
    if let Err(e) = account {
        window
            .emit("build-output", "Login failed or cancelled".to_string())
            .ok();
        window.emit("build-output", format!("{:?}", e)).ok();
        return Err(format!("{:?}", e));
    }
    let account = Arc::new(account.unwrap());
    window
        .emit("build-output", "Logged in successfully".to_string())
        .map_err(|e| e.to_string())?;

    Ok(account)
}

pub async fn ensure_device_registered(
    account: &AppleAccount,
    window: &tauri::Window,
    team: &DeveloperTeam,
    device: &DeviceInfo,
) -> Result<(), String> {
    let devices = account
        .list_devices(DeveloperDeviceType::Ios, team)
        .await
        .map_err(|e| {
            emit_error_and_return(window, &format!("Failed to list devices: {:?}", e))
                .err()
                .unwrap()
        })?;
    if !devices.iter().any(|d| d.device_number == device.uuid) {
        window
            .emit(
                "build-output",
                "Device not found in your account".to_string(),
            )
            .ok();
        // TODO: Actually test!
        account
            .add_device(DeveloperDeviceType::Ios, team, &device.name, &device.uuid)
            .await
            .map_err(|e| format!("Failed to add device: {:?}", e))?;
        window
            .emit("build-output", "Device added to your account".to_string())
            .ok();
    }
    window
        .emit("build-output", "Device is a development device".to_string())
        .ok();
    Ok(())
}
