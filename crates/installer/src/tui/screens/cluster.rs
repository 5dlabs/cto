//! Cluster selection screen with Atlas guidance

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

use crate::agent_defs::get_agent;
use crate::tui::app::ClusterOption;
use crate::tui::theme::Theme;
use crate::tui::widgets::{AgentCard, Banner, HelpBar, Menu, MenuItem};

/// Cluster selection screen state
pub struct ClusterScreen {
    /// Currently selected option
    selected: usize,
    /// Selected cluster option (after confirmation)
    pub selected_option: Option<ClusterOption>,
    /// Animation tick counter
    tick_count: u64,
}

impl ClusterScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            selected_option: None,
            tick_count: 0,
        }
    }

    /// Handle tick for animations
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ClusterOption> {
        let options = ClusterOption::all();

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < options.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => {
                let option = options[self.selected];
                if option.is_available() {
                    Some(option)
                } else {
                    None
                }
            }
            KeyCode::Char('1') => Some(ClusterOption::Kind),
            KeyCode::Char('2') => Some(ClusterOption::Remote),
            _ => None,
        }
    }

    /// Draw the cluster selection screen
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
                Constraint::Length(12), // Agent card
                Constraint::Length(2),  // Title
                Constraint::Min(15),    // Menu
                Constraint::Length(2),  // Help bar
            ])
            .split(area);

        // Banner (compact)
        frame.render_widget(Banner::new(), chunks[0]);

        // Agent card (Atlas)
        if let Some(atlas) = get_agent("Atlas") {
            let card = AgentCard::new(atlas, atlas.greeting);
            let card_area = centered_rect(60, 100, chunks[1]);
            frame.render_widget(card, card_area);
        }

        // Title
        let title = ratatui::widgets::Paragraph::new("Choose Your Environment")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[2]);

        // Menu items
        let menu_items: Vec<MenuItem> = ClusterOption::all()
            .iter()
            .map(|opt| {
                let mut item = MenuItem::new(opt.icon(), opt.label())
                    .with_description(opt.description());
                if !opt.is_available() {
                    item = item.disabled();
                }
                item
            })
            .collect();

        let menu = Menu::new(&menu_items, self.selected);
        let menu_area = centered_rect(70, 100, chunks[3]);
        frame.render_widget(menu, menu_area);

        // Help bar
        frame.render_widget(HelpBar::navigation(), chunks[4]);
    }
}

impl Default for ClusterScreen {
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

