use std::path::PathBuf;

use isideload::{
    device::{list_devices, DeviceInfo},
    sideload, Error, SideloadLogger,
};
use tauri::{Emitter, Manager, Window};

pub struct TauriLogger<'a> {
    window: &'a Window,
}

impl<'a> SideloadLogger for TauriLogger<'a> {
    fn log(&self, message: &str) {
        self.window.emit("build-output", message.to_string()).ok();
    }

    fn error(&self, error: &Error) {
        self.window
            .emit("build-output", format!("Error: {}", error))
            .ok();
    }
}

pub async fn sideload_app(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
    device: DeviceInfo,
    app_path: PathBuf,
) -> Result<(), String> {
    let dev_session =
        crate::sideloader::apple::get_developer_session(&handle, &window, anisette_server.clone())
            .await?;
    let logger = TauriLogger { window };
    let store_dir = handle.path().app_config_dir().map_err(|e| e.to_string())?;
    sideload::sideload_app(logger, &dev_session, &device, app_path, store_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn refresh_idevice(window: tauri::Window) {
    match list_devices().await {
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
