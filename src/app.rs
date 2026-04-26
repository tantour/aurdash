use crate::aur::{PkgEntry, comments::AurComment};
use crate::security::SecurityScore;
use tui_input::Input;

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Search,
    Results,
    Detail,
    Comments,
    Pkgbuild,
    InstallLog,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState {
    Idle,
    Loading,
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Combined search results (repo + AUR merged)
    SearchResults(Vec<PkgEntry>),
    /// Security score computed
    SecurityScore(String, SecurityScore),
    /// Comments fetched
    Comments(String, Vec<AurComment>),
    /// PKGBUILD text fetched
    Pkgbuild(String, String),
    /// Repo package extended info fetched
    RepoInfo(String, crate::aur::RepoPackage),
    /// Install finished
    InstallDone(bool, String),
    /// Paru availability check
    ParuAvailable(bool),
    /// Error
    Error(String),
    /// Tick (for spinner animation)
    Tick,
}

pub struct App {
    // --- Input ---
    pub search_input: Input,
    pub last_query: String,

    // --- Results ---
    pub results: Vec<PkgEntry>,
    pub selected_idx: usize,
    pub results_scroll: usize,

    // --- Detail ---
    pub selected_pkg: Option<PkgEntry>,
    pub security_score: Option<SecurityScore>,
    pub comments: Vec<AurComment>,
    pub pkgbuild_text: Option<String>,
    pub comments_scroll: usize,
    pub pkgbuild_scroll: usize,

    // --- Load states ---
    pub search_state: LoadState,
    pub security_state: LoadState,
    pub comments_state: LoadState,
    pub pkgbuild_state: LoadState,
    pub install_state: LoadState,

    // --- Install log ---
    pub install_log: Vec<String>,
    pub install_scroll: usize,

    // --- UI state ---
    pub active_panel: Panel,
    pub paru_available: bool,
    pub spinner_frame: usize,
    pub should_quit: bool,
    pub status_msg: Option<String>,
    pub status_is_error: bool,

    // --- Comment selection / popup ---
    pub selected_comment_idx: usize,
    pub comment_popup_open: bool,
    pub comment_popup_scroll: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            search_input: Input::default(),
            last_query: String::new(),
            results: Vec::new(),
            selected_idx: 0,
            results_scroll: 0,
            selected_pkg: None,
            security_score: None,
            comments: Vec::new(),
            pkgbuild_text: None,
            comments_scroll: 0,
            pkgbuild_scroll: 0,
            search_state: LoadState::Idle,
            security_state: LoadState::Idle,
            comments_state: LoadState::Idle,
            pkgbuild_state: LoadState::Idle,
            install_state: LoadState::Idle,
            install_log: Vec::new(),
            install_scroll: 0,
            active_panel: Panel::Search,
            paru_available: false,
            spinner_frame: 0,
            should_quit: false,
            status_msg: None,
            status_is_error: false,
            selected_comment_idx: 0,
            comment_popup_open: false,
            comment_popup_scroll: 0,
        }
    }

    pub fn selected_pkg_name(&self) -> Option<&str> {
        self.results.get(self.selected_idx).map(|p| p.name())
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected_idx = (self.selected_idx + 1).min(self.results.len() - 1);
            self.on_select_change();
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
            self.on_select_change();
        }
    }

    pub fn on_select_change(&mut self) {
        self.selected_pkg = self.results.get(self.selected_idx).cloned();
        self.security_score = None;
        self.comments = Vec::new();
        self.pkgbuild_text = None;
        self.security_state = LoadState::Loading;
        self.comments_state = LoadState::Loading;
        self.pkgbuild_state = LoadState::Loading;
        self.comments_scroll = 0;
        self.pkgbuild_scroll = 0;
        self.selected_comment_idx = 0;
        self.comment_popup_open = false;
        self.comment_popup_scroll = 0;

        // Repo packages: mark comments/pkgbuild as not applicable immediately
        if let Some(PkgEntry::Repo(_)) = &self.selected_pkg {
            self.comments_state = LoadState::Done;
            self.pkgbuild_state = LoadState::Done;
        }
    }

    pub fn on_search_results(&mut self, results: Vec<PkgEntry>) {
        self.results = results;
        self.selected_idx = 0;
        self.results_scroll = 0;
        self.search_state = LoadState::Done;
        if !self.results.is_empty() {
            self.selected_pkg = self.results.first().cloned();
            self.security_state = LoadState::Loading;
            self.comments_state = LoadState::Loading;
            self.pkgbuild_state = LoadState::Loading;
            // For repo pkg at top, skip those
            if let Some(PkgEntry::Repo(_)) = &self.selected_pkg {
                self.comments_state = LoadState::Done;
                self.pkgbuild_state = LoadState::Done;
            }
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_msg = Some(msg.into());
        self.status_is_error = is_error;
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn scroll_results_down(&mut self) {
        if self.results_scroll + 1 < self.results.len() {
            self.results_scroll += 1;
        }
    }

    pub fn scroll_results_up(&mut self) {
        self.results_scroll = self.results_scroll.saturating_sub(1);
    }

    pub fn scroll_comments_down(&mut self) {
        if self.comments_scroll + 1 < self.comments.len() {
            self.comments_scroll += 1;
        }
    }

    pub fn scroll_comments_up(&mut self) {
        self.comments_scroll = self.comments_scroll.saturating_sub(1);
    }

    pub fn comment_select_next(&mut self) {
        if !self.comments.is_empty() {
            self.selected_comment_idx =
                (self.selected_comment_idx + 1).min(self.comments.len() - 1);
        }
    }

    pub fn comment_select_prev(&mut self) {
        self.selected_comment_idx = self.selected_comment_idx.saturating_sub(1);
    }

    pub fn scroll_pkgbuild_down(&mut self, max: usize) {
        if self.pkgbuild_scroll + 1 < max {
            self.pkgbuild_scroll += 1;
        }
    }

    pub fn scroll_pkgbuild_up(&mut self) {
        self.pkgbuild_scroll = self.pkgbuild_scroll.saturating_sub(1);
    }
}
