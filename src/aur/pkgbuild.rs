use anyhow::Result;
use reqwest::Client;

/// Fetch raw PKGBUILD text from AUR cgit
pub async fn fetch_pkgbuild(client: &Client, pkg_name: &str) -> Result<String> {
    let url = format!(
        "https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h={}",
        pkg_name
    );
    let text = client
        .get(&url)
        .header("User-Agent", "aurdash/0.1")
        .send()
        .await?
        .text()
        .await?;
    Ok(text)
}
