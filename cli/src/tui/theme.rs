//! Color theme for the CTO TUI
//!
//! Uses an Indigo/Violet dark theme inspired by modern developer tools.

use ratatui::style::{Color, Modifier, Style};

/// CTO Platform color theme
#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // ─────────────────────────────────────────────────────────────────────────
    // Base Colors
    // ─────────────────────────────────────────────────────────────────────────

    /// Primary accent color (Indigo-500)
    pub const PRIMARY: Color = Color::Rgb(99, 102, 241);

    /// Secondary accent color (Violet-500)
    pub const SECONDARY: Color = Color::Rgb(139, 92, 246);

    /// Success color (Emerald-500)
    pub const SUCCESS: Color = Color::Rgb(16, 185, 129);

    /// Warning color (Amber-500)
    pub const WARNING: Color = Color::Rgb(245, 158, 11);

    /// Error color (Red-500)
    pub const ERROR: Color = Color::Rgb(239, 68, 68);

    /// Background color (Slate-900)
    pub const BACKGROUND: Color = Color::Rgb(15, 23, 42);

    /// Surface color (Slate-800)
    pub const SURFACE: Color = Color::Rgb(30, 41, 59);

    /// Border color (Slate-700)
    pub const BORDER: Color = Color::Rgb(51, 65, 85);

    /// Primary text color (Slate-50)
    pub const TEXT: Color = Color::Rgb(248, 250, 252);

    /// Secondary/muted text color (Slate-500)
    pub const TEXT_MUTED: Color = Color::Rgb(100, 116, 139);

    /// Dimmed text color (Slate-600)
    pub const TEXT_DIM: Color = Color::Rgb(71, 85, 105);

    /// Highlight background (Slate-700)
    pub const HIGHLIGHT_BG: Color = Color::Rgb(51, 65, 85);

    // ─────────────────────────────────────────────────────────────────────────
    // Agent Colors
    // ─────────────────────────────────────────────────────────────────────────

    /// Rex - Lead Developer (Green)
    pub const AGENT_REX: Color = Color::Rgb(34, 197, 94);

    /// Cleo - Code Reviewer (Blue)
    pub const AGENT_CLEO: Color = Color::Rgb(59, 130, 246);

    /// Blaze - Frontend Dev (Orange)
    pub const AGENT_BLAZE: Color = Color::Rgb(249, 115, 22);

    /// Tess - QA Engineer (Cyan)
    pub const AGENT_TESS: Color = Color::Rgb(6, 182, 212);

    /// Cipher - Security (Purple)
    pub const AGENT_CIPHER: Color = Color::Rgb(168, 85, 247);

    /// Morgan - Documentation (Pink)
    pub const AGENT_MORGAN: Color = Color::Rgb(236, 72, 153);

    /// Atlas - Infrastructure (Indigo)
    pub const AGENT_ATLAS: Color = Color::Rgb(99, 102, 241);

    /// Bolt - DevOps (Yellow)
    pub const AGENT_BOLT: Color = Color::Rgb(234, 179, 8);

    /// Stitch - PR Review (Teal)
    pub const AGENT_STITCH: Color = Color::Rgb(20, 184, 166);

    // ─────────────────────────────────────────────────────────────────────────
    // Styles
    // ─────────────────────────────────────────────────────────────────────────

    /// Default text style
    pub fn text() -> Style {
        Style::default().fg(Self::TEXT)
    }

    /// Muted text style
    pub fn text_muted() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }

    /// Dimmed text style
    pub fn text_dim() -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    /// Primary accent style
    pub fn primary() -> Style {
        Style::default().fg(Self::PRIMARY)
    }

    /// Secondary accent style
    pub fn secondary() -> Style {
        Style::default().fg(Self::SECONDARY)
    }

    /// Success style
    pub fn success() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    /// Warning style
    pub fn warning() -> Style {
        Style::default().fg(Self::WARNING)
    }

    /// Error style
    pub fn error() -> Style {
        Style::default().fg(Self::ERROR)
    }

    /// Title style (bold primary)
    pub fn title() -> Style {
        Style::default()
            .fg(Self::TEXT)
            .add_modifier(Modifier::BOLD)
    }

    /// Subtitle style
    pub fn subtitle() -> Style {
        Style::default().fg(Self::PRIMARY)
    }

    /// Header style (bold)
    pub fn header() -> Style {
        Style::default()
            .fg(Self::TEXT)
            .add_modifier(Modifier::BOLD)
    }

    /// Selected item style
    pub fn selected() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Highlighted item style
    pub fn highlighted() -> Style {
        Style::default()
            .fg(Self::TEXT)
            .bg(Self::HIGHLIGHT_BG)
    }

    /// Border style
    pub fn border() -> Style {
        Style::default().fg(Self::BORDER)
    }

    /// Active border style
    pub fn border_active() -> Style {
        Style::default().fg(Self::PRIMARY)
    }

    /// Progress bar style
    pub fn progress_bar() -> Style {
        Style::default().fg(Self::PRIMARY)
    }

    /// Progress bar background style
    pub fn progress_bar_bg() -> Style {
        Style::default().fg(Self::SURFACE)
    }

    /// Checkbox checked style
    pub fn checkbox_checked() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    /// Checkbox unchecked style
    pub fn checkbox_unchecked() -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    /// Help text style
    pub fn help() -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    /// Help key style
    pub fn help_key() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Status pending style
    pub fn status_pending() -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    /// Status installing style
    pub fn status_installing() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Status completed style
    pub fn status_completed() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    /// Status failed style
    pub fn status_failed() -> Style {
        Style::default().fg(Self::ERROR)
    }

    /// Agent style by name
    pub fn agent(name: &str) -> Style {
        let color = match name.to_lowercase().as_str() {
            "rex" => Self::AGENT_REX,
            "cleo" => Self::AGENT_CLEO,
            "blaze" => Self::AGENT_BLAZE,
            "tess" => Self::AGENT_TESS,
            "cipher" => Self::AGENT_CIPHER,
            "morgan" => Self::AGENT_MORGAN,
            "atlas" => Self::AGENT_ATLAS,
            "bolt" => Self::AGENT_BOLT,
            "stitch" => Self::AGENT_STITCH,
            _ => Self::PRIMARY,
        };
        Style::default().fg(color)
    }
}

