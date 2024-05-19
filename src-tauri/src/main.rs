// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn has_theos() -> bool {
    // If $THEOS is set, we have Theos
    std::env::var("THEOS").is_ok()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![has_theos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
