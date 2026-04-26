pub mod comments;
pub mod pkgbuild;
pub mod repo;
pub mod search;

pub use repo::{RepoPackage, search_repos, fetch_repo_info};
pub use search::{AurPackage, AurSearcher};

/// A unified package entry — either from official repos or AUR
#[derive(Debug, Clone)]
pub enum PkgEntry {
    Repo(RepoPackage),
    Aur(AurPackage),
}

impl PkgEntry {
    pub fn name(&self) -> &str {
        match self {
            PkgEntry::Repo(p) => &p.name,
            PkgEntry::Aur(p) => &p.name,
        }
    }

    pub fn version(&self) -> &str {
        match self {
            PkgEntry::Repo(p) => &p.version,
            PkgEntry::Aur(p) => &p.version,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            PkgEntry::Repo(p) => Some(p.description.as_str()),
            PkgEntry::Aur(p) => p.description.as_deref(),
        }
    }

    pub fn is_repo(&self) -> bool {
        matches!(self, PkgEntry::Repo(_))
    }

    pub fn is_aur(&self) -> bool {
        matches!(self, PkgEntry::Aur(_))
    }

    pub fn repo_name(&self) -> Option<&str> {
        match self {
            PkgEntry::Repo(p) => Some(&p.repo),
            PkgEntry::Aur(_) => None,
        }
    }

    pub fn is_installed(&self) -> bool {
        match self {
            PkgEntry::Repo(p) => p.installed,
            PkgEntry::Aur(_) => false,
        }
    }
}
