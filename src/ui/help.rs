use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::centered_rect;

pub fn render_help(f: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 70, area);
    f.render_widget(Clear, popup);

    let keys: Vec<(&str, &str)> = vec![
        ("/  or  s", "Focus search bar"),
        ("Tab", "Switch search → results"),
        ("j / ↓", "Next package"),
        ("k / ↑", "Previous package"),
        ("Enter / l / →", "Open detail panel"),
        ("h / ←  or  Esc", "Go back"),
        ("i", "Install selected package"),
        ("p", "View PKGBUILD"),
        ("c", "View comments"),
        ("?", "Toggle this help"),
        ("q / Ctrl+C", "Quit"),
        ("Mouse scroll", "Navigate lists"),
        ("Mouse click", "Select package"),
    ];

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(ICON_AUR, Style::default().fg(MAUVE).add_modifier(Modifier::BOLD)),
            Span::styled(
                "aurdash keybindings",
                Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "─".repeat(40),
            Style::default().fg(SURFACE1),
        )),
        Line::from(""),
    ];

    for (key, desc) in &keys {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", key),
                Style::default().fg(TEAL).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*desc, Style::default().fg(TEXT)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Security score", Style::default().fg(MAUVE).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  75–100",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  SAFE    — no issues found", Style::default().fg(TEXT)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  40–74 ",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  CAUTION — some concerns", Style::default().fg(TEXT)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  0–39  ",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  DANGER  — critical issues", Style::default().fg(TEXT)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Press any key to close",
            Style::default().fg(OVERLAY1).add_modifier(Modifier::ITALIC),
        ),
    ]));

    let help = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MAUVE))
            .title(Span::styled(
                format!("{}Help", ICON_HELP),
                Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(MANTLE)),
    );
    f.render_widget(help, popup);
}
