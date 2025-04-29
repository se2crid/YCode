use icloud_auth::{AnisetteConfiguration, AppleAccount};
use idevice::usbmuxd::{UsbmuxdAddr, UsbmuxdConnection};
use idevice::{lockdown::LockdownClient, IdeviceService};
use keyring::{Entry, Error as KeyringError};
use serde::Serialize;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;
use tauri::path::BaseDirectory;
use tauri::{Emitter, Listener, Manager};

#[derive(Serialize, Clone)]
struct DeviceInfo {
    name: String,
    id: u32,
}

#[tauri::command]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
pub async fn has_wsl() -> bool {
    if !is_windows() {
        return false;
    }

    let output = Command::new("wsl")
        .arg("echo")
        .arg("1")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);
    return output.trim() == "1";
}
#[tauri::command]
pub async fn has_theos() -> bool {
    if is_windows() {
        if !has_wsl().await {
            return false;
        }

        // Checks that $THEOS is set in wsl and that the directory contains theos's files
        // For some reason, without cmd /C the command doesn't work properly. I'm guessing its some sort of quoting issue but I couldn't figure it out.
        let output = Command::new("cmd")
            .args(&["/C", r#"wsl bash -ic 'test -d $THEOS/extras ; echo $?'"#])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8_lossy(&output.stdout);

        return stdout.trim() == "0";
    }
    // On linux, can just check if $THEOS is set and the directory exists
    if let Ok(theos) = std::env::var("THEOS") {
        return std::path::Path::new(&theos).exists();
    }
    false
}

#[tauri::command]
pub async fn update_theos(window: tauri::Window) {
    let mut command = if is_windows() {
        let mut cmd = Command::new("wsl");
        cmd.arg("bash").arg("-ic").arg("'$THEOS/bin/update-theos'");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("$THEOS/bin/update-theos");
        cmd
    };

    pipe_command(&mut command, window, "update-theos").await;
}

#[tauri::command]
pub async fn install_theos_windows(
    handle: tauri::AppHandle,
    window: tauri::Window,
    password: String,
) {
    let resource_path = match handle
        .path()
        .resolve("install_theos.sh", BaseDirectory::Resource)
    {
        Ok(path) => path,
        Err(_) => {
            window
                .emit("install-theos-output", "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let wsl_path = windows_to_wsl_path(&resource_path.to_string_lossy());
    let mut command = Command::new("wsl");
    // Windows line endings are \r\n, so we need to remove the \r for bash to work properly
    command.arg("sh").arg("-c").arg(format!(
        "export SUDO_PASSWORD={} ; tr -d '\r' < {} | bash",
        password, wsl_path
    ));

    pipe_command(&mut command, window, "install-theos").await;
}

#[tauri::command]
pub async fn install_theos(handle: tauri::AppHandle, window: tauri::Window) {
    let resource_path = match handle
        .path()
        .resolve("install_theos.sh", BaseDirectory::Resource)
    {
        Ok(path) => path,
        Err(_) => {
            window
                .emit("install-theos-output", "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("bash {}", resource_path.display()));

    pipe_command(&mut command, window, "install-theos").await;
}

async fn pipe_command(cmd: &mut Command, window: tauri::Window, cmd_name: &str) {
    let name = &format!("{}-output", cmd_name);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut command = match cmd.spawn() {
        Ok(cmd) => cmd,
        Err(_) => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let stdout = match command.stdout.take() {
        Some(out) => out,
        None => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let stderr = match command.stderr.take() {
        Some(err) => err,
        None => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let window_clone = window.clone();
    let name_clone = name.to_string();

    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    window_clone
                        .emit(&name_clone, line)
                        .expect("failed to send output");
                }
                Err(_) => {
                    window_clone
                        .emit(&name_clone, "command.done.999".to_string())
                        .expect("failed to send output");
                    return;
                }
            }
        }
    });

    let window_clone = window.clone();
    let name_clone = name.to_string();

    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    window_clone
                        .emit(&name_clone, line)
                        .expect("failed to send output");
                }
                Err(_) => {
                    window_clone
                        .emit(&name_clone, "command.done.999".to_string())
                        .expect("failed to send output");
                    return;
                }
            }
        }
    });

    stdout_handle.join().expect("stdout thread panicked");
    stderr_handle.join().expect("stderr thread panicked");

    let exit_status = match command.wait() {
        Ok(status) => status,
        Err(_) => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let exit_code = exit_status.code().unwrap_or(1);

    window
        .emit(name, format!("command.done.{}", exit_code))
        .expect("failed to send output");
}

fn windows_to_wsl_path(path: &str) -> String {
    let (drive_letter_index, rest_of_path_index) = if path.starts_with("\\\\?\\") {
        (4, 6)
    } else {
        (0, 2)
    };

    let drive_letter = path[drive_letter_index..]
        .chars()
        .next()
        .unwrap()
        .to_ascii_lowercase();
    let rest_of_path = path[rest_of_path_index..].replace("\\", "/");
    format!("/mnt/{}/{}", drive_letter, rest_of_path)
}

async fn build_theos_linux(window: tauri::Window, folder: &str) {
    // cd to the folder and run make clean package
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("cd {} && make clean package", folder));

    pipe_command(&mut command, window, "build").await;
}

async fn build_theos_windows(window: tauri::Window, folder: &str) {
    let mut command = Command::new("wsl");
    command.arg("bash").arg("-ic").arg(format!(
        "cd {} && make clean package",
        windows_to_wsl_path(folder)
    ));

    pipe_command(&mut command, window, "build").await;
}

#[tauri::command]
pub async fn build_theos(window: tauri::Window, folder: String) {
    if is_windows() {
        build_theos_windows(window, &folder).await;
    } else {
        build_theos_linux(window, &folder).await;
    }
}

// Helper functions for credential storage
fn store_credentials(email: &str, password: &str) -> Result<(), KeyringError> {
    // Store email under a fixed key, and password under the email key
    let email_entry = Entry::new("y-code", "apple_id_email")?;
    email_entry.set_password(email)?;
    let pass_entry = Entry::new("y-code", email)?;
    pass_entry.set_password(password)
}

fn get_stored_credentials() -> Option<(String, String)> {
    // Retrieve email from fixed key, then password from that email
    let email_entry = Entry::new("y-code", "apple_id_email").ok()?;
    let email = email_entry.get_password().ok()?;
    let pass_entry = Entry::new("y-code", &email).ok()?;
    let password = pass_entry.get_password().ok()?;
    Some((email, password))
}

#[tauri::command]
pub fn delete_stored_credentials() -> Result<(), String> {
    let email_entry =
        Entry::new("y-code", "apple_id_email").map_err(|e| format!("Keyring error: {:?}", e))?;
    let email = match email_entry.get_password() {
        Ok(email) => email,
        Err(_) => {
            // If email is not found, nothing to delete
            return Ok(());
        }
    };
    let pass_entry = Entry::new("y-code", &email).map_err(|e| format!("Keyring error: {:?}", e))?;
    let _ = pass_entry.delete_password();
    email_entry
        .delete_password()
        .map_err(|e| format!("Keyring error: {:?}", e))
}

#[tauri::command]
pub async fn deploy_theos(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    _folder: String,
) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let window_clone = window.clone();
    let appleid_closure = move || -> (String, String) {
        // Try to get stored credentials first
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
                    // Store both email and password securely
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
            .emit("build-output", "Login failed or cancelled!".to_string())
            .ok();
        window.emit("build-output", format!("{:?}", e)).ok();
        window
            .emit("build-output", "command.done.999".to_string())
            .ok();
        return Err(format!("{:?}", e));
    }
    let account = account.unwrap();
    window
        .emit("build-output", "Logged in successfully!".to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn reset_anisette(handle: tauri::AppHandle) -> Result<(), String> {
    let config_dir = handle.path().app_config_dir().map_err(|e| e.to_string())?;
    let status_path = config_dir.join("state.plist");
    if status_path.exists() {
        std::fs::remove_file(&status_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
#[tauri::command]
pub async fn refresh_idevice(window: tauri::Window) {
    let mut usbmuxd = UsbmuxdConnection::default()
        .await
        .expect("Unable to connect to usbmxud");
    let devs = usbmuxd.get_devices().await.unwrap();
    if devs.is_empty() {
        window
            .emit("idevices", Vec::<DeviceInfo>::new())
            .expect("Failed to send devices");
        return;
    }

    let device_info_futures: Vec<_> = devs
        .iter()
        .map(|d| async move {
            // Use current device (d) instead of always using devs[0]
            let provider = d.to_provider(UsbmuxdAddr::from_env_var().unwrap(), 0, "y-code");
            let device_uid = d.device_id;

            let mut lockdown_client = match LockdownClient::connect(&provider).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Unable to connect to lockdown: {e:?}");
                    return DeviceInfo {
                        name: String::from("Unknown Device"),
                        id: device_uid,
                    };
                }
            };

            let device_name = lockdown_client
                .get_value("DeviceName")
                .await
                .expect("Failed to get device name")
                .as_string()
                .expect("Failed to convert device name to string")
                .to_string();

            DeviceInfo {
                name: device_name,
                id: device_uid,
            }
        })
        .collect();

    let device_infos = futures::future::join_all(device_info_futures).await;

    window
        .emit("idevices", device_infos)
        .expect("Failed to send devices");
}
