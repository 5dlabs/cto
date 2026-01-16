//! Skill (SOP) model - the core learning unit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent type for skill association.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    /// Morgan - Project management
    Morgan,
    /// Rex - Rust implementation
    Rex,
    /// Blaze - React/TypeScript frontend
    Blaze,
    /// Grizz - Go implementation
    Grizz,
    /// Nova - Node.js/Bun implementation
    Nova,
    /// Tap - Expo mobile
    Tap,
    /// Spark - Electron desktop
    Spark,
    /// Bolt - Infrastructure
    Bolt,
    /// Cleo - Code quality
    Cleo,
    /// Cipher - Security
    Cipher,
    /// Tess - Testing
    Tess,
    /// Atlas - Integration/merge
    Atlas,
    /// Vex - Unity/XR
    Vex,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Morgan => "morgan",
            Self::Rex => "rex",
            Self::Blaze => "blaze",
            Self::Grizz => "grizz",
            Self::Nova => "nova",
            Self::Tap => "tap",
            Self::Spark => "spark",
            Self::Bolt => "bolt",
            Self::Cleo => "cleo",
            Self::Cipher => "cipher",
            Self::Tess => "tess",
            Self::Atlas => "atlas",
            Self::Vex => "vex",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "morgan" => Ok(Self::Morgan),
            "rex" => Ok(Self::Rex),
            "blaze" => Ok(Self::Blaze),
            "grizz" => Ok(Self::Grizz),
            "nova" => Ok(Self::Nova),
            "tap" => Ok(Self::Tap),
            "spark" => Ok(Self::Spark),
            "bolt" => Ok(Self::Bolt),
            "cleo" => Ok(Self::Cleo),
            "cipher" => Ok(Self::Cipher),
            "tess" => Ok(Self::Tess),
            "atlas" => Ok(Self::Atlas),
            "vex" => Ok(Self::Vex),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

/// A single step in a tool-call SOP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStep {
    /// Order in the sequence (1-based).
    pub order: u32,

    /// Name of the tool (e.g., "git_status", "write_file").
    pub tool_name: String,

    /// Description of what this step accomplishes.
    pub action: String,

    /// Optional hint about typical parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_hint: Option<String>,
}

impl ToolStep {
    /// Create a new tool step.
    #[must_use]
    pub fn new(order: u32, tool_name: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            order,
            tool_name: tool_name.into(),
            action: action.into(),
            parameters_hint: None,
        }
    }

    /// Add a parameters hint.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.parameters_hint = Some(hint.into());
        self
    }
}

/// A learned skill (Standard Operating Procedure) extracted from successful task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier.
    pub id: Uuid,

    /// Condition describing when this skill should be used.
    /// e.g., "implementing Rust HTTP handler with authentication"
    pub use_when: String,

    /// Agent type this skill is associated with.
    pub agent: AgentType,

    /// User/project preferences captured during learning.
    #[serde(default)]
    pub preferences: Vec<String>,

    /// Ordered sequence of tool calls that accomplish this task.
    pub tool_sops: Vec<ToolStep>,

    /// Complexity score (0.0 - 1.0). Higher = more complex task.
    pub complexity_score: f32,

    /// Number of times this pattern has succeeded.
    pub success_count: u32,

    /// Space (project/user scope) this skill belongs to.
    pub space_id: Uuid,

    /// Optional embedding vector for similarity search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// When this skill was first learned.
    pub created_at: DateTime<Utc>,

    /// When this skill was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Skill {
    /// Create a new skill.
    #[must_use]
    pub fn new(
        use_when: impl Into<String>,
        agent: AgentType,
        tool_sops: Vec<ToolStep>,
        space_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            use_when: use_when.into(),
            agent,
            preferences: Vec::new(),
            tool_sops,
            complexity_score: 0.5,
            success_count: 1,
            space_id,
            embedding: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a preference.
    pub fn add_preference(&mut self, preference: impl Into<String>) {
        self.preferences.push(preference.into());
    }

    /// Set the complexity score.
    #[must_use]
    pub fn with_complexity(mut self, score: f32) -> Self {
        self.complexity_score = score.clamp(0.0, 1.0);
        self
    }

    /// Set the embedding vector.
    #[must_use]
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Increment the success count.
    pub fn increment_success(&mut self) {
        self.success_count += 1;
        self.updated_at = Utc::now();
    }

    /// Generate a summary of this skill for inclusion in prompts.
    #[must_use]
    pub fn to_prompt_summary(&self) -> String {
        use std::fmt::Write;

        let mut summary = format!("### {}\n", self.use_when);

        if !self.preferences.is_empty() {
            let _ = writeln!(summary, "**Preferences**: {}", self.preferences.join(", "));
        }

        summary.push_str("**Approach**:\n");
        for step in &self.tool_sops {
            let _ = writeln!(summary, "{}. `{}`: {}", step.order, step.tool_name, step.action);
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_parsing() {
        assert_eq!("rex".parse::<AgentType>().unwrap(), AgentType::Rex);
        assert_eq!("BLAZE".parse::<AgentType>().unwrap(), AgentType::Blaze);
        assert!("unknown".parse::<AgentType>().is_err());
    }

    #[test]
    fn test_skill_creation() {
        let space_id = Uuid::new_v4();
        let skill = Skill::new(
            "implementing HTTP handler",
            AgentType::Rex,
            vec![
                ToolStep::new(1, "read_file", "Read existing handlers"),
                ToolStep::new(2, "write_file", "Create new handler"),
            ],
            space_id,
        );

        assert_eq!(skill.agent, AgentType::Rex);
        assert_eq!(skill.tool_sops.len(), 2);
        assert_eq!(skill.success_count, 1);
    }

    #[test]
    fn test_prompt_summary() {
        let space_id = Uuid::new_v4();
        let mut skill = Skill::new(
            "setting up authentication",
            AgentType::Blaze,
            vec![ToolStep::new(1, "search_files", "Find auth config")],
            space_id,
        );
        skill.add_preference("Use Better Auth");

        let summary = skill.to_prompt_summary();
        assert!(summary.contains("setting up authentication"));
        assert!(summary.contains("Better Auth"));
        assert!(summary.contains("search_files"));
    }
}
