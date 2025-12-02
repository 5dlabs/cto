//! Installation progress screen with Bolt

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use std::time::Instant;

use crate::agent_defs::get_agent;
use crate::tui::theme::Theme;
use crate::tui::widgets::{
    AgentCard, Banner, ComponentStatus, HelpBar, InstallProgress as ProgressWidget,
    ProgressBar, ProgressItem, TimeDisplay,
};

/// Installation progress screen state
pub struct InstallScreen {
    /// Installation items
    items: Vec<ProgressItem>,
    /// Current item being installed
    current_index: usize,
    /// Whether installation is complete
    is_complete: bool,
    /// Whether installation failed
    has_error: bool,
    /// Installation start time
    start_time: Option<Instant>,
    /// Animation tick counter
    tick_count: u64,
    /// Demo mode (simulated installation)
    demo_mode: bool,
    /// Demo tick for simulating progress
    demo_tick: u64,
}

impl InstallScreen {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: 0,
            is_complete: false,
            has_error: false,
            start_time: None,
            tick_count: 0,
            demo_mode: false,
            demo_tick: 0,
        }
    }

    /// Start the installation process
    pub fn start_installation(&mut self, demo_mode: bool) {
        self.demo_mode = demo_mode;
        self.demo_tick = 0;
        self.current_index = 0;
        self.is_complete = false;
        self.has_error = false;
        self.start_time = Some(Instant::now());

        // Initialize installation items
        self.items = vec![
            ProgressItem::new("Kubernetes cluster"),
            ProgressItem::new("Gateway API CRDs"),
            ProgressItem::new("Cert-Manager"),
            ProgressItem::new("Ingress NGINX"),
            ProgressItem::new("ArgoCD"),
            ProgressItem::new("ArgoCD Image Updater"),
            ProgressItem::new("Argo Workflows"),
            ProgressItem::new("Argo Events"),
            ProgressItem::new("Vault"),
            ProgressItem::new("Vault Secrets Operator"),
            ProgressItem::new("CTO Platform"),
        ];

        // Mark first item as installing
        if !self.items.is_empty() {
            self.items[0].status = ComponentStatus::Installing;
        }
    }

    /// Handle tick for animations and demo progress
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // In demo mode, simulate installation progress
        if self.demo_mode && !self.is_complete && !self.has_error {
            self.demo_tick += 1;

            // Every ~20 ticks (5 seconds at 250ms tick rate), complete current item
            if self.demo_tick % 20 == 0 && self.current_index < self.items.len() {
                // Complete current item
                let elapsed = self.demo_tick / 4; // Rough seconds
                self.items[self.current_index].status = ComponentStatus::Completed;
                self.items[self.current_index].elapsed_secs = Some(elapsed);

                // Move to next item
                self.current_index += 1;

                if self.current_index < self.items.len() {
                    self.items[self.current_index].status = ComponentStatus::Installing;
                } else {
                    self.is_complete = true;
                }
            }
        }
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&'static str> {
        match key.code {
            KeyCode::Enter if self.is_complete => Some("complete"),
            KeyCode::Char('r') if self.has_error => Some("retry"),
            _ => None,
        }
    }

    /// Get the progress percentage
    fn progress_percent(&self) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let completed = self.items.iter().filter(|i| {
            matches!(i.status, ComponentStatus::Completed | ComponentStatus::Skipped)
        }).count();

        (completed as f64 / self.items.len() as f64) * 100.0
    }

    /// Get elapsed time in seconds
    fn elapsed_secs(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    /// Estimate remaining time
    fn estimated_remaining_secs(&self) -> Option<u64> {
        let elapsed = self.elapsed_secs();
        let progress = self.progress_percent();

        if progress > 5.0 && progress < 100.0 {
            let total_estimated = (elapsed as f64 * 100.0 / progress) as u64;
            Some(total_estimated.saturating_sub(elapsed))
        } else {
            None
        }
    }

    /// Draw the installation screen
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear with background color
        frame.render_widget(
            ratatui::widgets::Block::default().style(Style::default().bg(Theme::BACKGROUND)),
            area,
        );

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),  // Banner
                Constraint::Length(10), // Agent card
                Constraint::Length(2),  // Current status
                Constraint::Min(12),    // Progress list
                Constraint::Length(3),  // Progress bar
                Constraint::Length(2),  // Time display
                Constraint::Length(2),  // Help bar
            ])
            .split(area);

        // Banner
        frame.render_widget(Banner::new(), chunks[0]);

        // Agent card (Bolt)
        if let Some(bolt) = get_agent("Bolt") {
            let message = if self.is_complete {
                "All components deployed successfully! ⚡"
            } else if self.has_error {
                "Encountered an error. Press 'r' to retry."
            } else {
                bolt.greeting
            };
            let card = AgentCard::new(bolt, message);
            let card_area = centered_rect(60, 100, chunks[1]);
            frame.render_widget(card, card_area);
        }

        // Current status
        let status_text = if self.is_complete {
            "✓ Installation Complete!".to_string()
        } else if self.has_error {
            "✗ Installation Failed".to_string()
        } else if self.current_index < self.items.len() {
            format!("Installing: {}", self.items[self.current_index].name)
        } else {
            "Preparing...".to_string()
        };

        let status_style = if self.is_complete {
            Theme::success()
        } else if self.has_error {
            Theme::error()
        } else {
            Theme::primary()
        };

        let status = ratatui::widgets::Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        // Progress list
        let progress_widget = ProgressWidget::new(&self.items, self.current_index);
        let progress_area = centered_rect(70, 100, chunks[3]);
        frame.render_widget(progress_widget, progress_area);

        // Progress bar
        let progress_bar = ProgressBar::new(self.progress_percent());
        let bar_area = centered_rect(80, 100, chunks[4]);
        frame.render_widget(progress_bar, bar_area);

        // Time display
        let time_display = TimeDisplay::new(self.elapsed_secs(), self.estimated_remaining_secs());
        let time_area = centered_rect(80, 100, chunks[5]);
        frame.render_widget(time_display, time_area);

        // Help bar
        let help = if self.is_complete {
            HelpBar::new(vec![("Enter", "Continue to secrets setup")])
        } else if self.has_error {
            HelpBar::new(vec![("r", "Retry"), ("q", "Quit")])
        } else {
            HelpBar::progress()
        };
        frame.render_widget(help, chunks[6]);
    }
}

impl Default for InstallScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, _percent_y: u16, r: Rect) -> Rect {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(r)[1]
}

