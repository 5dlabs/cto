//! Secrets configuration screen with Cipher and Vault integration

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::agent_defs::get_agent;
use crate::tui::theme::Theme;
use crate::tui::widgets::{AgentCard, Banner, HelpBar};

/// A secret field for input
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SecretField {
    name: &'static str,
    label: &'static str,
    description: &'static str,
    value: String,
    is_secret: bool,
}

impl SecretField {
    fn new(name: &'static str, label: &'static str, description: &'static str, is_secret: bool) -> Self {
        Self {
            name,
            label,
            description,
            value: String::new(),
            is_secret,
        }
    }

    /// Get the display value (masked if secret)
    fn display_value(&self) -> String {
        if self.is_secret && !self.value.is_empty() {
            "•".repeat(self.value.len().min(20))
        } else {
            self.value.clone()
        }
    }
}

/// Secrets configuration screen state
pub struct SecretsScreen {
    /// Secret fields
    fields: Vec<SecretField>,
    /// Currently selected field
    selected: usize,
    /// Whether we're in edit mode
    editing: bool,
    /// Animation tick counter
    tick_count: u64,
}

impl SecretsScreen {
    pub fn new() -> Self {
        let fields = vec![
            SecretField::new(
                "OPENAI_API_KEY",
                "OpenAI API Key",
                "Required for GPT models",
                true,
            ),
            SecretField::new(
                "ANTHROPIC_API_KEY",
                "Anthropic API Key",
                "Required for Claude models",
                true,
            ),
            SecretField::new(
                "GITHUB_TOKEN",
                "GitHub Token",
                "Personal access token for GitHub API",
                true,
            ),
            SecretField::new(
                "GHCR_TOKEN",
                "GHCR Token",
                "GitHub Container Registry token",
                true,
            ),
        ];

        Self {
            fields,
            selected: 0,
            editing: false,
            tick_count: 0,
        }
    }

    /// Check if we're in editing mode
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Handle tick for animations
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&'static str> {
        if self.editing {
            match key.code {
                KeyCode::Esc => {
                    self.editing = false;
                    None
                }
                KeyCode::Enter => {
                    self.editing = false;
                    // Move to next field
                    if self.selected < self.fields.len() - 1 {
                        self.selected += 1;
                    }
                    None
                }
                KeyCode::Backspace => {
                    if let Some(field) = self.fields.get_mut(self.selected) {
                        field.value.pop();
                    }
                    None
                }
                KeyCode::Char(c) => {
                    if let Some(field) = self.fields.get_mut(self.selected) {
                        field.value.push(c);
                    }
                    None
                }
                _ => None,
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected < self.fields.len() - 1 {
                        self.selected += 1;
                    }
                    None
                }
                KeyCode::Enter | KeyCode::Char('e') => {
                    self.editing = true;
                    None
                }
                KeyCode::Tab => {
                    if self.selected < self.fields.len() - 1 {
                        self.selected += 1;
                    } else {
                        self.selected = 0;
                    }
                    None
                }
                KeyCode::Char('s') => Some("save"),
                KeyCode::Esc => Some("skip"),
                _ => None,
            }
        }
    }

    /// Save secrets to Vault
    pub async fn save_to_vault(&self) -> Result<()> {
        // TODO: Implement actual Vault integration
        // For now, this is a placeholder

        // 1. Port-forward to Vault service
        // 2. Authenticate with Vault
        // 3. Write secrets to secret/cto/api-keys

        Ok(())
    }

    /// Draw the secrets screen
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
                Constraint::Length(2),  // Title
                Constraint::Min(15),    // Form fields
                Constraint::Length(2),  // Help bar
            ])
            .split(area);

        // Banner
        frame.render_widget(Banner::new(), chunks[0]);

        // Agent card (Cipher)
        if let Some(cipher) = get_agent("Cipher") {
            let card = AgentCard::new(cipher, cipher.greeting);
            let card_area = centered_rect(60, 100, chunks[1]);
            frame.render_widget(card, card_area);
        }

        // Title
        let title = Paragraph::new("Configure API Keys")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[2]);

        // Form fields
        let form_area = centered_rect(70, 100, chunks[3]);
        self.draw_form(frame, form_area);

        // Help bar
        let help = if self.editing {
            HelpBar::new(vec![
                ("Type", "Enter value"),
                ("Enter", "Confirm"),
                ("Esc", "Cancel"),
            ])
        } else {
            HelpBar::secrets()
        };
        frame.render_widget(help, chunks[4]);
    }

    /// Draw the form fields
    fn draw_form(&self, frame: &mut Frame, area: Rect) {
        let field_height = 4u16;
        let mut y = area.y;

        for (i, field) in self.fields.iter().enumerate() {
            if y + field_height > area.y + area.height {
                break;
            }

            let is_selected = i == self.selected;
            let field_area = Rect::new(area.x, y, area.width, field_height);

            self.draw_field(frame, field_area, field, is_selected);
            y += field_height;
        }

        // Skip hint at the bottom
        if y + 2 <= area.y + area.height {
            let hint = Paragraph::new("Press [Esc] to skip and configure later")
                .style(Theme::text_dim())
                .alignment(Alignment::Center);
            let hint_area = Rect::new(area.x, y + 1, area.width, 1);
            frame.render_widget(hint, hint_area);
        }
    }

    /// Draw a single form field
    fn draw_field(&self, frame: &mut Frame, area: Rect, field: &SecretField, is_selected: bool) {
        let border_style = if is_selected && self.editing {
            Theme::border_active()
        } else if is_selected {
            Theme::primary()
        } else {
            Theme::border()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(field.label)
            .title_style(if is_selected { Theme::selected() } else { Theme::text() });

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Value or placeholder
        let display = if field.value.is_empty() {
            field.description.to_string()
        } else {
            field.display_value()
        };

        let value_style = if field.value.is_empty() {
            Theme::text_dim()
        } else {
            Theme::text()
        };

        // Add cursor if editing
        let text = if is_selected && self.editing {
            format!("{display}▌")
        } else {
            display
        };

        let value = Paragraph::new(text).style(value_style);
        frame.render_widget(value, inner);
    }
}

impl Default for SecretsScreen {
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

