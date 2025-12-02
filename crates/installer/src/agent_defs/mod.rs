//! Agent definitions and ASCII art for the CTO Platform

mod ascii_art;

pub use ascii_art::AGENTS;

/// An AI agent in the CTO Platform
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Agent {
    /// Agent name
    pub name: &'static str,
    /// Agent role
    pub role: &'static str,
    /// Agent icon (emoji)
    pub icon: &'static str,
    /// Agent personality description
    pub personality: &'static str,
    /// ASCII art representation
    pub ascii_art: &'static str,
    /// Greeting message
    pub greeting: &'static str,
}

impl Agent {
    /// Get a short version of the role (for compact displays)
    pub const fn role_short(&self) -> &'static str {
        match self.name.as_bytes() {
            b"Rex" => "Dev",
            b"Cleo" => "Review",
            b"Blaze" => "UI/UX",
            b"Tess" => "QA",
            b"Cipher" => "Sec",
            b"Morgan" => "Docs",
            b"Atlas" => "Infra",
            b"Bolt" => "Deploy",
            b"Stitch" => "PR Bot",
            _ => "Agent",
        }
    }
}

/// Get an agent by name
pub fn get_agent(name: &str) -> Option<&'static Agent> {
    AGENTS.iter().find(|a| a.name.eq_ignore_ascii_case(name))
}

/// Get all agents
pub fn all_agents() -> &'static [Agent] {
    &AGENTS
}

