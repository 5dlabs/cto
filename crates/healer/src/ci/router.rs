//! CI failure routing logic.
//!
//! Routes CI failures to the appropriate specialist agent based on:
//! - Workflow name patterns
//! - Log content analysis
//! - Changed file types
//! - Security event indicators

use regex::Regex;
use std::sync::LazyLock;

use super::types::{Agent, ChangedFile, CiFailure, CiFailureType, RemediationContext, SecurityAlert};

/// Regex patterns for failure classification.
static RUST_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)clippy").unwrap(),
        Regex::new(r"(?i)cargo\s+(test|build|check)").unwrap(),
        Regex::new(r"(?i)rustc").unwrap(),
        Regex::new(r"error\[E\d+\]").unwrap(),                        // Rust compiler errors
        Regex::new(r"warning:\s*unused").unwrap(),                    // Rust warnings
        Regex::new(r"cannot\s+find\s+(crate|type|value)").unwrap(),
        Regex::new(r"Cargo\.toml").unwrap(),
    ]
});

static FRONTEND_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)npm\s+(install|run|test|build)").unwrap(),
        Regex::new(r"(?i)pnpm").unwrap(),
        Regex::new(r"(?i)yarn").unwrap(),
        Regex::new(r"(?i)typescript|tsc").unwrap(),
        Regex::new(r"(?i)eslint").unwrap(),
        Regex::new(r"TS\d{4}:").unwrap(),                             // TypeScript errors
        Regex::new(r"SyntaxError.*\.tsx?").unwrap(),
        Regex::new(r"Module not found").unwrap(),
    ]
});

static INFRA_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)docker\s+(build|push|pull)").unwrap(),
        Regex::new(r"(?i)helm\s+(template|install|upgrade)").unwrap(),
        Regex::new(r"(?i)kubectl").unwrap(),
        Regex::new(r"(?i)argocd").unwrap(),
        Regex::new(r"(?i)OutOfSync|sync\s+failed").unwrap(),
        Regex::new(r"Dockerfile").unwrap(),
        Regex::new(r"(?i)yaml\s*(syntax|error|invalid)").unwrap(),
        Regex::new(r"Chart\.yaml").unwrap(),
    ]
});

static SECURITY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)dependabot").unwrap(),
        Regex::new(r"(?i)vulnerability|CVE-\d{4}").unwrap(),
        Regex::new(r"(?i)security\s*advisory").unwrap(),
        Regex::new(r"(?i)code[\s_-]?scanning").unwrap(),
        Regex::new(r"(?i)secret[\s_-]?scanning").unwrap(),
    ]
});

static MERGE_CONFLICT_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)merge\s+conflict").unwrap(),
        Regex::new(r"(?i)CONFLICT\s*\(").unwrap(),
        Regex::new(r"(?i)cannot\s+merge").unwrap(),
        Regex::new(r"(?i)automatic\s+merge\s+failed").unwrap(),
    ]
});

/// CI failure router for intelligent agent selection.
pub struct CiRouter {
    /// Minimum confidence score to override routing
    confidence_threshold: f32,
}

impl Default for CiRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CiRouter {
    /// Create a new router with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.6,
        }
    }

    /// Route a CI failure to the appropriate agent.
    ///
    /// Uses multiple signals to determine the best agent:
    /// 1. Security events always go to Cipher
    /// 2. Workflow name patterns
    /// 3. Log content analysis
    /// 4. Changed file types
    /// 5. Default: Atlas (fallback)
    #[must_use]
    pub fn route(&self, ctx: &RemediationContext) -> Agent {
        // 1. Security events always go to Cipher
        if ctx.is_security_event() {
            return Agent::Cipher;
        }

        // 2. Use classified failure type if available
        if let Some(failure_type) = &ctx.failure_type {
            return Self::route_by_failure_type(failure_type);
        }

        // 3. Analyze context to determine routing
        let (agent, _confidence) = self.analyze_context(ctx);
        agent
    }

    /// Route based on classified failure type.
    #[must_use]
    fn route_by_failure_type(failure_type: &CiFailureType) -> Agent {
        match failure_type {
            // Rust -> Rex
            CiFailureType::RustClippy
            | CiFailureType::RustTest
            | CiFailureType::RustBuild
            | CiFailureType::RustDeps => Agent::Rex,

            // Frontend -> Blaze
            CiFailureType::FrontendDeps
            | CiFailureType::FrontendTypeScript
            | CiFailureType::FrontendLint
            | CiFailureType::FrontendTest
            | CiFailureType::FrontendBuild => Agent::Blaze,

            // Infrastructure -> Bolt
            CiFailureType::DockerBuild
            | CiFailureType::HelmTemplate
            | CiFailureType::K8sManifest
            | CiFailureType::ArgoCdSync
            | CiFailureType::YamlSyntax => Agent::Bolt,

            // Security -> Cipher
            CiFailureType::SecurityDependabot
            | CiFailureType::SecurityCodeScan
            | CiFailureType::SecuritySecret => Agent::Cipher,

            // Git/GitHub -> Atlas
            CiFailureType::GitMergeConflict
            | CiFailureType::GithubWorkflow
            | CiFailureType::GitPermission => Agent::Atlas,

            // General -> Atlas (fallback)
            CiFailureType::General => Agent::Atlas,
        }
    }

    /// Analyze context to determine agent and confidence.
    fn analyze_context(&self, ctx: &RemediationContext) -> (Agent, f32) {
        let mut scores: Vec<(Agent, f32)> = vec![
            (Agent::Rex, 0.0),
            (Agent::Blaze, 0.0),
            (Agent::Bolt, 0.0),
            (Agent::Cipher, 0.0),
            (Agent::Atlas, 0.1), // Small base score for fallback
        ];

        let mut signal_count = 0;

        // Score based on workflow logs
        if !ctx.workflow_logs.is_empty() {
            Self::score_from_logs(&ctx.workflow_logs, &mut scores);
            signal_count += 1;
        }

        // Score based on changed files
        if !ctx.changed_files.is_empty() {
            Self::score_from_files(&ctx.changed_files, &mut scores);
            signal_count += 1;
        }

        // Score based on workflow name
        if let Some(failure) = &ctx.failure {
            Self::score_from_workflow_name(&failure.workflow_name, &mut scores);
            if let Some(job_name) = &failure.job_name {
                Self::score_from_workflow_name(job_name, &mut scores);
            }
            signal_count += 1;
        }

        // Score based on ArgoCD status
        if let Some(argocd) = &ctx.argocd_status {
            if argocd.is_out_of_sync() || argocd.has_health_issues() {
                Self::add_score(&mut scores, Agent::Bolt, 0.4);
                signal_count += 1;
            }
        }

        // Find the highest scoring agent
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let (best_agent, best_score) = scores.first().map_or((Agent::Atlas, 0.0), |s| *s);

        // Adjust confidence threshold based on available signals
        // With fewer signals, we should be more lenient
        let effective_threshold = if signal_count <= 1 {
            self.confidence_threshold * 0.3 // Single signal: 0.18 threshold
        } else if signal_count == 2 {
            self.confidence_threshold * 0.5 // Two signals: 0.3 threshold
        } else {
            self.confidence_threshold // Full signals: 0.6 threshold
        };

        // If confidence is too low, fall back to Atlas
        if best_score < effective_threshold {
            return (Agent::Atlas, best_score);
        }

        (best_agent, best_score)
    }

    /// Score agents based on log content.
    #[allow(clippy::cast_precision_loss)]
    fn score_from_logs(logs: &str, scores: &mut [(Agent, f32)]) {
        // Check Rust patterns
        let rust_matches = RUST_PATTERNS.iter().filter(|p| p.is_match(logs)).count();
        if rust_matches > 0 {
            Self::add_score(scores, Agent::Rex, 0.2 * rust_matches as f32);
        }

        // Check frontend patterns
        let frontend_matches = FRONTEND_PATTERNS.iter().filter(|p| p.is_match(logs)).count();
        if frontend_matches > 0 {
            Self::add_score(scores, Agent::Blaze, 0.2 * frontend_matches as f32);
        }

        // Check infra patterns
        let infra_matches = INFRA_PATTERNS.iter().filter(|p| p.is_match(logs)).count();
        if infra_matches > 0 {
            Self::add_score(scores, Agent::Bolt, 0.2 * infra_matches as f32);
        }

        // Check security patterns
        let security_matches = SECURITY_PATTERNS.iter().filter(|p| p.is_match(logs)).count();
        if security_matches > 0 {
            Self::add_score(scores, Agent::Cipher, 0.3 * security_matches as f32);
        }

        // Check merge conflict patterns
        let merge_matches = MERGE_CONFLICT_PATTERNS.iter().filter(|p| p.is_match(logs)).count();
        if merge_matches > 0 {
            Self::add_score(scores, Agent::Atlas, 0.3 * merge_matches as f32);
        }
    }

    /// Score agents based on changed files.
    #[allow(clippy::cast_precision_loss)]
    fn score_from_files(files: &[ChangedFile], scores: &mut [(Agent, f32)]) {
        let total = files.len() as f32;
        if total == 0.0 {
            return;
        }

        let rust_count = files.iter().filter(|f| f.is_rust()).count() as f32;
        let frontend_count = files.iter().filter(|f| f.is_frontend()).count() as f32;
        let infra_count = files.iter().filter(|f| f.is_infra()).count() as f32;

        // Weight by proportion of files changed
        if rust_count > 0.0 {
            Self::add_score(scores, Agent::Rex, 0.3 * (rust_count / total));
        }
        if frontend_count > 0.0 {
            Self::add_score(scores, Agent::Blaze, 0.3 * (frontend_count / total));
        }
        if infra_count > 0.0 {
            Self::add_score(scores, Agent::Bolt, 0.3 * (infra_count / total));
        }
    }

    /// Score based on workflow name patterns.
    fn score_from_workflow_name(name: &str, scores: &mut [(Agent, f32)]) {
        let name_lower = name.to_lowercase();

        // Rust workflow names
        if name_lower.contains("rust")
            || name_lower.contains("clippy")
            || name_lower.contains("controller")
            || name_lower.contains("healer")
            || name_lower.contains("cargo")
        {
            Self::add_score(scores, Agent::Rex, 0.4);
        }

        // Frontend workflow names
        if name_lower.contains("frontend")
            || name_lower.contains("ui")
            || name_lower.contains("web")
            || name_lower.contains("npm")
            || name_lower.contains("pnpm")
        {
            Self::add_score(scores, Agent::Blaze, 0.4);
        }

        // Infrastructure workflow names
        if name_lower.contains("docker")
            || name_lower.contains("helm")
            || name_lower.contains("infra")
            || name_lower.contains("deploy")
            || name_lower.contains("gitops")
            || name_lower.contains("argocd")
        {
            Self::add_score(scores, Agent::Bolt, 0.4);
        }

        // Security workflow names
        if name_lower.contains("security")
            || name_lower.contains("audit")
            || name_lower.contains("scan")
        {
            Self::add_score(scores, Agent::Cipher, 0.4);
        }
    }

    /// Add score to an agent.
    fn add_score(scores: &mut [(Agent, f32)], agent: Agent, delta: f32) {
        for (a, score) in scores.iter_mut() {
            if *a == agent {
                *score += delta;
                break;
            }
        }
    }

    /// Classify a CI failure type from available context.
    #[must_use]
    pub fn classify_failure(&self, failure: &CiFailure, logs: &str) -> CiFailureType {
        // Check logs first (most specific)
        if let Some(failure_type) = Self::classify_from_logs(logs) {
            return failure_type;
        }

        // Check workflow/job name
        if let Some(failure_type) = Self::classify_from_workflow_name(failure) {
            return failure_type;
        }

        // Default to general
        CiFailureType::General
    }

    /// Classify from log content.
    fn classify_from_logs(logs: &str) -> Option<CiFailureType> {
        // Rust patterns
        if Regex::new(r"(?i)clippy").unwrap().is_match(logs) {
            return Some(CiFailureType::RustClippy);
        }
        if Regex::new(r"(?i)cargo\s+test").unwrap().is_match(logs)
            || logs.contains("test result: FAILED")
        {
            return Some(CiFailureType::RustTest);
        }
        if Regex::new(r"error\[E\d+\]").unwrap().is_match(logs) {
            return Some(CiFailureType::RustBuild);
        }

        // Frontend patterns
        if Regex::new(r"(?i)eslint").unwrap().is_match(logs) {
            return Some(CiFailureType::FrontendLint);
        }
        if Regex::new(r"TS\d{4}:").unwrap().is_match(logs) {
            return Some(CiFailureType::FrontendTypeScript);
        }
        if Regex::new(r"(?i)npm\s+ERR!").unwrap().is_match(logs)
            || Regex::new(r"(?i)pnpm\s+ERR!").unwrap().is_match(logs)
        {
            return Some(CiFailureType::FrontendDeps);
        }

        // Infrastructure patterns
        if Regex::new(r"(?i)docker\s+build.*failed").unwrap().is_match(logs) {
            return Some(CiFailureType::DockerBuild);
        }
        if Regex::new(r"(?i)helm.*template.*error").unwrap().is_match(logs) {
            return Some(CiFailureType::HelmTemplate);
        }
        if Regex::new(r"(?i)OutOfSync|sync\s+failed").unwrap().is_match(logs) {
            return Some(CiFailureType::ArgoCdSync);
        }
        if Regex::new(r"(?i)yaml.*error|error.*yaml").unwrap().is_match(logs) {
            return Some(CiFailureType::YamlSyntax);
        }

        // Security patterns
        if Regex::new(r"(?i)dependabot").unwrap().is_match(logs) {
            return Some(CiFailureType::SecurityDependabot);
        }
        if Regex::new(r"(?i)CVE-\d{4}").unwrap().is_match(logs) {
            return Some(CiFailureType::SecurityCodeScan);
        }

        // Git patterns
        if MERGE_CONFLICT_PATTERNS.iter().any(|p| p.is_match(logs)) {
            return Some(CiFailureType::GitMergeConflict);
        }

        None
    }

    /// Classify from workflow/job name.
    fn classify_from_workflow_name(failure: &CiFailure) -> Option<CiFailureType> {
        let name = failure
            .job_name
            .as_ref()
            .unwrap_or(&failure.workflow_name)
            .to_lowercase();

        if name.contains("clippy") {
            return Some(CiFailureType::RustClippy);
        }
        if name.contains("test") && (name.contains("rust") || name.contains("cargo")) {
            return Some(CiFailureType::RustTest);
        }
        if name.contains("build") && name.contains("rust") {
            return Some(CiFailureType::RustBuild);
        }

        if name.contains("eslint") || name.contains("lint") && name.contains("frontend") {
            return Some(CiFailureType::FrontendLint);
        }
        if name.contains("typescript") || name.contains("tsc") {
            return Some(CiFailureType::FrontendTypeScript);
        }
        if name.contains("npm") || name.contains("pnpm") {
            return Some(CiFailureType::FrontendDeps);
        }

        if name.contains("docker") {
            return Some(CiFailureType::DockerBuild);
        }
        if name.contains("helm") {
            return Some(CiFailureType::HelmTemplate);
        }
        if name.contains("argocd") || name.contains("gitops") {
            return Some(CiFailureType::ArgoCdSync);
        }

        if name.contains("security") || name.contains("audit") {
            return Some(CiFailureType::SecurityCodeScan);
        }

        None
    }

    /// Suggest a different agent when the current one is failing repeatedly.
    #[must_use]
    pub fn try_different_agent(&self, current: Agent, ctx: &RemediationContext) -> Agent {
        // If security event, stick with Cipher
        if ctx.is_security_event() {
            return Agent::Cipher;
        }

        // Try escalating to a more general agent
        match current {
            Agent::Rex => {
                // If Rex is failing, maybe it's actually infra (Cargo.lock in infra?)
                if ctx
                    .changed_files
                    .iter()
                    .any(|f| f.filename.contains("infra/"))
                {
                    return Agent::Bolt;
                }
                // Otherwise try Atlas as general fallback
                Agent::Atlas
            }
            Agent::Blaze => {
                // If Blaze is failing, check if there's infra involvement
                if ctx.changed_files.iter().any(|f| f.is_infra()) {
                    return Agent::Bolt;
                }
                Agent::Atlas
            }
            Agent::Bolt => {
                // Bolt failing might need Atlas for git issues
                Agent::Atlas
            }
            Agent::Cipher => {
                // Security issues might need code changes from Rex/Blaze
                if ctx.changed_files.iter().any(|f| f.is_rust()) {
                    return Agent::Rex;
                }
                if ctx.changed_files.iter().any(|f| f.is_frontend()) {
                    return Agent::Blaze;
                }
                Agent::Atlas
            }
            Agent::Atlas => {
                // Atlas is already the fallback, stay with it
                Agent::Atlas
            }
        }
    }
}

/// Route a security alert to the appropriate agent.
#[must_use]
pub fn route_security_alert(alert: &SecurityAlert) -> Agent {
    // All security alerts go to Cipher
    match alert.alert_type.as_str() {
        "dependabot_alert" | "code_scanning_alert" | "secret_scanning_alert" => Agent::Cipher,
        _ => Agent::Cipher, // Default to Cipher for any security-related event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> RemediationContext {
        RemediationContext::default()
    }

    #[test]
    fn test_route_rust_clippy() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.workflow_logs = "error: clippy found 3 errors".to_string();

        let (agent, _) = router.analyze_context(&ctx);
        assert_eq!(agent, Agent::Rex);
    }

    #[test]
    fn test_route_frontend_typescript() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.workflow_logs = "TS2304: Cannot find name 'foo'".to_string();

        let (agent, _) = router.analyze_context(&ctx);
        assert_eq!(agent, Agent::Blaze);
    }

    #[test]
    fn test_route_docker_build() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.workflow_logs = "docker build failed: COPY failed".to_string();

        let (agent, _) = router.analyze_context(&ctx);
        assert_eq!(agent, Agent::Bolt);
    }

    #[test]
    fn test_route_security_event() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.security_alert = Some(SecurityAlert {
            alert_type: "dependabot_alert".to_string(),
            severity: "high".to_string(),
            package_name: Some("lodash".to_string()),
            cve_id: Some("CVE-2021-23337".to_string()),
            description: "Prototype pollution".to_string(),
            repository: "5dlabs/cto".to_string(),
            branch: None,
            html_url: "https://github.com/advisories".to_string(),
            detected_at: chrono::Utc::now(),
        });

        let agent = router.route(&ctx);
        assert_eq!(agent, Agent::Cipher);
    }

    #[test]
    fn test_route_merge_conflict() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.workflow_logs = "CONFLICT (content): Merge conflict in src/main.rs".to_string();

        let (agent, _) = router.analyze_context(&ctx);
        assert_eq!(agent, Agent::Atlas);
    }

    #[test]
    fn test_route_by_changed_files() {
        let router = CiRouter::new();
        let mut ctx = create_test_context();
        ctx.changed_files = vec![
            ChangedFile {
                filename: "src/main.rs".to_string(),
                status: "modified".to_string(),
                additions: 10,
                deletions: 5,
            },
            ChangedFile {
                filename: "Cargo.toml".to_string(),
                status: "modified".to_string(),
                additions: 1,
                deletions: 1,
            },
        ];

        let (agent, _) = router.analyze_context(&ctx);
        assert_eq!(agent, Agent::Rex);
    }

    #[test]
    fn test_classify_failure_from_logs() {
        let router = CiRouter::new();
        let failure = CiFailure {
            workflow_run_id: 123,
            workflow_name: "CI".to_string(),
            job_name: None,
            conclusion: "failure".to_string(),
            branch: "main".to_string(),
            head_sha: "abc".to_string(),
            commit_message: "test".to_string(),
            html_url: "https://github.com".to_string(),
            repository: "test/repo".to_string(),
            sender: "user".to_string(),
            detected_at: chrono::Utc::now(),
            raw_event: None,
        };

        let ft = router.classify_failure(&failure, "error[E0382]: borrow of moved value");
        assert_eq!(ft, CiFailureType::RustBuild);

        let ft = router.classify_failure(&failure, "clippy::unwrap_used");
        assert_eq!(ft, CiFailureType::RustClippy);

        let ft = router.classify_failure(&failure, "TS2304: Cannot find name");
        assert_eq!(ft, CiFailureType::FrontendTypeScript);
    }
}

