//! Welcome screen with Rex introduction

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

use crate::agent_defs::get_agent;
use crate::tui::app::MainMenuOption;
use crate::tui::theme::Theme;
use crate::tui::widgets::{AgentCard, Banner, HelpBar, Menu, MenuItem, Subtitle};

/// Welcome screen state
pub struct WelcomeScreen {
    /// Currently selected menu item
    selected: usize,
    /// Animation tick counter
    tick_count: u64,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            tick_count: 0,
        }
    }

    /// Handle tick for animations
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MainMenuOption> {
        let options = MainMenuOption::all();

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
            KeyCode::Enter => Some(options[self.selected]),
            KeyCode::Char('1') => Some(MainMenuOption::InstallPlatform),
            KeyCode::Char('2') => Some(MainMenuOption::InstallCliOnly),
            KeyCode::Char('3') => Some(MainMenuOption::CheckRequirements),
            KeyCode::Char('4') => Some(MainMenuOption::ViewDocs),
            KeyCode::Char('5') | KeyCode::Esc => Some(MainMenuOption::Exit),
            _ => None,
        }
    }

    /// Draw the welcome screen
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear with background color
        frame.render_widget(
            ratatui::widgets::Block::default().style(Style::default().bg(Theme::BACKGROUND)),
            area,
        );

        // Layout: Banner + Subtitle | Agent Card | Menu | Help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Banner (7 lines + 1 padding)
                Constraint::Length(1),  // Subtitle
                Constraint::Length(10), // Agent card
                Constraint::Length(2),  // Title
                Constraint::Min(12),    // Menu
                Constraint::Length(1),  // Help bar
            ])
            .split(area);

        // Banner
        frame.render_widget(Banner::new(), chunks[0]);

        // Subtitle
        frame.render_widget(Subtitle::new(env!("CARGO_PKG_VERSION")), chunks[1]);

        // Agent card (Rex)
        if let Some(rex) = get_agent("Rex") {
            let card = AgentCard::new(rex, rex.greeting);
            let card_area = centered_rect(60, 100, chunks[2]);
            frame.render_widget(card, card_area);
        }

        // Menu title
        let title = ratatui::widgets::Paragraph::new("What would you like to do?")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[3]);

        // Menu items
        let menu_items: Vec<MenuItem> = MainMenuOption::all()
            .iter()
            .map(|opt| MenuItem::new(opt.icon(), opt.label()))
            .collect();

        let menu = Menu::new(&menu_items, self.selected).hide_descriptions();
        let menu_area = centered_rect(50, 100, chunks[4]);
        frame.render_widget(menu, menu_area);

        // Help bar
        frame.render_widget(HelpBar::navigation(), chunks[5]);
    }
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

