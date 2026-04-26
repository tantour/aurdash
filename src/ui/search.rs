use crate::app::{App, LoadState, Panel};
use crate::aur::PkgEntry;
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Search;
    let border_color = if is_active { MAUVE } else { SURFACE1 };

    let spinner = if app.search_state == LoadState::Loading {
        ICON_SPINNER[app.spinner_frame]
    } else {
        ""
    };

    let query = app.search_input.value();
    let placeholder = if query.is_empty() { "Search packages (AUR + official repos)…" } else { "" };

    let input_text = if query.is_empty() {
        Line::from(vec![
            Span::styled(ICON_SEARCH, Style::default().fg(OVERLAY1)),
            Span::styled(placeholder, Style::default().fg(SURFACE2).add_modifier(Modifier::ITALIC)),
            Span::styled(spinner, Style::default().fg(YELLOW)),
        ])
    } else {
        Line::from(vec![
            Span::styled(ICON_SEARCH, Style::default().fg(MAUVE)),
            Span::styled(query, Style::default().fg(TEXT)),
            Span::styled(spinner, Style::default().fg(YELLOW)),
        ])
    };

    let count_hint = if !app.results.is_empty() {
        format!(" {} results", app.results.len())
    } else {
        String::new()
    };

    let title = format!("Explorer Search{}", count_hint);

    let search = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(SUBTEXT0)))
            .style(Style::default().bg(ratatui::style::Color::Reset)),
    );

    f.render_widget(search, area);

    // Set cursor position if search is active
    if is_active {
        let cursor_x = area.x + 1 + ICON_SEARCH.width() as u16 + query.width() as u16;
        let cursor_y = area.y + 1;
        if cursor_x < area.x + area.width - 1 {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

pub fn render_results_list(f: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.active_panel, Panel::Results | Panel::Detail);
    let border_color = if is_active { BLUE } else { SURFACE1 };

    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            let is_selected = i == app.selected_idx;
            let name_color = if is_selected { LAVENDER } else { TEXT };
            let ver_color = if is_selected { TEAL } else { SUBTEXT0 };

            let name_str = pkg.name();
            let ver_str = pkg.version();

            // Truncate name to fit (reserve space for badge + version)
            let max_name = (area.width as usize).saturating_sub(14).min(24);
            let name = if name_str.len() > max_name {
                format!("{}…", &name_str[..max_name.saturating_sub(1)])
            } else {
                name_str.to_string()
            };

            let ver_short = if ver_str.len() > 12 {
                &ver_str[..12]
            } else {
                ver_str
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            // Badge: show repo name for official pkgs, AUR icon for AUR
            let (badge_text, badge_color) = match pkg {
                PkgEntry::Repo(r) => {
                    let label = format!("[{}]", r.repo.to_uppercase());
                    (label, GREEN)
                }
                PkgEntry::Aur(_) => (String::from("[AUR]"), MAUVE),
            };

            // Installed indicator
            let installed_span = if app.installed_pkgs.contains(pkg.name()) {
                Span::styled(ICON_CHECK, Style::default().fg(GREEN))
            } else {
                Span::raw(" ")
            };

            Line::from(vec![
                Span::styled(prefix, Style::default().fg(MAUVE)),
                Span::styled(badge_text, Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(name, Style::default().fg(name_color)),
                Span::raw(" "),
                Span::styled(ver_short, Style::default().fg(ver_color)),
                Span::raw(" "),
                installed_span,
            ])
            .into()
        })
        .collect();

    let title = if app.results.is_empty() && app.search_state == LoadState::Loading {
        format!("{}Searching…", ICON_SPINNER[app.spinner_frame])
    } else {
        format!("{}Results", ICON_PACKAGE)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(title, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)))
                .style(Style::default().bg(ratatui::style::Color::Reset)),
        )
        .highlight_style(
            Style::default()
                .bg(SURFACE0)
                .fg(LAVENDER)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if !app.results.is_empty() {
        // Adjust offset based on scroll
        let visible = area.height.saturating_sub(2) as usize;
        let offset = if app.selected_idx >= visible {
            app.selected_idx - visible + 1
        } else {
            0
        };
        state.select(Some(app.selected_idx));
        *state.offset_mut() = offset;
    }

    f.render_stateful_widget(list, area, &mut state);

    // Empty state
    if app.results.is_empty() && app.search_state == LoadState::Idle {
        let empty = Paragraph::new(Line::from(vec![
            Span::styled(ICON_SEARCH, Style::default().fg(OVERLAY0)),
            Span::styled(" Type to search AUR + official repos", Style::default().fg(OVERLAY0).add_modifier(Modifier::ITALIC)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SURFACE1))
                .style(Style::default().bg(ratatui::style::Color::Reset)),
        );
        f.render_widget(empty, area);
    }
}
