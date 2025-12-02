//! Selectable menu widget

use ratatui::{
    prelude::*,
    widgets::Widget,
};

use crate::tui::theme::Theme;

/// A menu item with icon, label, and optional description
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub icon: String,
    pub label: String,
    pub description: Option<String>,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
            description: None,
            enabled: true,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Selectable menu widget
pub struct Menu<'a> {
    items: &'a [MenuItem],
    selected: usize,
    show_descriptions: bool,
}

impl<'a> Menu<'a> {
    pub fn new(items: &'a [MenuItem], selected: usize) -> Self {
        Self {
            items,
            selected,
            show_descriptions: true,
        }
    }

    pub const fn hide_descriptions(mut self) -> Self {
        self.show_descriptions = false;
        self
    }

    /// Calculate the height needed for this menu
    #[allow(dead_code)]
    pub fn height(&self) -> u16 {
        let mut height = 0u16;
        for item in self.items {
            height += 1; // Main line
            if self.show_descriptions && item.description.is_some() {
                height += 1; // Description line
            }
            height += 1; // Spacing
        }
        height
    }
}

impl Widget for Menu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y;

        for (i, item) in self.items.iter().enumerate() {
            if y >= area.y + area.height {
                break;
            }

            let is_selected = i == self.selected;
            let is_enabled = item.enabled;

            // Selector indicator
            let selector = if is_selected { "▸" } else { " " };

            // Build the main line
            let style = if !is_enabled {
                Theme::text_dim()
            } else if is_selected {
                Theme::selected()
            } else {
                Theme::text()
            };

            let line = format!("  {}  {}  {}", selector, item.icon, item.label);
            let disabled_suffix = if !is_enabled { " [Coming Soon]" } else { "" };
            let full_line = format!("{line}{disabled_suffix}");

            buf.set_string(area.x, y, &full_line, style);
            y += 1;

            // Description line (if enabled and present)
            if self.show_descriptions {
                if let Some(desc) = &item.description {
                    if y < area.y + area.height {
                        let desc_style = if is_selected {
                            Theme::text_muted()
                        } else {
                            Theme::text_dim()
                        };
                        let desc_line = format!("        {desc}");
                        buf.set_string(area.x, y, &desc_line, desc_style);
                        y += 1;
                    }
                }
            }

            // Add spacing between items
            y += 1;
        }
    }
}

