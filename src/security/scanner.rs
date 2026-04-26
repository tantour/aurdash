/// Hand-rolled PKGBUILD security scanner.
///
/// Designed specifically to minimize false positives on legitimate AUR packages.
/// Only flags patterns that are genuinely suspicious with high confidence.
use anyhow::Result;
use raur::Package;
use regex::Regex;
use std::sync::OnceLock;

// ── Rule severity ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
    /// Penalty applied to score
    pub fn penalty(&self) -> i32 {
        match self {
            Severity::Critical => 45,
            Severity::High => 20,
            Severity::Medium => 8,
            Severity::Low => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub id: &'static str,
    pub severity: Severity,
    pub title: &'static str,
    pub detail: String, // context snippet
}

// ── Compiled patterns (lazy) ───────────────────────────────────────────────

fn re(pat: &str) -> Regex {
    Regex::new(pat).unwrap()
}

struct Patterns {
    // CRITICAL — very high confidence malicious
    curl_pipe_shell: Regex,      // curl ... | bash/sh
    wget_pipe_shell: Regex,      // wget ... | bash/sh
    curl_exec: Regex,            // curl ... -o ... && bash/chmod
    reverse_shell_tcp: Regex,    // /dev/tcp/
    reverse_shell_nc: Regex,     // nc -e /bin/
    python_reverse: Regex,       // socket.connect( ... os.dup2
    mining_pool: Regex,          // stratum+tcp://
    miner_binary: Regex,         // xmrig/minerd/cpuminer
    paste_download: Regex,       // pastebin.com/raw, ptpb.pw
    exfil_webhook: Regex,        // discord.com/api/webhooks, t.me/
    ld_preload_set: Regex,       // export LD_PRELOAD=

    // HIGH — suspicious but could occasionally be legitimate
    base64_exec: Regex,          // base64 -d | bash
    eval_decode: Regex,          // eval $(...)  or eval `...`
    sudoers_write: Regex,        // echo ... >> /etc/sudoers
    cred_access: Regex,          // ~/.ssh/id_rsa, /etc/shadow, ~/.gnupg
    browser_db: Regex,           // logins.json, cookies.sqlite, Login Data
    shell_config_write: Regex,   // >> ~/.bashrc, >> ~/.zshrc, >> ~/.profile

    // MEDIUM — informational / worth noting but common in legit packages
    suid_set: Regex,             // chmod 4xxx / chmod u+s  (but NOT 755, 644, etc.)
    url_shortener: Regex,        // bit.ly, tinyurl.com etc in source=()
    raw_ip_url: Regex,           // source from raw IP
    no_checksum_http: Regex,     // http:// in source= (downgrade attack)
    cron_create: Regex,          // crontab -, /etc/cron.
    persistence_autostart: Regex,// ~/.config/autostart/
}

static PATTERNS: OnceLock<Patterns> = OnceLock::new();

fn patterns() -> &'static Patterns {
    PATTERNS.get_or_init(|| Patterns {
        // CRITICAL
        curl_pipe_shell: re(r"curl\b[^|#\n]*\|\s*(ba)?sh\b"),
        wget_pipe_shell: re(r"wget\b[^|#\n]*\|\s*(ba)?sh\b"),
        curl_exec: re(r"curl\b[^;\n]+&&\s*(ba)?sh\b"),
        reverse_shell_tcp: re(r"/dev/tcp/"),
        reverse_shell_nc: re(r"\bnc\b[^#\n]+-e\s+/bin/(ba)?sh"),
        python_reverse: re(r"socket\.connect\(.*\bos\.dup2\b"),
        mining_pool: re(r"stratum\+tcp://"),
        miner_binary: re(r"\b(xmrig|minerd|cpuminer|nbminer|lolminer|t-rex-miner)\b"),
        paste_download: re(r"https?://(pastebin\.com/raw|ptpb\.pw|paste\.ee/r|0x0\.st|hastebin\.com/raw)"),
        exfil_webhook: re(r"https?://(discord\.com/api/webhooks|t\.me/|api\.telegram\.org/bot)"),
        ld_preload_set: re(r"\bexport\s+LD_PRELOAD="),

        // HIGH
        base64_exec: re(r"base64\s+-d\b[^|#\n]*\|\s*(ba)?sh\b"),
        eval_decode: re(r"\beval\s+[`$]\("),
        // sudoers modification: echo/tee writing to /etc/sudoers
        sudoers_write: re(r"(>>?\s*/etc/sudoers|tee\s+/etc/sudoers)"),
        // credential file access — only flag actual read/exfil, not path in comments
        cred_access: re(r"\bcat\b[^#\n]*(~/.ssh/id_|/etc/shadow|~/.gnupg/secring)"),
        browser_db: re(r"\bcp\b[^#\n]*(logins\.json|cookies\.sqlite|Login Data)"),
        shell_config_write: re(r">>\s*~/\.(bashrc|zshrc|profile|bash_profile|zprofile)\b"),

        // MEDIUM
        // Real SUID: chmod 4xxx or chmod +s or chmod u+s
        // Explicitly exclude: 755, 644, 777, 750 etc which are very common
        suid_set: re(r"chmod\s+[0-7]*[4][0-7]{3}\b|chmod\s+[ug]\+s\b"),
        url_shortener: re(r"https?://(bit\.ly|tinyurl\.com|is\.gd|cli\.gs|t\.co|ow\.ly)/"),
        raw_ip_url: re(r#"source=\([^)]*"https?://\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#),
        no_checksum_http: re(r#"source=\([^)]*"http://"#),
        cron_create: re(r"(crontab\s+-|>\s*/etc/cron\.(d|daily|hourly|weekly|monthly)/)"),
        persistence_autostart: re(r"~/\.config/autostart/"),
    })
}

// ── Scanner ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SecurityScore {
    pub score: u8,
    pub findings: Vec<Finding>,
    pub breakdown: Vec<(String, i32)>,
}

impl SecurityScore {
    pub fn unknown() -> Self {
        Self {
            score: 50,
            findings: vec![],
            breakdown: vec![("Scan pending".to_string(), 0)],
        }
    }
}

/// Scan a PKGBUILD text string and AUR metadata → produce a SecurityScore.
pub async fn compute_security_score(pkgbuild: &str, pkg: &Package) -> Result<SecurityScore> {
    let mut findings = Vec::new();
    let mut breakdown: Vec<(String, i32)> = Vec::new();
    let mut score: i32 = 100;

    // ── Static PKGBUILD scan ───────────────────────────────────────────────
    scan_pkgbuild(pkgbuild, &mut findings);

    // Aggregate penalties per severity, cap at 100 total deduction
    let mut penalty = 0i32;
    for f in &findings {
        penalty += f.severity.penalty();
    }
    penalty = penalty.min(100);

    if findings.is_empty() {
        breakdown.push(("No security issues in PKGBUILD".to_string(), 0));
    } else {
        // Group by severity for the breakdown display
        let critical: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Critical).collect();
        let high: Vec<_> = findings.iter().filter(|f| f.severity == Severity::High).collect();
        let medium: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Medium).collect();
        let low: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Low).collect();

        for (items, sev) in [
            (&critical, Severity::Critical),
            (&high, Severity::High),
            (&medium, Severity::Medium),
            (&low, Severity::Low),
        ] {
            if !items.is_empty() {
                let p = items.iter().map(|f| f.severity.penalty()).sum::<i32>().min(100);
                breakdown.push((
                    format!("{} {} finding(s)", items.len(), sev.label()),
                    -p,
                ));
            }
        }
        score -= penalty;
    }

    // ── AUR metadata trust signals ─────────────────────────────────────────

    // Votes
    let vote_adj: i32 = if pkg.num_votes >= 500 {
        5
    } else if pkg.num_votes >= 100 {
        2
    } else if pkg.num_votes < 5 && pkg.num_votes > 0 {
        -8
    } else if pkg.num_votes == 0 {
        -5
    } else {
        0
    };
    if vote_adj != 0 {
        breakdown.push((format!("Votes: {}", pkg.num_votes), vote_adj));
        score += vote_adj;
    } else {
        breakdown.push((format!("Votes: {}", pkg.num_votes), 0));
    }

    // Out of date
    if pkg.out_of_date.is_some() {
        breakdown.push(("Marked out-of-date".to_string(), -8));
        score -= 8;
    }

    // Orphaned
    if pkg.maintainer.is_none() {
        breakdown.push(("Orphaned (no maintainer)".to_string(), -10));
        score -= 10;
    }

    // Last modified age
    let now = chrono::Utc::now().timestamp();
    let age_days = (now - pkg.last_modified) / 86400;
    if age_days > 730 {
        breakdown.push((format!("Not updated in {}d", age_days), -3));
        score -= 3;
    }

    let final_score = score.clamp(0, 100) as u8;

    Ok(SecurityScore {
        score: final_score,
        findings,
        breakdown,
    })
}

/// Scan PKGBUILD text and push findings into the vec.
fn scan_pkgbuild(text: &str, findings: &mut Vec<Finding>) {
    let p = patterns();

    // Strip comment lines so we don't flag commented-out examples
    let stripped = strip_comments(text);
    let s = &stripped;

    check(s, &p.curl_pipe_shell,    findings, "MALW-001", Severity::Critical, "curl | shell execution",      text);
    check(s, &p.wget_pipe_shell,    findings, "MALW-002", Severity::Critical, "wget | shell execution",      text);
    check(s, &p.curl_exec,          findings, "MALW-003", Severity::Critical, "curl download + exec",        text);
    check(s, &p.reverse_shell_tcp,  findings, "RSHELL-001", Severity::Critical, "Bash TCP reverse shell",   text);
    check(s, &p.reverse_shell_nc,   findings, "RSHELL-002", Severity::Critical, "Netcat reverse shell",     text);
    check(s, &p.python_reverse,     findings, "RSHELL-003", Severity::Critical, "Python reverse shell",     text);
    check(s, &p.mining_pool,        findings, "MINE-001",  Severity::Critical, "Mining pool connection",     text);
    check(s, &p.miner_binary,       findings, "MINE-002",  Severity::Critical, "Cryptominer binary",         text);
    check(s, &p.paste_download,     findings, "PASTE-001", Severity::Critical, "Download from paste site",   text);
    check(s, &p.exfil_webhook,      findings, "EXFIL-001", Severity::Critical, "Discord/Telegram exfil",    text);
    check(s, &p.ld_preload_set,     findings, "PRIV-001",  Severity::Critical, "LD_PRELOAD injection",       text);

    check(s, &p.base64_exec,        findings, "OBF-001",   Severity::High,    "base64 decode + exec",       text);
    check(s, &p.eval_decode,        findings, "OBF-002",   Severity::High,    "eval with subshell",         text);
    check(s, &p.sudoers_write,      findings, "PRIV-002",  Severity::High,    "sudoers file modification",   text);
    check(s, &p.cred_access,        findings, "CRED-001",  Severity::High,    "Credential file access",     text);
    check(s, &p.browser_db,         findings, "CRED-002",  Severity::High,    "Browser credential theft",   text);
    check(s, &p.shell_config_write, findings, "PERSIST-001", Severity::High,  "Shell config modification",  text);

    check(s, &p.suid_set,           findings, "PRIV-003",  Severity::Medium,  "SUID/SGID bit set",          text);
    check(s, &p.url_shortener,      findings, "SRC-001",   Severity::Medium,  "URL shortener in source",    text);
    check(s, &p.raw_ip_url,         findings, "SRC-002",   Severity::Medium,  "Raw IP in source URL",       text);
    check(s, &p.no_checksum_http,   findings, "SRC-003",   Severity::Medium,  "HTTP source (no TLS)",       text);
    check(s, &p.cron_create,        findings, "PERSIST-002", Severity::Medium, "Cron job creation",         text);
    check(s, &p.persistence_autostart, findings, "PERSIST-003", Severity::Medium, "Autostart entry",        text);
}

/// Run a regex against stripped content, push a finding with an excerpt from the *original* text.
fn check(
    stripped: &str,
    re: &Regex,
    findings: &mut Vec<Finding>,
    id: &'static str,
    severity: Severity,
    title: &'static str,
    original: &str,
) {
    if let Some(m) = re.find(stripped) {
        // Get the matching line from the original text for context
        let line_no = stripped[..m.start()].chars().filter(|&c| c == '\n').count();
        let detail = original
            .lines()
            .nth(line_no)
            .map(|l| l.trim().to_string())
            .unwrap_or_default();

        findings.push(Finding {
            id,
            severity,
            title,
            detail,
        });
    }
}

/// Remove comment lines (lines where first non-whitespace char is `#`)
/// Also remove inline comments — anything after ` #` on a line.
fn strip_comments(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                return String::new();
            }
            // Remove inline # comments (crude but effective for PKGBUILD)
            // Only strip if there's whitespace before the #
            if let Some(pos) = line.find(" #") {
                line[..pos].to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
