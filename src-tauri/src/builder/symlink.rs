// on linux only import unix fs
#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;

#[cfg(target_os = "windows")]
use crate::windows::has_wsl;
#[cfg(target_os = "windows")]
use std::process::{Command, Stdio};

pub fn symlink(target: &str, link: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        symlink(target, link)?;
    }
    #[cfg(target_os = "windows")]
    {
        if !has_wsl() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "WSL is not available",
            ));
        }
        let output = Command::new("wsl")
            .arg("ln")
            .arg("-s")
            .arg(target)
            .arg(link)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("failed to execute process");
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create symlink: {}", String::from_utf8_lossy(&output.stderr)),
            ));
        }
    }
    Ok(())
}