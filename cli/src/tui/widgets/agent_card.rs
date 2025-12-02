//! Agent card widget with ASCII art and speech bubble

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::agent_defs::Agent;
use crate::tui::theme::Theme;

/// Agent card widget displaying an agent with their ASCII art and a message
pub struct AgentCard<'a> {
    agent: &'a Agent,
    message: &'a str,
    show_border: bool,
}

#[allow(dead_code)]
impl<'a> AgentCard<'a> {
    pub fn new(agent: &'a Agent, message: &'a str) -> Self {
        Self {
            agent,
            message,
            show_border: true,
        }
    }

    pub const fn without_border(mut self) -> Self {
        self.show_border = false;
        self
    }

    /// Calculate the height needed for this card
    pub fn height(&self) -> u16 {
        let art_lines = self.agent.ascii_art.lines().count() as u16;
        let message_lines = 2; // Estimate for wrapped message
        art_lines + message_lines + 4 // padding
    }
}

impl Widget for AgentCard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the content
        let agent_style = Theme::agent(self.agent.name);

        // Build the card content
        let mut lines = Vec::new();

        // Agent name and role header
        lines.push(Line::from(vec![
            Span::styled(self.agent.name, agent_style.add_modifier(Modifier::BOLD)),
            Span::styled(" - ", Theme::text_dim()),
            Span::styled(self.agent.role, Theme::text_muted()),
        ]));

        lines.push(Line::from(""));

        // Speech bubble with message
        lines.push(Line::from(vec![
            Span::styled("\"", Theme::text_dim()),
            Span::styled(self.message, Theme::text()),
            Span::styled("\"", Theme::text_dim()),
        ]));

        lines.push(Line::from(""));

        // ASCII art
        for line in self.agent.ascii_art.lines() {
            lines.push(Line::from(Span::styled(line, agent_style)));
        }

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        if self.show_border {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::border())
                .border_type(ratatui::widgets::BorderType::Rounded);

            let inner = block.inner(area);
            block.render(area, buf);
            paragraph.render(inner, buf);
        } else {
            paragraph.render(area, buf);
        }
    }
}

/// Agent grid widget for displaying multiple agents
pub struct AgentGrid<'a> {
    agents: &'a [&'a Agent],
    columns: u16,
}

impl<'a> AgentGrid<'a> {
    pub fn new(agents: &'a [&'a Agent], columns: u16) -> Self {
        Self { agents, columns }
    }
}

impl Widget for AgentGrid<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cell_width = area.width / self.columns;
        let cell_height = 6; // Compact agent display

        for (i, agent) in self.agents.iter().enumerate() {
            let col = (i as u16) % self.columns;
            let row = (i as u16) / self.columns;

            let x = area.x + col * cell_width;
            let y = area.y + row * cell_height;

            if y + cell_height > area.y + area.height {
                break;
            }

            let cell_area = Rect::new(x, y, cell_width, cell_height);
            AgentMini::new(agent).render(cell_area, buf);
        }
    }
}

/// Compact agent display for grids
pub struct AgentMini<'a> {
    agent: &'a Agent,
}

impl<'a> AgentMini<'a> {
    pub const fn new(agent: &'a Agent) -> Self {
        Self { agent }
    }
}

impl Widget for AgentMini<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let agent_style = Theme::agent(self.agent.name);

        // Icon
        let icon_x = area.x + (area.width / 2).saturating_sub(1);
        buf.set_string(icon_x, area.y, self.agent.icon, agent_style);

        // Name
        let name_x = area.x + (area.width / 2).saturating_sub(self.agent.name.len() as u16 / 2);
        buf.set_string(
            name_x,
            area.y + 2,
            self.agent.name,
            agent_style.add_modifier(Modifier::BOLD),
        );

        // Role (short)
        let role_short = self.agent.role_short();
        let role_x = area.x + (area.width / 2).saturating_sub(role_short.len() as u16 / 2);
        buf.set_string(role_x, area.y + 3, role_short, Theme::text_muted());
    }
}

