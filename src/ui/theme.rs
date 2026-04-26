// Catppuccin Mocha palette + Nerd Font icons
use ratatui::style::Color;

// --- Catppuccin Mocha ---
pub const BASE: Color = Color::Rgb(30, 30, 46);       // #1e1e2e - used as transparent fallback
pub const MANTLE: Color = Color::Rgb(24, 24, 37);     // #181825
pub const CRUST: Color = Color::Rgb(17, 17, 27);      // #11111b
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);   // #313244
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);   // #45475a
pub const SURFACE2: Color = Color::Rgb(88, 91, 112);  // #585b70
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);// #6c7086
pub const OVERLAY1: Color = Color::Rgb(127, 132, 156);// #7f849c
pub const OVERLAY2: Color = Color::Rgb(147, 153, 178);// #9399b2
pub const TEXT: Color = Color::Rgb(205, 214, 244);    // #cdd6f4
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);// #a6adc8
pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);// #bac2de
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);// #b4befe
pub const BLUE: Color = Color::Rgb(137, 180, 250);    // #89b4fa
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);// #74c7ec
pub const SKY: Color = Color::Rgb(137, 220, 235);     // #89dceb
pub const TEAL: Color = Color::Rgb(148, 226, 213);    // #94e2d5
pub const GREEN: Color = Color::Rgb(166, 227, 161);   // #a6e3a1
pub const YELLOW: Color = Color::Rgb(249, 226, 175);  // #f9e2af
pub const PEACH: Color = Color::Rgb(250, 179, 135);   // #fab387
pub const MAROON: Color = Color::Rgb(235, 160, 172);  // #eba0ac
pub const RED: Color = Color::Rgb(243, 139, 168);     // #f38ba8
pub const MAUVE: Color = Color::Rgb(203, 166, 247);   // #cba6f7
pub const PINK: Color = Color::Rgb(245, 194, 231);    // #f5c2e7
pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);// #f2cdcd
pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);// #f5e0dc

// --- Nerd Font Icons ---
pub const ICON_SEARCH: &str = " ";
pub const ICON_PACKAGE: &str = "󰏗 ";
pub const ICON_SECURITY: &str = "󱗂 ";
pub const ICON_VOTES: &str = " ";
pub const ICON_STAR: &str = "󰓎 ";
pub const ICON_INSTALL: &str = "󰇚 ";
pub const ICON_COMMENT: &str = " ";
pub const ICON_DATE: &str = " ";
pub const ICON_USER: &str = " ";
pub const ICON_LINK: &str = "󰌹 ";
pub const ICON_CHECK: &str = " ";
pub const ICON_WARN: &str = " ";
pub const ICON_DANGER: &str = " ";
pub const ICON_SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const ICON_AUR: &str = "󰣇 ";
pub const ICON_PKGBUILD: &str = " ";
pub const ICON_UP: &str = "󰁝 ";
pub const ICON_HELP: &str = "󰋖 ";
pub const ICON_QUIT: &str = "󰩈 ";

/// Score level for display
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreLevel {
    Safe,
    Warning,
    Danger,
    Unknown,
}

impl ScoreLevel {
    pub fn color(&self) -> Color {
        match self {
            ScoreLevel::Safe => GREEN,
            ScoreLevel::Warning => YELLOW,
            ScoreLevel::Danger => RED,
            ScoreLevel::Unknown => OVERLAY1,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ScoreLevel::Safe => ICON_CHECK,
            ScoreLevel::Warning => ICON_WARN,
            ScoreLevel::Danger => ICON_DANGER,
            ScoreLevel::Unknown => "? ",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ScoreLevel::Safe => "SAFE",
            ScoreLevel::Warning => "CAUTION",
            ScoreLevel::Danger => "DANGER",
            ScoreLevel::Unknown => "UNKNOWN",
        }
    }
}

pub fn score_level(score: u8) -> ScoreLevel {
    match score {
        75..=100 => ScoreLevel::Safe,
        40..=74 => ScoreLevel::Warning,
        _ => ScoreLevel::Danger,
    }
}

/// Build a score bar string like "██████████░░░░"
pub fn score_bar(score: u8, width: usize) -> String {
    let filled = (score as usize * width) / 100;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
