//! Framework/platform detection from package manifests and path patterns.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Frameworks and platforms that determine agent routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    // Rust frameworks
    Axum,
    Actix,
    Rocket,

    // Go frameworks
    Chi,
    Gin,
    Echo,
    Fiber,

    // TypeScript/JavaScript - Web
    NextJs,
    Remix,
    Astro,
    Vite,

    // TypeScript/JavaScript - Mobile
    Expo,
    ReactNative,

    // TypeScript/JavaScript - Desktop
    Electron,
    Tauri,

    // TypeScript/JavaScript - Backend
    Elysia,
    Express,
    Fastify,
    NestJs,
    Hono,

    // C#
    Unity,
    DotNet,
    Godot,

    // C++
    Unreal,
    Qt,

    // Python
    FastApi,
    Django,
    Flask,

    // Generic
    Unknown,
}

impl Framework {
    /// Human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Axum => "Axum",
            Self::Actix => "Actix",
            Self::Rocket => "Rocket",
            Self::Chi => "Chi",
            Self::Gin => "Gin",
            Self::Echo => "Echo",
            Self::Fiber => "Fiber",
            Self::NextJs => "Next.js",
            Self::Remix => "Remix",
            Self::Astro => "Astro",
            Self::Vite => "Vite",
            Self::Expo => "Expo",
            Self::ReactNative => "React Native",
            Self::Electron => "Electron",
            Self::Tauri => "Tauri",
            Self::Elysia => "Elysia",
            Self::Express => "Express",
            Self::Fastify => "Fastify",
            Self::NestJs => "NestJS",
            Self::Hono => "Hono",
            Self::Unity => "Unity",
            Self::DotNet => ".NET",
            Self::Godot => "Godot",
            Self::Unreal => "Unreal Engine",
            Self::Qt => "Qt",
            Self::FastApi => "FastAPI",
            Self::Django => "Django",
            Self::Flask => "Flask",
            Self::Unknown => "Unknown",
        }
    }

    /// Is this a mobile framework?
    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::Expo | Self::ReactNative)
    }

    /// Is this a desktop framework?
    pub fn is_desktop(&self) -> bool {
        matches!(self, Self::Electron | Self::Tauri)
    }

    /// Is this a web frontend framework?
    pub fn is_web_frontend(&self) -> bool {
        matches!(
            self,
            Self::NextJs | Self::Remix | Self::Astro | Self::Vite
        )
    }

    /// Is this a backend framework?
    pub fn is_backend(&self) -> bool {
        matches!(
            self,
            Self::Axum
                | Self::Actix
                | Self::Rocket
                | Self::Chi
                | Self::Gin
                | Self::Echo
                | Self::Fiber
                | Self::Elysia
                | Self::Express
                | Self::Fastify
                | Self::NestJs
                | Self::Hono
                | Self::FastApi
                | Self::Django
                | Self::Flask
        )
    }

    /// Is this a game engine?
    pub fn is_game_engine(&self) -> bool {
        matches!(self, Self::Unity | Self::Unreal | Self::Godot)
    }
}

/// Detect framework from file path patterns
pub fn detect_from_path(path: &str) -> Option<Framework> {
    let lower = path.to_lowercase();

    // Unity (C#)
    if lower.contains("assets/scripts") || lower.contains("assets/editor") {
        return Some(Framework::Unity);
    }

    // Unreal (C++)
    if lower.contains("source/") && (lower.ends_with(".cpp") || lower.ends_with(".h")) {
        return Some(Framework::Unreal);
    }

    // Godot
    if lower.ends_with(".gd") || lower.contains("addons/") && lower.ends_with(".tscn") {
        return Some(Framework::Godot);
    }

    // Expo (React Native) - path patterns
    if lower.contains("app/(tabs)")
        || lower.contains("app/_layout")
        || lower.contains("expo-router")
    {
        return Some(Framework::Expo);
    }

    // Electron - main process location
    if lower.contains("src/main/") && (lower.ends_with(".ts") || lower.ends_with(".js")) {
        return Some(Framework::Electron);
    }

    // Tauri
    if lower.contains("src-tauri/") {
        return Some(Framework::Tauri);
    }

    // Next.js app router
    if lower.starts_with("app/") && (lower.ends_with("page.tsx") || lower.ends_with("layout.tsx"))
    {
        return Some(Framework::NextJs);
    }

    // Server/API paths suggest backend
    if lower.contains("server/") || lower.contains("api/") || lower.contains("backend/") {
        // Will be refined by package.json
        return None;
    }

    None
}

/// Detect framework from package.json content
pub fn detect_from_package_json(content: &str) -> Option<Framework> {
    let pkg: Value = serde_json::from_str(content).ok()?;

    // Check dependencies and devDependencies
    let deps = merge_deps(&pkg);

    // Mobile (highest priority for TS disambiguation)
    if deps.contains_key("expo") {
        return Some(Framework::Expo);
    }
    if deps.contains_key("react-native") && !deps.contains_key("expo") {
        return Some(Framework::ReactNative);
    }

    // Desktop
    if deps.contains_key("electron") {
        return Some(Framework::Electron);
    }
    if deps.contains_key("@tauri-apps/api") || deps.contains_key("tauri") {
        return Some(Framework::Tauri);
    }

    // Web frameworks
    if deps.contains_key("next") {
        return Some(Framework::NextJs);
    }
    if deps.contains_key("@remix-run/react") || deps.contains_key("remix") {
        return Some(Framework::Remix);
    }
    if deps.contains_key("astro") {
        return Some(Framework::Astro);
    }

    // Backend Node frameworks
    if deps.contains_key("elysia") {
        return Some(Framework::Elysia);
    }
    if deps.contains_key("@nestjs/core") {
        return Some(Framework::NestJs);
    }
    if deps.contains_key("fastify") {
        return Some(Framework::Fastify);
    }
    if deps.contains_key("hono") {
        return Some(Framework::Hono);
    }
    if deps.contains_key("express") {
        return Some(Framework::Express);
    }

    // Generic React (default to web)
    if deps.contains_key("react") || deps.contains_key("react-dom") {
        return Some(Framework::Vite); // Generic web
    }

    None
}

/// Detect framework from Cargo.toml content
pub fn detect_from_cargo_toml(content: &str) -> Option<Framework> {
    let lower = content.to_lowercase();

    if lower.contains("axum") {
        return Some(Framework::Axum);
    }
    if lower.contains("actix") {
        return Some(Framework::Actix);
    }
    if lower.contains("rocket") {
        return Some(Framework::Rocket);
    }

    None
}

/// Detect framework from go.mod content
pub fn detect_from_go_mod(content: &str) -> Option<Framework> {
    let lower = content.to_lowercase();

    if lower.contains("go-chi/chi") {
        return Some(Framework::Chi);
    }
    if lower.contains("gin-gonic/gin") {
        return Some(Framework::Gin);
    }
    if lower.contains("labstack/echo") {
        return Some(Framework::Echo);
    }
    if lower.contains("gofiber/fiber") {
        return Some(Framework::Fiber);
    }

    None
}

fn merge_deps(pkg: &Value) -> std::collections::HashMap<String, bool> {
    let mut deps = std::collections::HashMap::new();

    if let Some(d) = pkg.get("dependencies").and_then(|v| v.as_object()) {
        for key in d.keys() {
            deps.insert(key.clone(), true);
        }
    }
    if let Some(d) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
        for key in d.keys() {
            deps.insert(key.clone(), true);
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expo_from_package_json() {
        let pkg = r#"{"dependencies": {"expo": "52.0.0", "react-native": "0.76.0"}}"#;
        assert_eq!(detect_from_package_json(pkg), Some(Framework::Expo));
    }

    #[test]
    fn test_nextjs_from_package_json() {
        let pkg = r#"{"dependencies": {"next": "15.0.0", "react": "19.0.0"}}"#;
        assert_eq!(detect_from_package_json(pkg), Some(Framework::NextJs));
    }

    #[test]
    fn test_electron_from_package_json() {
        let pkg = r#"{"dependencies": {"electron": "29.0.0", "react": "19.0.0"}}"#;
        assert_eq!(detect_from_package_json(pkg), Some(Framework::Electron));
    }

    #[test]
    fn test_elysia_from_package_json() {
        let pkg = r#"{"dependencies": {"elysia": "1.0.0", "effect": "3.0.0"}}"#;
        assert_eq!(detect_from_package_json(pkg), Some(Framework::Elysia));
    }

    #[test]
    fn test_unity_from_path() {
        assert_eq!(
            detect_from_path("Assets/Scripts/PlayerController.cs"),
            Some(Framework::Unity)
        );
    }

    #[test]
    fn test_unreal_from_path() {
        assert_eq!(
            detect_from_path("Source/MyGame/PlayerCharacter.cpp"),
            Some(Framework::Unreal)
        );
    }

    #[test]
    fn test_expo_from_path() {
        assert_eq!(
            detect_from_path("app/(tabs)/home.tsx"),
            Some(Framework::Expo)
        );
    }

    #[test]
    fn test_axum_from_cargo() {
        let cargo = r#"[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
"#;
        assert_eq!(detect_from_cargo_toml(cargo), Some(Framework::Axum));
    }

    #[test]
    fn test_chi_from_go_mod() {
        let gomod = r#"module example.com/myapp

require github.com/go-chi/chi/v5 v5.0.12
"#;
        assert_eq!(detect_from_go_mod(gomod), Some(Framework::Chi));
    }
}
