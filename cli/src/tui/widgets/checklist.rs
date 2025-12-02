//! Component checklist widget

use ratatui::{
    prelude::*,
    widgets::Widget,
};

use crate::tui::theme::Theme;

/// A checklist item
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChecklistItem {
    pub id: String,
    pub label: String,
    pub description: String,
    pub checked: bool,
    pub required: bool,
}

impl ChecklistItem {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            checked: required, // Required items start checked
            required,
        }
    }
}

/// A group of checklist items
#[derive(Debug, Clone)]
pub struct ChecklistGroup {
    pub title: String,
    pub items: Vec<ChecklistItem>,
}

impl ChecklistGroup {
    pub fn new(title: impl Into<String>, items: Vec<ChecklistItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// Checklist widget with groups
pub struct Checklist<'a> {
    groups: &'a [ChecklistGroup],
    selected_group: usize,
    selected_item: usize,
}

impl<'a> Checklist<'a> {
    pub fn new(groups: &'a [ChecklistGroup], selected_group: usize, selected_item: usize) -> Self {
        Self {
            groups,
            selected_group,
            selected_item,
        }
    }

    /// Calculate the height needed for this checklist
    #[allow(dead_code)]
    pub fn height(&self) -> u16 {
        let mut height = 0u16;
        for group in self.groups {
            height += 3; // Title + separator + spacing
            height += group.items.len() as u16;
            height += 1; // Group spacing
        }
        height
    }
}

impl Widget for Checklist<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y;

        for (group_idx, group) in self.groups.iter().enumerate() {
            if y >= area.y + area.height {
                break;
            }

            // Group title
            let title_style = Theme::header();
            buf.set_string(area.x + 2, y, &group.title, title_style);
            y += 1;

            // Separator
            let separator = "─".repeat((area.width as usize).saturating_sub(4));
            buf.set_string(area.x + 2, y, &separator, Theme::border());
            y += 1;

            // Items
            for (item_idx, item) in group.items.iter().enumerate() {
                if y >= area.y + area.height {
                    break;
                }

                let is_selected = group_idx == self.selected_group && item_idx == self.selected_item;

                // Checkbox
                let checkbox = if item.checked {
                    "[✓]"
                } else {
                    "[ ]"
                };

                let checkbox_style = if item.checked {
                    Theme::checkbox_checked()
                } else {
                    Theme::checkbox_unchecked()
                };

                // Label style
                let label_style = if is_selected {
                    Theme::selected()
                } else if item.required {
                    Theme::text()
                } else {
                    Theme::text_muted()
                };

                // Selection indicator
                let selector = if is_selected { "▸" } else { " " };

                // Build the line
                let x = area.x + 2;
                buf.set_string(x, y, selector, Theme::primary());
                buf.set_string(x + 2, y, checkbox, checkbox_style);
                buf.set_string(x + 6, y, &item.label, label_style);

                // Description (truncated if needed)
                let desc_x = x + 6 + item.label.len() as u16 + 2;
                let available_width = area.width.saturating_sub(desc_x - area.x + 2);
                if available_width > 10 {
                    let desc = if item.description.len() > available_width as usize {
                        format!("{}...", &item.description[..available_width as usize - 3])
                    } else {
                        item.description.clone()
                    };
                    buf.set_string(desc_x, y, &desc, Theme::text_dim());
                }

                y += 1;
            }

            // Group spacing
            y += 1;
        }
    }
}

