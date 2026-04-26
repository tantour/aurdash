use anyhow::Result;
use tokio::process::Command;

pub enum InstallStatus {
    Success,
    Failed(String),
    NotFound,
}

/// Invoke paru to install a package. Returns the combined output.
pub async fn install_package(pkg_name: &str) -> Result<(bool, String)> {
    let output = Command::new("paru")
        .args(["--noconfirm", "-S", pkg_name])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}{}", stdout, stderr);
    let success = output.status.success();
    Ok((success, combined))
}

/// Check if paru is installed
pub async fn paru_available() -> bool {
    Command::new("paru")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get list of installed AUR packages via paru
pub async fn list_installed() -> Result<Vec<String>> {
    let output = Command::new("paru")
        .args(["-Qm"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pkgs = stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next().map(String::from))
        .collect();
    Ok(pkgs)
}

/// Get available upgrades via paru
pub async fn list_upgrades() -> Result<Vec<String>> {
    let output = Command::new("paru")
        .args(["-Qua"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pkgs = stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next().map(String::from))
        .collect();
    Ok(pkgs)
}
