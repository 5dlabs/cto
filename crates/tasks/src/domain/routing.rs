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

/// Infer which agent should handle a task based on its title and description.
///
/// The inference is based on keyword matching in the combined content.
/// Order matters - more specific matches (like mobile) are checked before
/// more general ones (like frontend).
///
/// IMPORTANT: This function only considers title and description, not test_strategy
/// or other fields, to avoid false positives from generic keywords.
///
/// For dependency-aware routing (recommended), use `infer_agent_hint_with_deps` instead.
#[must_use]
pub fn infer_agent_hint(title: &str, description: &str) -> Agent {
    let content = format!("{} {}", title, description).to_lowercase();

    // HIGHEST PRIORITY: Explicit agent name in parentheses (e.g., "(Nova - Bun)")
    // This allows PRD authors to explicitly specify the agent
    if let Some(agent) = check_explicit_agent(title, description) {
        return agent;
    }

    // Check in order of specificity (most specific first)

    // Mobile (before frontend since React Native could match "react")
    // Expanded with screen patterns, push notifications, FCM/APNs
    if content.contains("mobile")
        || content.contains("react native")
        || content.contains("ios")
        || content.contains("android")
        || content.contains("expo")
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
        return Agent::Tap;
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
        return Agent::Spark;
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
        return Agent::Bolt;
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
        return Agent::Grizz;
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
            return Agent::Blaze;
        }
        // Backend context: service, api, delivery, kafka, queue, stream, elysia, bun
        // Default Effect to Nova (backend) since it's primarily used there
        return Agent::Nova;
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
    {
        return Agent::Nova;
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
        return Agent::Cipher;
    }

    // Generic backend keywords - default to Rex for these
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

/// Parse an agent hint string into an Agent enum.
#[must_use]
pub fn parse_agent(hint: &str) -> Agent {
    match hint.to_lowercase().as_str() {
        "blaze" => Agent::Blaze,
        "rex" => Agent::Rex,
        "grizz" => Agent::Grizz,
        "tap" => Agent::Tap,
        "spark" => Agent::Spark,
        "nova" => Agent::Nova,
        "tess" => Agent::Tess,
        "cipher" => Agent::Cipher,
        "bolt" => Agent::Bolt,
        "atlas" => Agent::Atlas,
        _ => Agent::Rex, // Default to Rex
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
/// Returns the agent if found (e.g., "(Nova)", "- Rex", etc.)
fn check_explicit_agent(title: &str, description: &str) -> Option<Agent> {
    let content = format!("{} {}", title, description).to_lowercase();

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
/// This is the recommended function to use for intake workflows where
/// task dependencies are available.
#[must_use]
pub fn infer_agent_hint_with_deps(task: &Task, all_tasks: &[Task]) -> Agent {
    // 1. Check explicit agent name first (highest priority)
    if let Some(agent) = check_explicit_agent(&task.title, &task.description) {
        return agent;
    }

    // 2. Check dependency chain (PRIMARY signal for implementation tasks)
    if let Some(agent) = infer_from_dependencies(task, all_tasks) {
        return agent;
    }

    // 3. Fall back to keyword matching
    infer_agent_hint(&task.title, &task.description)
}

/// Infer agent hint with dependencies and return as a string.
#[must_use]
pub fn infer_agent_hint_with_deps_str(task: &Task, all_tasks: &[Task]) -> &'static str {
    infer_agent_hint_with_deps(task, all_tasks).as_str()
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
        // Dashboard and admin panel should go to Blaze (frontend)
        assert_eq!(
            infer_agent_hint("Admin Dashboard", "Show user statistics"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Build admin panel", "Management interface"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Web app interface", "User-facing application"),
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
    fn test_security_audit_detection() {
        // ONLY explicit security audit/review tasks go to Cipher
        assert_eq!(
            infer_agent_hint("Security Audit", "Review authentication implementation"),
            Agent::Cipher
        );
        assert_eq!(
            infer_agent_hint("Vulnerability Scan", "Check for security issues"),
            Agent::Cipher
        );
        assert_eq!(
            infer_agent_hint("Penetration test", "Security testing"),
            Agent::Cipher
        );
        assert_eq!(
            infer_agent_hint("Security review", "Audit the codebase"),
            Agent::Cipher
        );
    }

    #[test]
    fn test_auth_implementation_not_cipher() {
        // Auth/JWT/OAuth implementation tasks should NOT go to Cipher
        // They should go to the appropriate implementation agent
        assert_eq!(
            infer_agent_hint("Auth system", "OAuth provider implementation"),
            Agent::Rex // Default backend agent
        );
        assert_eq!(
            infer_agent_hint("JWT validation", "Implement JWT middleware"),
            Agent::Rex // Default backend agent
        );
        assert_eq!(
            infer_agent_hint("RBAC implementation", "Role-based access control"),
            Agent::Rex // Default backend agent
        );
        // If language is specified, route to that agent
        assert_eq!(
            infer_agent_hint("JWT Authentication", "Go/gRPC backend with auth"),
            Agent::Grizz
        );
        assert_eq!(
            infer_agent_hint(
                "OAuth2 Token Management",
                "Effect TypeScript implementation"
            ),
            Agent::Nova
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
        // Provision keywords should route to Bolt
        assert_eq!(
            infer_agent_hint("Provision PostgreSQL", "Setup database cluster"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Setup Redis cluster", "Cache infrastructure"),
            Agent::Bolt
        );
        // Operator keywords should route to Bolt
        assert_eq!(
            infer_agent_hint("Deploy CloudNative-PG", "PostgreSQL operator"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Setup Strimzi Kafka", "Event streaming"),
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
        assert_eq!(infer_agent_hint_with_deps(&push_task, &tasks), Agent::Tap);
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
        assert_eq!(infer_agent_hint_with_deps(&tray_task, &tasks), Agent::Spark);
    }

    #[test]
    fn test_dependency_based_routing_blaze() {
        // Task depending on Blaze init should inherit Blaze
        let mut blaze_init = Task::new("32", "Initialize Next.js project", "Create Next.js setup");
        blaze_init.agent_hint = Some("blaze".to_string());

        let mut page_task = Task::new("35", "Create Notifications page", "History with filters");
        page_task.dependencies = vec!["32".to_string()];

        let tasks = vec![blaze_init, page_task.clone()];
        assert_eq!(infer_agent_hint_with_deps(&page_task, &tasks), Agent::Blaze);
    }

    #[test]
    fn test_dependency_mixed_agents_falls_through() {
        // Task with mixed dependencies should fall through to keywords
        let mut rex_task = Task::new("1", "Rust service", "Backend");
        rex_task.agent_hint = Some("rex".to_string());

        let mut nova_task = Task::new("2", "Node service", "Backend");
        nova_task.agent_hint = Some("nova".to_string());

        let mut mixed_dep_task = Task::new("3", "Combined service", "Uses both");
        mixed_dep_task.dependencies = vec!["1".to_string(), "2".to_string()];

        let tasks = vec![rex_task, nova_task, mixed_dep_task.clone()];
        // Mixed deps → falls through to keywords → "service" matches backend → Rex
        assert_eq!(
            infer_agent_hint_with_deps(&mixed_dep_task, &tasks),
            Agent::Rex
        );
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
            Agent::Nova
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
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Prometheus rules", "Alert configuration"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Loki logging", "Configure log aggregation"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Observability stack", "Metrics and logging"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Alertmanager setup", "Alert routing"),
            Agent::Bolt
        );
    }

    #[test]
    fn test_bolt_gitops() {
        assert_eq!(
            infer_agent_hint("ArgoCD application", "GitOps deployment"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Flux configuration", "Continuous delivery"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Kustomize overlays", "Environment configs"),
            Agent::Bolt
        );
    }

    #[test]
    fn test_bolt_k8s_resources() {
        assert_eq!(
            infer_agent_hint("Create ConfigMap", "Application config"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Ingress rules", "External access"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("Network policy", "Pod isolation"),
            Agent::Bolt
        );
        assert_eq!(
            infer_agent_hint("HPA configuration", "Auto-scaling"),
            Agent::Bolt
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
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Effect validation", "Component form validation"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Effect Schema", "Page state management"),
            Agent::Blaze
        );
    }

    #[test]
    fn test_effect_backend_context() {
        // Effect + backend keywords → Nova
        assert_eq!(
            infer_agent_hint("Slack Service", "Effect retry for delivery"),
            Agent::Nova
        );
        assert_eq!(
            infer_agent_hint("Effect Stream", "Kafka consumer with Effect"),
            Agent::Nova
        );
        assert_eq!(
            infer_agent_hint("Delivery service", "Effect error handling"),
            Agent::Nova
        );
    }

    #[test]
    fn test_effect_default_to_nova() {
        // Effect without clear context → Nova (backend default)
        assert_eq!(
            infer_agent_hint("Effect implementation", "Type-safe error handling"),
            Agent::Nova
        );
    }

    // ===========================================================================
    // NEW TESTS: Frontend Page Keywords
    // ===========================================================================

    #[test]
    fn test_frontend_pages() {
        assert_eq!(
            infer_agent_hint("Notifications page", "History with filters"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Analytics page", "Charts and metrics"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Settings page", "User preferences"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Create management page", "CRUD interface"),
            Agent::Blaze
        );
    }

    #[test]
    fn test_frontend_web_console() {
        assert_eq!(
            infer_agent_hint("Web console", "Admin interface"),
            Agent::Blaze
        );
        assert_eq!(
            infer_agent_hint("Recharts visualization", "Data charts"),
            Agent::Blaze
        );
    }

    // ===========================================================================
    // NEW TESTS: Mobile Screen Keywords
    // ===========================================================================

    #[test]
    fn test_mobile_screens() {
        assert_eq!(
            infer_agent_hint("Home screen", "Notification feed"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("Detail screen", "Full notification view"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("Settings screen", "Mobile preferences"),
            Agent::Tap
        );
    }

    #[test]
    fn test_mobile_push_notifications() {
        assert_eq!(
            infer_agent_hint("Push notification", "FCM registration"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("FCM integration", "Firebase messaging"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("APNs setup", "Apple push notifications"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("App badge count", "Unread notifications"),
            Agent::Tap
        );
    }

    #[test]
    fn test_mobile_deep_links() {
        assert_eq!(
            infer_agent_hint("Deep link", "URL scheme handling"),
            Agent::Tap
        );
        assert_eq!(
            infer_agent_hint("Biometric auth", "Face ID / fingerprint"),
            Agent::Tap
        );
    }

    // ===========================================================================
    // NEW TESTS: Desktop Window/Tray Keywords
    // ===========================================================================

    #[test]
    fn test_desktop_tray() {
        assert_eq!(
            infer_agent_hint("System tray", "Notification badge"),
            Agent::Spark
        );
        assert_eq!(
            infer_agent_hint("Tray icon", "Status indicator"),
            Agent::Spark
        );
        assert_eq!(infer_agent_hint("Tray menu", "Quick actions"), Agent::Spark);
    }

    #[test]
    fn test_desktop_windows() {
        assert_eq!(
            infer_agent_hint("Main window", "Full notification feed"),
            Agent::Spark
        );
        assert_eq!(
            infer_agent_hint("Mini window", "Quick view popup"),
            Agent::Spark
        );
        assert_eq!(
            infer_agent_hint("Popup window", "Notification toast"),
            Agent::Spark
        );
    }

    #[test]
    fn test_desktop_autostart() {
        assert_eq!(
            infer_agent_hint("Auto-start", "Boot preferences"),
            Agent::Spark
        );
        assert_eq!(
            infer_agent_hint("Auto start on boot", "Startup configuration"),
            Agent::Spark
        );
        assert_eq!(
            infer_agent_hint("Cross-platform", "Windows/macOS/Linux"),
            Agent::Spark
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
