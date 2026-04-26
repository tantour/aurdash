use crate::app::{App, Panel};
use crate::ui::theme::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::ui::centered_rect;
use unicode_width::UnicodeWidthStr;

pub fn render_manager(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let is_active = app.active_panel == Panel::Manager;
    let border_color = if is_active { BLUE } else { SURFACE1 };
    
    // 1. Search Bar
    let query = app.manager_search_input.value();
    let is_search_active = app.manager_search_active;
    
    let search_border_color = if is_search_active { MAUVE } else { SURFACE1 };
    let placeholder = if query.is_empty() { "Press '/' to search installed packages…" } else { "" };
    
    let input_text = if query.is_empty() {
        Line::from(vec![
            Span::styled(ICON_SEARCH, Style::default().fg(OVERLAY1)),
            Span::styled(placeholder, Style::default().fg(SURFACE2).add_modifier(Modifier::ITALIC)),
        ])
    } else {
        Line::from(vec![
            Span::styled(ICON_SEARCH, Style::default().fg(MAUVE)),
            Span::styled(query, Style::default().fg(TEXT)),
        ])
    };

    let search = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(search_border_color))
            .title(Span::styled("Manager Search", Style::default().fg(SUBTEXT0)))
            .style(Style::default().bg(ratatui::style::Color::Reset)),
    );

    f.render_widget(search, chunks[0]);

    if is_search_active {
        let cursor_x = chunks[0].x + 1 + ICON_SEARCH.width() as u16 + query.width() as u16;
        let cursor_y = chunks[0].y + 1;
        if cursor_x < chunks[0].x + chunks[0].width - 1 {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    // 2. List
    let items: Vec<ListItem> = app.manager_filtered_pkgs.iter().enumerate().map(|(i, pkg)| {
        let is_selected = i == app.manager_selected_idx;
        let prefix = if is_selected { "▶ " } else { "  " };
        let color = if is_selected { LAVENDER } else { TEXT };
        
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(MAUVE)),
            Span::styled(pkg, Style::default().fg(color)),
        ]).into()
    }).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(format!("{}Installed Packages ({})", ICON_PACKAGE, app.manager_filtered_pkgs.len()), Style::default().fg(BLUE).add_modifier(Modifier::BOLD)))
                .style(Style::default().bg(ratatui::style::Color::Reset)),
        )
        .highlight_style(
            Style::default()
                .bg(SURFACE0)
                .fg(LAVENDER)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if !app.manager_filtered_pkgs.is_empty() {
        let visible = chunks[1].height.saturating_sub(2) as usize;
        let offset = if app.manager_selected_idx >= visible {
            app.manager_selected_idx - visible + 1
        } else {
            0
        };
        app.manager_scroll; // Just touching to avoid unused warnings
        state.select(Some(app.manager_selected_idx));
        *state.offset_mut() = offset;
    }

    f.render_stateful_widget(list, chunks[1], &mut state);
}

pub fn render_uninstall_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(40, 20, area);
    f.render_widget(Clear, popup_area);

    let pkg_name = app.manager_filtered_pkgs.get(app.manager_selected_idx).map(|s| s.as_str()).unwrap_or("?");
    let text = vec![
        Line::from(Span::styled(format!("Uninstall {}?", pkg_name), Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("[n] ", Style::default().fg(GREEN)),
            Span::styled("Normal remove (-R)", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("[r] ", Style::default().fg(RED)),
            Span::styled("Recursive remove (-Rs)", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("[Esc] ", Style::default().fg(OVERLAY1)),
            Span::styled("Cancel", Style::default().fg(TEXT)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(RED))
                .title("Uninstall")
                .style(Style::default().bg(MANTLE)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, popup_area);
}
