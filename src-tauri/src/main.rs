// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod commands;
mod device;
mod sideloader;
mod theos;

use commands::*;
use tauri::Emitter;

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
            install_theos,
            install_theos_windows,
            is_windows,
            has_wsl,
            build_theos,
            deploy_theos,
            refresh_idevice,
            delete_stored_credentials,
            reset_anisette,
            get_apple_email,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn emit_error_and_return(window: &tauri::Window, msg: &str) -> Result<(), String> {
    window.emit("build-output", msg.to_string()).ok();
    window.emit("build-output", "command.done.999").ok();
    Err(msg.to_string())
}
