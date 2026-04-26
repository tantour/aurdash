use anyhow::Result;
use aur_scanner_core::{Scanner, ScanConfig, ScanResult, Severity};
use raur::Package;
use tempfile::NamedTempFile;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct SecurityScore {
    /// 0–100
    pub score: u8,
    /// Pkgbuild scan result
    pub scan_result: Option<ScanResult>,
    /// Human-readable breakdown
    pub breakdown: Vec<(String, i32)>,
}

impl SecurityScore {
    pub fn unknown() -> Self {
        Self {
            score: 50,
            scan_result: None,
            breakdown: vec![("Score pending...".to_string(), 0)],
        }
    }
}

/// Run aur-scanner-core on PKGBUILD text + compute trust signals from AUR metadata
pub async fn compute_security_score(
    pkgbuild_text: &str,
    pkg: &Package,
) -> Result<SecurityScore> {
    let mut breakdown: Vec<(String, i32)> = Vec::new();
    let mut score: i32 = 100;

    // ---- 1. Static PKGBUILD analysis via aur-scanner-core ----
    let scan_result = scan_pkgbuild_text(pkgbuild_text).await.ok();

    if let Some(ref result) = scan_result {
        let critical = result.findings_by_severity(Severity::Critical).len() as i32;
        let high = result.findings_by_severity(Severity::High).len() as i32;
        let medium = result.findings_by_severity(Severity::Medium).len() as i32;
        let low = result.findings_by_severity(Severity::Low).len() as i32;

        let critical_penalty = critical * 40;
        let high_penalty = high * 15;
        let medium_penalty = medium * 5;
        let low_penalty = low * 1;

        if critical > 0 {
            breakdown.push((format!("{} critical finding(s)", critical), -critical_penalty));
            score -= critical_penalty;
        }
        if high > 0 {
            breakdown.push((format!("{} high finding(s)", high), -high_penalty));
            score -= high_penalty;
        }
        if medium > 0 {
            breakdown.push((format!("{} medium finding(s)", medium), -medium_penalty));
            score -= medium_penalty;
        }
        if low > 0 {
            breakdown.push((format!("{} low finding(s)", low), -low_penalty));
            score -= low_penalty;
        }
        if result.findings.is_empty() {
            breakdown.push(("No security issues found".to_string(), 0));
        }
    } else {
        breakdown.push(("PKGBUILD scan unavailable".to_string(), -5));
        score -= 5;
    }

    // ---- 2. AUR metadata trust signals ----

    // Votes: >500 = great, 50-500 = ok, <10 = suspicious
    let vote_adj: i32 = if pkg.num_votes >= 500 {
        5
    } else if pkg.num_votes >= 100 {
        2
    } else if pkg.num_votes < 10 {
        -10
    } else {
        0
    };
    breakdown.push((format!("Votes: {}", pkg.num_votes), vote_adj));
    score += vote_adj;

    // Popularity
    let pop = pkg.popularity;
    let pop_adj: i32 = if pop > 5.0 {
        3
    } else if pop < 0.1 && pkg.num_votes < 5 {
        -5
    } else {
        0
    };
    if pop_adj != 0 {
        breakdown.push((format!("Popularity: {:.2}", pop), pop_adj));
        score += pop_adj;
    }

    // Out of date flag
    if pkg.out_of_date.is_some() {
        breakdown.push(("Marked out-of-date".to_string(), -8));
        score -= 8;
    }

    // Orphaned package (no maintainer)
    if pkg.maintainer.is_none() {
        breakdown.push(("Orphaned (no maintainer)".to_string(), -10));
        score -= 10;
    }

    // Last modified: if > 2 years ago, slight penalty
    let now = chrono::Utc::now().timestamp();
    let age_days = (now - pkg.last_modified) / 86400;
    if age_days > 730 {
        breakdown.push((format!("Not updated in {} days", age_days), -3));
        score -= 3;
    }

    let final_score = score.clamp(0, 100) as u8;

    Ok(SecurityScore {
        score: final_score,
        scan_result,
        breakdown,
    })
}

/// Write PKGBUILD to a temp file and scan it
async fn scan_pkgbuild_text(text: &str) -> Result<ScanResult> {
    // Write to temp file
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(text.as_bytes())?;
    tmp.flush()?;

    let config = ScanConfig {
        min_severity: Severity::Info,
        ..Default::default()
    };
    let scanner = Scanner::new(config)?;
    let result = scanner.scan_pkgbuild(tmp.path()).await?;
    Ok(result)
}
