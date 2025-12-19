//! Expected behavior patterns for agents.
//!
//! This module defines expected success and failure patterns for each agent type,
//! enabling real-time anomaly detection during play workflows.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Agent types in the platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Rex,
    Blaze,
    Cleo,
    Tess,
    Cipher,
    Atlas,
    Factory,
    Morgan,
    Unknown,
}

impl std::str::FromStr for AgentType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "rex" => Self::Rex,
            "blaze" => Self::Blaze,
            "cleo" => Self::Cleo,
            "tess" => Self::Tess,
            "cipher" => Self::Cipher,
            "atlas" => Self::Atlas,
            "factory" => Self::Factory,
            "morgan" => Self::Morgan,
            _ => Self::Unknown,
        })
    }
}

impl AgentType {
    /// Get the display name for this agent
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rex => "Rex (Implementation)",
            Self::Blaze => "Blaze (Frontend)",
            Self::Cleo => "Cleo (Code Review)",
            Self::Tess => "Tess (Testing)",
            Self::Cipher => "Cipher (Security)",
            Self::Atlas => "Atlas (Integration)",
            Self::Factory => "Factory (General)",
            Self::Morgan => "Morgan (PM)",
            Self::Unknown => "Unknown Agent",
        }
    }
}

/// A pattern that indicates expected behavior
#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    /// Human-readable description
    pub description: String,
    /// Compiled regex pattern
    pub pattern: Regex,
    /// Whether this is a success (true) or failure (false) indicator
    pub is_success: bool,
    /// Severity if this is a failure (critical, high, medium, low)
    pub severity: Option<String>,
}

/// Expected behaviors for a specific agent
#[derive(Debug, Clone)]
pub struct AgentBehaviors {
    /// The agent type
    pub agent: AgentType,
    /// Success patterns to look for
    pub success_patterns: Vec<BehaviorPattern>,
    /// Failure patterns to alert on
    pub failure_patterns: Vec<BehaviorPattern>,
    /// Anomaly patterns (unexpected but not necessarily failures)
    pub anomaly_patterns: Vec<BehaviorPattern>,
}

/// Result of analyzing a log line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysis {
    /// The log line that was analyzed
    pub line: String,
    /// Agent this log is from
    pub agent: AgentType,
    /// Type of detection
    pub detection_type: DetectionType,
    /// Matched pattern description
    pub matched_pattern: String,
    /// Severity level
    pub severity: String,
    /// Timestamp of the log
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Type of detection from log analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DetectionType {
    /// Expected success behavior
    Success,
    /// Known failure pattern
    Failure,
    /// Anomalous behavior (unexpected but not necessarily failure)
    Anomaly,
    /// No significant detection
    Normal,
}

/// Behavior analyzer for all agents
pub struct BehaviorAnalyzer {
    behaviors: HashMap<AgentType, AgentBehaviors>,
    /// Global failure patterns that apply to all agents
    global_failure_patterns: Vec<BehaviorPattern>,
}

impl Default for BehaviorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorAnalyzer {
    /// Create a new behavior analyzer with built-in patterns
    #[must_use]
    pub fn new() -> Self {
        let mut analyzer = Self {
            behaviors: HashMap::new(),
            global_failure_patterns: Vec::new(),
        };
        analyzer.load_builtin_patterns();
        analyzer
    }

    /// Load built-in behavior patterns for all agents
    #[allow(clippy::too_many_lines)]
    fn load_builtin_patterns(&mut self) {
        // Global failure patterns (apply to all agents)
        self.global_failure_patterns = vec![
            pattern("panic", r"(?i)panic(ked)?( at)?", false, "critical"),
            pattern("fatal error", r"(?i)fatal:", false, "critical"),
            pattern("segfault", r"(?i)segmentation fault", false, "critical"),
            pattern(
                "OOM killed",
                r"(?i)oom|out of memory|killed",
                false,
                "critical",
            ),
            pattern("permission denied", r"(?i)permission denied", false, "high"),
            pattern(
                "authentication failed",
                r"(?i)auth(entication)? failed",
                false,
                "high",
            ),
            pattern(
                "connection refused",
                r"(?i)connection refused",
                false,
                "medium",
            ),
            pattern("timeout", r"(?i)timed? ?out|timeout", false, "medium"),
        ];

        // Rex (Implementation Agent)
        self.behaviors.insert(
            AgentType::Rex,
            AgentBehaviors {
                agent: AgentType::Rex,
                success_patterns: vec![
                    pattern("git push success", r"(?i)git push|pushed to", true, "info"),
                    pattern("commit created", r"(?i)git commit|committed", true, "info"),
                    pattern(
                        "PR created",
                        r"(?i)pr created|pull request created",
                        true,
                        "info",
                    ),
                    pattern(
                        "PR updated",
                        r"(?i)pr updated|pull request updated",
                        true,
                        "info",
                    ),
                    pattern(
                        "implementation complete",
                        r"(?i)implementation complete|task complete",
                        true,
                        "info",
                    ),
                    pattern("changes committed", r"(?i)changes committed", true, "info"),
                ],
                failure_patterns: vec![
                    pattern(
                        "git conflict",
                        r"(?i)conflict|merge conflict",
                        false,
                        "high",
                    ),
                    pattern(
                        "push failed",
                        r"(?i)failed to push|push failed",
                        false,
                        "high",
                    ),
                    pattern("git error", r"^error:|^fatal:", false, "high"),
                    pattern("clippy error", r"error\[E\d+\]", false, "medium"),
                    pattern(
                        "cargo build failed",
                        r"(?i)error: could not compile",
                        false,
                        "high",
                    ),
                ],
                anomaly_patterns: vec![
                    pattern(
                        "unusual retry",
                        r"(?i)retry|retrying|attempt \d+",
                        false,
                        "low",
                    ),
                    pattern("force push", r"(?i)force push|--force", false, "medium"),
                ],
            },
        );

        // Blaze (Frontend Agent)
        self.behaviors.insert(
            AgentType::Blaze,
            AgentBehaviors {
                agent: AgentType::Blaze,
                success_patterns: vec![
                    pattern("git push success", r"(?i)git push|pushed to", true, "info"),
                    pattern(
                        "npm build success",
                        r"(?i)npm run build.*success|build succeeded",
                        true,
                        "info",
                    ),
                    pattern(
                        "PR created",
                        r"(?i)pr created|pull request created",
                        true,
                        "info",
                    ),
                    pattern("changes committed", r"(?i)changes committed", true, "info"),
                ],
                failure_patterns: vec![
                    pattern("npm error", r"(?i)npm err!|npm error", false, "high"),
                    pattern("eslint error", r"(?i)eslint.*error", false, "medium"),
                    pattern(
                        "typescript error",
                        r"(?i)ts\d+:|typescript.*error",
                        false,
                        "high",
                    ),
                    pattern(
                        "build failed",
                        r"(?i)build failed|compilation failed",
                        false,
                        "high",
                    ),
                    pattern(
                        "git conflict",
                        r"(?i)conflict|merge conflict",
                        false,
                        "high",
                    ),
                ],
                anomaly_patterns: vec![
                    pattern("deprecation warning", r"(?i)deprecat(ed|ion)", false, "low"),
                    pattern(
                        "peer dependency",
                        r"(?i)peer dep|peerDependenc",
                        false,
                        "low",
                    ),
                ],
            },
        );

        // Cleo (Code Review Agent)
        self.behaviors.insert(
            AgentType::Cleo,
            AgentBehaviors {
                agent: AgentType::Cleo,
                success_patterns: vec![
                    pattern(
                        "review submitted",
                        r"(?i)review submitted|posted review",
                        true,
                        "info",
                    ),
                    pattern("approved", r"(?i)\bapproved\b", true, "info"),
                    pattern(
                        "changes requested",
                        r"(?i)changes requested|request(ed)? changes",
                        true,
                        "info",
                    ),
                    pattern(
                        "comment posted",
                        r"(?i)comment posted|posted comment",
                        true,
                        "info",
                    ),
                    pattern(
                        "review complete",
                        r"(?i)review complete|code review (done|complete)",
                        true,
                        "info",
                    ),
                ],
                failure_patterns: vec![
                    pattern(
                        "review not submitted",
                        r"(?i)review not submitted|failed to (post|submit) review",
                        false,
                        "high",
                    ),
                    pattern(
                        "API rate limit",
                        r"(?i)rate limit|too many requests",
                        false,
                        "medium",
                    ),
                    pattern(
                        "could not fetch PR",
                        r"(?i)could not fetch|failed to fetch.*pr",
                        false,
                        "high",
                    ),
                ],
                anomaly_patterns: vec![pattern(
                    "long review",
                    r"(?i)still reviewing|review taking",
                    false,
                    "low",
                )],
            },
        );

        // Tess (Testing Agent)
        self.behaviors.insert(
            AgentType::Tess,
            AgentBehaviors {
                agent: AgentType::Tess,
                success_patterns: vec![
                    pattern(
                        "tests passed",
                        r"(?i)test result: ok|\d+ passed.*0 failed|all tests passed",
                        true,
                        "info",
                    ),
                    pattern(
                        "cargo test success",
                        r"(?i)cargo test.*ok|running.*tests.*ok",
                        true,
                        "info",
                    ),
                    pattern(
                        "npm test success",
                        r"(?i)npm test.*pass|jest.*pass",
                        true,
                        "info",
                    ),
                    pattern(
                        "tests complete",
                        r"(?i)tests? complete|testing complete",
                        true,
                        "info",
                    ),
                ],
                failure_patterns: vec![
                    pattern(
                        "test failed",
                        r"(?i)test result: failed|\d+ failed|tests? failed",
                        false,
                        "high",
                    ),
                    pattern(
                        "assertion failed",
                        r"(?i)assertion failed|assert.*failed",
                        false,
                        "high",
                    ),
                    pattern(
                        "panic in test",
                        r"(?i)panicked at|thread.*panicked",
                        false,
                        "critical",
                    ),
                    pattern(
                        "test compilation error",
                        r"error\[E\d+\].*test",
                        false,
                        "high",
                    ),
                    pattern(
                        "approved despite failures",
                        r"(?i)approved.*fail|fail.*approved",
                        false,
                        "critical",
                    ),
                ],
                anomaly_patterns: vec![
                    pattern(
                        "flaky test",
                        r"(?i)flaky|intermittent|retry",
                        false,
                        "medium",
                    ),
                    pattern("skipped tests", r"(?i)skipped|ignored.*test", false, "low"),
                    pattern("slow tests", r"(?i)slow test|test.*\d{3,}s", false, "low"),
                ],
            },
        );

        // Cipher (Security Agent)
        self.behaviors.insert(
            AgentType::Cipher,
            AgentBehaviors {
                agent: AgentType::Cipher,
                success_patterns: vec![
                    pattern(
                        "security check passed",
                        r"(?i)security (check|scan) passed|no vulnerabilities",
                        true,
                        "info",
                    ),
                    pattern(
                        "secrets verified",
                        r"(?i)secrets? (verified|secure|ok)",
                        true,
                        "info",
                    ),
                    pattern(
                        "audit passed",
                        r"(?i)audit passed|cargo audit.*ok",
                        true,
                        "info",
                    ),
                ],
                failure_patterns: vec![
                    pattern(
                        "vulnerability found",
                        r"(?i)vulnerabilit(y|ies) found|security issue",
                        false,
                        "critical",
                    ),
                    pattern(
                        "secret exposed",
                        r"(?i)secret exposed|credential leak|hardcoded (secret|password|key)",
                        false,
                        "critical",
                    ),
                    pattern(
                        "audit failed",
                        r"(?i)audit failed|security audit.*fail",
                        false,
                        "high",
                    ),
                    pattern(
                        "insecure dependency",
                        r"(?i)insecure dep|vulnerable package",
                        false,
                        "high",
                    ),
                ],
                anomaly_patterns: vec![pattern(
                    "advisory notice",
                    r"(?i)advisory|cve-\d+",
                    false,
                    "medium",
                )],
            },
        );

        // Atlas (Integration Agent)
        self.behaviors.insert(
            AgentType::Atlas,
            AgentBehaviors {
                agent: AgentType::Atlas,
                success_patterns: vec![
                    pattern(
                        "rebase successful",
                        r"(?i)rebase successful|rebased",
                        true,
                        "info",
                    ),
                    pattern(
                        "merge successful",
                        r"(?i)merge successful|merged",
                        true,
                        "info",
                    ),
                    pattern(
                        "conflicts resolved",
                        r"(?i)conflicts? resolved",
                        true,
                        "info",
                    ),
                    pattern(
                        "branch updated",
                        r"(?i)branch updated|updated branch",
                        true,
                        "info",
                    ),
                    pattern(
                        "integration complete",
                        r"(?i)integration complete",
                        true,
                        "info",
                    ),
                    pattern(
                        "PR ready to merge",
                        r"(?i)ready to merge|pr (is )?mergeable",
                        true,
                        "info",
                    ),
                ],
                failure_patterns: vec![
                    pattern(
                        "merge conflict",
                        r"(?i)conflict|merge conflict|cannot merge",
                        false,
                        "high",
                    ),
                    pattern(
                        "rebase failed",
                        r"(?i)rebase failed|could not rebase",
                        false,
                        "high",
                    ),
                    pattern(
                        "diverged branches",
                        r"(?i)diverged|branches have diverged",
                        false,
                        "medium",
                    ),
                    pattern("git error", r"^error:|^fatal:", false, "high"),
                ],
                anomaly_patterns: vec![
                    pattern(
                        "complex merge",
                        r"(?i)complex merge|many conflicts",
                        false,
                        "medium",
                    ),
                    pattern("force required", r"(?i)force|--force", false, "medium"),
                ],
            },
        );

        // Factory (General Agent)
        self.behaviors.insert(
            AgentType::Factory,
            AgentBehaviors {
                agent: AgentType::Factory,
                success_patterns: vec![
                    pattern(
                        "task complete",
                        r"(?i)task complete|completed successfully",
                        true,
                        "info",
                    ),
                    pattern("fix applied", r"(?i)fix applied|fixed", true, "info"),
                ],
                failure_patterns: vec![pattern(
                    "task failed",
                    r"(?i)task failed|failed to complete",
                    false,
                    "high",
                )],
                anomaly_patterns: vec![],
            },
        );

        // Morgan (PM Agent)
        self.behaviors.insert(
            AgentType::Morgan,
            AgentBehaviors {
                agent: AgentType::Morgan,
                success_patterns: vec![
                    pattern(
                        "issue created",
                        r"(?i)issue created|created issue",
                        true,
                        "info",
                    ),
                    pattern(
                        "comment posted",
                        r"(?i)comment (posted|added)",
                        true,
                        "info",
                    ),
                    pattern("project updated", r"(?i)project updated", true, "info"),
                ],
                failure_patterns: vec![
                    pattern("API error", r"(?i)api error|github api", false, "high"),
                    pattern("rate limited", r"(?i)rate limit", false, "medium"),
                ],
                anomaly_patterns: vec![],
            },
        );
    }

    /// Analyze a log line for a specific agent
    #[must_use]
    pub fn analyze_line(&self, line: &str, agent: AgentType) -> LogAnalysis {
        // First check global failure patterns
        for pattern in &self.global_failure_patterns {
            if pattern.pattern.is_match(line) {
                return LogAnalysis {
                    line: line.to_string(),
                    agent,
                    detection_type: DetectionType::Failure,
                    matched_pattern: pattern.description.clone(),
                    severity: pattern
                        .severity
                        .clone()
                        .unwrap_or_else(|| "high".to_string()),
                    timestamp: None,
                };
            }
        }

        // Then check agent-specific patterns
        if let Some(behaviors) = self.behaviors.get(&agent) {
            // Check failure patterns first (higher priority)
            for pattern in &behaviors.failure_patterns {
                if pattern.pattern.is_match(line) {
                    return LogAnalysis {
                        line: line.to_string(),
                        agent,
                        detection_type: DetectionType::Failure,
                        matched_pattern: pattern.description.clone(),
                        severity: pattern
                            .severity
                            .clone()
                            .unwrap_or_else(|| "high".to_string()),
                        timestamp: None,
                    };
                }
            }

            // Check anomaly patterns
            for pattern in &behaviors.anomaly_patterns {
                if pattern.pattern.is_match(line) {
                    return LogAnalysis {
                        line: line.to_string(),
                        agent,
                        detection_type: DetectionType::Anomaly,
                        matched_pattern: pattern.description.clone(),
                        severity: pattern
                            .severity
                            .clone()
                            .unwrap_or_else(|| "medium".to_string()),
                        timestamp: None,
                    };
                }
            }

            // Check success patterns
            for pattern in &behaviors.success_patterns {
                if pattern.pattern.is_match(line) {
                    return LogAnalysis {
                        line: line.to_string(),
                        agent,
                        detection_type: DetectionType::Success,
                        matched_pattern: pattern.description.clone(),
                        severity: "info".to_string(),
                        timestamp: None,
                    };
                }
            }
        }

        // No significant detection
        LogAnalysis {
            line: line.to_string(),
            agent,
            detection_type: DetectionType::Normal,
            matched_pattern: String::new(),
            severity: "none".to_string(),
            timestamp: None,
        }
    }

    /// Analyze multiple log lines and return only significant detections
    #[must_use]
    pub fn analyze_logs(&self, lines: &[String], agent: AgentType) -> Vec<LogAnalysis> {
        lines
            .iter()
            .map(|line| self.analyze_line(line, agent))
            .filter(|analysis| analysis.detection_type != DetectionType::Normal)
            .collect()
    }

    /// Get a summary of behaviors for an agent
    #[must_use]
    pub fn get_agent_behaviors(&self, agent: AgentType) -> Option<&AgentBehaviors> {
        self.behaviors.get(&agent)
    }

    /// Detect agent type from pod name or labels
    #[must_use]
    pub fn detect_agent_from_pod(pod_name: &str, labels: &HashMap<String, String>) -> AgentType {
        // Check labels first (most reliable)
        if let Some(agent_label) = labels
            .get("agents.platform/agent")
            .or_else(|| labels.get("healer/agent"))
            .or_else(|| labels.get("agent"))
        {
            return agent_label.parse().unwrap_or(AgentType::Unknown);
        }

        // Fall back to pod name patterns
        let name_lower = pod_name.to_lowercase();
        for agent in [
            AgentType::Rex,
            AgentType::Blaze,
            AgentType::Cleo,
            AgentType::Tess,
            AgentType::Cipher,
            AgentType::Atlas,
            AgentType::Factory,
            AgentType::Morgan,
        ] {
            let agent_name = format!("{agent:?}").to_lowercase();
            if name_lower.contains(&agent_name) {
                return agent;
            }
        }

        debug!(pod_name = %pod_name, "Could not detect agent type from pod");
        AgentType::Unknown
    }
}

/// Helper to create a behavior pattern
fn pattern(desc: &str, regex: &str, is_success: bool, severity: &str) -> BehaviorPattern {
    BehaviorPattern {
        description: desc.to_string(),
        pattern: Regex::new(regex).unwrap_or_else(|e| {
            warn!(pattern = %regex, error = %e, "Failed to compile regex pattern");
            Regex::new("^$").unwrap() // Match nothing as fallback
        }),
        is_success,
        severity: Some(severity.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!("rex".parse::<AgentType>().unwrap(), AgentType::Rex);
        assert_eq!("REX".parse::<AgentType>().unwrap(), AgentType::Rex);
        assert_eq!("Tess".parse::<AgentType>().unwrap(), AgentType::Tess);
        assert_eq!(
            "unknown_agent".parse::<AgentType>().unwrap(),
            AgentType::Unknown
        );
    }

    #[test]
    fn test_analyze_success_pattern() {
        let analyzer = BehaviorAnalyzer::new();

        let analysis = analyzer.analyze_line("All tests passed!", AgentType::Tess);
        assert_eq!(analysis.detection_type, DetectionType::Success);
        assert!(analysis.matched_pattern.contains("pass"));
    }

    #[test]
    fn test_analyze_failure_pattern() {
        let analyzer = BehaviorAnalyzer::new();

        let analysis = analyzer.analyze_line("error[E0382]: borrow of moved value", AgentType::Rex);
        assert_eq!(analysis.detection_type, DetectionType::Failure);
    }

    #[test]
    fn test_analyze_global_panic() {
        let analyzer = BehaviorAnalyzer::new();

        // Panic should be detected for any agent
        let analysis = analyzer.analyze_line(
            "thread 'main' panicked at 'index out of bounds'",
            AgentType::Cleo,
        );
        assert_eq!(analysis.detection_type, DetectionType::Failure);
        assert_eq!(analysis.severity, "critical");
    }

    #[test]
    fn test_analyze_normal_line() {
        let analyzer = BehaviorAnalyzer::new();

        let analysis = analyzer.analyze_line("Processing request...", AgentType::Rex);
        assert_eq!(analysis.detection_type, DetectionType::Normal);
    }

    #[test]
    fn test_detect_agent_from_labels() {
        let mut labels = HashMap::new();
        labels.insert("agents.platform/agent".to_string(), "tess".to_string());

        let agent = BehaviorAnalyzer::detect_agent_from_pod("some-random-pod-abc123", &labels);
        assert_eq!(agent, AgentType::Tess);
    }

    #[test]
    fn test_detect_agent_from_pod_name() {
        let labels = HashMap::new();

        let agent = BehaviorAnalyzer::detect_agent_from_pod("play-task-1-rex-step-abc123", &labels);
        assert_eq!(agent, AgentType::Rex);
    }
}
