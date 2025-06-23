use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::path::BaseDirectory;
use tauri::{Emitter, Manager};

use crate::device::DeviceInfo;
use crate::emit_error_and_return;
use crate::sideloader::sideload::sideload_ipa;

pub async fn pipe_command(
    cmd: &mut Command,
    window: tauri::Window,
    cmd_name: &str,
    emit_exit_code: bool,
) -> Result<(), String> {
    let name = &format!("{}-output", cmd_name);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut command = match cmd.spawn() {
        Ok(cmd) => cmd,
        Err(_) => {
            return emit_error_and_return(&window, "Failed to spawn build command");
        }
    };

    let stdout = match command.stdout.take() {
        Some(out) => out,
        None => {
            return emit_error_and_return(&window, "Failed to get stdout");
        }
    };

    let stderr = match command.stderr.take() {
        Some(err) => err,
        None => {
            return emit_error_and_return(&window, "Failed to get stderr");
        }
    };

    let stdout_handle = spawn_output_thread(stdout, window.clone(), name.to_string());
    let stderr_handle = spawn_output_thread(stderr, window.clone(), name.to_string());

    stdout_handle.join().expect("stdout thread panicked");
    stderr_handle.join().expect("stderr thread panicked");

    let exit_status = match command.wait() {
        Ok(status) => status,
        Err(_) => {
            return emit_error_and_return(&window, "Failed to wait for command");
        }
    };

    let exit_code = exit_status.code().unwrap_or(1);

    if emit_exit_code {
        window
            .emit(name, format!("command.done.{}", exit_code))
            .expect("failed to send output");
    }

    if exit_code != 0 {
        return Err(format!("Command exited with code {}", exit_code));
    }

    Ok(())
}

fn spawn_output_thread<R: std::io::Read + Send + 'static>(
    reader: R,
    window: tauri::Window,
    name: String,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    window.emit(&name, line).expect("failed to send output");
                }
                Err(err) => {
                    window
                        .emit(&name, "command.done.999".to_string())
                        .expect("failed to send output");
                    eprintln!("Error reading output: {}", err);
                    return;
                }
            }
        }
    })
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

#[tauri::command]
pub async fn has_wsl() -> bool {
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
pub async fn has_theos() -> bool {
    if is_windows() {
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

#[tauri::command]
pub async fn install_theos_windows(
    handle: tauri::AppHandle,
    window: tauri::Window,
    password: String,
) -> Result<(), String> {
    let resource_path = match handle
        .path()
        .resolve("install_theos.sh", BaseDirectory::Resource)
    {
        Ok(path) => path,
        Err(_) => {
            return emit_error_and_return(&window, "");
        }
    };

    let wsl_path = windows_to_wsl_path(&resource_path.to_string_lossy());
    let mut command = Command::new("wsl");
    // Windows line endings are \r\n, so we need to remove the \r for bash to work properly
    command.arg("sh").arg("-c").arg(format!(
        "export SUDO_PASSWORD={} ; tr -d '\r' < {} | bash",
        password, wsl_path
    ));

    pipe_command(&mut command, window, "install-theos", true).await
}

#[tauri::command]
pub async fn install_theos_linux(
    handle: tauri::AppHandle,
    window: tauri::Window,
) -> Result<(), String> {
    let resource_path = match handle
        .path()
        .resolve("install_theos.sh", BaseDirectory::Resource)
    {
        Ok(path) => path,
        Err(_) => {
            return emit_error_and_return(&window, "");
        }
    };

    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("bash {}", resource_path.display()));

    pipe_command(&mut command, window, "install-theos", true).await
}

#[tauri::command]
pub async fn update_theos(window: tauri::Window) -> Result<(), String> {
    let mut command = if is_windows() {
        let mut cmd = Command::new("wsl");
        cmd.arg("bash").arg("-ic").arg("'$THEOS/bin/update-theos'");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("$THEOS/bin/update-theos");
        cmd
    };

    pipe_command(&mut command, window, "update-theos", true).await
}

pub async fn build_theos_linux(
    window: tauri::Window,
    folder: &str,
    emit_exit_code: bool,
) -> Result<(), String> {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("cd {} && make package", folder));

    pipe_command(&mut command, window, "build", emit_exit_code).await
}

pub async fn build_theos_windows(
    window: tauri::Window,
    folder: &str,
    emit_exit_code: bool,
) -> Result<(), String> {
    let mut command = Command::new("wsl");
    command.arg("bash").arg("-ic").arg(format!(
        "cd {} && make package",
        windows_to_wsl_path(folder)
    ));

    pipe_command(&mut command, window, "build", emit_exit_code).await
}

async fn build_theos_int(
    window: tauri::Window,
    folder: String,
    emit_exit_code: bool,
) -> Result<(), String> {
    if is_windows() {
        return build_theos_windows(window, &folder, emit_exit_code).await;
    } else {
        return build_theos_linux(window, &folder, emit_exit_code).await;
    }
}

#[tauri::command]
pub async fn build_theos(window: tauri::Window, folder: String) -> Result<(), String> {
    build_theos_int(window, folder, true).await
}

#[tauri::command]
pub async fn deploy_theos(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    device: DeviceInfo,
    folder: String,
) -> Result<(), String> {
    let packages_path = std::path::PathBuf::from(&folder).join("packages");
    // delete everything in the packages directory
    if packages_path.exists() {
        std::fs::remove_dir_all(&packages_path)
            .map_err(|e| format!("Failed to remove packages directory: {}", e.to_string()))?;
    }

    build_theos_int(window.clone(), folder.clone(), false).await?;
    window
        .emit("build-output", "App Built Succesfully!".to_string())
        .ok();
    let ipa_path = std::fs::read_dir(&packages_path)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().map_or(false, |ext| ext == "ipa"))
        .map(|entry| entry.path());

    if ipa_path.is_none() {
        return emit_error_and_return(&window, "No IPA file found in packages directory");
    }
    let ipa_path = ipa_path.unwrap();

    sideload_ipa(&handle, window.clone(), anisette_server, device, ipa_path).await?;

    window
        .emit("build-output", "App installed!".to_string())
        .ok();
    window
        .emit("build-output", "command.done.0".to_string())
        .ok();

    Ok(())
}

#[tauri::command]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}
