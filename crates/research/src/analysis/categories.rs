//! Content categories for research items.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Content category for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// AI/LLM agent development patterns.
    Agents,
    /// Rust ecosystem updates.
    Rust,
    /// Kubernetes/infrastructure.
    Infrastructure,
    /// MCP/tool integrations.
    Tooling,
    /// Software architecture.
    Architecture,
    /// DevOps/CI-CD.
    DevOps,
    /// Security practices.
    Security,
    /// Research papers/academic.
    Research,
    /// Product launches/announcements.
    Announcements,
    /// Other/general.
    Other,
}

impl Category {
    /// Get all categories.
    #[must_use]
    pub fn all() -> &'static [Category] {
        &[
            Category::Agents,
            Category::Rust,
            Category::Infrastructure,
            Category::Tooling,
            Category::Architecture,
            Category::DevOps,
            Category::Security,
            Category::Research,
            Category::Announcements,
            Category::Other,
        ]
    }

    /// Parse category from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "agents" => Some(Category::Agents),
            "rust" => Some(Category::Rust),
            "infrastructure" | "infra" => Some(Category::Infrastructure),
            "tooling" | "tools" => Some(Category::Tooling),
            "architecture" | "arch" => Some(Category::Architecture),
            "devops" | "ci-cd" | "cicd" => Some(Category::DevOps),
            "security" | "sec" => Some(Category::Security),
            "research" | "papers" => Some(Category::Research),
            "announcements" | "announce" => Some(Category::Announcements),
            "other" => Some(Category::Other),
            _ => None,
        }
    }

    /// Get the category description.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Category::Agents => "AI/LLM agent development patterns and autonomous systems",
            Category::Rust => "Rust language, crates, and tooling",
            Category::Infrastructure => "Kubernetes, cloud, and DevOps infrastructure",
            Category::Tooling => "Developer tools, MCP, and IDEs",
            Category::Architecture => "Software design and architecture patterns",
            Category::DevOps => "CI/CD, automation, and deployment",
            Category::Security => "Security practices and vulnerabilities",
            Category::Research => "Academic papers and research studies",
            Category::Announcements => "Product launches and releases",
            Category::Other => "Content that doesn't fit other categories",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Category::Agents => "agents",
            Category::Rust => "rust",
            Category::Infrastructure => "infrastructure",
            Category::Tooling => "tooling",
            Category::Architecture => "architecture",
            Category::DevOps => "devops",
            Category::Security => "security",
            Category::Research => "research",
            Category::Announcements => "announcements",
            Category::Other => "other",
        };
        write!(f, "{s}")
    }
}
