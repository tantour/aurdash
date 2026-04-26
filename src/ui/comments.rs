use crate::app::{App, LoadState, Panel};
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_comments(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Comments;
    let border_color = if is_active { PEACH } else { SURFACE1 };

    let count = app.comments.len();
    let title = format!(
        "{}Comments{}{}",
        ICON_COMMENT,
        if count > 0 {
            format!(" ({})  [↑↓/jk] navigate  [Enter/click] read full", count)
        } else {
            String::new()
        },
        if is_active { "" } else { "" }
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
            let p = Paragraph::new(Line::from(vec![Span::styled(
                format!("{}Fetching comments…", spinner),
                Style::default().fg(OVERLAY1).add_modifier(Modifier::ITALIC),
            )]))
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

    // Inner area width for word-wrap calculation
    let inner_width = area.width.saturating_sub(4) as usize; // 2 borders + 2 indent
    let inner_height = area.height.saturating_sub(2) as usize;

    // Build all comment "cards" as lines, tracking which card each line belongs to
    // so we can highlight the selected one and scroll correctly.
    let mut all_lines: Vec<(Line<'static>, usize)> = Vec::new(); // (line, comment_idx)

    for (idx, comment) in app.comments.iter().enumerate() {
        let is_selected = idx == app.selected_comment_idx && is_active;
        let (name_color, body_color, bg) = if is_selected {
            (MAUVE, TEXT, Some(SURFACE0))
        } else {
            (LAVENDER, SUBTEXT1, None)
        };

        // Header line
        let mut header = Line::from(vec![
            Span::styled(
                if is_selected { "▶ " } else { "  " },
                Style::default().fg(PEACH),
            ),
            Span::styled(ICON_USER, Style::default().fg(BLUE)),
            Span::styled(
                comment.author.clone(),
                Style::default()
                    .fg(name_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(ICON_DATE, Style::default().fg(OVERLAY2)),
            Span::styled(comment.date.clone(), Style::default().fg(OVERLAY1)),
            if is_selected {
                Span::styled(
                    "  [Enter] expand",
                    Style::default()
                        .fg(PEACH)
                        .add_modifier(Modifier::ITALIC),
                )
            } else {
                Span::raw("")
            },
        ]);
        if let Some(bg) = bg {
            header = header.style(Style::default().bg(bg));
        }
        all_lines.push((header, idx));

        // Body — word-wrap into multiple lines
        let body_text = if comment.body.is_empty() {
            "(no text)".to_string()
        } else {
            comment.body.clone()
        };

        let wrapped = word_wrap(&body_text, inner_width.saturating_sub(2));
        let wrap_count = wrapped.len();
        // Show at most 2 body lines in the list view (truncated with indicator)
        let preview_lines = if is_selected { 3 } else { 2 };
        for (i, wline) in wrapped.into_iter().enumerate().take(preview_lines) {
            let is_last_preview = i == preview_lines - 1 && wrap_count > preview_lines;
            let display = if is_last_preview {
                // Truncate with ellipsis indicator
                let trunc = if wline.len() > inner_width.saturating_sub(6) {
                    format!("{}…", &wline[..inner_width.saturating_sub(7).min(wline.len())])
                } else {
                    wline
                };
                format!("  {}  ↩ more", trunc)
            } else {
                format!("  {}", wline)
            };

            let mut body_line = Line::from(Span::styled(
                display,
                Style::default().fg(if is_last_preview { OVERLAY1 } else { body_color }),
            ));
            if let Some(bg) = bg {
                body_line = body_line.style(Style::default().bg(bg));
            }
            all_lines.push((body_line, idx));
        }

        // Separator
        all_lines.push((
            Line::from(Span::styled(
                " ".repeat(inner_width.max(1)),
                Style::default().fg(SURFACE0),
            )),
            idx,
        ));
    }

    // Auto-scroll to keep selected comment visible
    // Find the first line index belonging to selected_comment_idx
    let selected_line_start = all_lines
        .iter()
        .position(|(_, ci)| *ci == app.selected_comment_idx)
        .unwrap_or(0);

    // Compute scroll offset: ensure selected_line_start is within view
    let scroll = {
        let current = app.comments_scroll;
        if selected_line_start < current {
            selected_line_start
        } else if selected_line_start >= current + inner_height {
            selected_line_start.saturating_sub(inner_height / 2)
        } else {
            current
        }
    };

    let visible_lines: Vec<Line<'static>> = all_lines
        .into_iter()
        .skip(scroll)
        .take(inner_height)
        .map(|(l, _)| l)
        .collect();

    let para = Paragraph::new(Text::from(visible_lines)).block(block);
    f.render_widget(para, area);
}

/// Render the full-comment popup overlay.
pub fn render_comment_popup(f: &mut Frame, app: &App, area: Rect) {
    let Some(comment) = app.comments.get(app.selected_comment_idx) else {
        return;
    };

    let popup = super::centered_rect(75, 75, area);
    f.render_widget(Clear, popup);

    let inner_width = popup.width.saturating_sub(4) as usize;
    let inner_height = popup.height.saturating_sub(2) as usize;

    // Build full wrapped body
    let body_lines = word_wrap(&comment.body, inner_width);
    let total = body_lines.len();

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Author / date header
    lines.push(Line::from(vec![
        Span::styled(ICON_USER, Style::default().fg(BLUE)),
        Span::styled(
            comment.author.clone(),
            Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(ICON_DATE, Style::default().fg(OVERLAY2)),
        Span::styled(comment.date.clone(), Style::default().fg(OVERLAY1)),
    ]));

    // Divider
    lines.push(Line::from(Span::styled(
        "─".repeat(inner_width),
        Style::default().fg(SURFACE1),
    )));
    lines.push(Line::from("")); // blank

    // Full body, scrolled
    let scroll = app.comment_popup_scroll.min(total.saturating_sub(1));
    for line in body_lines.iter().skip(scroll).take(inner_height.saturating_sub(3)) {
        lines.push(Line::from(Span::styled(
            line.clone(),
            Style::default().fg(TEXT),
        )));
    }

    // Scroll indicator at bottom
    if total > inner_height.saturating_sub(3) {
        lines.push(Line::from(Span::styled(
            format!("  ↑↓ scroll  {}/{}  [Esc] close", scroll + 1, total),
            Style::default().fg(OVERLAY1).add_modifier(Modifier::ITALIC),
        )));
    }

    let title = format!(
        "{}Comment — {}  [Esc] close",
        ICON_COMMENT, comment.author
    );

    let para = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PEACH))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(PEACH)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(MANTLE)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(para, popup);
}

/// Simple word-wrap: split text into lines of at most `width` chars.
pub fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
