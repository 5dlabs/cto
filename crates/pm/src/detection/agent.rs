//! Agent selection based on detection results.

use super::{DetectionResult, Framework, Language};
use serde::{Deserialize, Serialize};

/// Implementation agents that can handle remediation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Agent {
    /// Rust backend (axum, tokio, serde, sqlx)
    Rex,
    /// Go backend (chi, grpc, pgx, redis)
    Grizz,
    /// Node.js/Bun backend (Elysia, Effect, Better Auth, Drizzle)
    Nova,
    /// React/Next.js web frontend (Next.js 15, shadcn/ui, TailwindCSS)
    Blaze,
    /// Expo/React Native mobile (expo-router, react-native)
    Tap,
    /// Electron desktop (electron-builder, react)
    Spark,
    /// Unity/C# XR (XR Interaction Toolkit, OpenXR, Meta XR SDK)
    Vex,
    /// Unreal/Godot (Unreal Engine 5, C++, Blueprints, GDScript)
    Forge,
    /// Generic/fallback - uses Rex for infrastructure
    Generic,
}

impl Agent {
    /// Human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rex => "Rex",
            Self::Grizz => "Grizz",
            Self::Nova => "Nova",
            Self::Blaze => "Blaze",
            Self::Tap => "Tap",
            Self::Spark => "Spark",
            Self::Vex => "Vex",
            Self::Forge => "Forge",
            Self::Generic => "Rex", // Generic falls back to Rex
        }
    }

    /// Identifier used in button actions
    pub fn id(&self) -> &'static str {
        match self {
            Self::Rex => "rex",
            Self::Grizz => "grizz",
            Self::Nova => "nova",
            Self::Blaze => "blaze",
            Self::Tap => "tap",
            Self::Spark => "spark",
            Self::Vex => "vex",
            Self::Forge => "forge",
            Self::Generic => "rex",
        }
    }

    /// GitHub App name for this agent
    pub fn github_app(&self) -> &'static str {
        match self {
            Self::Rex => "5DLabs-Rex",
            Self::Grizz => "5DLabs-Grizz",
            Self::Nova => "5DLabs-Nova",
            Self::Blaze => "5DLabs-Blaze",
            Self::Tap => "5DLabs-Tap",
            Self::Spark => "5DLabs-Spark",
            Self::Vex => "5DLabs-Vex",
            Self::Forge => "5DLabs-Forge",
            Self::Generic => "5DLabs-Rex",
        }
    }

    /// Emoji for display
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Rex => "🦖",
            Self::Grizz => "🐻",
            Self::Nova => "⭐",
            Self::Blaze => "🔥",
            Self::Tap => "📱",
            Self::Spark => "⚡",
            Self::Vex => "🥽",
            Self::Forge => "🎮",
            Self::Generic => "🤖",
        }
    }

    /// Button label for remediation
    pub fn button_label(&self) -> String {
        format!("{} Fix with {}", self.emoji(), self.display_name())
    }

    /// Parse agent from string identifier
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.to_lowercase().as_str() {
            "rex" => Self::Rex,
            "grizz" => Self::Grizz,
            "nova" => Self::Nova,
            "blaze" => Self::Blaze,
            "tap" => Self::Tap,
            "spark" => Self::Spark,
            "vex" => Self::Vex,
            "forge" => Self::Forge,
            _ => return None,
        })
    }
}

/// Select the best agent based on detection result
pub fn select_agent(result: &DetectionResult) -> Agent {
    // Priority 1: Framework (most specific)
    if let Some(framework) = result.framework {
        if let Some(agent) = agent_for_framework(framework) {
            return agent;
        }
    }

    // Priority 2: Primary language
    if let Some(language) = result.primary_language {
        return agent_for_language(language);
    }

    // Fallback
    Agent::Generic
}

/// Map framework to agent
fn agent_for_framework(framework: Framework) -> Option<Agent> {
    Some(match framework {
        // Rust
        Framework::Axum | Framework::Actix | Framework::Rocket => Agent::Rex,

        // Go
        Framework::Chi | Framework::Gin | Framework::Echo | Framework::Fiber => Agent::Grizz,

        // TypeScript - Web Frontend
        Framework::NextJs | Framework::Remix | Framework::Astro | Framework::Vite => Agent::Blaze,

        // TypeScript - Mobile
        Framework::Expo | Framework::ReactNative => Agent::Tap,

        // TypeScript - Desktop
        Framework::Electron | Framework::Tauri => Agent::Spark,

        // TypeScript - Backend
        Framework::Elysia
        | Framework::Express
        | Framework::Fastify
        | Framework::NestJs
        | Framework::Hono => Agent::Nova,

        // C# - Game/XR
        Framework::Unity => Agent::Vex,
        Framework::DotNet => Agent::Nova, // .NET backend goes to Nova for now

        // C++ - Game
        Framework::Unreal | Framework::Qt => Agent::Forge,
        Framework::Godot => Agent::Forge,

        // Python - Backend (no dedicated agent yet, falls back to Generic/Rex)
        Framework::FastApi | Framework::Django | Framework::Flask => Agent::Generic,

        Framework::Unknown => return None,
    })
}

/// Map language to agent (fallback when no framework detected)
fn agent_for_language(language: Language) -> Agent {
    match language {
        Language::Rust => Agent::Rex,
        Language::Go => Agent::Grizz,
        Language::TypeScript | Language::JavaScript => Agent::Blaze, // Default TS to web
        Language::CSharp => Agent::Vex,
        Language::Cpp => Agent::Forge,
        Language::Python => Agent::Generic, // No dedicated Python agent yet
        Language::Swift | Language::Kotlin => Agent::Tap, // Mobile languages
        Language::Java => Agent::Generic,   // No dedicated Java agent yet
        Language::Ruby | Language::Php => Agent::Generic, // No dedicated Ruby/PHP agent yet
        Language::Shell | Language::Yaml | Language::Json | Language::Markdown | Language::Other => {
            Agent::Rex // Infra/config goes to Rex
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_result(
        language: Option<Language>,
        framework: Option<Framework>,
    ) -> DetectionResult {
        let mut language_counts = HashMap::new();
        if let Some(lang) = language {
            language_counts.insert(lang, 1);
        }

        DetectionResult {
            primary_language: language,
            language_counts,
            framework,
            confidence: 1.0,
            signals: vec![],
        }
    }

    #[test]
    fn test_rust_to_rex() {
        let result = make_result(Some(Language::Rust), None);
        assert_eq!(select_agent(&result), Agent::Rex);
    }

    #[test]
    fn test_go_to_grizz() {
        let result = make_result(Some(Language::Go), None);
        assert_eq!(select_agent(&result), Agent::Grizz);
    }

    #[test]
    fn test_typescript_defaults_to_blaze() {
        let result = make_result(Some(Language::TypeScript), None);
        assert_eq!(select_agent(&result), Agent::Blaze);
    }

    #[test]
    fn test_expo_overrides_typescript() {
        let result = make_result(Some(Language::TypeScript), Some(Framework::Expo));
        assert_eq!(select_agent(&result), Agent::Tap);
    }

    #[test]
    fn test_electron_overrides_typescript() {
        let result = make_result(Some(Language::TypeScript), Some(Framework::Electron));
        assert_eq!(select_agent(&result), Agent::Spark);
    }

    #[test]
    fn test_elysia_to_nova() {
        let result = make_result(Some(Language::TypeScript), Some(Framework::Elysia));
        assert_eq!(select_agent(&result), Agent::Nova);
    }

    #[test]
    fn test_unity_to_vex() {
        let result = make_result(Some(Language::CSharp), Some(Framework::Unity));
        assert_eq!(select_agent(&result), Agent::Vex);
    }

    #[test]
    fn test_unreal_to_forge() {
        let result = make_result(Some(Language::Cpp), Some(Framework::Unreal));
        assert_eq!(select_agent(&result), Agent::Forge);
    }

    #[test]
    fn test_agent_button_label() {
        assert_eq!(Agent::Rex.button_label(), "🦖 Fix with Rex");
        assert_eq!(Agent::Blaze.button_label(), "🔥 Fix with Blaze");
        assert_eq!(Agent::Tap.button_label(), "📱 Fix with Tap");
    }
}
