use std::process::{Command, Stdio};
use wslpath2::{convert, Conversion};

pub fn windows_to_wsl_path(path: &str) -> String {
    convert(path, None, Conversion::WslToWindows, false).unwrap()
}

#[tauri::command]
pub fn has_wsl() -> bool {
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
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}
