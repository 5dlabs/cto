//! Agent routing - Infer which agent should handle a task based on its content.
//!
//! This module provides logic to automatically assign agent hints to tasks
//! based on keywords in the task title and description, with support for
//! dependency-based inference as the primary signal.

use crate::entities::Task;

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

/// Check if a word appears as a complete word in the content.
/// This avoids false positives like "expo" matching "exponential".
fn is_word_match(content: &str, word: &str) -> bool {
    // Check for common word boundary patterns
    content == word
        || content.starts_with(&format!("{} ", word))
        || content.ends_with(&format!(" {}", word))
        || content.contains(&format!(" {} ", word))
        || content.contains(&format!(" {}-", word))
        || content.contains(&format!("-{} ", word))
        || content.contains(&format!("({}", word))
        || content.contains(&format!("{})", word))
}

/// Infer which agent should handle a task based on its title and description.
///
/// The inference is based on keyword matching in the combined content.
/// Order matters - more specific matches (like mobile) are checked before
/// more general ones (like frontend).
///
/// IMPORTANT: This function only considers title and description, not test_strategy
/// or other fields, to avoid false positives from generic keywords.
///
/// Returns `None` if no agent can be determined - this is intentional to force
/// explicit handling of routing gaps rather than silent misrouting.
///
/// For dependency-aware routing (recommended), use `infer_agent_hint_with_deps` instead.
#[must_use]
pub fn infer_agent_hint(title: &str, description: &str) -> Option<Agent> {
    let content = format!("{} {}", title, description).to_lowercase();

    // HIGHEST PRIORITY: Explicit agent name in parentheses (e.g., "(Nova - Bun)")
    // This allows PRD authors to explicitly specify the agent
    if let Some(agent) = check_explicit_agent(title, description) {
        return Some(agent);
    }

    // Check in order of specificity (most specific first)

    // Mobile (before frontend since React Native could match "react")
    // Expanded with screen patterns, push notifications, FCM/APNs
    // NOTE: Use word boundaries for short keywords to avoid false positives
    // (e.g., "expo" should not match "exponential")
    if content.contains("mobile")
        || content.contains("react native")
        || content.contains("ios")
        || content.contains("android")
        // Expo - use word boundaries to avoid matching "exponential", "export", etc.
        || is_word_match(&content, "expo")
        // Mobile screen patterns
        || content.contains("home screen")
        || content.contains("detail screen")
        || content.contains("settings screen")
        || content.contains("profile screen")
        || content.contains("notification screen")
        // Push notification infrastructure
        || content.contains("push notification")
        || content.contains("fcm")
        || content.contains("apns")
        || content.contains("app badge")
        || content.contains("deep link")
        || content.contains("biometric")
    {
        return Some(Agent::Tap);
    }

    // Desktop/Electron (before frontend)
    // Expanded with tray, window, auto-start patterns
    if content.contains("electron")
        || content.contains("desktop")
        || content.contains("native app")
        || content.contains("tauri")
        // Desktop-specific patterns
        || content.contains("system tray")
        || content.contains("tray icon")
        || content.contains("tray menu")
        || content.contains("notification badge")
        || content.contains("auto-start")
        || content.contains("auto start")
        || content.contains("startup on boot")
        || content.contains("main window")
        || content.contains("mini window")
        || content.contains("popup window")
        || content.contains("cross-platform")
    {
        return Some(Agent::Spark);
    }

    // NOTE: Security keywords (auth, jwt, oauth, rbac) are checked MUCH LATER
    // because these are usually IMPLEMENTATION tasks, not security audits.
    // Cipher is only for explicit security audit/review tasks.

    // DevOps/Deployment/Infrastructure (early - deploy/docker/k8s are strong signals)
    // EXPANDED: Includes observability (Grafana, Prometheus, Loki), GitOps tools,
    // K8s resources, and storage infrastructure.
    // This MUST be checked BEFORE frontend to catch "Grafana dashboard" correctly.
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
        || content.contains("provision")
        || content.contains("setup cluster")
        || content.contains("setup database")
        || content.contains("setup redis")
        || content.contains("setup kafka")
        || content.contains("operator")
        || content.contains("cloudnative-pg")
        || content.contains("strimzi")
        || content.contains("percona")
        // Observability stack (checked BEFORE frontend "dashboard")
        || content.contains("grafana")
        || content.contains("prometheus")
        || content.contains("loki")
        || content.contains("alertmanager")
        || content.contains("observability")
        || content.contains("monitoring setup")
        || content.contains("jaeger")
        || content.contains("opentelemetry")
        || content.contains("otel")
        || content.contains("fluent")
        // GitOps tools
        || content.contains("argocd")
        || content.contains("flux")
        || content.contains("kustomize")
        // K8s resources (provisioning, not app usage)
        || content.contains("configmap")
        || content.contains("ingress")
        || content.contains("network policy")
        || content.contains("service mesh")
        || content.contains(" hpa")
        || content.contains("hpa ")
        || content.contains(" pvc")
        || content.contains("pvc ")
        || content.contains("persistentvolume")
        || content.contains("deployment manifest")
        || content.contains("kubernetes manifest")
        // Storage infrastructure
        || content.contains("seaweedfs")
        || content.contains("minio")
        || content.contains("s3 bucket")
    {
        return Some(Agent::Bolt);
    }

    // Go - check BEFORE general backend since Go services are often APIs too
    // Be more generous with Go detection patterns
    // NOTE: "gin " removed because it matches "login " - use "gin framework" or explicit go hints
    if content.contains("golang")
        || content.contains("goroutine")
        || content.contains(" go ")
        || content.contains("go/")
        || content.contains("/go")
        || content.contains("(go)")
        || content.contains("gin framework")
        || content.contains("fiber")
        || content.contains("echo ")
        || content.contains("chi ")
        || content.contains("grpc")
        || content.contains("protobuf")
    {
        return Some(Agent::Grizz);
    }

    // Effect TypeScript - CONTEXT DETERMINES frontend vs backend
    // Effect can be used in both Blaze (frontend) and Nova (backend)
    // Check context keywords to determine which agent
    if content.contains("effect") {
        // Frontend context: component, page, form, ui, react, next, shadcn
        if content.contains("component")
            || content.contains(" page")
            || content.contains("page ")
            || content.contains(" form")
            || content.contains("form ")
            || content.contains(" ui")
            || content.contains("ui ")
            || content.contains("react")
            || content.contains("next")
            || content.contains("shadcn")
            || content.contains("frontend")
            || content.contains("validation")
        // Form validation context
        {
            return Some(Agent::Blaze);
        }
        // Backend context: service, api, delivery, kafka, queue, stream, elysia, bun
        // Default Effect to Nova (backend) since it's primarily used there
        return Some(Agent::Nova);
    }

    // Node.js - check BEFORE general backend
    // Include modern JS/TS runtime and framework keywords
    // NOTE: "effect" moved to its own context-aware block above
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
        || content.contains("drizzle")
        || content.contains("prisma")
        || content.contains("kafkajs")
        // Async/queue processing patterns (typically Node.js/Bun)
        || content.contains("background worker")
        || content.contains("worker pool")
        || content.contains("job queue")
        || content.contains("priority queue")
        || content.contains("message queue")
        || content.contains("task queue")
        // Rate limiting and processing patterns (NOT generic "middleware" - too broad)
        || content.contains("rate limit")
        || content.contains("rate-limit")
        || content.contains("throttling")
        || content.contains("deduplication")
        || content.contains("sliding window")
    {
        return Some(Agent::Nova);
    }

    // Frontend (check before generic backend to catch "admin dashboard", "web app", etc.)
    // EXPANDED: Added page patterns for web pages
    // NOTE: "dashboard" still here but Grafana/observability caught by Bolt earlier
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
        || content.contains("dashboard")
        || content.contains("admin panel")
        || content.contains("web app")
        || content.contains("website")
        || content.contains("shadcn")
        || content.contains("landing page")
        || content.contains("web interface")
        // Web page patterns (after mobile "screen" check)
        || content.contains(" page")
        || content.contains("page ")
        // Specific page types
        || content.contains("history page")
        || content.contains("management page")
        || content.contains("settings page")
        || content.contains("analytics page")
        || content.contains("notifications page")
        || content.contains("integrations page")
        || content.contains("rules page")
        // Web console patterns
        || content.contains("web console")
        || content.contains("console ui")
        || content.contains("recharts")
        || content.contains("chart.js")
        // Theme/styling patterns
        || content.contains("theme")
        || content.contains("dark mode")
        || content.contains("light mode")
        || content.contains("color scheme")
        || content.contains("local storage")
    {
        return Some(Agent::Blaze);
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
        return Some(Agent::Rex);
    }

    // Security Audit (BEFORE generic backend to catch "security testing of the API")
    // NOTE: Generic keywords like "auth", "jwt", "oauth", "rbac", "permission"
    // are NOT matched here because they are usually implementation tasks.
    // Cipher is a SUPPORT agent for security reviews, not implementation.
    if content.contains("security audit")
        || content.contains("security review")
        || content.contains("vulnerability scan")
        || content.contains("penetration test")
        || content.contains("security scan")
        || content.contains("security analysis")
        || content.contains("security testing")
    {
        return Some(Agent::Cipher);
    }

    // Generic backend keywords - route to Rex
    // Note: This is NOT a default fallback - these are explicit backend patterns
    // Note: "admin" removed (now caught by frontend "admin panel" or "dashboard")
    // Note: "postgresql", "redis" removed (if it's provisioning, caught by Bolt earlier)
    if content.contains("backend")
        || content.contains("api ")
        || content.contains(" api")
        || content.contains("endpoint")
        || content.contains("schema")
        || content.contains("migration")
        || content.contains("user service")
        || content.contains("profile service")
        || content.contains("crud")
        || content.contains("microservice")
        || content.contains("service layer")
    {
        return Some(Agent::Rex);
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
        return Some(Agent::Tess);
    }

    // Integration/Merge (LAST - these are very generic keywords)
    // Only match if no other agent was matched
    if content.contains("merge")
        || content.contains("conflict")
        || content.contains("consolidate")
        || content.contains("combine")
    {
        return Some(Agent::Atlas);
    }
    // Note: "integration" removed from Atlas - too generic and conflicts with
    // "Integration Service" which should go to a backend agent

    // NO DEFAULT - return None to force explicit handling
    // This ensures we catch routing gaps rather than silently misrouting
    None
}

/// Infer agent hint and return as a string.
/// Returns `None` if no agent can be determined.
#[must_use]
pub fn infer_agent_hint_str(title: &str, description: &str) -> Option<&'static str> {
    infer_agent_hint(title, description).map(|a| a.as_str())
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

/// Parse an agent hint string into an Agent enum.
#[must_use]
pub fn parse_agent(hint: &str) -> Agent {
    match hint.to_lowercase().as_str() {
        "blaze" => Agent::Blaze,
        "grizz" => Agent::Grizz,
        "tap" => Agent::Tap,
        "spark" => Agent::Spark,
        "nova" => Agent::Nova,
        "tess" => Agent::Tess,
        "cipher" => Agent::Cipher,
        "bolt" => Agent::Bolt,
        "atlas" => Agent::Atlas,
        // Default to Rex (including explicit "rex")
        _ => Agent::Rex,
    }
}

/// Check if an agent is an implementation agent (can write code).
/// Support agents (Tess, Cipher, Atlas) only review/test/merge.
#[must_use]
pub const fn is_implementation_agent(agent: Agent) -> bool {
    matches!(
        agent,
        Agent::Blaze
            | Agent::Rex
            | Agent::Grizz
            | Agent::Nova
            | Agent::Tap
            | Agent::Spark
            | Agent::Bolt
    )
}

/// Infer agent from dependency chain.
///
/// If ALL dependencies point to the same implementation agent, inherit that agent.
/// This is useful for tasks like "Create Home screen" that depend on "Initialize Expo project"
/// - the dependency on Tap implies the task should also be Tap.
///
/// Returns `None` if:
/// - Task has no dependencies
/// - Dependencies have mixed agents
/// - Dependencies are support agents (Tess, Cipher, Atlas)
fn infer_from_dependencies(task: &Task, all_tasks: &[Task]) -> Option<Agent> {
    if task.dependencies.is_empty() {
        return None;
    }

    let dep_agents: Vec<Agent> = task
        .dependencies
        .iter()
        .filter_map(|dep_id| {
            all_tasks
                .iter()
                .find(|t| {
                    t.id == *dep_id
                        || format!("task-{:03}", t.id.parse::<u32>().unwrap_or(0)) == *dep_id
                })
                .and_then(|t| t.agent_hint.as_ref())
                .map(|hint| parse_agent(hint))
        })
        .collect();

    if dep_agents.is_empty() {
        return None;
    }

    // All deps must be the same IMPLEMENTATION agent
    let first = dep_agents[0];
    if is_implementation_agent(first) && dep_agents.iter().all(|&a| a == first) {
        Some(first)
    } else {
        None
    }
}

/// Check for explicit agent name in title/description.
/// Returns the agent if found (e.g., "(Nova)", "- Rex", "Rex:", etc.)
///
/// Supports multiple patterns:
/// - Parentheses: "(Nova - Bun)"
/// - Dash prefix: "- nova"
/// - Colon prefix in title: "Rex: Some Task" (common LLM output)
fn check_explicit_agent(title: &str, description: &str) -> Option<Agent> {
    let content = format!("{} {}", title, description).to_lowercase();
    let title_lower = title.to_lowercase();

    // Check for "AgentName:" prefix in title (common LLM output pattern)
    // This handles cases like "Rex: Dead Letter Queue" or "Grizz: JWT Auth"
    if title_lower.starts_with("nova:") || title_lower.starts_with("nova :")  {
        return Some(Agent::Nova);
    }
    if title_lower.starts_with("grizz:") || title_lower.starts_with("grizz :") {
        return Some(Agent::Grizz);
    }
    if title_lower.starts_with("rex:") || title_lower.starts_with("rex :") {
        return Some(Agent::Rex);
    }
    if title_lower.starts_with("blaze:") || title_lower.starts_with("blaze :") {
        return Some(Agent::Blaze);
    }
    if title_lower.starts_with("tap:") || title_lower.starts_with("tap :") {
        return Some(Agent::Tap);
    }
    if title_lower.starts_with("spark:") || title_lower.starts_with("spark :") {
        return Some(Agent::Spark);
    }
    if title_lower.starts_with("bolt:") || title_lower.starts_with("bolt :") {
        return Some(Agent::Bolt);
    }
    if title_lower.starts_with("cipher:") || title_lower.starts_with("cipher :") {
        return Some(Agent::Cipher);
    }
    if title_lower.starts_with("tess:") || title_lower.starts_with("tess :") {
        return Some(Agent::Tess);
    }
    if title_lower.starts_with("atlas:") || title_lower.starts_with("atlas :") {
        return Some(Agent::Atlas);
    }
    if title_lower.starts_with("cleo:") || title_lower.starts_with("cleo :") {
        return Some(Agent::Tess); // Cleo maps to Tess
    }

    // Check for parentheses and dash patterns in full content
    if content.contains("(nova") || content.contains("- nova") {
        return Some(Agent::Nova);
    }
    if content.contains("(grizz") || content.contains("- grizz") {
        return Some(Agent::Grizz);
    }
    if content.contains("(rex") || content.contains("- rex") {
        return Some(Agent::Rex);
    }
    if content.contains("(blaze") || content.contains("- blaze") {
        return Some(Agent::Blaze);
    }
    if content.contains("(tap") || content.contains("- tap") {
        return Some(Agent::Tap);
    }
    if content.contains("(spark") || content.contains("- spark") {
        return Some(Agent::Spark);
    }
    if content.contains("(bolt") || content.contains("- bolt") {
        return Some(Agent::Bolt);
    }
    if content.contains("(cipher") || content.contains("- cipher") {
        return Some(Agent::Cipher);
    }
    if content.contains("(tess") || content.contains("- tess") {
        return Some(Agent::Tess);
    }
    if content.contains("(atlas") || content.contains("- atlas") {
        return Some(Agent::Atlas);
    }
    if content.contains("(cleo") || content.contains("- cleo") {
        // Cleo is an alias for quality review, maps to Tess for now
        return Some(Agent::Tess);
    }

    None
}

/// Infer agent hint with dependency context (PRIMARY signal).
///
/// Priority order:
/// 1. Explicit agent name in title/description (e.g., "(Nova)")
/// 2. Dependency chain inheritance (if ALL deps have same agent)
/// 3. Keyword-based inference (fallback)
///
/// Returns `None` if no agent can be determined - this is intentional to force
/// explicit handling of routing gaps rather than silent misrouting.
///
/// This is the recommended function to use for intake workflows where
/// task dependencies are available.
#[must_use]
pub fn infer_agent_hint_with_deps(task: &Task, all_tasks: &[Task]) -> Option<Agent> {
    // 1. Check explicit agent name first (highest priority)
    if let Some(agent) = check_explicit_agent(&task.title, &task.description) {
        return Some(agent);
    }

    // 2. Check dependency chain (PRIMARY signal for implementation tasks)
    if let Some(agent) = infer_from_dependencies(task, all_tasks) {
        return Some(agent);
    }

    // 3. Fall back to keyword matching (may return None)
    infer_agent_hint(&task.title, &task.description)
}

/// Infer agent hint with dependencies and return as a string.
/// Returns `None` if no agent can be determined.
#[must_use]
pub fn infer_agent_hint_with_deps_str(task: &Task, all_tasks: &[Task]) -> Option<&'static str> {
    infer_agent_hint_with_deps(task, all_tasks).map(|a| a.as_str())
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
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Setup Admin API (Grizz - Go/gRPC)", "Backend service"),
            Some(Agent::Grizz)
        );
        assert_eq!(
            infer_agent_hint(
                "Router Service (Rex - Rust/Axum)",
                "High-performance router"
            ),
            Some(Agent::Rex)
        );
        assert_eq!(
            infer_agent_hint("Dashboard (Blaze - React)", "Admin UI"),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_explicit_agent_colon_prefix() {
        // Test "AgentName:" prefix pattern (common LLM output)
        assert_eq!(
            infer_agent_hint("Rex: Dead Letter Queue", "Implement dead letter queue"),
            Some(Agent::Rex)
        );
        assert_eq!(
            infer_agent_hint("Rex: Kafka Event Publishing", "Implement Kafka publishing"),
            Some(Agent::Rex)
        );
        assert_eq!(
            infer_agent_hint("Grizz: PostgreSQL Repository Layer", "Implement repository"),
            Some(Agent::Grizz)
        );
        assert_eq!(
            infer_agent_hint("Grizz: JWT Authentication Middleware", "Implement JWT auth"),
            Some(Agent::Grizz)
        );
        assert_eq!(
            infer_agent_hint("Blaze: Authentication with Better Auth", "Implement auth flow"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Tap: Core Screens Implementation", "Implement screens"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("Nova: Slack Delivery Service", "Implement Slack integration"),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Bolt: Infrastructure Setup", "Deploy Kubernetes"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Spark: Desktop Notifications", "System tray"),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_frontend_detection() {
        assert_eq!(
            infer_agent_hint("Build React component", "Create a UI form"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Frontend layout", "CSS styling"),
            Some(Agent::Blaze)
        );
        // Dashboard and admin panel should go to Blaze (frontend)
        assert_eq!(
            infer_agent_hint("Admin Dashboard", "Show user statistics"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Build admin panel", "Management interface"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Web app interface", "User-facing application"),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_rust_detection() {
        assert_eq!(
            infer_agent_hint("Implement service", "Rust axum server"),
            Some(Agent::Rex)
        );
        assert_eq!(
            infer_agent_hint("Cargo workspace", "Build system"),
            Some(Agent::Rex)
        );
    }

    #[test]
    fn test_mobile_detection() {
        assert_eq!(
            infer_agent_hint("Mobile app", "React Native screen"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("iOS feature", "Push notifications"),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_go_detection() {
        assert_eq!(
            infer_agent_hint("Go service", "Golang microservice"),
            Some(Agent::Grizz)
        );
        // Test Go/gRPC pattern
        assert_eq!(
            infer_agent_hint("Admin API", "Go/gRPC backend"),
            Some(Agent::Grizz)
        );
        // Test gRPC alone
        assert_eq!(
            infer_agent_hint("gRPC service", "Protocol buffers"),
            Some(Agent::Grizz)
        );
    }

    #[test]
    fn test_nodejs_detection() {
        // Test Elysia/Effect (modern Bun stack)
        assert_eq!(
            infer_agent_hint("Integration Service", "Bun with Elysia framework"),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("API Service", "Effect TypeScript"),
            Some(Agent::Nova)
        );
        // Test traditional Node patterns
        assert_eq!(
            infer_agent_hint("Express API", "Node.js server"),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_queue_processing_detection() {
        // Queue and worker patterns should go to Nova
        assert_eq!(
            infer_agent_hint(
                "Implement Priority Queue Processing",
                "Create background worker for processing notifications from priority queues"
            ),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Worker Pool", "Background worker for job processing"),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Message Queue", "Process messages from queue"),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_rate_limiting_detection() {
        // Rate limiting patterns should go to Nova
        assert_eq!(
            infer_agent_hint(
                "Implement Rate Limiting",
                "Create rate limiting using Redis sliding window algorithm"
            ),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Deduplication Service", "Notification deduplication with TTL"),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_theme_support_detection() {
        // Theme/styling patterns should go to Blaze (frontend)
        assert_eq!(
            infer_agent_hint(
                "Implement Dark/Light Theme Support",
                "Add theme switching with dark and light modes"
            ),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Color Scheme", "Implement color scheme selection"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Settings", "Persist to local storage"),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_security_audit_detection() {
        // ONLY explicit security audit/review tasks go to Cipher
        assert_eq!(
            infer_agent_hint("Security Audit", "Review authentication implementation"),
            Some(Agent::Cipher)
        );
        assert_eq!(
            infer_agent_hint("Vulnerability Scan", "Check for security issues"),
            Some(Agent::Cipher)
        );
        assert_eq!(
            infer_agent_hint("Penetration test", "Security testing"),
            Some(Agent::Cipher)
        );
        assert_eq!(
            infer_agent_hint("Security review", "Audit the codebase"),
            Some(Agent::Cipher)
        );
    }

    #[test]
    fn test_auth_implementation_returns_none() {
        // Auth/JWT/OAuth implementation tasks without language hints
        // should return None - they need explicit routing
        assert_eq!(
            infer_agent_hint("Auth system", "OAuth provider implementation"),
            None // No default - needs explicit hint
        );
        assert_eq!(
            infer_agent_hint("JWT validation", "Implement JWT middleware"),
            None // No default - needs explicit hint
        );
        assert_eq!(
            infer_agent_hint("RBAC implementation", "Role-based access control"),
            None // No default - needs explicit hint
        );
        // If language is specified, route to that agent
        assert_eq!(
            infer_agent_hint("JWT Authentication", "Go/gRPC backend with auth"),
            Some(Agent::Grizz)
        );
        assert_eq!(
            infer_agent_hint(
                "OAuth2 Token Management",
                "Effect TypeScript implementation"
            ),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_devops_detection() {
        assert_eq!(
            infer_agent_hint("Deploy pipeline", "Kubernetes manifest"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("CI/CD setup", "GitHub Actions"),
            Some(Agent::Bolt)
        );
        // Test infrastructure keyword
        assert_eq!(
            infer_agent_hint("Infrastructure setup", "Database provisioning"),
            Some(Agent::Bolt)
        );
        // Provision keywords should route to Bolt
        assert_eq!(
            infer_agent_hint("Provision PostgreSQL", "Setup database cluster"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Setup Redis cluster", "Cache infrastructure"),
            Some(Agent::Bolt)
        );
        // Operator keywords should route to Bolt
        assert_eq!(
            infer_agent_hint("Deploy CloudNative-PG", "PostgreSQL operator"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Setup Strimzi Kafka", "Event streaming"),
            Some(Agent::Bolt)
        );
    }

    #[test]
    fn test_testing_detection() {
        assert_eq!(
            infer_agent_hint("Write tests", "Unit test coverage"),
            Some(Agent::Tess)
        );
        assert_eq!(
            infer_agent_hint("E2E testing", "Playwright"),
            Some(Agent::Tess)
        );
    }

    #[test]
    fn test_no_default_returns_none() {
        // Tasks without matching keywords should return None
        assert_eq!(
            infer_agent_hint("Generic task", "No specific keywords"),
            None
        );
    }

    #[test]
    fn test_integration_service_returns_none() {
        // "Integration Service" should NOT match Atlas (too generic)
        // Without explicit agent hint, it should return None
        assert_eq!(
            infer_agent_hint("Integration Service", "Connects systems together"),
            None
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

    // ===========================================================================
    // NEW TESTS: Dependency-Based Routing
    // ===========================================================================

    #[test]
    fn test_dependency_based_routing_tap() {
        // Task depending on Tap init should inherit Tap
        let mut tap_init = Task::new("41", "Initialize Expo project", "Create Expo SDK setup");
        tap_init.agent_hint = Some("tap".to_string());

        let mut push_task = Task::new(
            "42",
            "Implement push notification registration",
            "Setup FCM/APNs",
        );
        push_task.dependencies = vec!["41".to_string()];

        let tasks = vec![tap_init, push_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&push_task, &tasks),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_dependency_based_routing_spark() {
        // Task depending on Spark init should inherit Spark
        let mut spark_init =
            Task::new("45", "Initialize Electron project", "Create Electron setup");
        spark_init.agent_hint = Some("spark".to_string());

        let mut tray_task = Task::new("46", "Implement system tray", "Notification badge");
        tray_task.dependencies = vec!["45".to_string()];

        let tasks = vec![spark_init, tray_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&tray_task, &tasks),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_dependency_based_routing_blaze() {
        // Task depending on Blaze init should inherit Blaze
        let mut blaze_init = Task::new("32", "Initialize Next.js project", "Create Next.js setup");
        blaze_init.agent_hint = Some("blaze".to_string());

        let mut page_task = Task::new("35", "Create Notifications page", "History with filters");
        page_task.dependencies = vec!["32".to_string()];

        let tasks = vec![blaze_init, page_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&page_task, &tasks),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_dependency_mixed_agents_returns_none() {
        // Task with mixed dependencies and no keywords should return None
        let mut rex_task = Task::new("1", "Rust service", "Backend");
        rex_task.agent_hint = Some("rex".to_string());

        let mut nova_task = Task::new("2", "Node service", "Backend");
        nova_task.agent_hint = Some("nova".to_string());

        // Note: avoid keywords - this should return None
        let mut mixed_dep_task = Task::new("3", "Shared work", "Interoperates with multiple");
        mixed_dep_task.dependencies = vec!["1".to_string(), "2".to_string()];

        let tasks = vec![rex_task, nova_task, mixed_dep_task.clone()];
        // Mixed deps and no keywords → None
        assert_eq!(infer_agent_hint_with_deps(&mixed_dep_task, &tasks), None);
    }

    #[test]
    fn test_explicit_agent_overrides_dependency() {
        // Explicit agent name should override dependency inference
        let mut tap_init = Task::new("41", "Initialize Expo project", "Create Expo setup");
        tap_init.agent_hint = Some("tap".to_string());

        let mut explicit_task = Task::new(
            "42",
            "Some task (Nova - special case)",
            "Needs Nova despite dependency",
        );
        explicit_task.dependencies = vec!["41".to_string()];

        let tasks = vec![tap_init, explicit_task.clone()];
        // Explicit "(Nova" should win over Tap dependency
        assert_eq!(
            infer_agent_hint_with_deps(&explicit_task, &tasks),
            Some(Agent::Nova)
        );
    }

    // ===========================================================================
    // NEW TESTS: Bolt Observability Keywords
    // ===========================================================================

    #[test]
    fn test_bolt_observability() {
        // Grafana dashboards should go to Bolt, not Blaze
        assert_eq!(
            infer_agent_hint(
                "Setup Grafana dashboards",
                "Create dashboards for monitoring"
            ),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Prometheus rules", "Alert configuration"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Loki logging", "Configure log aggregation"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Observability stack", "Metrics and logging"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Alertmanager setup", "Alert routing"),
            Some(Agent::Bolt)
        );
    }

    #[test]
    fn test_bolt_gitops() {
        assert_eq!(
            infer_agent_hint("ArgoCD application", "GitOps deployment"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Flux configuration", "Continuous delivery"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Kustomize overlays", "Environment configs"),
            Some(Agent::Bolt)
        );
    }

    #[test]
    fn test_bolt_k8s_resources() {
        assert_eq!(
            infer_agent_hint("Create ConfigMap", "Application config"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Ingress rules", "External access"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("Network policy", "Pod isolation"),
            Some(Agent::Bolt)
        );
        assert_eq!(
            infer_agent_hint("HPA configuration", "Auto-scaling"),
            Some(Agent::Bolt)
        );
    }

    // ===========================================================================
    // NEW TESTS: Effect Context-Aware Routing
    // ===========================================================================

    #[test]
    fn test_effect_frontend_context() {
        // Effect + frontend keywords → Blaze
        assert_eq!(
            infer_agent_hint("Settings Form", "Effect Schema validation in React"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Effect validation", "Component form validation"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Effect Schema", "Page state management"),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_effect_backend_context() {
        // Effect + backend keywords → Nova
        assert_eq!(
            infer_agent_hint("Slack Service", "Effect retry for delivery"),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Effect Stream", "Kafka consumer with Effect"),
            Some(Agent::Nova)
        );
        assert_eq!(
            infer_agent_hint("Delivery service", "Effect error handling"),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_effect_default_to_nova() {
        // Effect without clear context → Nova (backend default)
        assert_eq!(
            infer_agent_hint("Effect implementation", "Type-safe error handling"),
            Some(Agent::Nova)
        );
    }

    // ===========================================================================
    // NEW TESTS: Frontend Page Keywords
    // ===========================================================================

    #[test]
    fn test_frontend_pages() {
        assert_eq!(
            infer_agent_hint("Notifications page", "History with filters"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Analytics page", "Charts and metrics"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Settings page", "User preferences"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Create management page", "CRUD interface"),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_frontend_web_console() {
        assert_eq!(
            infer_agent_hint("Web console", "Admin interface"),
            Some(Agent::Blaze)
        );
        assert_eq!(
            infer_agent_hint("Recharts visualization", "Data charts"),
            Some(Agent::Blaze)
        );
    }

    // ===========================================================================
    // NEW TESTS: Mobile Screen Keywords
    // ===========================================================================

    #[test]
    fn test_mobile_screens() {
        assert_eq!(
            infer_agent_hint("Home screen", "Notification feed"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("Detail screen", "Full notification view"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("Settings screen", "Mobile preferences"),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_mobile_push_notifications() {
        assert_eq!(
            infer_agent_hint("Push notification", "FCM registration"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("FCM integration", "Firebase messaging"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("APNs setup", "Apple push notifications"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("App badge count", "Unread notifications"),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_mobile_deep_links() {
        assert_eq!(
            infer_agent_hint("Deep link", "URL scheme handling"),
            Some(Agent::Tap)
        );
        assert_eq!(
            infer_agent_hint("Biometric auth", "Face ID / fingerprint"),
            Some(Agent::Tap)
        );
    }

    // ===========================================================================
    // NEW TESTS: Desktop Window/Tray Keywords
    // ===========================================================================

    #[test]
    fn test_desktop_tray() {
        assert_eq!(
            infer_agent_hint("System tray", "Notification badge"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Tray icon", "Status indicator"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Tray menu", "Quick actions"),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_desktop_windows() {
        assert_eq!(
            infer_agent_hint("Main window", "Full notification feed"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Mini window", "Quick view popup"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Popup window", "Notification toast"),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_desktop_autostart() {
        assert_eq!(
            infer_agent_hint("Auto-start", "Boot preferences"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Auto start on boot", "Startup configuration"),
            Some(Agent::Spark)
        );
        assert_eq!(
            infer_agent_hint("Cross-platform", "Windows/macOS/Linux"),
            Some(Agent::Spark)
        );
    }

    // ===========================================================================
    // NEW TESTS: Helper Functions
    // ===========================================================================

    #[test]
    fn test_parse_agent() {
        assert_eq!(parse_agent("blaze"), Agent::Blaze);
        assert_eq!(parse_agent("BLAZE"), Agent::Blaze);
        assert_eq!(parse_agent("Rex"), Agent::Rex);
        assert_eq!(parse_agent("unknown"), Agent::Rex); // Default
    }

    #[test]
    fn test_is_implementation_agent() {
        assert!(is_implementation_agent(Agent::Blaze));
        assert!(is_implementation_agent(Agent::Rex));
        assert!(is_implementation_agent(Agent::Grizz));
        assert!(is_implementation_agent(Agent::Nova));
        assert!(is_implementation_agent(Agent::Tap));
        assert!(is_implementation_agent(Agent::Spark));
        assert!(is_implementation_agent(Agent::Bolt));
        // Support agents
        assert!(!is_implementation_agent(Agent::Tess));
        assert!(!is_implementation_agent(Agent::Cipher));
        assert!(!is_implementation_agent(Agent::Atlas));
    }
}
