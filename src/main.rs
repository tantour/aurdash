#![allow(dead_code)]
mod app;
mod aur;
mod events;
mod install;
mod security;
mod ui;

use anyhow::Result;
use app::{App, AppEvent};
use raur::Raur;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{handle_app_event, handle_event, Action};
use futures::StreamExt;
use reqwest::Client;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::{
    sync::mpsc,
    time::{interval, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let result = run_app(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("aurdash error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();
    let http = Client::builder()
        .user_agent("aurdash/0.1")
        .timeout(Duration::from_secs(15))
        .build()?;

    let _searcher = aur::AurSearcher::new();

    // Channel for background → main thread events
    let (tx, mut rx) = mpsc::channel::<AppEvent>(64);

    // Check paru availability
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let avail = install::paru_available().await;
            let _ = tx.send(AppEvent::ParuAvailable(avail)).await;
        });
    }

    // Tick interval for spinner animation
    let mut tick_interval = interval(Duration::from_millis(80));

    // Debounce: track last search time
    let mut last_search_time = Instant::now();
    let debounce = Duration::from_millis(300);

    // Crossterm event stream
    let mut event_stream = EventStream::new();

    // Current pending search query (debounced)
    let mut pending_search: Option<String> = None;
    // Track active detail loads to avoid stale overwrites
    let mut current_detail_pkg: Option<String> = None;

    loop {
        // Draw
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            // Terminal events
            maybe_event = event_stream.next() => {
                if let Some(Ok(event)) = maybe_event {
                    // Tick on any event too
                    if matches!(event, Event::Key(_) | Event::Mouse(_)) {
                        let action = handle_event(&mut app, &event);
                        match action {
                            Action::Quit => break,
                            Action::Search(query) => {
                                pending_search = Some(query);
                                last_search_time = Instant::now();
                            }
                            Action::LoadDetail(name) => {
                                if Some(&name) != current_detail_pkg.as_ref() {
                                    current_detail_pkg = Some(name.clone());
                                    spawn_detail_load(name, tx.clone(), http.clone());
                                }
                            }
                            Action::Install(name) => {
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    match install::install_package(&name).await {
                                        Ok((success, log)) => {
                                            let _ = tx.send(AppEvent::InstallDone(success, log)).await;
                                        }
                                        Err(e) => {
                                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                                        }
                                    }
                                });
                            }
                            Action::Continue => {}
                        }
                    }
                }
            }

            // Background events (search results, scores, comments, etc.)
            Some(app_event) = rx.recv() => {
                let action = handle_app_event(&mut app, app_event);
                match action {
                    Action::LoadDetail(name) => {
                        if Some(&name) != current_detail_pkg.as_ref() {
                            current_detail_pkg = Some(name.clone());
                            spawn_detail_load(name, tx.clone(), http.clone());
                        }
                    }
                    Action::Quit => break,
                    _ => {}
                }
            }

            // Tick
            _ = tick_interval.tick() => {
                let _ = tx.send(AppEvent::Tick).await;

                // Fire debounced search
                if let Some(ref query) = pending_search {
                    if last_search_time.elapsed() >= debounce {
                        let q = query.clone();
                        pending_search = None;
                        let tx2 = tx.clone();
                        // We need searcher — clone via Arc would be ideal, but raur::Handle is Clone
                        let searcher_inner = aur::AurSearcher::new();
                        tokio::spawn(async move {
                            match searcher_inner.search(&q).await {
                                Ok(results) => {
                                    let _ = tx2.send(AppEvent::SearchResults(results)).await;
                                }
                                Err(e) => {
                                    let _ = tx2.send(AppEvent::Error(e.to_string())).await;
                                }
                            }
                        });
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Spawn concurrent tasks to load all detail data for a package
fn spawn_detail_load(
    pkg_name: String,
    tx: mpsc::Sender<AppEvent>,
    http: Client,
) {
    // Fetch PKGBUILD + security scan (they're linked)
    {
        let name = pkg_name.clone();
        let tx = tx.clone();
        let http = http.clone();
        tokio::spawn(async move {
            match aur::pkgbuild::fetch_pkgbuild(&http, &name).await {
                Ok(text) => {
                    let _ = tx.send(AppEvent::Pkgbuild(name.clone(), text.clone())).await;

                    // Now run security scan on the PKGBUILD text
                    // We need the AUR Package for metadata signals - do a quick info lookup
                    let raur = raur::Handle::new();
                    let score = match raur.info(&[&name]).await {
                        Ok(pkgs) if !pkgs.is_empty() => {
                            security::compute_security_score(&text, &pkgs[0]).await
                        }
                        _ => {
                            // No AUR metadata, scan only
                            let fake_pkg = make_minimal_pkg(&name);
                            security::compute_security_score(&text, &fake_pkg).await
                        }
                    };
                    match score {
                        Ok(s) => {
                            let _ = tx.send(AppEvent::SecurityScore(name, s)).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(format!("Security scan: {}", e))).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(format!("PKGBUILD fetch: {}", e))).await;
                }
            }
        });
    }

    // Fetch comments independently
    {
        let name = pkg_name.clone();
        let tx = tx.clone();
        let http = http.clone();
        tokio::spawn(async move {
            match aur::comments::fetch_comments(&http, &name).await {
                Ok(comments) => {
                    let _ = tx.send(AppEvent::Comments(name, comments)).await;
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(format!("Comments: {}", e))).await;
                }
            }
        });
    }
}

/// Create a minimal fake Package when RPC info fails
fn make_minimal_pkg(name: &str) -> raur::Package {
    raur::Package {
        id: 0,
        name: name.to_string(),
        package_base_id: 0,
        package_base: name.to_string(),
        version: String::new(),
        description: None,
        url: None,
        num_votes: 0,
        popularity: 0.0,
        out_of_date: None,
        maintainer: None,
        co_maintainers: vec![],
        submitter: None,
        first_submitted: 0,
        last_modified: 0,
        url_path: String::new(),
        depends: vec![],
        make_depends: vec![],
        opt_depends: vec![],
        check_depends: vec![],
        conflicts: vec![],
        provides: vec![],
        replaces: vec![],
        groups: vec![],
        license: vec![],
        keywords: vec![],
    }
}
