use crate::app::{App, AppEvent, LoadState, Panel};
use crate::aur::PkgEntry;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use tui_input::backend::crossterm::EventHandler;

pub enum Action {
    /// Normal handled action
    Continue,
    /// Trigger a new search (repo + AUR)
    Search(String),
    /// Load detail for selected package
    LoadDetail(PkgEntry),
    /// Install selected package
    Install(String),
    /// Uninstall selected package
    Uninstall(String, bool),
    /// Quit
    Quit,
}

pub fn handle_event(app: &mut App, event: &Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        _ => Action::Continue,
    }
}

pub fn handle_app_event(app: &mut App, event: AppEvent) -> Action {
    match event {
        AppEvent::SearchResults(results) => {
            let first = results.first().cloned();
            app.on_search_results(results);
            if let Some(entry) = first {
                return Action::LoadDetail(entry);
            }
        }
        AppEvent::SecurityScore(pkg, score) => {
            if app.selected_pkg.as_ref().map(|p| p.name()) == Some(pkg.as_str()) {
                app.security_score = Some(score);
                app.security_state = LoadState::Done;
            }
        }
        AppEvent::Comments(pkg, comments) => {
            if app.selected_pkg.as_ref().map(|p| p.name()) == Some(pkg.as_str()) {
                app.comments = comments;
                app.comments_state = LoadState::Done;
            }
        }
        AppEvent::Pkgbuild(pkg, text) => {
            if app.selected_pkg.as_ref().map(|p| p.name()) == Some(pkg.as_str()) {
                app.pkgbuild_text = Some(text);
                app.pkgbuild_state = LoadState::Done;
            }
        }
        AppEvent::RepoInfo(pkg, info) => {
            if app.selected_pkg.as_ref().map(|p| p.name()) == Some(pkg.as_str()) {
                // Update the selected_pkg and results entry with enriched info
                let enriched = PkgEntry::Repo(info);
                if let Some(r) = app.results.get_mut(app.selected_idx) {
                    *r = enriched.clone();
                }
                app.selected_pkg = Some(enriched);
                app.security_state = LoadState::Done;
            }
        }
        AppEvent::InstallDone(success, log) => {
            app.install_log = log.lines().map(String::from).collect();
            app.install_state = if success {
                LoadState::Done
            } else {
                LoadState::Error("Operation failed".to_string())
            };
            let msg = if success {
                "Operation succeeded!".to_string()
            } else {
                "Operation failed. Check install log.".to_string()
            };
            app.set_status(msg, !success);

            if success {
                let mut new_installed = std::collections::HashSet::new();
                if let Ok(output) = std::process::Command::new("pacman").arg("-Qq").output() {
                    if let Ok(s) = String::from_utf8(output.stdout) {
                        for line in s.lines() {
                            new_installed.insert(line.to_string());
                        }
                    }
                }
                app.installed_pkgs = new_installed;
                app.manager_pkgs = app.installed_pkgs.iter().cloned().collect();
                app.manager_pkgs.sort();
                let query = app.manager_search_input.value().to_lowercase();
                app.manager_filtered_pkgs = app.manager_pkgs
                    .iter()
                    .filter(|p| p.to_lowercase().contains(&query))
                    .cloned()
                    .collect();
            }
        }
        AppEvent::ParuAvailable(avail) => {
            app.paru_available = avail;
            if !avail {
                app.set_status("paru not found — install actions disabled", true);
            }
        }
        AppEvent::Tick => {
            app.spinner_frame = (app.spinner_frame + 1) % 10;
        }
        AppEvent::Error(e) => {
            app.set_status(format!("Error: {}", e), true);
        }
    }
    Action::Continue
}

fn handle_key(app: &mut App, key: &KeyEvent) -> Action {
    // Global quit
    if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
        if app.active_panel == Panel::Search {
            return Action::Quit;
        }
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    // Escape - close popup first, then go back / close overlay
    if key.code == KeyCode::Esc {
        if app.comment_popup_open {
            app.comment_popup_open = false;
            app.comment_popup_scroll = 0;
            return Action::Continue;
        }
        match app.active_panel {
            Panel::Help | Panel::InstallLog | Panel::Pkgbuild => {
                app.active_panel = Panel::Results;
            }
            Panel::Comments => {
                app.active_panel = Panel::Detail;
            }
            Panel::Detail | Panel::Results => {
                app.active_panel = Panel::Search;
            }
            Panel::Manager => {
                app.switch_to_explorer();
            }
            Panel::ManagerUninstallPopup => {
                app.active_panel = Panel::Manager;
            }
            Panel::Search => {}
        }
        return Action::Continue;
    }

    // Help toggle
    if key.code == KeyCode::Char('?') {
        if app.active_panel == Panel::Help {
            app.active_panel = Panel::Results;
        } else {
            app.active_panel = Panel::Help;
        }
        return Action::Continue;
    }

    match app.active_panel {
        Panel::Search => handle_search_key(app, key),
        Panel::Results => handle_results_key(app, key),
        Panel::Detail => handle_detail_key(app, key),
        Panel::Comments => handle_comments_key(app, key),
        Panel::Pkgbuild => handle_pkgbuild_key(app, key),
        Panel::InstallLog => handle_installlog_key(app, key),
        Panel::Help => {
            app.active_panel = Panel::Results;
            Action::Continue
        }
        Panel::Manager => {
            if app.manager_search_active {
                match key.code {
                    KeyCode::Enter | KeyCode::Down | KeyCode::Esc => {
                        app.manager_search_active = false;
                        return Action::Continue;
                    }
                    KeyCode::Tab => {
                        app.switch_to_explorer();
                        return Action::Continue;
                    }
                    _ => {}
                }
                
                let prev = app.manager_search_input.value().to_string();
                app.manager_search_input.handle_event(&Event::Key(*key));
                let current = app.manager_search_input.value().to_string();
                
                if current != prev {
                    let query = current.to_lowercase();
                    app.manager_filtered_pkgs = app.manager_pkgs
                        .iter()
                        .filter(|p| p.to_lowercase().contains(&query))
                        .cloned()
                        .collect();
                    app.manager_selected_idx = 0;
                }
                return Action::Continue;
            }

            match key.code {
                KeyCode::Tab => {
                    app.switch_to_explorer();
                }
                KeyCode::Char('/') => {
                    app.manager_search_active = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.manager_selected_idx = app.manager_selected_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.manager_selected_idx + 1 < app.manager_filtered_pkgs.len() {
                        app.manager_selected_idx += 1;
                    }
                }
                KeyCode::Char('u') => {
                    if !app.manager_filtered_pkgs.is_empty() {
                        app.active_panel = Panel::ManagerUninstallPopup;
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('i') => {
                    if let Some(pkg) = app.manager_filtered_pkgs.get(app.manager_selected_idx) {
                        app.active_panel = Panel::InstallLog;
                        app.install_state = LoadState::Loading;
                        app.install_log.clear();
                        return Action::Install(pkg.clone());
                    }
                }
                _ => {}
            }
            Action::Continue
        }
        Panel::ManagerUninstallPopup => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.active_panel = Panel::Manager;
                }
                KeyCode::Char('n') | KeyCode::Char('r') => {
                    if let Some(pkg) = app.manager_filtered_pkgs.get(app.manager_selected_idx) {
                        let recursive = key.code == KeyCode::Char('r');
                        app.active_panel = Panel::InstallLog;
                        app.install_state = LoadState::Loading;
                        app.install_log.clear();
                        return Action::Uninstall(pkg.clone(), recursive);
                    }
                }
                _ => {}
            }
            Action::Continue
        }
    }
}

fn handle_search_key(app: &mut App, key: &KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter | KeyCode::Down => {
            if !app.results.is_empty() {
                app.active_panel = Panel::Results;
            }
            return Action::Continue;
        }
        KeyCode::Tab => {
            app.switch_to_manager();
            return Action::Continue;
        }
        _ => {}
    }

    let prev = app.search_input.value().to_string();
    app.search_input.handle_event(&Event::Key(*key));
    let current = app.search_input.value().to_string();

    if current != prev {
        if current.len() >= 2 {
            app.search_state = LoadState::Loading;
            app.last_query = current.clone();
            return Action::Search(current);
        } else if current.is_empty() {
            app.results.clear();
            app.selected_pkg = None;
            app.search_state = LoadState::Idle;
        }
    }

    Action::Continue
}

fn load_detail_for_selected(app: &App) -> Action {
    if let Some(entry) = app.results.get(app.selected_idx) {
        Action::LoadDetail(entry.clone())
    } else {
        Action::Continue
    }
}

fn handle_results_key(app: &mut App, key: &KeyEvent) -> Action {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
            return load_detail_for_selected(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            return load_detail_for_selected(app);
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            app.active_panel = Panel::Detail;
        }
        KeyCode::Char('/') | KeyCode::Char('s') => {
            app.active_panel = Panel::Search;
        }
        KeyCode::Tab => {
            app.switch_to_manager();
        }
        KeyCode::Char('i') => {
            if let Some(name) = app.selected_pkg_name().map(String::from) {
                app.active_panel = Panel::InstallLog;
                app.install_state = LoadState::Loading;
                app.install_log.clear();
                return Action::Install(name);
            }
        }
        KeyCode::Char('p') => {
            // Only for AUR packages
            if app.selected_pkg.as_ref().map(|p| p.is_aur()).unwrap_or(false) {
                app.active_panel = Panel::Pkgbuild;
            }
        }
        KeyCode::Char('c') => {
            if app.selected_pkg.as_ref().map(|p| p.is_aur()).unwrap_or(false) {
                app.active_panel = Panel::Comments;
            }
        }
        _ => {}
    }
    Action::Continue
}

fn handle_detail_key(app: &mut App, key: &KeyEvent) -> Action {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
            return load_detail_for_selected(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            return load_detail_for_selected(app);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.active_panel = Panel::Results;
        }
        KeyCode::Tab => {
            app.switch_to_manager();
        }
        KeyCode::Char('i') => {
            if let Some(name) = app.selected_pkg_name().map(String::from) {
                app.active_panel = Panel::InstallLog;
                app.install_state = LoadState::Loading;
                app.install_log.clear();
                return Action::Install(name);
            }
        }
        KeyCode::Char('p') => {
            if app.selected_pkg.as_ref().map(|p| p.is_aur()).unwrap_or(false) {
                app.active_panel = Panel::Pkgbuild;
            }
        }
        KeyCode::Char('c') => {
            if app.selected_pkg.as_ref().map(|p| p.is_aur()).unwrap_or(false) {
                app.active_panel = Panel::Comments;
            }
        }
        _ => {}
    }
    Action::Continue
}

fn handle_comments_key(app: &mut App, key: &KeyEvent) -> Action {
    if app.comment_popup_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.comment_popup_open = false;
                app.comment_popup_scroll = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.comment_popup_scroll = app.comment_popup_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.comment_popup_scroll += 1;
            }
            _ => {}
        }
        return Action::Continue;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.comment_select_prev();
            app.comments_scroll = app.selected_comment_idx;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.comment_select_next();
            app.comments_scroll = app.selected_comment_idx;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if !app.comments.is_empty() {
                app.comment_popup_open = true;
                app.comment_popup_scroll = 0;
            }
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
            app.active_panel = Panel::Detail;
        }
        _ => {}
    }
    Action::Continue
}

fn handle_pkgbuild_key(app: &mut App, key: &KeyEvent) -> Action {
    let lines = app
        .pkgbuild_text
        .as_ref()
        .map(|t| t.lines().count())
        .unwrap_or(0);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.scroll_pkgbuild_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_pkgbuild_down(lines),
        KeyCode::Left | KeyCode::Char('h') => {
            app.active_panel = Panel::Results;
        }
        _ => {}
    }
    Action::Continue
}

fn handle_installlog_key(app: &mut App, key: &KeyEvent) -> Action {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.install_scroll = app.install_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.install_scroll + 1 < app.install_log.len() {
                app.install_scroll += 1;
            }
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Enter => {
            app.active_panel = Panel::Results;
        }
        _ => {}
    }
    Action::Continue
}

fn handle_mouse(app: &mut App, mouse: &MouseEvent) -> Action {
    if app.comment_popup_open {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                app.comment_popup_scroll += 1;
            }
            MouseEventKind::ScrollUp => {
                app.comment_popup_scroll = app.comment_popup_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return Action::Continue;
    }

    match mouse.kind {
        MouseEventKind::ScrollDown => match app.active_panel {
            Panel::Results | Panel::Detail => {
                app.select_next();
                return load_detail_for_selected(app);
            }
            Panel::Comments => {
                app.comment_select_next();
                app.comments_scroll = app.selected_comment_idx;
            }
            Panel::Pkgbuild => {
                let lines = app
                    .pkgbuild_text
                    .as_ref()
                    .map(|t| t.lines().count())
                    .unwrap_or(0);
                app.scroll_pkgbuild_down(lines);
            }
            Panel::Manager => {
                if app.manager_selected_idx + 1 < app.manager_filtered_pkgs.len() {
                    app.manager_selected_idx += 1;
                }
            }
            Panel::InstallLog => {
                if app.install_scroll + 1 < app.install_log.len() {
                    app.install_scroll += 1;
                }
            }
            _ => {}
        },
        MouseEventKind::ScrollUp => match app.active_panel {
            Panel::Results | Panel::Detail => {
                app.select_prev();
                return load_detail_for_selected(app);
            }
            Panel::Comments => {
                app.comment_select_prev();
                app.comments_scroll = app.selected_comment_idx;
            }
            Panel::Pkgbuild => app.scroll_pkgbuild_up(),
            Panel::Manager => {
                app.manager_selected_idx = app.manager_selected_idx.saturating_sub(1);
            }
            Panel::InstallLog => {
                app.install_scroll = app.install_scroll.saturating_sub(1);
            }
            _ => {}
        },
        MouseEventKind::Down(MouseButton::Left) => {
            if app.active_panel == Panel::Manager {
                if mouse.row > 4 { // List starts below search bar
                    // Approximate the offset by assuming ~30 items fit.
                    // This is rough, but since we don't track true terminal size in App state, 
                    // we'll try our best.
                    let visible = 30;
                    let offset = if app.manager_selected_idx >= visible {
                        app.manager_selected_idx - visible + 1
                    } else {
                        0
                    };
                    let clicked_idx = offset + (mouse.row as usize).saturating_sub(5);
                    if clicked_idx < app.manager_filtered_pkgs.len() {
                        app.manager_selected_idx = clicked_idx;
                    }
                }
                return Action::Continue;
            }

            if mouse.column < 38 && mouse.row > 2 {
                let clicked_idx = app.results_scroll + (mouse.row as usize).saturating_sub(3);
                if clicked_idx < app.results.len() {
                    app.selected_idx = clicked_idx;
                    app.active_panel = Panel::Results;
                    app.on_select_change();
                    return load_detail_for_selected(app);
                }
            }
            if app.active_panel == Panel::Comments && mouse.column >= 38 {
                if !app.comments.is_empty() {
                    app.comment_popup_open = true;
                    app.comment_popup_scroll = 0;
                }
            }
        }
        _ => {}
    }
    Action::Continue
}
