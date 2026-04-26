use anyhow::Result;
use tokio::process::Command;

pub enum InstallStatus {
    Success,
    Failed(String),
    NotFound,
}

pub async fn install_package(pkg_name: &str) -> Result<(bool, String)> {
    let output = Command::new("paru")
        .args(["--noconfirm", "-S", pkg_name])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    Ok((success, format!("{}{}", stdout, stderr)))
}

pub async fn uninstall_package(pkg_name: &str, recursive: bool) -> Result<(bool, String)> {
    let mut args = vec!["--noconfirm", "-R"];
    if recursive {
        args = vec!["--noconfirm", "-Rs"];
    }
    args.push(pkg_name);

    let output = Command::new("paru")
        .args(args)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    Ok((success, format!("{}{}", stdout, stderr)))
}

pub async fn paru_available() -> bool {
    Command::new("paru")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub async fn list_installed() -> Result<Vec<String>> {
    let output = Command::new("paru")
        .args(["-Qm"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(|l| l.split_whitespace().next().map(String::from)).collect())
}

pub async fn list_upgrades() -> Result<Vec<String>> {
    let output = Command::new("paru")
        .args(["-Qua"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(|l| l.split_whitespace().next().map(String::from)).collect())
}
