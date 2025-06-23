use crate::device::{install_app, DeviceInfo};
use crate::emit_error_and_return;
use crate::sideloader::apple::ensure_device_registered;
use crate::sideloader::certificate::CertificateIdentity;
use crate::theos::{build_theos_linux, build_theos_windows, pipe_command};
use icloud_auth::DeveloperDeviceType;
use std::io::Write;
use std::process::Command;
use tauri::{Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

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
        println!(
            "{}, {}, {}",
            cert.machine_name, cert.name, cert.serial_number
        );
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
    let mut list_app_id_response = match account
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
    let is_sidestore = app.bundle.bundle_identifier().unwrap_or("") == "com.SideStore.SideStore";
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
            list_app_id_response
                .app_ids
                .iter()
                .any(|app_id| app_id.app_id_id == bundle_id)
        })
        .collect::<Vec<_>>();

    if app_ids_to_register.len() > list_app_id_response.available_quantity.try_into().unwrap() {
        return emit_error_and_return(
            &window,
            &format!(
                "This app requires {} app ids, but you only have {} available",
                app_ids_to_register.len(),
                list_app_id_response.available_quantity
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
    list_app_id_response = match account
        .list_app_ids(icloud_auth::DeveloperDeviceType::Ios, team)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to list app IDs: {:?}", e));
        }
    };

    let mut app_ids: Vec<_> = list_app_id_response
        .app_ids
        .into_iter()
        .filter(|app_id| {
            bundles_with_app_id
                .iter()
                .any(|bundle| app_id.identifier == bundle.bundle_identifier().unwrap_or(""))
        })
        .collect();
    let main_app_id = app_ids
        .iter()
        .find(|app_id| app_id.identifier == main_app_id_str)
        .cloned()
        .ok_or("Main app ID not found")?;

    window
        .emit("build-output", "Registered app IDs".to_string())
        .ok();

    for app_id in app_ids.iter_mut() {
        let app_group_feature_enabled = app_id
            .features
            .get(
                "APG3427HIY", /* Gotta love apple and their magic strings! */
            )
            .and_then(|v| v.as_boolean())
            .ok_or("App group feature not found in app id")?;
        if !app_group_feature_enabled {
            let mut body = plist::Dictionary::new();
            body.insert("APG3427HIY".to_string(), plist::Value::Boolean(true));
            let new_features = match account
                .update_app_id(DeveloperDeviceType::Ios, &team, &app_id, &body)
                .await
            {
                Ok(new_feats) => new_feats,
                Err(e) => {
                    return emit_error_and_return(
                        &window,
                        &format!("Failed to update app ID features: {:?}", e),
                    );
                }
            };
            app_id.features = new_features;
        }
    }

    let group_identifier = format!("group.{}", main_app_id_str);

    if is_sidestore {
        app.bundle.app_info.insert(
            "ALTAppGroups".to_string(),
            plist::Value::Array(vec![plist::Value::String(group_identifier.clone())]),
        );
    }

    let app_groups = match account
        .list_application_groups(DeveloperDeviceType::Ios, &team)
        .await
    {
        Ok(groups) => groups,
        Err(e) => {
            return emit_error_and_return(&window, &format!("Failed to list app groups: {:?}", e));
        }
    };

    let matching_app_groups = app_groups
        .iter()
        .filter(|group| group.identifier == group_identifier.clone())
        .collect::<Vec<_>>();

    let app_group = if matching_app_groups.is_empty() {
        match account
            .add_application_group(
                DeveloperDeviceType::Ios,
                &team,
                &group_identifier,
                &main_app_name,
            )
            .await
        {
            Ok(group) => group,
            Err(e) => {
                return emit_error_and_return(
                    &window,
                    &format!("Failed to register app group: {:?}", e),
                );
            }
        }
    } else {
        matching_app_groups[0].clone()
    };

    // let mut provisioning_profiles: HashMap<String, ProvisioningProfile> = HashMap::new();
    // for app_id in app_ids {
    //     let assign_res = account
    //         .assign_application_group_to_app_id(
    //             DeveloperDeviceType::Ios,
    //             &team,
    //             &app_id,
    //             &app_group,
    //         )
    //         .await;
    //     if assign_res.is_err() {
    //         return emit_error_and_return(
    //             &window,
    //             &format!(
    //                 "Failed to assign app group to app ID: {:?}",
    //                 assign_res.err()
    //             ),
    //         );
    //     }
    //     let provisioning_profile = match account
    //         // This doesn't seem right to me, but it's what Sideloader does... Shouldn't it be downloading the provisioning profile for this app ID, not the main?
    //         .download_team_provisioning_profile(DeveloperDeviceType::Ios, &team, &main_app_id)
    //         .await
    //     {
    //         Ok(pp /* tee hee */) => pp,
    //         Err(e) => {
    //             return emit_error_and_return(
    //                 &window,
    //                 &format!("Failed to download provisioning profile: {:?}", e),
    //             );
    //         }
    //     };
    //     provisioning_profiles.insert(app_id.identifier.clone(), provisioning_profile);
    // }

    // println!("Provisioning profiles:");
    // for (id, profile) in &provisioning_profiles {
    //     println!("{}: {}", id, profile.name);
    // }

    window
        .emit("build-output", "Registered app groups".to_string())
        .ok();

    let provisioning_profile = match account
        // This doesn't seem right to me, but it's what Sideloader does... Shouldn't it be downloading the provisioning profile for this app ID, not the main?
        .download_team_provisioning_profile(DeveloperDeviceType::Ios, &team, &main_app_id)
        .await
    {
        Ok(pp /* tee hee */) => pp,
        Err(e) => {
            return emit_error_and_return(
                &window,
                &format!("Failed to download provisioning profile: {:?}", e),
            );
        }
    };

    // write provisioning profile to disk
    let profile_path = std::path::PathBuf::from(&folder).join("dev.prov");
    let mut file = std::fs::File::create(&profile_path).map_err(|e| e.to_string())?;
    file.write_all(&provisioning_profile.encoded_profile)
        .map_err(|e| e.to_string())?;

    // TODO: Recursive for sub-bundles?
    app.bundle.write_info().map_err(|e| e.to_string())?;

    window
        .emit("build-output", "Signining app...".to_string())
        .ok();

    let zsign_command = handle.shell().sidecar("zsign").unwrap().args([
        "-k",
        cert.get_private_key_file_path().to_str().unwrap(),
        "-c",
        cert.get_certificate_file_path().to_str().unwrap(),
        "-m",
        profile_path.to_str().unwrap(),
        app.bundle.bundle_dir.to_str().unwrap(),
    ]);
    let (mut rx, mut _child) = zsign_command.spawn().expect("Failed to spawn zsign");

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) | CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    window
                        .emit("build-output", Some(line))
                        .expect("failed to emit event");
                }
                CommandEvent::Terminated(result) => {
                    if result.code != Some(0) {
                        window
                            .emit("build-output", "App signing failed!".to_string())
                            .ok();
                        return;
                    }
                    window.emit("build-output", "App signed!").ok();

                    window
                        .emit(
                            "build-output",
                            "Installing app (Transfer)... 0%".to_string(),
                        )
                        .ok();

                    let res = install_app(&device, &app.bundle.bundle_dir, |percentage| {
                        window
                            .emit("build-output", format!("Installing app... {}%", percentage))
                            .expect("failed to emit event");
                    })
                    .await;
                    if let Err(e) = res {
                        window
                            .emit("build-output", format!("Failed to install app: {:?}", e))
                            .ok();
                        return;
                    }
                    window
                        .emit("build-output", "App installed!".to_string())
                        .ok();
                }
                _ => {}
            }
        }
    });

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
