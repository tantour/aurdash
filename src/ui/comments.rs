use crate::app::{App, LoadState, Panel};
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_comments(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Comments;
    let border_color = if is_active { PEACH } else { SURFACE1 };

    let count = app.comments.len();
    let title = format!(
        "{}Comments{}",
        ICON_COMMENT,
        if count > 0 { format!(" ({})", count) } else { String::new() }
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title,
            Style::default().fg(PEACH).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(ratatui::style::Color::Reset));

    match &app.comments_state {
        LoadState::Loading => {
            let spinner = ICON_SPINNER[app.spinner_frame];
            let p = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{}Fetching comments…", spinner),
                    Style::default().fg(OVERLAY1).add_modifier(Modifier::ITALIC),
                ),
            ]))
            .block(block);
            f.render_widget(p, area);
            return;
        }
        LoadState::Error(e) => {
            let p = Paragraph::new(Line::from(Span::styled(
                format!("Error: {}", e),
                Style::default().fg(RED),
            )))
            .block(block);
            f.render_widget(p, area);
            return;
        }
        _ => {}
    }

    if app.comments.is_empty() {
        let p = Paragraph::new(Line::from(Span::styled(
            "No comments yet",
            Style::default().fg(OVERLAY0).add_modifier(Modifier::ITALIC),
        )))
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = app
        .comments
        .iter()
        .skip(app.comments_scroll)
        .take(visible_height)
        .flat_map(|c| {
            let header = Line::from(vec![
                Span::styled(ICON_USER, Style::default().fg(BLUE)),
                Span::styled(&c.author, Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(ICON_DATE, Style::default().fg(OVERLAY2)),
                Span::styled(&c.date, Style::default().fg(OVERLAY1)),
            ]);
            let body = Line::from(vec![
                Span::raw("  "),
                Span::styled(&c.body, Style::default().fg(SUBTEXT1)),
            ]);
            let sep = Line::from(Span::styled(
                " ",
                Style::default().fg(SURFACE0),
            ));
            vec![
                ListItem::new(header),
                ListItem::new(body),
                ListItem::new(sep),
            ]
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
