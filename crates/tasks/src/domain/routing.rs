//! Agent routing - Infer which agent should handle a task based on its content.
//!
//! This module provides logic to automatically assign agent hints to tasks
//! based on keywords in the task title and description.

/// Agent types available for task routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    /// Frontend/React engineer (Blaze)
    Blaze,
    /// Rust/backend engineer (Rex)
    Rex,
    /// Go engineer (Grizz)
    Grizz,
    /// Mobile/React Native engineer (Tap)
    Tap,
    /// Desktop/Electron engineer (Spark)
    Spark,
    /// Node.js engineer (Nova)
    Nova,
    /// QA/Testing engineer (Tess)
    Tess,
    /// Security engineer (Cipher)
    Cipher,
    /// DevOps/Deployment engineer (Bolt)
    Bolt,
    /// Integration/Merge engineer (Atlas)
    Atlas,
}

impl Agent {
    /// Get the agent name as a lowercase string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Blaze => "blaze",
            Self::Rex => "rex",
            Self::Grizz => "grizz",
            Self::Tap => "tap",
            Self::Spark => "spark",
            Self::Nova => "nova",
            Self::Tess => "tess",
            Self::Cipher => "cipher",
            Self::Bolt => "bolt",
            Self::Atlas => "atlas",
        }
    }

    /// Get the role description for this agent.
    #[must_use]
    pub const fn role_description(&self) -> &'static str {
        match self {
            Self::Blaze => {
                "Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX"
            }
            Self::Rex => "Senior Rust Engineer with expertise in systems programming and APIs",
            Self::Grizz => {
                "Senior Go Engineer with expertise in concurrent systems and microservices"
            }
            Self::Tap => {
                "Senior Mobile Engineer with expertise in React Native and cross-platform development"
            }
            Self::Spark => {
                "Senior Desktop Engineer with expertise in Electron and native integrations"
            }
            Self::Nova => {
                "Senior Node.js Engineer with expertise in server-side JavaScript and APIs"
            }
            Self::Tess => {
                "Senior QA Engineer with expertise in test automation and quality assurance"
            }
            Self::Cipher => {
                "Senior Security Engineer with expertise in authentication and secure coding"
            }
            Self::Bolt => "Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD",
            Self::Atlas => {
                "Senior Integration Engineer with expertise in system integration and merging"
            }
        }
    }
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Infer which agent should handle a task based on its title and description.
///
/// The inference is based on keyword matching in the combined content.
/// Order matters - more specific matches (like mobile) are checked before
/// more general ones (like frontend).
///
/// IMPORTANT: This function only considers title and description, not test_strategy
/// or other fields, to avoid false positives from generic keywords.
#[must_use]
pub fn infer_agent_hint(title: &str, description: &str) -> Agent {
    let content = format!("{} {}", title, description).to_lowercase();

    // HIGHEST PRIORITY: Explicit agent name in parentheses (e.g., "(Nova - Bun)")
    // This allows PRD authors to explicitly specify the agent
    if content.contains("(nova") || content.contains("- nova") {
        return Agent::Nova;
    }
    if content.contains("(grizz") || content.contains("- grizz") {
        return Agent::Grizz;
    }
    if content.contains("(rex") || content.contains("- rex") {
        return Agent::Rex;
    }
    if content.contains("(blaze") || content.contains("- blaze") {
        return Agent::Blaze;
    }
    if content.contains("(tap") || content.contains("- tap") {
        return Agent::Tap;
    }
    if content.contains("(spark") || content.contains("- spark") {
        return Agent::Spark;
    }
    if content.contains("(bolt") || content.contains("- bolt") {
        return Agent::Bolt;
    }
    if content.contains("(cipher") || content.contains("- cipher") {
        return Agent::Cipher;
    }
    if content.contains("(tess") || content.contains("- tess") {
        return Agent::Tess;
    }
    if content.contains("(atlas") || content.contains("- atlas") {
        return Agent::Atlas;
    }

    // Check in order of specificity (most specific first)

    // Mobile (before frontend since React Native could match "react")
    if content.contains("mobile")
        || content.contains("react native")
        || content.contains("ios")
        || content.contains("android")
        || content.contains("expo")
    {
        return Agent::Tap;
    }

    // Desktop/Electron (before frontend)
    if content.contains("electron")
        || content.contains("desktop")
        || content.contains("native app")
        || content.contains("tauri")
    {
        return Agent::Spark;
    }

    // Security (early - auth/jwt/encrypt are strong signals)
    if content.contains("security")
        || content.contains("auth")
        || content.contains("encrypt")
        || content.contains("oauth")
        || content.contains("jwt")
        || content.contains("permission")
        || content.contains("rbac")
        || content.contains("vulnerability")
        || content.contains("password")
    {
        return Agent::Cipher;
    }

    // DevOps/Deployment (early - deploy/docker/k8s are strong signals)
    if content.contains("deploy")
        || content.contains("ci/cd")
        || content.contains("ci cd")
        || content.contains("kubernetes")
        || content.contains("k8s")
        || content.contains("helm")
        || content.contains("docker")
        || content.contains("terraform")
        || content.contains("ansible")
        || content.contains("pipeline")
        || content.contains("gitops")
        || content.contains("containerize")
        || content.contains("infrastructure")
    {
        return Agent::Bolt;
    }

    // Go - check BEFORE general backend since Go services are often APIs too
    // Be more generous with Go detection patterns
    if content.contains("golang")
        || content.contains("goroutine")
        || content.contains(" go ")
        || content.contains("go/")
        || content.contains("/go")
        || content.contains("(go)")
        || content.contains("gin ")
        || content.contains("fiber")
        || content.contains("echo ")
        || content.contains("chi ")
        || content.contains("grpc")
        || content.contains("protobuf")
    {
        return Agent::Grizz;
    }

    // Node.js - check BEFORE general backend
    // Include modern JS/TS runtime and framework keywords
    if content.contains("node.js")
        || content.contains("nodejs")
        || content.contains("express")
        || content.contains("fastify")
        || content.contains("nestjs")
        || content.contains("npm")
        || content.contains("yarn")
        || content.contains("bun")
        || content.contains("deno")
        || content.contains("elysia")
        || content.contains("hono")
        || content.contains("effect")
        || content.contains("drizzle")
        || content.contains("prisma")
    {
        return Agent::Nova;
    }

    // Frontend
    if content.contains("frontend")
        || content.contains("react")
        || content.contains("ui ")
        || content.contains(" ui")
        || content.contains("css")
        || content.contains("component")
        || content.contains("tailwind")
        || content.contains("next.js")
        || content.contains("nextjs")
        || content.contains("vue")
        || content.contains("angular")
        || content.contains("svelte")
    {
        return Agent::Blaze;
    }

    // Rust/Backend - now checked after Go/Node to avoid overshadowing
    if content.contains("rust")
        || content.contains("cargo")
        || content.contains("actix")
        || content.contains("axum")
        || content.contains("tokio")
        || content.contains("wasm")
        || content.contains("sqlx")
    {
        return Agent::Rex;
    }

    // Generic backend keywords - default to Rex for these
    if content.contains("backend")
        || content.contains("api ")
        || content.contains(" api")
        || content.contains("endpoint")
        || content.contains("database")
        || content.contains("schema")
        || content.contains("migration")
        || content.contains("user")
        || content.contains("profile")
        || content.contains("admin")
        || content.contains("crud")
        || content.contains("postgresql")
        || content.contains("redis")
    {
        return Agent::Rex;
    }

    // Testing/QA (late - require more specific markers to avoid false positives)
    // Note: "test" alone is too generic, require "testing", "test suite", etc.
    if content.contains("testing")
        || content.contains("test suite")
        || content.contains("test coverage")
        || content.contains("qa ")
        || content.contains("quality assurance")
        || content.contains("jest")
        || content.contains("cypress")
        || content.contains("playwright")
        || content.contains("e2e test")
    {
        return Agent::Tess;
    }

    // Integration/Merge (LAST - these are very generic keywords)
    // Only match if no other agent was matched
    if content.contains("merge")
        || content.contains("conflict")
        || content.contains("consolidate")
        || content.contains("combine")
    {
        return Agent::Atlas;
    }
    // Note: "integration" removed from Atlas - too generic and conflicts with
    // "Integration Service" which should go to a backend agent

    // Default to Rex (Rust/backend)
    Agent::Rex
}

/// Infer agent hint and return as a string.
#[must_use]
pub fn infer_agent_hint_str(title: &str, description: &str) -> &'static str {
    infer_agent_hint(title, description).as_str()
}

/// Get role description for an agent hint string.
#[must_use]
pub fn get_role_for_hint(hint: &str) -> &'static str {
    match hint.to_lowercase().as_str() {
        "blaze" => Agent::Blaze.role_description(),
        "rex" => Agent::Rex.role_description(),
        "grizz" => Agent::Grizz.role_description(),
        "tap" => Agent::Tap.role_description(),
        "spark" => Agent::Spark.role_description(),
        "nova" => Agent::Nova.role_description(),
        "tess" => Agent::Tess.role_description(),
        "cipher" => Agent::Cipher.role_description(),
        "bolt" => Agent::Bolt.role_description(),
        "atlas" => Agent::Atlas.role_description(),
        _ => "Senior Software Engineer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_agent_names() {
        // Test explicit agent names in parentheses (highest priority)
        assert_eq!(
            infer_agent_hint(
                "Setup Integration Service (Nova - Bun/Elysia)",
                "API service"
            ),
            Agent::Nova
        );
        assert_eq!(
            infer_agent_hint("Setup Admin API (Grizz - Go/gRPC)", "Backend service"),
            Agent::Grizz
        );
        assert_eq!(
            infer_agent_hint(
                "Router Service (Rex - Rust/Axum)",
                "High-performance router"
            ),
            Agent::Rex
        );
        assert_eq!(
            infer_agent_hint("Dashboard (Blaze - React)", "Admin UI"),
            Agent::Blaze
        );
    }

    #[test]
    fn test_frontend_detection() {
        assert_eq!(
            infer_agent_hint("Build React component", "Create a UI form"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Frontend layout", "CSS styling"),
            Agent::Blaze
        );
    }

    #[test]
    fn test_rust_detection() {
        assert_eq!(
            infer_agent_hint("Implement service", "Rust axum server"),
            Agent::Rex
        );
        assert_eq!(
            infer_agent_hint("Cargo workspace", "Build system"),
            Agent::Rex
        );
    }

    #[test]
    fn test_mobile_detection() {
        assert_eq!(
            infer_agent_hint("Mobile app", "React Native screen"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("iOS feature", "Push notifications"),
            Agent::Tap
        );
    }

    #[test]
    fn test_go_detection() {
        assert_eq!(
            infer_agent_hint("Go service", "Golang microservice"),
            Agent::Grizz
        );
        // Test Go/gRPC pattern
        assert_eq!(
            infer_agent_hint("Admin API", "Go/gRPC backend"),
            Agent::Grizz
        );
        // Test gRPC alone
        assert_eq!(
            infer_agent_hint("gRPC service", "Protocol buffers"),
            Agent::Grizz
        );
    }

    #[test]
    fn test_nodejs_detection() {
        // Test Elysia/Effect (modern Bun stack)
        assert_eq!(
            infer_agent_hint("Integration Service", "Bun with Elysia framework"),
            Agent::Nova
        );
        assert_eq!(
            infer_agent_hint("API Service", "Effect TypeScript"),
            Agent::Nova
        );
        // Test traditional Node patterns
        assert_eq!(
            infer_agent_hint("Express API", "Node.js server"),
            Agent::Nova
        );
    }

    #[test]
    fn test_security_detection() {
        assert_eq!(
            infer_agent_hint("Auth system", "OAuth provider"),
            Agent::Cipher
        );
        assert_eq!(
            infer_agent_hint("JWT validation", "Security middleware"),
            Agent::Cipher
        );
    }

    #[test]
    fn test_devops_detection() {
        assert_eq!(
            infer_agent_hint("Deploy pipeline", "Kubernetes manifest"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("CI/CD setup", "GitHub Actions"),
            Agent::Bolt
        );
        // Test infrastructure keyword
        assert_eq!(
            infer_agent_hint("Infrastructure setup", "Database provisioning"),
            Agent::Bolt
        );
    }

    #[test]
    fn test_testing_detection() {
        assert_eq!(
            infer_agent_hint("Write tests", "Unit test coverage"),
            Agent::Tess
        );
        assert_eq!(infer_agent_hint("E2E testing", "Playwright"), Agent::Tess);
    }

    #[test]
    fn test_default_to_rex() {
        assert_eq!(
            infer_agent_hint("Generic task", "No specific keywords"),
            Agent::Rex
        );
    }

    #[test]
    fn test_integration_not_atlas() {
        // "Integration Service" should NOT match Atlas (too generic)
        // Without explicit agent hint, it should fall through to generic backend (Rex)
        assert_eq!(
            infer_agent_hint("Integration Service", "Connects systems together"),
            Agent::Rex
        );
    }

    #[test]
    fn test_role_descriptions() {
        assert!(Agent::Blaze.role_description().contains("Frontend"));
        assert!(Agent::Rex.role_description().contains("Rust"));
        assert!(Agent::Cipher.role_description().contains("Security"));
    }

    #[test]
    fn test_agent_str() {
        assert_eq!(Agent::Blaze.as_str(), "blaze");
        assert_eq!(Agent::Rex.as_str(), "rex");
        assert_eq!(Agent::Tap.as_str(), "tap");
    }
}
