use crate::device::DeviceInfo;
use crate::emit_error_and_return;
use crate::sideloader::apple::ensure_device_registered;
use crate::sideloader::certificate::CertificateIdentity;
use crate::theos::{build_theos_linux, build_theos_windows, pipe_command};
use icloud_auth::DeveloperDeviceType;
use std::process::Command;
use tauri::{Emitter, Manager};

#[tauri::command]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[tauri::command]
pub async fn has_wsl() -> bool {
    crate::theos::has_wsl().await
}

#[tauri::command]
pub async fn has_theos() -> bool {
    crate::theos::has_theos().await
}

#[tauri::command]
pub async fn update_theos(window: tauri::Window) {
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
pub async fn install_theos_windows(
    handle: tauri::AppHandle,
    window: tauri::Window,
    password: String,
) {
    crate::theos::install_theos_windows(handle, window, password).await;
}

#[tauri::command]
pub async fn install_theos(handle: tauri::AppHandle, window: tauri::Window) {
    crate::theos::install_theos_linux(handle, window).await;
}

#[tauri::command]
pub async fn build_theos(window: tauri::Window, folder: String) {
    if is_windows() {
        build_theos_windows(window, &folder).await;
    } else {
        build_theos_linux(window, &folder).await;
    }
}

#[tauri::command]
pub fn delete_stored_credentials() -> Result<(), String> {
    crate::sideloader::apple::delete_stored_credentials()
}

#[tauri::command]
pub fn get_apple_email() -> String {
    let credentials = crate::sideloader::apple::get_stored_credentials();
    if credentials.is_none() {
        return "".to_string();
    }
    let (email, _) = credentials.unwrap();
    email
}

#[tauri::command]
pub async fn deploy_theos(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    device: DeviceInfo,
    folder: String,
) -> Result<(), String> {
    if device.uuid.is_empty() {
        return emit_error_and_return(&window, "No device selected");
    }
    let account =
        match crate::sideloader::apple::get_account(&handle, &window, anisette_server).await {
            Ok(acc) => acc,
            Err(e) => {
                return emit_error_and_return(
                    &window,
                    &format!("Failed to login to Apple account: {:?}", e),
                );
            }
        };
    let teams = match account.list_teams().await {
        Ok(t) => t,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to list teams: {:?}", e));
        }
    };
    let team = &teams[0];
    window
        .emit("build-output", "Successfully retrieved team".to_string())
        .ok();
    ensure_device_registered(&account, &window, team, &device).await?;

    let certs = account
        .list_all_development_certs(icloud_auth::DeveloperDeviceType::Ios, team)
        .await;
    if certs.is_err() {
        return emit_error_and_return(
            &window,
            &format!("Failed to list certificates: {:?}", certs.err()),
        );
    }
    let certs = certs.unwrap();
    print!("Available certificates:\n");
    for cert in &certs {
        println!("{}: {}", cert.name, cert.serial_number);
    }
    let config_dir = handle.path().app_config_dir().map_err(|e| e.to_string())?;
    let cert = match CertificateIdentity::new(config_dir, &account, get_apple_email()).await {
        Ok(c) => c,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to get certificate: {:?}", e));
        }
    };
    window
        .emit(
            "build-output",
            "Certificate acquired succesfully".to_string(),
        )
        .ok();
    let app_ids = match account
        .list_app_ids(icloud_auth::DeveloperDeviceType::Ios, team)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to list app IDs: {:?}", e));
        }
    };
    let packages_path = std::path::PathBuf::from(&folder).join("packages");

    let ipa_path = std::fs::read_dir(&packages_path)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().map_or(false, |ext| ext == "ipa"))
        .map(|entry| entry.path());
    if ipa_path.is_none() {
        return emit_error_and_return(&window, "No IPA file found in packages directory");
    }
    let ipa_path = ipa_path.unwrap();
    let mut app = crate::sideloader::application::Application::new(ipa_path);
    let main_app_bundle_id = match app.bundle.bundle_identifier() {
        Some(id) => id.to_string(),
        None => {
            return emit_error_and_return(&window, "No bundle identifier found in IPA");
        }
    };
    let main_app_id_str = format!("{}.{}", main_app_bundle_id, team.team_id);
    let main_app_name = match app.bundle.bundle_name() {
        Some(name) => name.to_string(),
        None => {
            return emit_error_and_return(&window, "No bundle name found in IPA");
        }
    };

    let extensions = app.bundle.app_extensions_mut();
    // for each extension, ensure it has a unique bundle identifier that starts with the main app's bundle identifier
    for ext in extensions.iter_mut() {
        if let Some(id) = ext.bundle_identifier() {
            if !(id.starts_with(&main_app_bundle_id) && id.len() > main_app_bundle_id.len()) {
                return emit_error_and_return(
                    &window,
                    &format!(
                        "Extension {} is not part of the main app bundle identifier: {}",
                        ext.bundle_name().unwrap_or("Unknown"),
                        id
                    ),
                );
            } else {
                ext.set_bundle_identifier(&format!(
                    "{}{}",
                    main_app_id_str,
                    &id[main_app_bundle_id.len()..]
                ));
            }
        }
    }
    app.bundle.set_bundle_identifier(&main_app_id_str);

    let extension_refs: Vec<_> = app.bundle.app_extensions().into_iter().collect();
    let mut bundles_with_app_id = vec![&app.bundle];
    bundles_with_app_id.extend(extension_refs);

    let app_ids_to_register = bundles_with_app_id
        .iter()
        .filter(|bundle| {
            let bundle_id = bundle.bundle_identifier().unwrap_or("");
            app_ids
                .app_ids
                .iter()
                .any(|app_id| app_id.app_id_id == bundle_id)
        })
        .collect::<Vec<_>>();

    if app_ids_to_register.len() > app_ids.available_quantity.try_into().unwrap() {
        return emit_error_and_return(
            &window,
            &format!(
                "This app requires {} app ids, but you only have {} available",
                app_ids_to_register.len(),
                app_ids.available_quantity
            ),
        );
    }

    for bundle in app_ids_to_register {
        let id = bundle.bundle_identifier().unwrap_or("");
        let name = bundle.bundle_name().unwrap_or("");
        if let Err(e) = account
            .add_app_id(DeveloperDeviceType::Ios, &team, &id, &name)
            .await
        {
            return emit_error_and_return(&window, &format!("Failed to register app ID: {:?}", e));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn reset_anisette(handle: tauri::AppHandle) -> Result<(), String> {
    let config_dir = handle.path().app_config_dir().map_err(|e| e.to_string())?;
    let status_path = config_dir.join("state.plist");
    if status_path.exists() {
        std::fs::remove_file(&status_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn refresh_idevice(window: tauri::Window) {
    match crate::device::list_devices().await {
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
