use crate::apple::ensure_device_registered;
use crate::device::DeviceInfo;
use crate::emit_error_and_return;
use crate::theos::{build_theos_linux, build_theos_windows, pipe_command};
use std::process::Command;
use tauri::{Emitter, Manager};

#[tauri::command]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
pub async fn has_wsl() -> bool {
    crate::theos::has_wsl().await
}

#[tauri::command]
pub async fn has_theos() -> bool {
    crate::theos::has_theos().await
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
    crate::theos::install_theos_windows(handle, window, password).await;
}

#[tauri::command]
pub async fn install_theos(handle: tauri::AppHandle, window: tauri::Window) {
    crate::theos::install_theos_linux(handle, window).await;
}

#[tauri::command]
pub async fn build_theos(window: tauri::Window, folder: String) {
    if is_windows() {
        build_theos_windows(window, &folder).await;
    } else {
        build_theos_linux(window, &folder).await;
    }
}

#[tauri::command]
pub fn delete_stored_credentials() -> Result<(), String> {
    crate::apple::delete_stored_credentials()
}

#[tauri::command]
pub fn get_apple_email() -> String {
    let credentials = crate::apple::get_stored_credentials();
    if credentials.is_none() {
        return "".to_string();
    }
    let (email, _) = credentials.unwrap();
    email
}

#[tauri::command]
pub async fn deploy_theos(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    device: DeviceInfo,
    _folder: String,
) -> Result<(), String> {
    if device.uuid.is_empty() {
        return emit_error_and_return(&window, "No device selected");
    }
    let account = match crate::apple::get_account(&handle, &window, anisette_server).await {
        Ok(acc) => acc,
        Err(e) => {
            return emit_error_and_return(
                &window,
                &format!("Failed to login to Apple account: {:?}", e),
            );
        }
    };
    let teams = match account.list_teams().await {
        Ok(t) => t,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to list teams: {:?}", e));
        }
    };
    let team = &teams[0];
    window
        .emit("build-output", "Successfully retrieved team".to_string())
        .ok();
    ensure_device_registered(&account, &window, team, &device).await?;
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
    match crate::device::list_devices().await {
        Ok(devices) => {
            window
                .emit("idevices", devices)
                .expect("Failed to send devices");
        }
        Err(e) => {
            window
                .emit("idevices", Vec::<DeviceInfo>::new())
                .expect("Failed to send error");
            eprintln!("Failed to list devices: {}", e);
        }
    };
}
