use keyring::{Entry, Error as KeyringError};
use tauri::Manager;

use crate::sideloader::apple::APPLE_ACCOUNT;

pub fn store_credentials(email: &str, password: &str) -> Result<(), KeyringError> {
    let email_entry = Entry::new("y-code", "apple_id_email")?;
    email_entry.set_password(email)?;
    let pass_entry = Entry::new("y-code", email)?;
    pass_entry.set_password(password)
}

pub fn get_stored_credentials() -> Option<(String, String)> {
    let email_entry = Entry::new("y-code", "apple_id_email").ok()?;
    let email = email_entry.get_password().ok()?;
    let pass_entry = Entry::new("y-code", &email).ok()?;
    let password = pass_entry.get_password().ok()?;
    Some((email, password))
}

#[tauri::command]
pub fn get_apple_email() -> String {
    let credentials = get_stored_credentials();
    if credentials.is_none() {
        return "".to_string();
    }
    let (email, _) = credentials.unwrap();
    email
}

#[tauri::command]
pub fn delete_stored_credentials() -> Result<(), String> {
    let email_entry =
        Entry::new("y-code", "apple_id_email").map_err(|e| format!("Keyring error: {:?}", e))?;
    let email = match email_entry.get_password() {
        Ok(email) => email,
        Err(_) => {
            return Ok(());
        }
    };
    let pass_entry = Entry::new("y-code", &email).map_err(|e| format!("Keyring error: {:?}", e))?;

    let _ = pass_entry.delete_password();
    email_entry
        .delete_password()
        .map_err(|e| format!("Keyring error: {:?}", e))?;

    if let Some(account) = APPLE_ACCOUNT.get() {
        let mut account_guard = account.lock().unwrap();
        *account_guard = None;
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
