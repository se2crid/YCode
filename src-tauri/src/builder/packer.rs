use std::{fs, path::PathBuf};

use dircpy::CopyBuilder;

use crate::builder::config::{BuildSettings, ProjectConfig};

pub fn pack(
    project_path: PathBuf,
    config: &ProjectConfig,
    build_settings: &BuildSettings,
) -> Result<PathBuf, String> {
    let workdir = project_path.join(".ycode");
    if !workdir.exists() {
        std::fs::create_dir_all(&workdir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;
    }
    let app_path = workdir.join(format!("{}.app", config.product));
    if app_path.exists() {
        std::fs::remove_dir_all(&app_path)
            .map_err(|e| format!("Failed to remove existing app directory: {}", e))?;
    }
    std::fs::create_dir_all(&app_path)
        .map_err(|e| format!("Failed to create app directory: {}", e))?;

    let exec = project_path
        .join(".build")
        .join("arm64-apple-ios")
        .join(if build_settings.debug {
            "debug"
        } else {
            "release"
        })
        .join(&config.product);

    if !exec.exists() {
        return Err(format!("Executable not found at: {}", exec.display()));
    }

    fs::copy(exec, app_path.join(&config.product))
        .map_err(|e| format!("Failed to copy executable: {}", e))?;

    // TODO: Create default Info.plist if it doesn't exist
    let info_plist = project_path.join("Info.plist");
    if !info_plist.exists() {
        return Err(format!("Info.plist not found at: {}", info_plist.display()));
    }

    let info_content = fs::read_to_string(&info_plist)
        .map_err(|e| format!("Failed to read Info.plist: {}", e))?
        .replace("[[bundle_id]]", &config.bundle_id)
        .replace("[[product]]", &config.product)
        .replace("[[version_num]]", &config.version_num)
        .replace("[[version_string]]", &config.version_string);
    fs::write(&app_path.join("Info.plist"), info_content)
        .map_err(|e| format!("Failed to write Info.plist: {}", e))?;

    let resources = project_path.join("Resources");

    if !resources.exists() {
        std::fs::create_dir_all(&resources)
            .map_err(|e| format!("Failed to create Resources directory: {}", e))?;
    }

    CopyBuilder::new(&resources, &app_path)
        .run()
        .map_err(|e| format!("Failed to copy resources: {}", e))?;

    Ok(app_path)
}
