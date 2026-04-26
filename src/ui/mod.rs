pub mod theme;
pub mod search;
pub mod detail;
pub mod comments;
pub mod help;

use crate::app::{App, LoadState, Panel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use theme::*;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Full transparent base — use Reset so terminal bg shows through
    f.render_widget(
        Block::default().style(Style::default().bg(ratatui::style::Color::Reset)),
        size,
    );

    // Top layout: header | search
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header + search bar
            Constraint::Min(1),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_header(f, app, root[0]);
    render_main(f, app, root[1]);
    render_statusbar(f, app, root[2]);

    // Overlays
    match app.active_panel {
        Panel::Help => help::render_help(f, size),
        Panel::InstallLog => render_install_log(f, app, size),
        Panel::Pkgbuild => render_pkgbuild_overlay(f, app, size),
        _ => {}
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(1)])
        .split(area);

    // Logo
    let logo = Paragraph::new(Line::from(vec![
        Span::styled(ICON_AUR, Style::default().fg(MAUVE).add_modifier(Modifier::BOLD)),
        Span::styled("aurdash", Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SURFACE1))
            .style(Style::default().bg(ratatui::style::Color::Reset)),
    );
    f.render_widget(logo, layout[0]);

    // Search bar
    search::render_search_bar(f, app, layout[1]);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(38), Constraint::Min(1)])
        .split(area);

    search::render_results_list(f, app, layout[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(layout[1]);

    detail::render_detail(f, app, right[0]);
    comments::render_comments(f, app, right[1]);
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let hints = if app.active_panel == Panel::Search {
        " [Tab] results  [?] help  [Ctrl+C] quit"
    } else {
        " [/] search  [i] install  [p] PKGBUILD  [c] comments  [?] help  [q] quit"
    };

    let mut spans = vec![
        Span::styled(hints, Style::default().fg(OVERLAY1)),
    ];

    if let Some(ref msg) = app.status_msg {
        let color = if app.status_is_error { RED } else { GREEN };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(msg.as_str(), Style::default().fg(color).add_modifier(Modifier::BOLD)));
    }

    let bar = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(ratatui::style::Color::Reset));
    f.render_widget(bar, area);
}

fn render_install_log(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(80, 70, area);
    f.render_widget(Clear, popup);

    let title_style = match &app.install_state {
        LoadState::Done => Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        LoadState::Error(_) => Style::default().fg(RED).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(YELLOW),
    };

    let title = match &app.install_state {
        LoadState::Done => format!("{}Install Log — Success", ICON_CHECK),
        LoadState::Error(_) => format!("{}Install Log — Failed", ICON_DANGER),
        _ => format!("{}Installing...", ICON_SPINNER[app.spinner_frame]),
    };

    let lines: Vec<Line> = if app.install_log.is_empty() {
        vec![Line::from(Span::styled("Running paru...", Style::default().fg(OVERLAY1)))]
    } else {
        app.install_log
            .iter()
            .skip(app.install_scroll)
            .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(TEXT))))
            .collect()
    };

    let log = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MAUVE))
                .title(Span::styled(title, title_style))
                .style(Style::default().bg(MANTLE)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(log, popup);
}

fn render_pkgbuild_overlay(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(90, 85, area);
    f.render_widget(Clear, popup);

    let content: Vec<Line> = match &app.pkgbuild_text {
        None => vec![Line::from(Span::styled(
            "Loading PKGBUILD...",
            Style::default().fg(OVERLAY1),
        ))],
        Some(text) => text
            .lines()
            .skip(app.pkgbuild_scroll)
            .map(|l| {
                // Simple syntax coloring
                let color = if l.starts_with('#') {
                    OVERLAY1
                } else if l.starts_with("pkgname") || l.starts_with("pkgver") || l.starts_with("pkgrel") {
                    BLUE
                } else if l.starts_with("depends") || l.starts_with("makedepends") {
                    TEAL
                } else if l.starts_with("source") || l.starts_with("sha") || l.starts_with("b2") {
                    YELLOW
                } else if l.contains("() {") || l.starts_with("build()") || l.starts_with("package()") {
                    MAUVE
                } else {
                    TEXT
                };
                Line::from(Span::styled(l, Style::default().fg(color)))
            })
            .collect(),
    };

    let pkg_name = app
        .selected_pkg
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("PKGBUILD");

    let pkgbuild = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SAPPHIRE))
                .title(Span::styled(
                    format!("{}PKGBUILD — {}  [↑↓/jk scroll] [Esc close]", ICON_PKGBUILD, pkg_name),
                    Style::default().fg(SAPPHIRE).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(MANTLE)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(pkgbuild, popup);
}

/// Create a centered popup rect with given % of area
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
