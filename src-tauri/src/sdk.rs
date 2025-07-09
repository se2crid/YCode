// Reference: https://github.com/xtool-org/xtool/blob/main/Sources/XToolSupport/SDKBuilder.swift
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

const DARWIN_TOOLS_VERSION: &str = "1.0.1";

#[tauri::command]
pub async fn install_sdk(
    app: AppHandle,
    xcode_path: String,
    toolchain_path: String,
) -> Result<(), String> {
    if xcode_path.is_empty() {
        return Err("Xcode not found".to_string());
    }
    if toolchain_path.is_empty() {
        return Err("Toolchain not found".to_string());
    }
    let output_dir = std::env::temp_dir()
        .join("DarwinSDKBuild")
        .join("darwin.artifactbundle");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .map_err(|e| format!("Failed to remove existing output directory: {}", e))?;
    }
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    install_toolset(&output_dir).await?;

    let dev = install_developer(&app, &output_dir, &xcode_path).await?;

    let iphone_os_sdk = sdk(&dev, "iPhoneOS")?;
    let mac_os_sdk = sdk(&dev, "MacOSX")?;
    let iphone_simulator_sdk = sdk(&dev, "iPhoneSimulator")?;

    let info = "
        {
            \"schemaVersion\": \"1.0\",
            \"artifacts\": {
                \"darwin\": {
                    \"type\": \"swiftSDK\",
                    \"version\": \"0.0.1\",
                    \"variants\": [
                        {
                            \"path\": \".\",
                            \"supportedTriples\": [\"aarch64-unknown-linux-gnu\", \"x86_64-unknown-linux-gnu\"]
                        }
                    ]
                }
            }
        }
        ";
    fs::write(output_dir.join("info.json"), info)
        .map_err(|e| format!("Failed to write info.json: {}", e))?;

    let toolset = "
        {
            \"schemaVersion\": \"1.0\",
            \"rootPath\": \"toolset/bin\",
            \"linker\": {
                \"path\": \"ld64.lld\"
            },
            \"swiftCompiler\": {
                \"extraCLIOptions\": [
                    \"-use-ld=lld\"
                ]
            }
        }
        ";
    fs::write(output_dir.join("toolset.json"), toolset)
        .map_err(|e| format!("Failed to write toolset.json: {}", e))?;

    Ok(())
}

fn sdk(dev: &PathBuf, platform: &str) -> Result<String, String> {
    let dir = dev.join(format!("Platforms/{}.platform/Developer/SDKs", platform));
    let regex = Regex::new(&format!(
        r"^{}[0-9]+\.[0-9]+\.sdk$",
        regex::escape(platform)
    ))
    .map_err(|e| format!("Invalid regex: {}", e))?;

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read SDKs directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if regex.is_match(&name_str) {
            return Ok(name_str.into_owned());
        }
    }

    Err(format!("Could not find SDK for {}/{}", platform, platform))
}

async fn install_toolset(output_path: &PathBuf) -> Result<(), String> {
    let toolset_dir = output_path.join("toolset");
    fs::create_dir_all(&toolset_dir)
        .map_err(|e| format!("Failed to create toolset directory: {}", e))?;

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err("Unsupported architecture".to_string());
    };
    let toolset_url = format!(
        "https://github.com/xtool-org/darwin-tools-linux-llvm/releases/download/v{}/toolset-{}.tar.gz",
        DARWIN_TOOLS_VERSION, arch
    );

    let response = reqwest::get(&toolset_url)
        .await
        .map_err(|e| format!("Failed to download toolset: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Failed to download toolset: {}", response.status()));
    }
    let tar_gz = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&*tar_gz));
    archive
        .unpack(&toolset_dir)
        .map_err(|e| format!("Failed to extract toolset: {}", e))?;

    Ok(())
}

async fn install_developer(
    app: &AppHandle,
    output_path: &PathBuf,
    xcode_path: &str,
) -> Result<PathBuf, String> {
    let dev_stage = output_path.join("DeveloperStage");
    fs::create_dir_all(&dev_stage)
        .map_err(|e| format!("Failed to create DeveloperStage directory: {}", e))?;

    let unxip_path = app
        .path()
        .resolve("unxip", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve unxip path: {}", e))?;

    let status = Command::new(unxip_path)
        .current_dir(&dev_stage)
        .arg(xcode_path)
        .status();
    if let Err(e) = status {
        return Err(format!("Failed to run unxip: {}", e));
    }
    if !status.unwrap().success() {
        return Err("Failed to unxip Xcode".to_string());
    }

    let app_dirs = fs::read_dir(&dev_stage)
        .map_err(|e| format!("Failed to read DeveloperStage directory: {}", e))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "app"))
        .collect::<Vec<_>>();
    if app_dirs.len() != 1 {
        return Err(format!(
            "Expected one .app in DeveloperStage, found {}",
            app_dirs.len()
        ));
    }

    let app_path = app_dirs[0].path();
    let dev = output_path.join("Developer");
    fs::create_dir_all(&dev).map_err(|e| format!("Failed to create Developer directory: {}", e))?;

    let contents_developer = app_path.join("Contents/Developer");
    if !contents_developer.exists() {
        return Err("Contents/Developer not found in .app".to_string());
    }
    copy_developer(&contents_developer, &dev, Path::new("Contents/Developer"))
        .map_err(|e| format!("Failed to copy Developer: {}", e))?;
    fs::remove_dir_all(&dev_stage)
        .map_err(|e| format!("Failed to remove DeveloperStage directory: {}", e))?;

    for platform in ["iPhoneOS", "MacOSX", "iPhoneSimulator"] {
        let lib = "../../../../../Library";
        let dest = dev.join(format!(
            "Platforms/{}.platform/Developer/SDKs/{}.sdk/System/Library/Frameworks",
            platform, platform
        ));

        let links = [
            (
                "Testing.framework",
                format!("{}/Frameworks/Testing.framework", lib),
            ),
            (
                "XCTest.framework",
                format!("{}/Frameworks/XCTest.framework", lib),
            ),
            (
                "XCUIAutomation.framework",
                format!("{}/Frameworks/XCUIAutomation.framework", lib),
            ),
            (
                "XCTestCore.framework",
                format!("{}/PrivateFrameworks/XCTestCore.framework", lib),
            ),
        ];

        for (name, target) in &links {
            let link_path = dest.join(name);
            symlink(target, &link_path).map_err(|e| {
                format!(
                    "Failed to create symlink {:?} -> {:?}: {}",
                    link_path, target, e
                )
            })?;
        }
    }

    Ok(dev)
}

fn copy_developer(src: &Path, dst: &Path, rel: &Path) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_name = entry.file_name();
        let rel_path = rel.join(&file_name);
        if !is_wanted(&rel_path) {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&rel_path);
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        if file_type.is_dir() {
            fs::create_dir_all(&dst_path).map_err(|e| format!("Failed to create dir: {}", e))?;
            copy_developer(&src_path, dst, &rel_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            fs::copy(&src_path, &dst_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Triple {
    sdk_root_path: String,
    include_search_paths: Vec<String>,
    library_search_paths: Vec<String>,
    swift_resources_path: String,
    swift_static_resources_path: String,
    toolset_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct SDKDefinition {
    schema_version: String,
    target_triples: HashMap<String, Triple>,
}

#[derive(Debug, Clone)]
struct SDKEntry {
    names: HashSet<String>,
    values: Vec<SDKEntry>,
}

impl SDKEntry {
    // empty = wildcard
    fn new(names: HashSet<String>, values: Vec<SDKEntry>) -> Self {
        SDKEntry { names, values }
    }

    fn from_name(name: &str, values: Vec<SDKEntry>) -> Self {
        let mut set = HashSet::new();
        set.insert(name.to_string());
        SDKEntry::new(set, values)
    }

    fn matches<'a, I>(&self, path: I) -> bool
    where
        I: Iterator<Item = &'a str> + Clone,
    {
        let mut path_clone = path.clone();
        let first = path_clone.next();
        if first.is_none() {
            return true;
        }
        let first = first.unwrap();
        if !self.names.is_empty() && !self.names.contains(first) {
            return false;
        }
        if self.values.is_empty() {
            return true;
        }
        let after_name = path_clone;
        for value in &self.values {
            if value.matches(after_name.clone()) {
                return true;
            }
        }
        false
    }

    fn e(name: Option<&str>, values: Vec<SDKEntry>) -> SDKEntry {
        if let Some(name) = name {
            let parts: Vec<&str> = name.split('/').collect();
            let mut entry = SDKEntry::from_name(parts.last().unwrap(), values);
            for part in parts.iter().rev().skip(1) {
                entry = SDKEntry::from_name(part, vec![entry]);
            }
            entry
        } else {
            SDKEntry::new(HashSet::new(), values)
        }
    }
}

// Build the wanted tree
fn wanted_sdk_entry() -> SDKEntry {
    SDKEntry::e(
        Some("Contents/Developer"),
        vec![
            SDKEntry::e(
                Some("Toolchains/XcodeDefault.xctoolchain/usr/lib"),
                vec![
                    SDKEntry::e(Some("swift"), vec![]),
                    SDKEntry::e(Some("swift_static"), vec![]),
                    SDKEntry::e(Some("clang"), vec![]),
                ],
            ),
            SDKEntry::e(
                Some("Platforms"),
                ["iPhoneOS", "MacOSX", "iPhoneSimulator"]
                    .iter()
                    .map(|plat| {
                        SDKEntry::e(
                            Some(&format!("{}.platform/Developer", plat)),
                            vec![
                                SDKEntry::e(Some("SDKs"), vec![]),
                                SDKEntry::e(
                                    Some("Library"),
                                    vec![
                                        SDKEntry::e(Some("Frameworks"), vec![]),
                                        SDKEntry::e(Some("PrivateFrameworks"), vec![]),
                                    ],
                                ),
                                SDKEntry::e(Some("usr/lib"), vec![]),
                            ],
                        )
                    })
                    .collect(),
            ),
        ],
    )
}

fn is_wanted(path: &Path) -> bool {
    let mut components: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(os) => Some(os.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    if let Some(first) = components.first() {
        if first == "." {
            components.remove(0);
        }
    }
    if let Some(first) = components.first() {
        if first.ends_with(".app") {
            components.remove(0);
        }
    }

    if !wanted_sdk_entry().matches(components.iter().map(|s| s.as_str())) {
        return false;
    }

    if components.len() >= 10
        && components[9] == "prebuilt-modules"
        && components.starts_with(
            &[
                "Contents",
                "Developer",
                "Toolchains",
                "XcodeDefault.xctoolchain",
                "usr",
                "lib",
                "swift",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        )
    {
        return false;
    }

    true
}
