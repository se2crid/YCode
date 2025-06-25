// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod theos;
#[macro_use]
mod device;
mod sideloader;

use device::refresh_idevice;
use sideloader::apple_commands::{
    delete_stored_credentials, get_apple_email, get_certificates, reset_anisette,
    revoke_certificate,
};
use tauri::Emitter;
use theos::{
    build_theos, clean_theos, deploy_theos, has_theos, has_wsl, install_theos_linux,
    install_theos_windows, is_windows, update_theos,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            has_theos,
            update_theos,
            install_theos_linux,
            install_theos_windows,
            is_windows,
            has_wsl,
            build_theos,
            deploy_theos,
            clean_theos,
            refresh_idevice,
            delete_stored_credentials,
            reset_anisette,
            get_apple_email,
            revoke_certificate,
            get_certificates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn emit_error_and_return(window: &tauri::Window, msg: &str) -> Result<(), String> {
    window.emit("build-output", msg.to_string()).ok();
    window.emit("build-output", "command.done.999").ok();
    Err(msg.to_string())
}
