use crate::app::{App, LoadState, Panel};
use crate::aur::{PkgEntry, RepoPackage};
use crate::aur::search::AurPackage;
use crate::ui::theme::*;
use chrono::{TimeZone, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Detail;
    let border_color = if is_active { SAPPHIRE } else { SURFACE1 };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!("{}Package Detail", ICON_PACKAGE),
            Style::default().fg(SAPPHIRE).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(ratatui::style::Color::Reset));

    match &app.selected_pkg {
        None => {
            let p = Paragraph::new(Line::from(Span::styled(
                "Select a package from the list",
                Style::default().fg(OVERLAY0).add_modifier(Modifier::ITALIC),
            )))
            .block(block);
            f.render_widget(p, area);
        }
        Some(PkgEntry::Repo(pkg)) => {
            let inner = block.inner(area);
            f.render_widget(block, area);
            render_repo_detail(f, app, pkg, inner);
        }
        Some(PkgEntry::Aur(pkg)) => {
            let inner = block.inner(area);
            f.render_widget(block, area);
            render_aur_detail(f, app, pkg, inner);
        }
    }
}

fn render_repo_detail(f: &mut Frame, _app: &App, pkg: &RepoPackage, inner: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // name + version
            Constraint::Length(1), // description
            Constraint::Length(1), // meta row 1: repo, installed size
            Constraint::Length(1), // meta row 2: url, licenses
            Constraint::Length(3), // "Official — Verified" badge area
            Constraint::Length(1), // sep
            Constraint::Min(1),    // deps
        ])
        .split(inner);

    // Name + Version
    let name_line = Line::from(vec![
        Span::styled(&pkg.name, Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(&pkg.version, Style::default().fg(TEAL)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", pkg.repo.to_uppercase()),
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        if pkg.installed {
            Span::styled(
                format!("  {}installed", ICON_CHECK),
                Style::default().fg(GREEN),
            )
        } else {
            Span::raw("")
        },
    ]);
    f.render_widget(Paragraph::new(name_line), layout[0]);

    // Description
    let desc_line = Line::from(Span::styled(
        pkg.description.as_str(),
        Style::default().fg(SUBTEXT1),
    ));
    f.render_widget(
        Paragraph::new(desc_line).wrap(Wrap { trim: true }),
        layout[1],
    );

    // Meta row 1: repo name, install size
    let install_size = pkg.install_size.as_deref().unwrap_or("unknown");
    let meta1 = Line::from(vec![
        Span::styled(ICON_PACKAGE, Style::default().fg(BLUE)),
        Span::styled(
            format!("repo: {}", pkg.repo),
            Style::default().fg(TEXT),
        ),
        Span::raw("  "),
        Span::styled(ICON_DATE, Style::default().fg(OVERLAY2)),
        Span::styled(
            format!("size: {}", install_size),
            Style::default().fg(SUBTEXT0),
        ),
    ]);
    f.render_widget(Paragraph::new(meta1), layout[2]);

    // Meta row 2: url, licenses
    let url = pkg.url.as_deref().unwrap_or("no url");
    let licenses = if pkg.licenses.is_empty() {
        "unknown".to_string()
    } else {
        pkg.licenses.join(", ")
    };
    let meta2 = Line::from(vec![
        Span::styled(ICON_LINK, Style::default().fg(BLUE)),
        Span::styled(url, Style::default().fg(OVERLAY1)),
        Span::raw("  "),
        Span::styled("License: ", Style::default().fg(OVERLAY2)),
        Span::styled(licenses, Style::default().fg(SUBTEXT0)),
    ]);
    f.render_widget(Paragraph::new(meta2), layout[3]);

    // Official verified badge (instead of security scan)
    let badge_lines = vec![
        Line::from(vec![
            Span::styled(ICON_CHECK, Style::default().fg(GREEN)),
            Span::styled(
                "Official Repository — Verified",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "  · Maintained by the Arch Linux team",
            Style::default().fg(TEAL),
        )),
    ];
    f.render_widget(Paragraph::new(badge_lines), layout[4]);

    // Separator
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(SURFACE1),
        ))),
        layout[5],
    );

    // Dependencies
    let deps = pkg.depends.join(", ");
    let deps_text = if deps.is_empty() {
        "No dependencies".to_string()
    } else {
        format!("Deps: {}", deps)
    };
    let deps_p = Paragraph::new(Line::from(Span::styled(
        deps_text,
        Style::default().fg(OVERLAY1),
    )))
    .wrap(Wrap { trim: true });
    f.render_widget(deps_p, layout[6]);
}

fn render_aur_detail(f: &mut Frame, app: &App, pkg: &AurPackage, inner: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // name + version
            Constraint::Length(1), // description
            Constraint::Length(1), // meta row 1
            Constraint::Length(1), // meta row 2
            Constraint::Length(3), // security score
            Constraint::Length(1), // sep
            Constraint::Min(1),    // deps
        ])
        .split(inner);

    // Name + Version
    let name_line = Line::from(vec![
        Span::styled(&pkg.name, Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(&pkg.version, Style::default().fg(TEAL)),
        if pkg.out_of_date.is_some() {
            Span::styled("  [OUT OF DATE]", Style::default().fg(RED).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("")
        },
    ]);
    f.render_widget(Paragraph::new(name_line), layout[0]);

    // Description
    let desc = pkg.description.as_deref().unwrap_or("No description");
    let desc_line = Line::from(Span::styled(desc, Style::default().fg(SUBTEXT1)));
    f.render_widget(
        Paragraph::new(desc_line).wrap(Wrap { trim: true }),
        layout[1],
    );

    // Meta row 1: votes, popularity, maintainer
    let maintainer = pkg.maintainer.as_deref().unwrap_or("orphaned");
    let maintainer_color = if pkg.maintainer.is_none() { RED } else { GREEN };
    let meta1 = Line::from(vec![
        Span::styled(ICON_VOTES, Style::default().fg(YELLOW)),
        Span::styled(
            format!("{} votes", pkg.num_votes),
            Style::default().fg(TEXT),
        ),
        Span::raw("  "),
        Span::styled(ICON_STAR, Style::default().fg(PEACH)),
        Span::styled(
            format!("{:.2} pop", pkg.popularity),
            Style::default().fg(TEXT),
        ),
        Span::raw("  "),
        Span::styled(ICON_USER, Style::default().fg(SKY)),
        Span::styled(maintainer, Style::default().fg(maintainer_color)),
    ]);
    f.render_widget(Paragraph::new(meta1), layout[2]);

    // Meta row 2: last modified, url
    let last_mod = Utc
        .timestamp_opt(pkg.last_modified, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let url = pkg.url.as_deref().unwrap_or("no url");
    let meta2 = Line::from(vec![
        Span::styled(ICON_DATE, Style::default().fg(OVERLAY2)),
        Span::styled(&last_mod, Style::default().fg(SUBTEXT0)),
        Span::raw("  "),
        Span::styled(ICON_LINK, Style::default().fg(BLUE)),
        Span::styled(url, Style::default().fg(OVERLAY1)),
    ]);
    f.render_widget(Paragraph::new(meta2), layout[3]);

    // Security score
    render_security_score(f, app, layout[4]);

    // Separator
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(SURFACE1),
        ))),
        layout[5],
    );

    // Dependencies
    let deps = pkg.depends.join(", ");
    let deps_text = if deps.is_empty() {
        "No dependencies".to_string()
    } else {
        format!("Deps: {}", deps)
    };
    let deps_p = Paragraph::new(Line::from(Span::styled(
        deps_text,
        Style::default().fg(OVERLAY1),
    )))
    .wrap(Wrap { trim: true });
    f.render_widget(deps_p, layout[6]);
}

fn render_security_score(f: &mut Frame, app: &App, area: Rect) {
    match &app.security_state {
        LoadState::Loading => {
            let spinner = ICON_SPINNER[app.spinner_frame];
            let line = Line::from(vec![
                Span::styled(ICON_SECURITY, Style::default().fg(OVERLAY1)),
                Span::styled(
                    format!("{}Analyzing PKGBUILD…", spinner),
                    Style::default().fg(OVERLAY1).add_modifier(Modifier::ITALIC),
                ),
            ]);
            f.render_widget(Paragraph::new(line), area);
        }
        LoadState::Error(e) => {
            let line = Line::from(vec![
                Span::styled(ICON_SECURITY, Style::default().fg(RED)),
                Span::styled(
                    format!("Scan error: {}", e),
                    Style::default().fg(RED),
                ),
            ]);
            f.render_widget(Paragraph::new(line), area);
        }
        LoadState::Done | LoadState::Idle => {
            if let Some(score) = &app.security_score {
                let level = score_level(score.score);
                let color = level.color();
                let bar_width = (area.width as usize).saturating_sub(20).min(20);
                let bar = score_bar(score.score, bar_width);

                let lines = vec![
                    Line::from(vec![
                        Span::styled(ICON_SECURITY, Style::default().fg(color)),
                        Span::styled(
                            format!("Security  {} {}/100  ", bar, score.score),
                            Style::default().fg(color),
                        ),
                        Span::styled(
                            format!("{}{}", level.icon(), level.label()),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    // First breakdown items
                    Line::from(
                        score.breakdown.iter().take(2).flat_map(|(msg, adj)| {
                            let color = if *adj < 0 { RED } else if *adj > 0 { GREEN } else { OVERLAY1 };
                            let adj_str = if *adj != 0 { format!(" ({:+})", adj) } else { String::new() };
                            vec![
                                Span::styled(
                                    format!("  · {}{}", msg, adj_str),
                                    Style::default().fg(color),
                                ),
                                Span::raw("  "),
                            ]
                        })
                        .collect::<Vec<_>>(),
                    ),
                ];

                f.render_widget(Paragraph::new(lines), area);
            } else {
                let line = Line::from(vec![
                    Span::styled(ICON_SECURITY, Style::default().fg(OVERLAY0)),
                    Span::styled("Select a package to scan", Style::default().fg(OVERLAY0)),
                ]);
                f.render_widget(Paragraph::new(line), area);
            }
        }
    }
}
