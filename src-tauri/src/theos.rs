use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::path::BaseDirectory;
use tauri::{Emitter, Manager};

use crate::emit_error_and_return;

pub async fn pipe_command(cmd: &mut Command, window: tauri::Window, cmd_name: &str) {
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

pub fn windows_to_wsl_path(path: &str) -> String {
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

pub async fn build_theos_linux(window: tauri::Window, folder: &str) {
    // cd to the folder and run make clean package
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("cd {} && make clean package", folder));

    pipe_command(&mut command, window, "build").await;
}

pub async fn build_theos_windows(window: tauri::Window, folder: &str) {
    let mut command = Command::new("wsl");
    command.arg("bash").arg("-ic").arg(format!(
        "cd {} && make clean package",
        windows_to_wsl_path(folder)
    ));

    pipe_command(&mut command, window, "build").await;
}

pub async fn has_wsl() -> bool {
    if !cfg!(target_os = "windows") {
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

pub async fn has_theos() -> bool {
    if cfg!(target_os = "windows") {
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
    // On linux, can just check if $THEOS is set and the directory exists (i love linux)
    if let Ok(theos) = std::env::var("THEOS") {
        return std::path::Path::new(&theos).exists();
    }
    false
}

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
            emit_error_and_return(&window, "").ok();
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

pub async fn install_theos_linux(handle: tauri::AppHandle, window: tauri::Window) {
    let resource_path = match handle
        .path()
        .resolve("install_theos.sh", BaseDirectory::Resource)
    {
        Ok(path) => path,
        Err(_) => {
            emit_error_and_return(&window, "").ok();
            return;
        }
    };

    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("bash {}", resource_path.display()));

    pipe_command(&mut command, window, "install-theos").await;
}
