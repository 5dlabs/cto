//! CTO Platform ASCII art banner widget

use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

use crate::tui::theme::Theme;

/// ASCII art banner for the CTO Platform
pub struct Banner {
    /// Whether to show the full banner or compact version
    compact: bool,
}

#[allow(dead_code)]
impl Banner {
    /// Create a new banner widget
    pub const fn new() -> Self {
        Self { compact: false }
    }

    /// Create a compact banner (for smaller screens)
    pub const fn compact() -> Self {
        Self { compact: true }
    }

    /// Full ASCII art banner (fits ~35 char width)
    const FULL_BANNER: &'static str = r#"
  ██████╗████████╗ ██████╗ 
 ██╔════╝╚══██╔══╝██╔═══██╗
 ██║        ██║   ██║   ██║
 ██║        ██║   ██║   ██║
 ╚██████╗   ██║   ╚██████╔╝
  ╚═════╝   ╚═╝    ╚═════╝ "#;

    /// Compact ASCII art banner (for very narrow terminals)
    const COMPACT_BANNER: &'static str = r#"
  ╔═══╗╔════╗╔═══╗
  ║       ║   ║   ║
  ║       ║   ║   ║
  ╚═══╝   ║   ╚═══╝
"#;

    /// Get the banner text based on mode
    fn banner_text(&self) -> &'static str {
        if self.compact {
            Self::COMPACT_BANNER
        } else {
            Self::FULL_BANNER
        }
    }

    /// Get the height of the banner
    pub fn height(&self) -> u16 {
        if self.compact { 6 } else { 8 }
    }
}

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let banner_text = self.banner_text();

        let paragraph = Paragraph::new(banner_text)
            .style(Theme::primary())
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }
}

/// Subtitle widget showing version and tagline
pub struct Subtitle {
    version: &'static str,
}

impl Subtitle {
    pub const fn new(version: &'static str) -> Self {
        Self { version }
    }
}

impl Widget for Subtitle {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = format!(
            "Multi-Agent Development Platform  •  v{}  •  5D Labs",
            self.version
        );

        let paragraph = Paragraph::new(text)
            .style(Theme::text_muted())
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }
}

/// Help bar at the bottom of the screen
pub struct HelpBar {
    items: Vec<(&'static str, &'static str)>,
}

impl HelpBar {
    pub fn new(items: Vec<(&'static str, &'static str)>) -> Self {
        Self { items }
    }

    /// Default help bar for navigation
    pub fn navigation() -> Self {
        Self::new(vec![
            ("↑/↓", "Navigate"),
            ("Enter", "Select"),
            ("Esc", "Back"),
            ("q", "Quit"),
            ("?", "Help"),
        ])
    }

    /// Help bar for checklist
    pub fn checklist() -> Self {
        Self::new(vec![
            ("↑/↓", "Navigate"),
            ("Space", "Toggle"),
            ("a", "Select All"),
            ("n", "Select None"),
            ("Enter", "Continue"),
        ])
    }

    /// Help bar for installation progress
    pub fn progress() -> Self {
        Self::new(vec![
            ("", "Installing..."),
        ])
    }

    /// Help bar for secrets input
    pub fn secrets() -> Self {
        Self::new(vec![
            ("Tab", "Next Field"),
            ("Enter", "Save"),
            ("Esc", "Skip"),
        ])
    }

    /// Help bar for completion screen
    pub fn complete() -> Self {
        Self::new(vec![
            ("Enter", "Open Dashboard"),
            ("c", "Copy Commands"),
            ("q", "Exit"),
        ])
    }
}

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();

        for (i, (key, desc)) in self.items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Theme::text_dim()));
            }
            spans.push(Span::styled(format!("[{key}]"), Theme::help_key()));
            spans.push(Span::styled(format!(" {desc}"), Theme::help()));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).alignment(Alignment::Center);

        paragraph.render(area, buf);
    }
}

