use std::process::Command;
use std::str;

fn parse_pacman_ss(output: &str) -> Vec<(String, String, String)> {
    let mut packages = Vec::new();
    let mut lines = output.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(' ') || line.is_empty() { continue; }
        let Some(slash) = line.find('/') else { continue };
        let repo = line[..slash].trim().to_string();
        let rest = &line[slash + 1..];
        let mut parts = rest.splitn(2, ' ');
        let name = parts.next().unwrap_or("").trim().to_string();
        let remainder = parts.next().unwrap_or("").trim();
        let mut ver_parts = remainder.splitn(2, ' ');
        let version = ver_parts.next().unwrap_or("").trim().to_string();
        if name.is_empty() || version.is_empty() { continue; }
        // consume description line
        if lines.peek().map(|l| l.starts_with("    ")).unwrap_or(false) {
            lines.next();
        }
        packages.push((repo, name, version));
    }
    packages
}

fn main() {
    let out = Command::new("pacman").args(["-Ss", "firefox"]).output().unwrap();
    let stdout = str::from_utf8(&out.stdout).unwrap();
    let pkgs = parse_pacman_ss(stdout);
    println!("status: {}", out.status);
    println!("repo results: {}", pkgs.len());
    for (repo, name, ver) in pkgs.iter().take(5) {
        println!("  [{repo}] {name} {ver}");
    }
}
