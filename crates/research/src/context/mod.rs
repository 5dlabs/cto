//! Platform context module for context-aware research analysis.
//!
//! This module loads information about the CTO platform to help
//! the AI make more relevant, platform-specific recommendations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::Path;

/// Profile of an agent in the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    /// Agent name (e.g., "Rex", "Blaze").
    pub name: String,
    /// Primary language/technology.
    pub language: String,
    /// Key technologies the agent works with.
    pub technologies: Vec<String>,
    /// Agent role (implementation, support, etc.).
    pub role: AgentRole,
}

/// Role classification for agents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRole {
    /// Implementation agents that write code.
    Implementation,
    /// Support agents for specific tasks.
    Support,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Implementation => write!(f, "Implementation"),
            Self::Support => write!(f, "Support"),
        }
    }
}

/// Platform context for research analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformContext {
    /// Summary of the platform architecture.
    pub architecture_summary: String,
    /// Available agents and their capabilities.
    pub agents: Vec<AgentProfile>,
    /// Technology stack used by the platform.
    pub tech_stack: Vec<String>,
    /// Recent pull requests (titles/descriptions).
    pub recent_prs: Vec<String>,
    /// Known pain points or areas for improvement.
    pub pain_points: Vec<String>,
}

impl Default for PlatformContext {
    fn default() -> Self {
        Self::built_in()
    }
}

impl PlatformContext {
    /// Create a platform context with built-in knowledge.
    ///
    /// This is used when we can't load from files (e.g., in container).
    #[must_use]
    pub fn built_in() -> Self {
        Self {
            architecture_summary: BUILT_IN_ARCHITECTURE.to_string(),
            agents: built_in_agents(),
            tech_stack: built_in_tech_stack(),
            recent_prs: Vec::new(),
            pain_points: built_in_pain_points(),
        }
    }

    /// Load platform context from the repository.
    ///
    /// Attempts to load AGENTS.md and architecture docs.
    /// Falls back to built-in knowledge if files aren't available.
    pub async fn load(repo_path: &Path) -> Result<Self> {
        let mut context = Self::built_in();

        // Try to load AGENTS.md for agent information
        let agents_path = repo_path.join("AGENTS.md");
        if agents_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&agents_path).await {
                context.architecture_summary = extract_architecture_summary(&content);
            }
        }

        // Try to load recent PRs if we have GitHub access
        if let Ok(prs) = load_recent_prs().await {
            context.recent_prs = prs;
        }

        Ok(context)
    }

    /// Load platform context from environment or built-in defaults.
    ///
    /// Checks for REPO_PATH environment variable, otherwise uses built-in.
    pub async fn from_env() -> Self {
        if let Ok(repo_path) = std::env::var("REPO_PATH") {
            Self::load(Path::new(&repo_path)).await.unwrap_or_default()
        } else {
            Self::built_in()
        }
    }

    /// Format the context for inclusion in an AI prompt.
    #[must_use]
    pub fn to_prompt_context(&self) -> String {
        let mut output = String::new();

        output.push_str("## Platform Architecture\n\n");
        output.push_str(&self.architecture_summary);
        output.push_str("\n\n");

        output.push_str("## Technology Stack\n\n");
        for tech in &self.tech_stack {
            let _ = writeln!(output, "- {tech}");
        }
        output.push('\n');

        output.push_str("## Available Agents\n\n");
        for agent in &self.agents {
            let _ = writeln!(
                output,
                "- **{}** ({}) - {} agent: {}",
                agent.name,
                agent.language,
                agent.role,
                agent.technologies.join(", ")
            );
        }
        output.push('\n');

        if !self.recent_prs.is_empty() {
            output.push_str("## Recent Work\n\n");
            for pr in self.recent_prs.iter().take(5) {
                let _ = writeln!(output, "- {pr}");
            }
            output.push('\n');
        }

        if !self.pain_points.is_empty() {
            output.push_str("## Known Pain Points\n\n");
            for point in &self.pain_points {
                let _ = writeln!(output, "- {point}");
            }
        }

        output
    }

    /// Get agent names that match a technology.
    #[must_use]
    pub fn agents_for_tech(&self, tech: &str) -> Vec<&str> {
        let tech_lower = tech.to_lowercase();
        self.agents
            .iter()
            .filter(|a| {
                a.language.to_lowercase().contains(&tech_lower)
                    || a.technologies
                        .iter()
                        .any(|t| t.to_lowercase().contains(&tech_lower))
            })
            .map(|a| a.name.as_str())
            .collect()
    }
}

/// Extract architecture summary from AGENTS.md content.
fn extract_architecture_summary(content: &str) -> String {
    // Extract key sections from AGENTS.md
    let mut summary = String::new();

    // Look for architecture-related sections
    if content.contains("CTO platform") || content.contains("multi-agent") {
        summary.push_str(
            "The CTO platform is a multi-agent software development system that orchestrates \
             specialized AI agents for different tasks. Each agent has specific capabilities \
             and works together through the Play workflow to deliver features from PRD to production.",
        );
    }

    if summary.is_empty() {
        summary = BUILT_IN_ARCHITECTURE.to_string();
    }

    summary
}

/// Load recent PRs from GitHub using the gh CLI.
async fn load_recent_prs() -> Result<Vec<String>> {
    let output = tokio::process::Command::new("gh")
        .args([
            "pr",
            "list",
            "--repo",
            "5dlabs/cto",
            "--state",
            "merged",
            "--limit",
            "10",
            "--json",
            "title",
            "-q",
            ".[].title",
        ])
        .output()
        .await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(String::from).collect())
    } else {
        Ok(Vec::new())
    }
}

/// Built-in architecture summary.
const BUILT_IN_ARCHITECTURE: &str = r"The CTO platform is a multi-agent software development system built on Kubernetes.

Key components:
- **Agent Controller**: Orchestrates CodeRun CRDs for agent execution
- **PM Server**: Handles Linear integration and webhook processing
- **Healer**: Self-healing system that monitors and remediates failures
- **Play Workflow**: Argo Workflows pipeline from PRD to merged PR

The platform uses:
- Rust for backend services (controller, healer, intake, research)
- React/Next.js for web interfaces
- Kubernetes operators for infrastructure
- ArgoCD for GitOps deployments";

/// Built-in agent profiles.
fn built_in_agents() -> Vec<AgentProfile> {
    vec![
        AgentProfile {
            name: "Rex".to_string(),
            language: "Rust".to_string(),
            technologies: vec![
                "axum".to_string(),
                "tokio".to_string(),
                "serde".to_string(),
                "sqlx".to_string(),
                "tracing".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Grizz".to_string(),
            language: "Go".to_string(),
            technologies: vec![
                "chi".to_string(),
                "grpc".to_string(),
                "pgx".to_string(),
                "redis".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Nova".to_string(),
            language: "Node.js/Bun".to_string(),
            technologies: vec![
                "Elysia".to_string(),
                "Effect".to_string(),
                "Better Auth".to_string(),
                "Drizzle".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Blaze".to_string(),
            language: "React/TypeScript".to_string(),
            technologies: vec![
                "Next.js 15".to_string(),
                "shadcn/ui".to_string(),
                "TailwindCSS".to_string(),
                "Better Auth".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Tap".to_string(),
            language: "React Native".to_string(),
            technologies: vec![
                "Expo".to_string(),
                "expo-router".to_string(),
                "Better Auth".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Spark".to_string(),
            language: "Electron".to_string(),
            technologies: vec![
                "electron-builder".to_string(),
                "React".to_string(),
                "Better Auth".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Vex".to_string(),
            language: "Unity/C#".to_string(),
            technologies: vec![
                "XR Interaction Toolkit".to_string(),
                "OpenXR".to_string(),
                "Meta XR SDK".to_string(),
            ],
            role: AgentRole::Implementation,
        },
        AgentProfile {
            name: "Morgan".to_string(),
            language: "N/A".to_string(),
            technologies: vec!["PRD intake".to_string(), "Linear integration".to_string()],
            role: AgentRole::Support,
        },
        AgentProfile {
            name: "Bolt".to_string(),
            language: "N/A".to_string(),
            technologies: vec!["Infrastructure setup".to_string(), "Kubernetes".to_string()],
            role: AgentRole::Support,
        },
        AgentProfile {
            name: "Cleo".to_string(),
            language: "N/A".to_string(),
            technologies: vec!["Code review".to_string(), "Quality analysis".to_string()],
            role: AgentRole::Support,
        },
        AgentProfile {
            name: "Cipher".to_string(),
            language: "N/A".to_string(),
            technologies: vec![
                "Security analysis".to_string(),
                "Vulnerability scanning".to_string(),
            ],
            role: AgentRole::Support,
        },
        AgentProfile {
            name: "Tess".to_string(),
            language: "N/A".to_string(),
            technologies: vec!["Testing".to_string(), "Test generation".to_string()],
            role: AgentRole::Support,
        },
        AgentProfile {
            name: "Atlas".to_string(),
            language: "N/A".to_string(),
            technologies: vec!["CI/CD".to_string(), "Merge coordination".to_string()],
            role: AgentRole::Support,
        },
    ]
}

/// Built-in technology stack.
fn built_in_tech_stack() -> Vec<String> {
    vec![
        "Rust (backend services)".to_string(),
        "TypeScript/React (web frontend)".to_string(),
        "Kubernetes (orchestration)".to_string(),
        "ArgoCD (GitOps)".to_string(),
        "Argo Workflows (CI/CD pipelines)".to_string(),
        "PostgreSQL (databases)".to_string(),
        "Linear (project management)".to_string(),
        "GitHub (source control)".to_string(),
        "Prometheus/Grafana (observability)".to_string(),
        "OpenBao (secrets management)".to_string(),
    ]
}

/// Built-in pain points.
fn built_in_pain_points() -> Vec<String> {
    vec![
        "Agent context window limits during long tasks".to_string(),
        "CI pipeline reliability and speed".to_string(),
        "Healer remediation success rate".to_string(),
        "Agent coordination in complex workflows".to_string(),
        "Cost optimization for LLM API calls".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_context() {
        let ctx = PlatformContext::built_in();
        assert!(!ctx.agents.is_empty());
        assert!(!ctx.tech_stack.is_empty());
        assert!(!ctx.architecture_summary.is_empty());
    }

    #[test]
    fn test_agents_for_tech() {
        let ctx = PlatformContext::built_in();
        let rust_agents = ctx.agents_for_tech("rust");
        assert!(rust_agents.contains(&"Rex"));

        let react_agents = ctx.agents_for_tech("react");
        assert!(react_agents.contains(&"Blaze"));
    }

    #[test]
    fn test_to_prompt_context() {
        let ctx = PlatformContext::built_in();
        let prompt = ctx.to_prompt_context();
        assert!(prompt.contains("## Platform Architecture"));
        assert!(prompt.contains("## Technology Stack"));
        assert!(prompt.contains("## Available Agents"));
        assert!(prompt.contains("Rex"));
    }
}
