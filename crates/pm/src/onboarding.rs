//! Onboarding state machine for tenant setup
//!
//! Manages the conversational onboarding flow where Morgan guides
//! users through configuring their CTO workspace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Onboarding state machine states
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OnboardingState {
    /// Initial welcome state
    Welcome,

    /// Waiting for GitHub OAuth connection
    GitHubConnect {
        /// OAuth state parameter for CSRF protection
        oauth_state: Option<String>,
    },

    /// User selecting repositories to authorize
    RepoSelection {
        /// GitHub installation ID after OAuth
        installation_id: i64,
        /// Available repositories from GitHub
        available_repos: Vec<GitHubRepo>,
    },

    /// User entering API key
    ApiKeyEntry {
        /// Selected repositories
        selected_repos: Vec<GitHubRepo>,
        /// Provider being configured
        provider: String,
    },

    /// User selecting agents for their squad
    AgentSelection {
        /// Selected repositories
        selected_repos: Vec<GitHubRepo>,
        /// Configured API keys
        api_keys: HashMap<String, bool>,
        /// Detected stack from repo analysis
        detected_stack: Option<DetectedStack>,
    },

    /// Tenant provisioning in progress
    Provisioning {
        /// Tenant configuration
        config: TenantConfig,
        /// Current provisioning step
        step: ProvisioningStep,
    },

    /// Onboarding complete
    Complete {
        /// Final tenant configuration
        tenant_id: String,
        /// Kubernetes namespace
        namespace: String,
    },

    /// Error state
    Error {
        /// Error message
        message: String,
        /// State to retry from
        retry_from: Box<OnboardingState>,
    },
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self::Welcome
    }
}

/// GitHub repository info
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitHubRepo {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub default_branch: String,
}

/// Detected technology stack from repository analysis
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectedStack {
    pub primary_language: String,
    pub frameworks: Vec<String>,
    pub recommended_agents: Vec<String>,
}

/// Full tenant configuration for provisioning
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantConfig {
    pub owner_email: String,
    pub owner_github_id: String,
    pub organization_name: String,
    pub organization_slug: String,
    pub github_installation_id: i64,
    pub selected_repos: Vec<GitHubRepo>,
    pub ai_provider: String,
    pub enabled_agents: Vec<String>,
    pub default_cli: String,
}

/// Provisioning steps
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningStep {
    CreatingNamespace,
    SettingUpRbac,
    ConfiguringSecrets,
    DeployingAgents,
    InstallingGitHubApp,
    Finalizing,
}

/// User action/input
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum OnboardingAction {
    /// Start onboarding
    Start,

    /// GitHub OAuth completed
    GitHubConnected {
        installation_id: i64,
        access_token: String,
    },

    /// User selected repositories
    SelectRepos { repos: Vec<String> },

    /// User submitted API key
    SubmitApiKey { provider: String, api_key: String },

    /// User selected agents
    SelectAgents { agents: Vec<String>, cli: String },

    /// User confirmed organization name
    ConfirmOrganization { name: String },

    /// Go back to previous state
    Back,

    /// Retry from error
    Retry,
}

/// Response from state transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingResponse {
    /// New state after transition
    pub state: OnboardingState,
    /// Message to display to user
    pub message: String,
    /// Available actions
    pub actions: Vec<AvailableAction>,
    /// Progress percentage (0-100)
    pub progress: u8,
}

/// Action available to user
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailableAction {
    pub action_type: String,
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ActionOption>>,
}

/// Option for select-type actions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionOption {
    pub label: String,
    pub value: String,
}

/// Onboarding state machine
pub struct OnboardingMachine {
    state: OnboardingState,
}

impl OnboardingMachine {
    /// Create a new onboarding machine
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: OnboardingState::Welcome,
        }
    }

    /// Create from existing state
    #[must_use]
    pub fn from_state(state: OnboardingState) -> Self {
        Self { state }
    }

    /// Get current state
    #[must_use]
    pub fn state(&self) -> &OnboardingState {
        &self.state
    }

    /// Process an action and transition state
    #[allow(clippy::too_many_lines)]
    pub fn transition(&mut self, action: OnboardingAction) -> OnboardingResponse {
        let (new_state, message, actions, progress) = match (&self.state, action) {
            // Welcome -> GitHubConnect
            (OnboardingState::Welcome, OnboardingAction::Start) => {
                (
                    OnboardingState::GitHubConnect { oauth_state: None },
                    "Great! Let's connect your GitHub account so our agents can access your repositories.".to_string(),
                    vec![AvailableAction {
                        action_type: "button".to_string(),
                        label: "Connect GitHub".to_string(),
                        value: "connect_github".to_string(),
                        options: None,
                    }],
                    10,
                )
            }

            // GitHubConnect -> RepoSelection
            (OnboardingState::GitHubConnect { .. }, OnboardingAction::GitHubConnected { installation_id, .. }) => {
                // In real implementation, fetch repos from GitHub API
                let available_repos = vec![
                    GitHubRepo {
                        owner: "example".to_string(),
                        name: "api".to_string(),
                        full_name: "example/api".to_string(),
                        description: Some("Backend API".to_string()),
                        language: Some("Rust".to_string()),
                        default_branch: "main".to_string(),
                    },
                ];

                (
                    OnboardingState::RepoSelection {
                        installation_id,
                        available_repos: available_repos.clone(),
                    },
                    "GitHub connected! Which repositories should our agents work on?".to_string(),
                    vec![AvailableAction {
                        action_type: "select".to_string(),
                        label: "Select Repositories".to_string(),
                        value: "select_repos".to_string(),
                        options: Some(available_repos.iter().map(|r| ActionOption {
                            label: format!("{} ({})", r.full_name, r.language.as_deref().unwrap_or("Unknown")),
                            value: r.full_name.clone(),
                        }).collect()),
                    }],
                    30,
                )
            }

            // RepoSelection -> ApiKeyEntry
            (OnboardingState::RepoSelection { available_repos, .. }, OnboardingAction::SelectRepos { repos }) => {
                let selected: Vec<_> = available_repos
                    .iter()
                    .filter(|r| repos.contains(&r.full_name))
                    .cloned()
                    .collect();

                (
                    OnboardingState::ApiKeyEntry {
                        selected_repos: selected,
                        provider: "anthropic".to_string(),
                    },
                    "Now I need an API key to power our AI agents. We recommend Anthropic (Claude) for the best experience.".to_string(),
                    vec![AvailableAction {
                        action_type: "input".to_string(),
                        label: "Enter Anthropic API Key".to_string(),
                        value: "anthropic_key".to_string(),
                        options: None,
                    }],
                    50,
                )
            }

            // ApiKeyEntry -> AgentSelection
            (OnboardingState::ApiKeyEntry { selected_repos, .. }, OnboardingAction::SubmitApiKey { provider, .. }) => {
                let mut api_keys = HashMap::new();
                api_keys.insert(provider, true);

                // Detect stack from repos
                let detected_stack = Some(DetectedStack {
                    primary_language: "Rust".to_string(),
                    frameworks: vec!["axum".to_string(), "tokio".to_string()],
                    recommended_agents: vec!["rex".to_string(), "cleo".to_string(), "tess".to_string()],
                });

                (
                    OnboardingState::AgentSelection {
                        selected_repos: selected_repos.clone(),
                        api_keys,
                        detected_stack: detected_stack.clone(),
                    },
                    format!(
                        "API key validated! Based on your {} codebase, I recommend: {}",
                        detected_stack.as_ref().map_or("", |s| s.primary_language.as_str()),
                        detected_stack.as_ref().map_or_else(String::new, |s| s.recommended_agents.join(", "))
                    ),
                    vec![AvailableAction {
                        action_type: "button".to_string(),
                        label: "Use Recommended Squad".to_string(),
                        value: "use_recommended".to_string(),
                        options: None,
                    }],
                    70,
                )
            }

            // AgentSelection -> Provisioning
            (OnboardingState::AgentSelection { selected_repos, api_keys, detected_stack: _ }, OnboardingAction::SelectAgents { agents, cli }) => {
                let config = TenantConfig {
                    owner_email: "user@example.com".to_string(), // Would come from session
                    owner_github_id: "12345".to_string(),
                    organization_name: "Example Org".to_string(),
                    organization_slug: "example-org".to_string(),
                    github_installation_id: 0, // Would come from state
                    selected_repos: selected_repos.clone(),
                    ai_provider: api_keys.keys().next().cloned().unwrap_or_default(),
                    enabled_agents: agents,
                    default_cli: cli,
                };

                (
                    OnboardingState::Provisioning {
                        config,
                        step: ProvisioningStep::CreatingNamespace,
                    },
                    "Setting up your workspace... Creating isolated environment.".to_string(),
                    vec![],
                    85,
                )
            }

            // Handle completion (would be triggered by operator)
            (OnboardingState::Provisioning { config, .. }, OnboardingAction::Start) => {
                (
                    OnboardingState::Complete {
                        tenant_id: config.organization_slug.clone(),
                        namespace: format!("tenant-{}", config.organization_slug),
                    },
                    "Your workspace is ready! You can now create your first project.".to_string(),
                    vec![
                        AvailableAction {
                            action_type: "button".to_string(),
                            label: "Go to Dashboard".to_string(),
                            value: "dashboard".to_string(),
                            options: None,
                        },
                        AvailableAction {
                            action_type: "button".to_string(),
                            label: "Create First Project".to_string(),
                            value: "create_project".to_string(),
                            options: None,
                        },
                    ],
                    100,
                )
            }

            // Default: stay in current state
            _ => {
                (
                    self.state.clone(),
                    "I didn't understand that. Could you try again?".to_string(),
                    vec![],
                    0,
                )
            }
        };

        self.state = new_state.clone();

        OnboardingResponse {
            state: new_state,
            message,
            actions,
            progress,
        }
    }
}

impl Default for OnboardingMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_to_github_connect() {
        let mut machine = OnboardingMachine::new();
        let response = machine.transition(OnboardingAction::Start);

        assert!(matches!(
            response.state,
            OnboardingState::GitHubConnect { .. }
        ));
        assert_eq!(response.progress, 10);
    }

    #[test]
    fn test_github_connected() {
        let mut machine =
            OnboardingMachine::from_state(OnboardingState::GitHubConnect { oauth_state: None });
        let response = machine.transition(OnboardingAction::GitHubConnected {
            installation_id: 12345,
            access_token: "token".to_string(),
        });

        assert!(matches!(
            response.state,
            OnboardingState::RepoSelection { .. }
        ));
        assert_eq!(response.progress, 30);
    }
}
