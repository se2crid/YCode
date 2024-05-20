// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};


#[tauri::command]
fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
fn has_wsl() -> bool {
    if !is_windows() {
        return false;
    }
    // Check for wsl
    let output = Command::new("wsl")
        .arg("echo")
        .arg("1")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);
    if output.trim() == "1" {
        return true;
    }
    false
}
#[tauri::command]
fn has_theos() -> bool {
    // If $THEOS is set and the directory exists, return true
    if let Ok(theos) = std::env::var("THEOS") {
        if std::path::Path::new(&theos).exists() {
            return true;
        }
    }
    false
}

#[tauri::command]
async fn update_theos(window: tauri::Window) {
    let mut command = match Command::new("sh")
        .arg("-c")
        .arg("$THEOS/bin/update-theos")
        .stdout(Stdio::piped())
        .spawn() {
            Ok(cmd) => cmd,
            Err(_) => {
                window.emit("update-theos-output", "command.done.999".to_string()).expect("failed to send output");
                return;
            }
        };

    let output = match command.stdout.take() {
        Some(out) => out,
        None => {
            window.emit("update-theos-output", "command.done.999".to_string()).expect("failed to send output");
            return;
        }
    };

    let reader = BufReader::new(output);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                window.emit("update-theos-output", line).expect("failed to send output");
            },
            Err(_) => {
                window.emit("update-theos-output", "command.done.999".to_string()).expect("failed to send output");
                return;
            }
        }
    }

    let exit_status = match command.wait() {
        Ok(status) => status,
        Err(_) => {
            window.emit("update-theos-output", "command.done.999".to_string()).expect("failed to send output");
            return;
        }
    };

    let exit_code = exit_status.code().unwrap_or(1);

    window.emit("update-theos-output", format!("command.done.{}", exit_code)).expect("failed to send output");
}

#[tauri::command]
async fn install_theos(handle: tauri::AppHandle, window: tauri::Window) {
    let resource_path = match handle.path_resolver().resolve_resource("install_theos.sh") {
        Some(path) => path,
        None => {
            window.emit("install-theos-output", "command.done.999".to_string()).expect("failed to send output");
            return;
        }
    };

    let mut command = match Command::new("sh")
        .arg("-c")
        .arg(format!("bash {}", resource_path.display()))
        .stdout(Stdio::piped())
        .spawn() {
            Ok(cmd) => cmd,
            Err(_) => {
                window.emit("install-theos-output", "command.done.999".to_string()).expect("failed to send output");
                return;
            }
        };

    let output = match command.stdout.take() {
        Some(out) => out,
        None => {
            window.emit("install-theos-output", "command.done.999".to_string()).expect("failed to send output");
            return;
        }
    };

    let reader = BufReader::new(output);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                window.emit("install-theos-output", line).expect("failed to send output");
            },
            Err(_) => {
                window.emit("install-theos-output", "command.done.999".to_string()).expect("failed to send output");
                return;
            }
        }
    }

    let exit_status = match command.wait() {
        Ok(status) => status,
        Err(_) => {
            window.emit("install-theos-output", "command.done.999".to_string()).expect("failed to send output");
            return;
        }
    };

    let exit_code = exit_status.code().unwrap_or(1);

    window.emit("install-theos-output", format!("command.done.{}", exit_code)).expect("failed to send output");
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![has_theos, update_theos, install_theos, is_windows, has_wsl])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}