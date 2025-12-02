//! Application state machine for the TUI installer

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

use super::event::{Event, EventHandler};
use super::screens::{
    ClusterScreen, CompleteScreen, ComponentsScreen, InstallScreen, SecretsScreen, WelcomeScreen,
};
use super::Tui;
use crate::config::{ClusterType, InstallConfig, InstallProfile};

/// Result type for app operations
pub type AppResult<T> = Result<T>;

/// Application screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Cluster,
    Components,
    Install,
    Secrets,
    Complete,
}

/// Main menu options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuOption {
    InstallPlatform,
    InstallCliOnly,
    CheckRequirements,
    ViewDocs,
    Exit,
}

impl MainMenuOption {
    pub const fn all() -> [Self; 5] {
        [
            Self::InstallPlatform,
            Self::InstallCliOnly,
            Self::CheckRequirements,
            Self::ViewDocs,
            Self::Exit,
        ]
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::InstallPlatform => "Install CTO Platform",
            Self::InstallCliOnly => "Install CLI Only",
            Self::CheckRequirements => "Check System Requirements",
            Self::ViewDocs => "View Documentation",
            Self::Exit => "Exit",
        }
    }

    pub const fn icon(&self) -> &'static str {
        match self {
            Self::InstallPlatform => "🚀",
            Self::InstallCliOnly => "🔧",
            Self::CheckRequirements => "📊",
            Self::ViewDocs => "📖",
            Self::Exit => "❌",
        }
    }
}

/// Cluster options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterOption {
    Kind,
    Remote,
    Talos,
}

impl ClusterOption {
    pub const fn all() -> [Self; 3] {
        [Self::Kind, Self::Remote, Self::Talos]
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Kind => "Local Development (Kind)",
            Self::Remote => "Existing Kubernetes Cluster",
            Self::Talos => "Bare Metal (Talos) [Coming Soon]",
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            Self::Kind => "Perfect for trying out CTO on your machine",
            Self::Remote => "Deploy to your existing cluster",
            Self::Talos => "Full production setup with Talos Linux",
        }
    }

    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Kind => "🏠",
            Self::Remote => "☁️",
            Self::Talos => "🔧",
        }
    }

    pub const fn is_available(&self) -> bool {
        !matches!(self, Self::Talos)
    }
}

/// Component categories for installation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ComponentCategory {
    Core,
    Infrastructure,
    Observability,
    Additional,
}

impl ComponentCategory {
    #[allow(dead_code)]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Core => "CORE PLATFORM (Required)",
            Self::Infrastructure => "INFRASTRUCTURE (Recommended)",
            Self::Observability => "OBSERVABILITY (Optional)",
            Self::Additional => "ADDITIONAL (Optional)",
        }
    }
}

/// Individual component
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Component {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: ComponentCategory,
    pub required: bool,
    pub selected: bool,
}

impl Component {
    #[allow(dead_code)]
    pub const fn new(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        category: ComponentCategory,
        required: bool,
    ) -> Self {
        Self {
            id,
            name,
            description,
            category,
            required,
            selected: required,
        }
    }
}

/// Installation status for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InstallStatus {
    Pending,
    Installing,
    Completed,
    Failed,
    Skipped,
}

/// Installation progress entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InstallProgress {
    pub component: String,
    pub status: InstallStatus,
    pub elapsed_secs: Option<u64>,
    pub message: Option<String>,
}

/// Application state
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,

    /// Current screen
    pub current_screen: Screen,

    /// Demo mode (no actual installation)
    pub demo_mode: bool,

    /// Welcome screen state
    pub welcome: WelcomeScreen,

    /// Cluster selection screen state
    pub cluster: ClusterScreen,

    /// Components selection screen state
    pub components: ComponentsScreen,

    /// Installation progress screen state
    pub install: InstallScreen,

    /// Secrets configuration screen state
    pub secrets: SecretsScreen,

    /// Completion screen state
    pub complete: CompleteScreen,

    /// Final installation config
    pub install_config: Option<InstallConfig>,
}

impl App {
    /// Create a new application instance
    pub fn new(demo_mode: bool) -> Self {
        Self {
            should_quit: false,
            current_screen: Screen::Welcome,
            demo_mode,
            welcome: WelcomeScreen::new(),
            cluster: ClusterScreen::new(),
            components: ComponentsScreen::new(),
            install: InstallScreen::new(),
            secrets: SecretsScreen::new(),
            complete: CompleteScreen::new(),
            install_config: None,
        }
    }

    /// Run the application main loop
    pub async fn run(&mut self, terminal: &mut Tui, mut events: EventHandler) -> AppResult<()> {
        while !self.should_quit {
            // Draw the current screen
            terminal.draw(|frame| self.draw(frame))?;

            // Handle events
            match events.next().await? {
                Event::Tick => self.tick(),
                Event::Key(key_event) => self.handle_key_event(key_event).await?,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        Ok(())
    }

    /// Draw the current screen
    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            Screen::Welcome => self.welcome.draw(frame),
            Screen::Cluster => self.cluster.draw(frame),
            Screen::Components => self.components.draw(frame),
            Screen::Install => self.install.draw(frame),
            Screen::Secrets => self.secrets.draw(frame),
            Screen::Complete => self.complete.draw(frame),
        }
    }

    /// Handle tick events (for animations)
    fn tick(&mut self) {
        match self.current_screen {
            Screen::Welcome => self.welcome.tick(),
            Screen::Cluster => self.cluster.tick(),
            Screen::Components => self.components.tick(),
            Screen::Install => self.install.tick(),
            Screen::Secrets => self.secrets.tick(),
            Screen::Complete => self.complete.tick(),
        }
    }

    /// Handle key events
    async fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<()> {
        // Global quit shortcuts
        if key.code == KeyCode::Char('q') && !self.is_input_mode() {
            self.should_quit = true;
            return Ok(());
        }

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        // Delegate to current screen
        match self.current_screen {
            Screen::Welcome => {
                if let Some(action) = self.welcome.handle_key(key) {
                    self.handle_welcome_action(action);
                }
            }
            Screen::Cluster => {
                if let Some(action) = self.cluster.handle_key(key) {
                    self.handle_cluster_action(action);
                }
            }
            Screen::Components => {
                if let Some(action) = self.components.handle_key(key) {
                    self.handle_components_action(action);
                }
            }
            Screen::Install => {
                if let Some(action) = self.install.handle_key(key) {
                    self.handle_install_action(action).await?;
                }
            }
            Screen::Secrets => {
                if let Some(action) = self.secrets.handle_key(key) {
                    self.handle_secrets_action(action).await?;
                }
            }
            Screen::Complete => {
                if let Some(action) = self.complete.handle_key(key) {
                    self.handle_complete_action(action);
                }
            }
        }

        Ok(())
    }

    /// Check if we're in an input mode (for text entry)
    fn is_input_mode(&self) -> bool {
        matches!(self.current_screen, Screen::Secrets) && self.secrets.is_editing()
    }

    /// Handle welcome screen actions
    fn handle_welcome_action(&mut self, action: MainMenuOption) {
        match action {
            MainMenuOption::InstallPlatform => {
                self.current_screen = Screen::Cluster;
            }
            MainMenuOption::InstallCliOnly => {
                // TODO: Implement CLI-only installation
                self.should_quit = true;
            }
            MainMenuOption::CheckRequirements => {
                // TODO: Show requirements check screen
            }
            MainMenuOption::ViewDocs => {
                // TODO: Open documentation
            }
            MainMenuOption::Exit => {
                self.should_quit = true;
            }
        }
    }

    /// Handle cluster selection actions
    fn handle_cluster_action(&mut self, action: ClusterOption) {
        if !action.is_available() {
            return;
        }

        self.cluster.selected_option = Some(action);
        self.current_screen = Screen::Components;
    }

    /// Handle components selection actions
    fn handle_components_action(&mut self, action: &str) {
        match action {
            "continue" => {
                self.build_install_config();
                self.current_screen = Screen::Install;
                self.install.start_installation(self.demo_mode);
            }
            "back" => {
                self.current_screen = Screen::Cluster;
            }
            _ => {}
        }
    }

    /// Handle installation actions
    async fn handle_install_action(&mut self, action: &str) -> AppResult<()> {
        match action {
            "complete" => {
                self.current_screen = Screen::Secrets;
            }
            "retry" => {
                self.install.start_installation(self.demo_mode);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle secrets configuration actions
    async fn handle_secrets_action(&mut self, action: &str) -> AppResult<()> {
        match action {
            "save" => {
                if !self.demo_mode {
                    self.secrets.save_to_vault().await?;
                }
                self.current_screen = Screen::Complete;
            }
            "skip" => {
                self.current_screen = Screen::Complete;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle completion screen actions
    fn handle_complete_action(&mut self, action: &str) {
        match action {
            "dashboard" => {
                // TODO: Open ArgoCD dashboard
            }
            "exit" => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    /// Build the installation configuration from user selections
    fn build_install_config(&mut self) {
        let cluster_type = match self.cluster.selected_option {
            Some(ClusterOption::Kind) => ClusterType::Kind,
            Some(ClusterOption::Remote) | Some(ClusterOption::Talos) | None => ClusterType::Remote,
        };

        self.install_config = Some(InstallConfig {
            profile: InstallProfile::Standard,
            cluster_type,
            namespace: "cto".to_string(),
            github_org: None,
            github_repo: None,
            registry: "ghcr.io".to_string(),
            registry_namespace: Some("5dlabs".to_string()),
            domain: None,
            install_monitoring: self.components.is_category_selected(ComponentCategory::Observability),
            install_databases: false,
            auto_generate_config: true,
        });
    }
}

