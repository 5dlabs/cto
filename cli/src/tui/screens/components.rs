//! Component selection screen

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

use crate::tui::app::ComponentCategory;
use crate::tui::theme::Theme;
use crate::tui::widgets::{Banner, Checklist, ChecklistGroup, ChecklistItem, HelpBar};

/// Component selection screen state
pub struct ComponentsScreen {
    /// Component groups
    groups: Vec<ChecklistGroup>,
    /// Currently selected group index
    selected_group: usize,
    /// Currently selected item within group
    selected_item: usize,
    /// Animation tick counter
    tick_count: u64,
}

impl ComponentsScreen {
    pub fn new() -> Self {
        let groups = vec![
            ChecklistGroup::new(
                "CORE PLATFORM (Required)",
                vec![
                    ChecklistItem::new("argocd", "ArgoCD", "GitOps continuous delivery", true),
                    ChecklistItem::new("argo-workflows", "Argo Workflows", "Workflow orchestration", true),
                    ChecklistItem::new("argo-events", "Argo Events", "Event-driven automation", true),
                    ChecklistItem::new("cto-platform", "CTO Platform", "Agent orchestration & tools", true),
                ],
            ),
            ChecklistGroup::new(
                "INFRASTRUCTURE (Recommended)",
                vec![
                    ChecklistItem::new("vault", "Vault", "Secrets management", false),
                    ChecklistItem::new("cert-manager", "Cert-Manager", "TLS certificates", false),
                    ChecklistItem::new("ingress-nginx", "Ingress NGINX", "Load balancing", false),
                    ChecklistItem::new("external-dns", "External DNS", "DNS automation", false),
                ],
            ),
            ChecklistGroup::new(
                "OBSERVABILITY (Optional)",
                vec![
                    ChecklistItem::new("victoria-metrics", "VictoriaMetrics", "Metrics storage", false),
                    ChecklistItem::new("victoria-logs", "VictoriaLogs", "Log aggregation", false),
                    ChecklistItem::new("grafana", "Grafana", "Dashboards & visualization", false),
                    ChecklistItem::new("otel-collector", "OTEL Collector", "Distributed tracing", false),
                ],
            ),
            ChecklistGroup::new(
                "ADDITIONAL (Optional)",
                vec![
                    ChecklistItem::new("minio", "MinIO", "Object storage", false),
                    ChecklistItem::new("postgres-operator", "PostgreSQL Operator", "Database management", false),
                    ChecklistItem::new("redis-operator", "Redis Operator", "Cache management", false),
                    ChecklistItem::new("arc-controller", "ARC Controller", "GitHub runners", false),
                ],
            ),
        ];

        Self {
            groups,
            selected_group: 0,
            selected_item: 0,
            tick_count: 0,
        }
    }

    /// Handle tick for animations
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Check if a category has any selected items
    pub fn is_category_selected(&self, category: ComponentCategory) -> bool {
        let group_idx = match category {
            ComponentCategory::Core => 0,
            ComponentCategory::Infrastructure => 1,
            ComponentCategory::Observability => 2,
            ComponentCategory::Additional => 3,
        };

        if group_idx < self.groups.len() {
            self.groups[group_idx].items.iter().any(|item| item.checked)
        } else {
            false
        }
    }

    /// Get selected component IDs
    #[allow(dead_code)]
    pub fn selected_components(&self) -> Vec<String> {
        self.groups
            .iter()
            .flat_map(|g| g.items.iter())
            .filter(|item| item.checked)
            .map(|item| item.id.clone())
            .collect()
    }

    /// Handle key events, returns action if one should be taken
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&'static str> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                None
            }
            KeyCode::Char(' ') => {
                self.toggle_current();
                None
            }
            KeyCode::Char('a') => {
                self.select_all();
                None
            }
            KeyCode::Char('n') => {
                self.select_none();
                None
            }
            KeyCode::Enter => Some("continue"),
            KeyCode::Esc => Some("back"),
            _ => None,
        }
    }

    /// Move selection up
    fn move_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        } else if self.selected_group > 0 {
            self.selected_group -= 1;
            self.selected_item = self.groups[self.selected_group].items.len().saturating_sub(1);
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        let current_group = &self.groups[self.selected_group];
        if self.selected_item < current_group.items.len().saturating_sub(1) {
            self.selected_item += 1;
        } else if self.selected_group < self.groups.len().saturating_sub(1) {
            self.selected_group += 1;
            self.selected_item = 0;
        }
    }

    /// Toggle the currently selected item
    fn toggle_current(&mut self) {
        if let Some(item) = self.groups
            .get_mut(self.selected_group)
            .and_then(|g| g.items.get_mut(self.selected_item))
        {
            // Don't allow toggling required items
            if !item.required {
                item.checked = !item.checked;
            }
        }
    }

    /// Select all optional items
    fn select_all(&mut self) {
        for group in &mut self.groups {
            for item in &mut group.items {
                item.checked = true;
            }
        }
    }

    /// Deselect all optional items (keep required)
    fn select_none(&mut self) {
        for group in &mut self.groups {
            for item in &mut group.items {
                if !item.required {
                    item.checked = false;
                }
            }
        }
    }

    /// Draw the components screen
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
                Constraint::Length(2),  // Title
                Constraint::Min(20),    // Checklist
                Constraint::Length(2),  // Help bar
            ])
            .split(area);

        // Banner (compact)
        frame.render_widget(Banner::new(), chunks[0]);

        // Title
        let title = ratatui::widgets::Paragraph::new("Select Components to Install")
            .style(Theme::header())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[1]);

        // Checklist
        let checklist = Checklist::new(&self.groups, self.selected_group, self.selected_item);
        let checklist_area = centered_rect(80, 100, chunks[2]);
        frame.render_widget(checklist, checklist_area);

        // Help bar
        frame.render_widget(HelpBar::checklist(), chunks[3]);
    }
}

impl Default for ComponentsScreen {
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

