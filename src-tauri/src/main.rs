// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[tauri::command]
fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
async fn has_wsl() -> bool {
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
async fn has_theos() -> bool {
    if is_windows() {
        if !has_wsl().await {
            return false;
        }

        // Checks that $THEOS is set in wsl and that the directory contains theos's files
        // For some reason, without cmd /C the command doesn't work properly. I'm guessing its some sort of quoting issue but I couldn't figure it out.
        let output = Command::new("cmd")
            .args(&["/C", r#"bash -ic 'test -d $THEOS/extras ; echo $?'"#])
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
async fn update_theos(window: tauri::Window) {
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
async fn install_theos_windows(handle: tauri::AppHandle, window: tauri::Window, password: String) {
    let resource_path = match handle.path_resolver().resolve_resource("install_theos.sh") {
        Some(path) => path,
        None => {
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
async fn install_theos(handle: tauri::AppHandle, window: tauri::Window) {
    let resource_path = match handle.path_resolver().resolve_resource("install_theos.sh") {
        Some(path) => path,
        None => {
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

// Handles taking a command and piping the stdout and exit code to the window
async fn pipe_command(cmd: &mut Command, window: tauri::Window, cmd_name: &str) {
    let name = &format!("{}-output", cmd_name);
    cmd.stdout(Stdio::piped());

    let mut command = match cmd.spawn() {
        Ok(cmd) => cmd,
        Err(_) => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let output = match command.stdout.take() {
        Some(out) => out,
        None => {
            window
                .emit(name, "command.done.999".to_string())
                .expect("failed to send output");
            return;
        }
    };

    let reader = BufReader::new(output);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                window.emit(name, line).expect("failed to send output");
            }
            Err(_) => {
                window
                    .emit(name, "command.done.999".to_string())
                    .expect("failed to send output");
                return;
            }
        }
    }

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
    let drive_letter = path[4..].chars().next().unwrap().to_ascii_lowercase();
    let rest_of_path = path[6..].replace("\\", "/");
    format!("/mnt/{}/{}", drive_letter, rest_of_path)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            has_theos,
            update_theos,
            install_theos,
            install_theos_windows,
            is_windows,
            has_wsl
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
