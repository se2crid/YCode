// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod commands;

use commands::*;

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
            reset_anisette
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
