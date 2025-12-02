//! Installation progress widget

use ratatui::{
    prelude::*,
    widgets::{Gauge, Widget},
};

use crate::tui::theme::Theme;

/// Status of a component installation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ComponentStatus {
    Pending,
    Installing,
    Completed,
    Failed,
    Skipped,
}

impl ComponentStatus {
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Installing => "◉",
            Self::Completed => "✓",
            Self::Failed => "✗",
            Self::Skipped => "⊘",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            Self::Pending => Theme::status_pending(),
            Self::Installing => Theme::status_installing(),
            Self::Completed => Theme::status_completed(),
            Self::Failed => Theme::status_failed(),
            Self::Skipped => Theme::text_dim(),
        }
    }
}

/// A component being installed
#[derive(Debug, Clone)]
pub struct ProgressItem {
    pub name: String,
    pub status: ComponentStatus,
    pub elapsed_secs: Option<u64>,
    pub message: Option<String>,
}

#[allow(dead_code)]
impl ProgressItem {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ComponentStatus::Pending,
            elapsed_secs: None,
            message: None,
        }
    }

    pub fn installing(mut self) -> Self {
        self.status = ComponentStatus::Installing;
        self
    }

    pub fn completed(mut self, elapsed_secs: u64) -> Self {
        self.status = ComponentStatus::Completed;
        self.elapsed_secs = Some(elapsed_secs);
        self
    }

    pub fn failed(mut self, message: impl Into<String>) -> Self {
        self.status = ComponentStatus::Failed;
        self.message = Some(message.into());
        self
    }
}

/// Installation progress list widget
#[allow(dead_code)]
pub struct InstallProgress<'a> {
    items: &'a [ProgressItem],
    current_index: usize,
}

#[allow(dead_code)]
impl<'a> InstallProgress<'a> {
    pub fn new(items: &'a [ProgressItem], current_index: usize) -> Self {
        Self {
            items,
            current_index,
        }
    }

    /// Calculate progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let completed = self.items.iter().filter(|i| {
            matches!(i.status, ComponentStatus::Completed | ComponentStatus::Skipped)
        }).count();

        (completed as f64 / self.items.len() as f64) * 100.0
    }
}

impl Widget for InstallProgress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y;

        for item in self.items.iter() {
            if y >= area.y + area.height {
                break;
            }

            let icon = item.status.icon();
            let style = item.status.style();

            // Status icon
            buf.set_string(area.x + 2, y, icon, style);

            // Component name
            let name_style = if matches!(item.status, ComponentStatus::Installing) {
                Theme::selected()
            } else {
                Theme::text()
            };
            buf.set_string(area.x + 5, y, &item.name, name_style);

            // Elapsed time or message
            let suffix_x = area.x + 5 + item.name.len() as u16 + 2;
            if let Some(elapsed) = item.elapsed_secs {
                let time_str = format_duration(elapsed);
                buf.set_string(suffix_x, y, &format!("[{time_str}]"), Theme::text_dim());
            } else if matches!(item.status, ComponentStatus::Installing) {
                buf.set_string(suffix_x, y, "installing...", Theme::text_muted());
            }

            // Error message on next line if failed
            if let Some(msg) = &item.message {
                if matches!(item.status, ComponentStatus::Failed) {
                    y += 1;
                    if y < area.y + area.height {
                        let truncated = if msg.len() > (area.width as usize - 7) {
                            format!("{}...", &msg[..area.width as usize - 10])
                        } else {
                            msg.clone()
                        };
                        buf.set_string(area.x + 5, y, &truncated, Theme::error());
                    }
                }
            }

            y += 1;
        }
    }
}

/// Progress bar widget
pub struct ProgressBar {
    percent: f64,
    label: Option<String>,
}

#[allow(dead_code)]
impl ProgressBar {
    pub fn new(percent: f64) -> Self {
        Self {
            percent,
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = self.label.unwrap_or_else(|| format!("{:.0}%", self.percent));

        let gauge = Gauge::default()
            .gauge_style(Theme::progress_bar())
            .ratio(self.percent / 100.0)
            .label(label);

        gauge.render(area, buf);
    }
}

/// Elapsed and estimated time display
pub struct TimeDisplay {
    elapsed_secs: u64,
    estimated_remaining_secs: Option<u64>,
}

impl TimeDisplay {
    pub fn new(elapsed_secs: u64, estimated_remaining_secs: Option<u64>) -> Self {
        Self {
            elapsed_secs,
            estimated_remaining_secs,
        }
    }
}

impl Widget for TimeDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let elapsed = format_duration(self.elapsed_secs);
        let mut text = format!("Elapsed: {elapsed}");

        if let Some(remaining) = self.estimated_remaining_secs {
            let remaining_str = format_duration(remaining);
            text.push_str(&format!("  |  Estimated remaining: {remaining_str}"));
        }

        buf.set_string(area.x, area.y, &text, Theme::text_muted());
    }
}

/// Format a duration in seconds to MM:SS format
fn format_duration(secs: u64) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{mins:02}:{secs:02}")
}

