use std::collections::HashMap;

use dircpy::CopyBuilder;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn create_template(
    app: AppHandle,
    template: String,
    name: String,
    parameters: HashMap<String, String>,
) -> Result<String, String> {
    let template_dir = app
        .path()
        .resolve("templates", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve template directory: {}", e))?;
    let template_path = template_dir.join(&template);
    if !template_path.exists() {
        return Err(format!("Template '{}' does not exist", template));
    }
    let file_path = app
        .dialog()
        .file()
        .set_title("Project Location")
        .blocking_pick_folder();
    if file_path.is_none() {
        return Err("No folder selected".to_string());
    }
    let file_path = file_path.unwrap();
    let target_path = file_path.as_path().unwrap().join(&name);
    if target_path.exists() {
        return Err(format!(
            "Target path '{}' already exists",
            target_path.display()
        ));
    }
    std::fs::create_dir_all(&target_path)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    if !template_path.is_dir() {
        return Err(format!(
            "Template path '{}' is not a directory",
            template_path.display()
        ));
    }

    CopyBuilder::new(&template_path, &target_path)
        .run()
        .map_err(|e| format!("Failed to copy template: {}", e))?;

    let walker = walkdir::WalkDir::new(&target_path)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        let path = entry.path();
        if path.is_file() {
            let mut content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;
            for (key, value) in &parameters {
                content = content.replace(&format!("{{{{{}}}}}", key), value);
            }
            std::fs::write(&path, content)
                .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))?;
        }
    }

    Ok(target_path.to_string_lossy().to_string())
}
