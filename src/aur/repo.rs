use anyhow::Result;
use tokio::process::Command;

/// A package from official Arch repos (from `pacman -Ss`)
#[derive(Debug, Clone)]
pub struct RepoPackage {
    pub repo: String,       // e.g. "core", "extra", "multilib"
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
    // Extended info from `pacman -Si` (fetched on select)
    pub url: Option<String>,
    pub licenses: Vec<String>,
    pub depends: Vec<String>,
    pub install_size: Option<String>,
}

/// Search official repos via pacman -Ss
pub async fn search_repos(query: &str) -> Result<Vec<RepoPackage>> {
    let output = Command::new("pacman")
        .env("LC_ALL", "C")
        .args(["-Ss", query])
        .output()
        .await?;

    if !output.status.success() && output.stdout.is_empty() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_pacman_ss(&stdout))
}

/// Parse `pacman -Ss` output.
/// Format:
///   repo/name version [flags]
///       description text
fn parse_pacman_ss(output: &str) -> Vec<RepoPackage> {
    let mut packages = Vec::new();
    let mut lines = output.lines().peekable();

    while let Some(line) = lines.next() {
        // Header line: "repo/name version [installed]"
        if line.starts_with(' ') || line.is_empty() {
            continue;
        }
        // Split on '/'
        let Some(slash) = line.find('/') else { continue };
        let repo = line[..slash].trim().to_string();
        let rest = &line[slash + 1..];

        // Split name and version on first space
        let mut parts = rest.splitn(2, ' ');
        let name = parts.next().unwrap_or("").trim().to_string();
        let remainder = parts.next().unwrap_or("").trim();

        // Version is first token; rest may contain [installed], [local], etc.
        let mut ver_parts = remainder.splitn(2, ' ');
        let version = ver_parts.next().unwrap_or("").trim().to_string();
        let flags = ver_parts.next().unwrap_or("");
        let installed = flags.contains("[installed]") || line.contains("[installed]");

        // Description is the next indented line
        let description = if lines.peek().map(|l| l.starts_with("    ") || l.starts_with('\t')).unwrap_or(false) {
            lines.next().unwrap_or("").trim().to_string()
        } else {
            String::new()
        };

        if name.is_empty() || version.is_empty() {
            continue;
        }

        packages.push(RepoPackage {
            repo,
            name,
            version,
            description,
            installed,
            url: None,
            licenses: Vec::new(),
            depends: Vec::new(),
            install_size: None,
        });
    }

    packages
}

/// Fetch extended info for a repo package via `pacman -Si`
pub async fn fetch_repo_info(pkg_name: &str) -> Result<RepoPackage> {
    let output = Command::new("pacman")
        .env("LC_ALL", "C")
        .args(["-Si", pkg_name])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    parse_pacman_si(&stdout, pkg_name)
}

fn parse_pacman_si(output: &str, fallback_name: &str) -> Result<RepoPackage> {
    let mut pkg = RepoPackage {
        repo: String::new(),
        name: fallback_name.to_string(),
        version: String::new(),
        description: String::new(),
        installed: false,
        url: None,
        licenses: Vec::new(),
        depends: Vec::new(),
        install_size: None,
    };

    for line in output.lines() {
        let Some(colon) = line.find(':') else { continue };
        let key = line[..colon].trim();
        let val = line[colon + 1..].trim().to_string();

        match key {
            "Repository" => pkg.repo = val,
            "Name" => pkg.name = val,
            "Version" => pkg.version = val,
            "Description" => pkg.description = val,
            "URL" => pkg.url = Some(val),
            "Licenses" => {
                pkg.licenses = val.split_whitespace().map(String::from).collect()
            }
            "Depends On" => {
                if val != "None" {
                    pkg.depends = val.split_whitespace().map(String::from).collect();
                }
            }
            "Installed Size" => pkg.install_size = Some(val),
            _ => {}
        }
    }

    Ok(pkg)
}
