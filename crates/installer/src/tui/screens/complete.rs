//! Completion screen with "Meet Your Team" agent grid

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::agent_defs::{all_agents, Agent};
use crate::tui::theme::Theme;
use crate::tui::widgets::{AgentGrid, Banner, HelpBar};

/// Completion screen state
pub struct CompleteScreen {
    /// Animation tick counter
    tick_count: u64,
}

impl CompleteScreen {
    pub fn new() -> Self {
        Self { tick_count: 0 }
    }

    /// Handle tick for animations
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&'static str> {
        match key.code {
            KeyCode::Enter => Some("dashboard"),
            KeyCode::Char('c') => Some("copy"),
            KeyCode::Char('q') | KeyCode::Esc => Some("exit"),
            _ => None,
        }
    }

    /// Draw the completion screen
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
                Constraint::Length(3),  // Success message
                Constraint::Length(2),  // Meet your team title
                Constraint::Length(14), // Agent grid
                Constraint::Length(2),  // Getting started title
                Constraint::Min(8),     // Next steps
                Constraint::Length(2),  // Help bar
            ])
            .split(area);

        // Banner
        frame.render_widget(Banner::new(), chunks[0]);

        // Success message
        let success = Paragraph::new("🎉 Installation Complete!")
            .style(Theme::success().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(success, chunks[1]);

        // Meet your team title
        let team_title = Paragraph::new("Meet Your AI Development Team")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(team_title, chunks[2]);

        // Agent grid
        let agents: Vec<&Agent> = all_agents().iter().collect();
        let grid = AgentGrid::new(&agents, 5);
        let grid_area = centered_rect(90, 100, chunks[3]);
        frame.render_widget(grid, grid_area);

        // Getting started title
        let started_title = Paragraph::new("Getting Started")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(started_title, chunks[4]);

        // Next steps
        self.draw_next_steps(frame, chunks[5]);

        // Help bar
        frame.render_widget(HelpBar::complete(), chunks[6]);
    }

    /// Draw the next steps section
    fn draw_next_steps(&self, frame: &mut Frame, area: Rect) {
        let steps = vec![
            ("1.", "Access ArgoCD:", "https://localhost:8080"),
            ("2.", "View Workflows:", "https://localhost:2746"),
            ("3.", "Configure GitHub:", "cto configure github"),
            ("4.", "Start your first task:", "cto task create \"Build login page\""),
        ];

        let inner = centered_rect(70, 100, area);
        let mut y = inner.y;

        for (num, label, value) in steps {
            if y >= inner.y + inner.height {
                break;
            }

            // Number
            let num_span = Span::styled(format!("{num}  "), Theme::primary());
            // Label
            let label_span = Span::styled(format!("{label} "), Theme::text());
            // Value
            let value_span = Span::styled(value, Theme::secondary());

            let line = Line::from(vec![num_span, label_span, value_span]);
            let paragraph = Paragraph::new(line);
            let line_area = Rect::new(inner.x, y, inner.width, 1);
            frame.render_widget(paragraph, line_area);

            y += 2; // Add spacing between steps
        }
    }
}

impl Default for CompleteScreen {
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

