//! Log scanner for periodic error detection and remediation triggering.
//!
//! Scans Loki logs for errors and warnings across platform namespaces,
//! analyzing patterns to determine if automated remediation should be triggered.
//!
//! The scanner uses log-level-aware patterns to avoid false positives from:
//! - INFO-level messages containing the word "error" as a command name
//! - JSON field names like "errorMessages" regardless of value
//! - Empty error arrays that indicate success, not failure

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{debug, info};

use crate::loki::{LogEntry, LokiClient};

/// Patterns that indicate an actual error log level (not just keyword matches)
const ERROR_LEVEL_PATTERNS: &[&str] = &[
    // Structured log level indicators (key-value style)
    r#"level[=:]\s*["']?error"#,
    r#"level[=:]\s*["']?fatal"#,
    r#"level[=:]\s*["']?ERROR"#,
    r#"level[=:]\s*["']?FATAL"#,
    // JSON-style log levels (with quoted keys)
    r#""level"\s*:\s*"error""#,
    r#""level"\s*:\s*"fatal""#,
    r#""level"\s*:\s*"ERROR""#,
    r#""level"\s*:\s*"FATAL""#,
    // Bracket-style log levels
    r"\[ERROR\]",
    r"\[ERR\]",
    r"\[FATAL\]",
    r"\[PANIC\]",
    // Prefix-style log levels
    r"^ERROR:",
    r"^FATAL:",
    r"^PANIC:",
    r"^E\s",
    // Rust-style errors
    r"error\[E\d{4}\]",
    r"thread '.*' panicked at",
    // Go-style errors
    r"level=error",
    r"level=fatal",
    // Python-style errors
    r"ERROR\s+-\s+",
    r"CRITICAL\s+-\s+",
    // Stack traces - more specific patterns to avoid false positives
    r"(?i)stack\s+trace",
    r"(?i)traceback\s*\(",
    r"(?i)Traceback\s+\(most recent",
    r"(?i)exception\s*:",
];

/// Patterns that indicate false positives (INFO-level messages with "error" keyword)
const FALSE_POSITIVE_PATTERNS: &[&str] = &[
    // Command registration (the word "error" is a command name)
    r"(?i)Register.*command.*extension.*for.*command.*error",
    r"(?i)action.*command.*extension.*for.*command.*error",
    // Empty error arrays in JSON (indicates NO errors)
    r#""errorMessages"\s*:\s*\[\s*\]"#,
    r#""errors"\s*:\s*\[\s*\]"#,
    r"errorMessages.*\[\]",
    // INFO-level messages that contain "error" as a keyword
    r"(?i)\bINFO\b.*\berror\b",
    r"(?i)\[INFO\].*\berror\b",
    // Debug/trace messages about error handling
    r"(?i)\bDEBUG\b.*error.*handler",
    r"(?i)\bTRACE\b.*error.*handling",
    // Error count reports showing zero errors
    r"(?i)errors:\s*0\b",
    r"(?i)error.*count.*:\s*0\b",
    // Test/mock error messages
    r"(?i)test.*error",
    r"(?i)mock.*error",
    r"(?i)fake.*error",
    // Go-style structured log levels indicating non-error (info, debug, trace, warn)
    // These are explicitly NOT errors even if the message contains error-like keywords
    r#"level[=:]\s*["']?info"#,
    r#"level[=:]\s*["']?INFO"#,
    r#"level[=:]\s*["']?debug"#,
    r#"level[=:]\s*["']?DEBUG"#,
    r#"level[=:]\s*["']?trace"#,
    r#"level[=:]\s*["']?TRACE"#,
    // JSON-style log levels indicating non-error
    r#""level"\s*:\s*"info""#,
    r#""level"\s*:\s*"INFO""#,
    r#""level"\s*:\s*"debug""#,
    r#""level"\s*:\s*"DEBUG""#,
    // ArgoCD/Helm manifest cache hits (informational messages)
    r"(?i)manifest\s+cache\s+hit",
];

/// Get compiled regex patterns for error level detection (cached)
fn get_error_level_regexes() -> &'static Vec<Regex> {
    static ERROR_LEVEL_REGEX: OnceLock<Vec<Regex>> = OnceLock::new();
    ERROR_LEVEL_REGEX.get_or_init(|| {
        ERROR_LEVEL_PATTERNS
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Failed to compile error level regex '{p}': {e}");
                    None
                }
            })
            .collect()
    })
}

/// Get compiled regex patterns for false positive detection (cached)
fn get_false_positive_regexes() -> &'static Vec<Regex> {
    static FALSE_POSITIVE_REGEX: OnceLock<Vec<Regex>> = OnceLock::new();
    FALSE_POSITIVE_REGEX.get_or_init(|| {
        FALSE_POSITIVE_PATTERNS
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Failed to compile false positive regex '{p}': {e}");
                    None
                }
            })
            .collect()
    })
}

/// Default namespaces to scan for platform health
pub const DEFAULT_NAMESPACES: &[&str] = &["cto", "automation", "argocd", "infra", "observability"];

/// Configuration for the log scanner
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Namespaces to scan
    pub namespaces: Vec<String>,
    /// Minimum error count to report a service
    pub error_threshold: u32,
    /// Minimum warning count to report a service
    pub warn_threshold: u32,
    /// Whether to include resolved (info-level) entries
    pub include_info: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            namespaces: DEFAULT_NAMESPACES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            // Lower thresholds to catch platform issues early
            error_threshold: 3,
            warn_threshold: 5,
            include_info: false,
        }
    }
}

/// A service with detected issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIssue {
    /// Service/pod name pattern
    pub service: String,
    /// Namespace
    pub namespace: String,
    /// Number of error-level logs
    pub error_count: u32,
    /// Number of warning-level logs
    pub warn_count: u32,
    /// Sample error messages (up to 5)
    pub sample_errors: Vec<String>,
    /// First error timestamp
    pub first_seen: Option<DateTime<Utc>>,
    /// Last error timestamp
    pub last_seen: Option<DateTime<Utc>>,
    /// Affected pods
    pub affected_pods: Vec<String>,
}

/// Report from a log scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    /// When the scan was performed
    pub scan_time: DateTime<Utc>,
    /// Time window scanned
    pub window_minutes: u64,
    /// Namespaces that were scanned
    pub namespaces_scanned: Vec<String>,
    /// Services with issues above threshold
    pub services_with_issues: Vec<ServiceIssue>,
    /// Total errors found
    pub total_errors: u32,
    /// Total warnings found
    pub total_warnings: u32,
    /// Whether remediation is recommended
    pub remediation_recommended: bool,
    /// Reason for recommendation
    pub recommendation_reason: Option<String>,
}

/// A candidate for automated remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationCandidate {
    /// Service that needs remediation
    pub service: String,
    /// Namespace
    pub namespace: String,
    /// Severity (critical, high, medium, low)
    pub severity: String,
    /// Reason for remediation
    pub reason: String,
    /// Suggested agent to handle
    pub suggested_agent: String,
    /// Sample log context
    pub log_context: String,
}

/// Check if a log line contains an actual error-level indicator.
///
/// This function checks for structured log level markers rather than
/// naive keyword matching, reducing false positives significantly.
#[must_use]
pub fn is_actual_error(line: &str) -> bool {
    // Fast path: if the line doesn't contain any error-like keywords, skip regex
    let line_lower = line.to_lowercase();
    if !line_lower.contains("error")
        && !line_lower.contains("fatal")
        && !line_lower.contains("panic")
        && !line_lower.contains("exception")
        && !line_lower.contains("traceback")
    {
        return false;
    }

    // Check against actual error level patterns
    get_error_level_regexes().iter().any(|re| re.is_match(line))
}

/// Check if a log line matches known false positive patterns.
///
/// These are INFO-level or other non-error messages that contain
/// the word "error" as a command name, field name, or in other
/// non-error contexts.
#[must_use]
pub fn is_false_positive(line: &str) -> bool {
    get_false_positive_regexes()
        .iter()
        .any(|re| re.is_match(line))
}

/// Filter log entries to remove false positives and keep only actual errors.
///
/// This applies log-level-aware filtering to distinguish between:
/// - Actual ERROR/FATAL level logs that need attention
/// - INFO-level logs that happen to contain the word "error"
pub fn filter_actual_errors(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    let original_count = entries.len();

    let filtered: Vec<LogEntry> = entries
        .into_iter()
        .filter(|entry| {
            // Check if it's a known false positive first (fast reject)
            if is_false_positive(&entry.line) {
                debug!(
                    "Filtering out false positive: {}",
                    truncate_line(&entry.line, 100)
                );
                return false;
            }

            // Check if it contains actual error level indicators
            // If it doesn't match our error patterns but was caught by keyword search,
            // it's likely a false positive
            let has_error_keyword = entry.line.to_lowercase().contains("error")
                || entry.line.to_lowercase().contains("fatal")
                || entry.line.to_lowercase().contains("panic");

            if has_error_keyword && !is_actual_error(&entry.line) {
                debug!(
                    "Filtering out keyword-only match: {}",
                    truncate_line(&entry.line, 100)
                );
                return false;
            }

            true
        })
        .collect();

    let filtered_count = filtered.len();
    if original_count != filtered_count {
        info!(
            "Filtered {} false positives from {} entries ({} remaining)",
            original_count - filtered_count,
            original_count,
            filtered_count
        );
    }

    filtered
}

/// Truncate a line for logging purposes.
fn truncate_line(line: &str, max_len: usize) -> String {
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

/// Log scanner for periodic health checks
pub struct LogScanner {
    loki: LokiClient,
    config: ScannerConfig,
}

impl LogScanner {
    /// Create a new log scanner with default configuration.
    #[must_use]
    pub fn new(loki: LokiClient) -> Self {
        Self {
            loki,
            config: ScannerConfig::default(),
        }
    }

    /// Create a new log scanner with custom configuration.
    #[must_use]
    pub fn with_config(loki: LokiClient, config: ScannerConfig) -> Self {
        Self { loki, config }
    }

    /// Scan logs for errors and warnings in the given time window.
    ///
    /// # Arguments
    /// * `window` - Time window to scan (e.g., 1 hour)
    ///
    /// # Returns
    /// A scan report with detected issues
    ///
    /// # Errors
    /// Returns an error if Loki queries fail.
    #[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
    pub async fn scan(&self, window: Duration) -> Result<ScanReport> {
        let end = Utc::now();
        let start = end - window;
        // Window duration is always positive by construction
        let window_minutes = window.num_minutes().unsigned_abs();

        info!(
            "Scanning logs from {} to {} ({} minutes)",
            start, end, window_minutes
        );

        let mut all_issues: HashMap<String, ServiceIssue> = HashMap::new();
        let mut total_errors = 0u32;
        let mut total_warnings = 0u32;

        for namespace in &self.config.namespaces {
            debug!("Scanning namespace: {}", namespace);

            // Query for error-level logs
            // We use a more flexible query that handles missing namespace labels
            // and supports both namespace and service_namespace labels

            // Try namespace-specific query first, fallback to broader if it returns nothing
            let mut error_entries = self
                .loki
                .query_logs(
                    &format!(r#"{{namespace="{namespace}"}} |~ "(?i)(error|fatal|panic)""#),
                    start,
                    end,
                    1000,
                )
                .await
                .unwrap_or_default();

            if error_entries.is_empty() {
                debug!("No results for namespace={namespace}, trying service_name fallback");
                let fallback_query = r#"{service_name=~".+"} |~ "(?i)(error|fatal|panic)""#;
                let fallback_entries = self
                    .loki
                    .query_logs(fallback_query, start, end, 1000)
                    .await
                    .unwrap_or_default();

                // Filter fallback entries by namespace if possible
                for entry in fallback_entries {
                    let ns_label = entry
                        .labels
                        .get("namespace")
                        .or_else(|| entry.labels.get("service_namespace"))
                        .map_or("", String::as_str);

                    let pod_name = entry
                        .labels
                        .get("pod")
                        .or_else(|| entry.labels.get("pod_name"))
                        .map_or("", String::as_str);

                    let is_match = ns_label == namespace
                        || entry.line.contains(&format!("_{namespace}_"))
                        || pod_name.contains(&format!("-{namespace}-"))
                        || pod_name.contains(&format!("_{namespace}_"))
                        || (ns_label.is_empty()
                            && (pod_name.contains(namespace)
                                || entry
                                    .labels
                                    .get("service_name")
                                    .is_some_and(|s| s.contains("unknown"))));

                    if is_match {
                        error_entries.push(entry);
                    }
                }
            }

            // Filter out false positives from error entries
            // This removes INFO-level logs that contain "error" as a keyword but are not actual errors
            let pre_filter_count = error_entries.len();
            error_entries = filter_actual_errors(error_entries);
            if pre_filter_count != error_entries.len() {
                info!(
                    "Namespace {}: filtered {} false positives from error entries",
                    namespace,
                    pre_filter_count - error_entries.len()
                );
            }

            // Query for warning-level logs (including platform-specific patterns)
            let mut warn_entries = self
                .loki
                .query_logs(&format!(r#"{{namespace="{namespace}"}} |~ "(?i)(warn|warning|invalid.*signature|unauthorized|forbidden|timeout|connection refused)""#), start, end, 500)
                .await
                .unwrap_or_default();

            if warn_entries.is_empty() {
                debug!(
                    "No results for namespace={namespace} warnings, trying service_name fallback"
                );
                let fallback_query = r#"{service_name=~".+"} |~ "(?i)(warn|warning|invalid.*signature|unauthorized|forbidden|timeout|connection refused)""#;
                let fallback_entries = self
                    .loki
                    .query_logs(fallback_query, start, end, 500)
                    .await
                    .unwrap_or_default();

                for entry in fallback_entries {
                    let ns_label = entry
                        .labels
                        .get("namespace")
                        .or_else(|| entry.labels.get("service_namespace"))
                        .map_or("", String::as_str);

                    let pod_name = entry
                        .labels
                        .get("pod")
                        .or_else(|| entry.labels.get("pod_name"))
                        .map_or("", String::as_str);

                    let is_match = ns_label == namespace
                        || entry.line.contains(&format!("_{namespace}_"))
                        || pod_name.contains(&format!("-{namespace}-"))
                        || pod_name.contains(&format!("_{namespace}_"))
                        || (ns_label.is_empty()
                            && (pod_name.contains(namespace)
                                || entry
                                    .labels
                                    .get("service_name")
                                    .is_some_and(|s| s.contains("unknown"))));

                    if is_match {
                        warn_entries.push(entry);
                    }
                }
            }

            // Group by service/pod
            self.process_entries(namespace, &error_entries, "error", &mut all_issues);
            self.process_entries(namespace, &warn_entries, "warn", &mut all_issues);

            total_errors += u32::try_from(error_entries.len()).unwrap_or(u32::MAX);
            total_warnings += u32::try_from(warn_entries.len()).unwrap_or(u32::MAX);
        }

        // Filter to only services above threshold
        let services_with_issues: Vec<ServiceIssue> = all_issues
            .into_values()
            .filter(|issue| {
                issue.error_count >= self.config.error_threshold
                    || issue.warn_count >= self.config.warn_threshold
            })
            .collect();

        // Determine if remediation is recommended
        let (remediation_recommended, recommendation_reason) =
            self.analyze_for_remediation(&services_with_issues);

        Ok(ScanReport {
            scan_time: end,
            window_minutes,
            namespaces_scanned: self.config.namespaces.clone(),
            services_with_issues,
            total_errors,
            total_warnings,
            remediation_recommended,
            recommendation_reason,
        })
    }

    /// Process log entries and group by service.
    #[allow(clippy::unused_self)]
    fn process_entries(
        &self,
        namespace: &str,
        entries: &[LogEntry],
        level: &str,
        issues: &mut HashMap<String, ServiceIssue>,
    ) {
        for entry in entries {
            // Extract pod name from labels or line content
            let pod = entry
                .labels
                .get("pod")
                .or_else(|| entry.labels.get("pod_name"))
                .cloned()
                .unwrap_or_else(|| {
                    // Try to extract from service_name label if it's not "unknown_service"
                    if let Some(svc) = entry.labels.get("service_name") {
                        if svc != "unknown_service" {
                            return svc.clone();
                        }
                    }
                    "unknown".to_string()
                });

            // Extract service name (strip random suffix from pod name)
            let service = if let Some(svc) = entry.labels.get("service_name") {
                if svc == "unknown_service" {
                    extract_service_name(&pod)
                } else {
                    svc.clone()
                }
            } else {
                extract_service_name(&pod)
            };

            let key = format!("{namespace}/{service}");

            let issue = issues.entry(key).or_insert_with(|| ServiceIssue {
                service: service.clone(),
                namespace: namespace.to_string(),
                error_count: 0,
                warn_count: 0,
                sample_errors: Vec::new(),
                first_seen: None,
                last_seen: None,
                affected_pods: Vec::new(),
            });

            // Update counts
            match level {
                "error" => issue.error_count += 1,
                "warn" => issue.warn_count += 1,
                _ => {}
            }

            // Update timestamps
            if issue.first_seen.is_none() || Some(entry.timestamp) < issue.first_seen {
                issue.first_seen = Some(entry.timestamp);
            }
            if issue.last_seen.is_none() || Some(entry.timestamp) > issue.last_seen {
                issue.last_seen = Some(entry.timestamp);
            }

            // Add sample error (up to 5)
            if level == "error" && issue.sample_errors.len() < 5 {
                let truncated = if entry.line.len() > 200 {
                    format!("{}...", &entry.line[..200])
                } else {
                    entry.line.clone()
                };
                issue.sample_errors.push(truncated);
            }

            // Track affected pods
            if !issue.affected_pods.contains(&pod) {
                issue.affected_pods.push(pod);
            }
        }
    }

    /// Analyze issues to determine if remediation should be triggered.
    #[allow(clippy::unused_self)]
    fn analyze_for_remediation(&self, issues: &[ServiceIssue]) -> (bool, Option<String>) {
        if issues.is_empty() {
            return (false, None);
        }

        // Check for critical services with errors
        // Platform services that require immediate attention
        let critical_services = [
            "controller",
            "healer",
            "pm",
            "cto-pm",
            "tools",
            "cto-tools",
            "argo-workflows",
            "argocd",
        ];
        for issue in issues {
            if critical_services.iter().any(|s| issue.service.contains(s))
                && issue.error_count >= 10
            {
                return (
                    true,
                    Some(format!(
                        "Critical service '{}' has {} errors in scan window",
                        issue.service, issue.error_count
                    )),
                );
            }
        }

        // Check for high error counts anywhere
        let max_errors = issues.iter().map(|i| i.error_count).max().unwrap_or(0);
        if max_errors >= 50 {
            let worst = issues.iter().max_by_key(|i| i.error_count).unwrap();
            return (
                true,
                Some(format!(
                    "High error volume: '{}' has {} errors",
                    worst.service, worst.error_count
                )),
            );
        }

        // Check for multiple services with errors (systemic issue)
        let services_with_errors = issues.iter().filter(|i| i.error_count > 0).count();
        if services_with_errors >= 3 {
            return (
                true,
                Some(format!(
                    "Multiple services ({services_with_errors}) have errors - possible systemic issue"
                )),
            );
        }

        (false, None)
    }

    /// Determine remediation candidates from a scan report.
    #[must_use]
    pub fn determine_candidates(&self, report: &ScanReport) -> Vec<RemediationCandidate> {
        let mut candidates = Vec::new();

        for issue in &report.services_with_issues {
            if issue.error_count < self.config.error_threshold {
                continue;
            }

            let severity = if issue.error_count >= 50 {
                "critical"
            } else if issue.error_count >= 20 {
                "high"
            } else if issue.error_count >= 10 {
                "medium"
            } else {
                "low"
            };

            let suggested_agent = determine_agent_for_service(&issue.service, &issue.sample_errors);

            let log_context = issue.sample_errors.join("\n");

            candidates.push(RemediationCandidate {
                service: issue.service.clone(),
                namespace: issue.namespace.clone(),
                severity: severity.to_string(),
                reason: format!(
                    "{} errors detected from {} pods",
                    issue.error_count,
                    issue.affected_pods.len()
                ),
                suggested_agent,
                log_context,
            });
        }

        // Sort by severity
        candidates.sort_by(|a, b| {
            let severity_order = |s: &str| match s {
                "critical" => 0,
                "high" => 1,
                "medium" => 2,
                _ => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        candidates
    }
}

/// Extract service name from pod name (strips random suffixes).
#[must_use]
pub fn extract_service_name(pod_name: &str) -> String {
    // Pod names typically follow: {service}-{deployment-hash}-{pod-hash}
    // or {service}-{random} for StatefulSets
    // Examples:
    //   controller-7b9f8c6d5-abc12 -> controller
    //   cto-healer-5d4f3e2c1-xyz -> cto-healer
    //   argo-workflows-controller-abc123-def -> argo-workflows-controller

    let parts: Vec<&str> = pod_name.split('-').collect();

    if parts.len() <= 2 {
        return pod_name.to_string();
    }

    // Work backwards to find hash-like suffixes
    // Kubernetes typically adds 1-2 hash suffixes at the end
    // Hashes are alphanumeric, often 3-10 chars
    let mut last_non_hash_index = parts.len();

    for i in (0..parts.len()).rev() {
        let part = parts[i];

        // A part is likely a hash if:
        // - It's short (3-10 chars) and alphanumeric
        // - OR it contains digits mixed with letters
        let is_short_alphanum =
            part.len() >= 3 && part.len() <= 10 && part.chars().all(|c| c.is_ascii_alphanumeric());
        let has_mixed = part.chars().any(|c| c.is_ascii_digit())
            && part.chars().any(|c| c.is_ascii_lowercase());

        // Check if it looks like a hash
        let is_likely_hash = is_short_alphanum && (has_mixed || part.len() <= 5);

        if is_likely_hash {
            last_non_hash_index = i;
        } else {
            // Found a non-hash part, stop
            break;
        }
    }

    // If we found hashes at the end, strip them
    if last_non_hash_index < parts.len() && last_non_hash_index > 0 {
        return parts[..last_non_hash_index].join("-");
    }

    pod_name.to_string()
}

/// Determine which agent should handle remediation based on service and error patterns.
/// Determine which agent should handle remediation for a service based on error patterns.
#[must_use]
pub fn determine_agent_for_service(service: &str, sample_errors: &[String]) -> String {
    let errors_lower: Vec<String> = sample_errors.iter().map(|e| e.to_lowercase()).collect();
    let errors_joined = errors_lower.join(" ");

    // Check for Rust-specific errors
    if errors_joined.contains("error[e")
        || errors_joined.contains("cargo")
        || errors_joined.contains("rustc")
        || errors_joined.contains("clippy")
    {
        return "rex".to_string();
    }

    // Check for frontend/TypeScript errors
    if errors_joined.contains("typescript")
        || errors_joined.contains("eslint")
        || errors_joined.contains("react")
        || errors_joined.contains("npm")
        || errors_joined.contains("node")
    {
        return "blaze".to_string();
    }

    // Check for infrastructure errors
    if errors_joined.contains("kubernetes")
        || errors_joined.contains("docker")
        || errors_joined.contains("container")
        || errors_joined.contains("pod")
        || errors_joined.contains("deployment")
        || errors_joined.contains("helm")
    {
        return "bolt".to_string();
    }

    // Check for security issues
    if errors_joined.contains("permission")
        || errors_joined.contains("unauthorized")
        || errors_joined.contains("forbidden")
        || errors_joined.contains("secret")
        || errors_joined.contains("certificate")
    {
        return "cipher".to_string();
    }

    // Check for git issues
    if errors_joined.contains("git")
        || errors_joined.contains("merge")
        || errors_joined.contains("conflict")
    {
        return "atlas".to_string();
    }

    // Default based on service name
    if service.contains("controller") || service.contains("healer") || service.contains("pm") {
        "rex".to_string() // CTO platform is Rust
    } else if service.contains("argo") || service.contains("flux") {
        "bolt".to_string() // GitOps/workflow infra
    } else {
        "atlas".to_string() // General fallback
    }
}

/// Format a scan report as text output.
#[must_use]
pub fn format_report_text(report: &ScanReport) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    writeln!(output, "=== Log Scan Report ===").unwrap();
    writeln!(output, "Time: {}", report.scan_time).unwrap();
    writeln!(output, "Window: {} minutes", report.window_minutes).unwrap();
    writeln!(
        output,
        "Namespaces: {}",
        report.namespaces_scanned.join(", ")
    )
    .unwrap();
    writeln!(output).unwrap();

    writeln!(
        output,
        "Total Errors: {} | Total Warnings: {}",
        report.total_errors, report.total_warnings
    )
    .unwrap();
    writeln!(output).unwrap();

    if report.services_with_issues.is_empty() {
        writeln!(output, "No services with issues above threshold.").unwrap();
    } else {
        writeln!(
            output,
            "Services with Issues ({}):",
            report.services_with_issues.len()
        )
        .unwrap();
        for issue in &report.services_with_issues {
            writeln!(
                output,
                "  - {}/{}: {} errors, {} warnings ({} pods)",
                issue.namespace,
                issue.service,
                issue.error_count,
                issue.warn_count,
                issue.affected_pods.len()
            )
            .unwrap();
            for (i, sample) in issue.sample_errors.iter().take(2).enumerate() {
                writeln!(output, "      [{}] {}", i + 1, sample).unwrap();
            }
        }
    }

    writeln!(output).unwrap();
    if report.remediation_recommended {
        writeln!(output, "⚠️  REMEDIATION RECOMMENDED").unwrap();
        if let Some(reason) = &report.recommendation_reason {
            writeln!(output, "    Reason: {reason}").unwrap();
        }
    } else {
        writeln!(output, "✓ No remediation needed").unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            extract_service_name("controller-7b9f8c6d5-abc12"),
            "controller"
        );
        assert_eq!(
            extract_service_name("cto-healer-5d4f3e2c1-xyz"),
            "cto-healer"
        );
        assert_eq!(extract_service_name("simple-pod"), "simple-pod");
        assert_eq!(
            extract_service_name("argo-workflows-controller-abc123-def"),
            "argo-workflows-controller"
        );
    }

    #[test]
    fn test_determine_agent_for_service() {
        let rust_errors = vec!["error[E0382]: borrow of moved value".to_string()];
        assert_eq!(
            determine_agent_for_service("controller", &rust_errors),
            "rex"
        );

        let k8s_errors = vec!["pod not found in namespace".to_string()];
        assert_eq!(
            determine_agent_for_service("some-service", &k8s_errors),
            "bolt"
        );

        let git_errors = vec!["merge conflict detected".to_string()];
        assert_eq!(
            determine_agent_for_service("some-service", &git_errors),
            "atlas"
        );
    }

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        // Lower threshold (3) to catch platform issues early
        assert_eq!(config.error_threshold, 3);
        assert!(config.namespaces.contains(&"cto".to_string()));
    }

    // Tests for false positive detection
    #[test]
    fn test_is_false_positive_command_registration() {
        // These are INFO-level messages where "error" is a command name, not an error
        assert!(is_false_positive(
            "F [WORKER 2026-01-01 22:54:07Z INFO ActionCommandManager] Register action command extension for command error"
        ));
        assert!(is_false_positive(
            "[INFO] Register action command extension for command error"
        ));
    }

    #[test]
    fn test_is_false_positive_empty_error_arrays() {
        // Empty error arrays indicate NO errors occurred
        assert!(is_false_positive(r#"  "errorMessages": [],"#));
        assert!(is_false_positive(r#"{"errors": [], "warnings": []}"#));
        assert!(is_false_positive(r"errorMessages: []"));
    }

    #[test]
    fn test_is_false_positive_info_level_with_error_keyword() {
        // INFO-level logs that happen to contain the word "error"
        assert!(is_false_positive(
            "[INFO] Processing error handler registration"
        ));
        assert!(is_false_positive("INFO error recovery completed"));
    }

    #[test]
    fn test_is_false_positive_zero_error_counts() {
        // Reports showing zero errors
        assert!(is_false_positive("errors: 0"));
        assert!(is_false_positive("error count: 0"));
    }

    #[test]
    fn test_is_actual_error_rust_errors() {
        // Rust compiler errors should be detected
        assert!(is_actual_error("error[E0382]: borrow of moved value"));
        assert!(is_actual_error(
            "thread 'main' panicked at 'assertion failed'"
        ));
    }

    #[test]
    fn test_is_actual_error_structured_log_levels() {
        // Structured log level indicators
        assert!(is_actual_error("level=error msg=\"something failed\""));
        assert!(is_actual_error(r#"{"level": "error", "msg": "failed"}"#));
        assert!(is_actual_error("[ERROR] Connection refused"));
        assert!(is_actual_error("[FATAL] Database unavailable"));
    }

    #[test]
    fn test_is_actual_error_stack_traces() {
        // Stack traces and exceptions
        assert!(is_actual_error("exception: NullPointerException"));
        assert!(is_actual_error("Traceback (most recent call last):"));
        assert!(is_actual_error(
            "java.lang.RuntimeException: Error\n\tat stack trace"
        ));
    }

    #[test]
    fn test_is_not_actual_error_info_messages() {
        // INFO-level messages should NOT be detected as actual errors
        assert!(!is_actual_error("INFO: Processing error handler"));
        assert!(!is_actual_error("[INFO] Error recovery complete"));
        assert!(!is_actual_error(
            "Register command extension for command error"
        ));
    }

    #[test]
    fn test_filter_actual_errors_removes_false_positives() {
        use chrono::Utc;

        let entries = vec![
            // False positive: command registration
            LogEntry {
                timestamp: Utc::now(),
                line: "F [WORKER 2026-01-01 22:54:07Z INFO ActionCommandManager] Register action command extension for command error".to_string(),
                labels: HashMap::new(),
            },
            // False positive: empty error array
            LogEntry {
                timestamp: Utc::now(),
                line: r#"  "errorMessages": [],"#.to_string(),
                labels: HashMap::new(),
            },
            // Actual error: Rust compiler error
            LogEntry {
                timestamp: Utc::now(),
                line: "error[E0382]: borrow of moved value".to_string(),
                labels: HashMap::new(),
            },
            // Actual error: ERROR level log
            LogEntry {
                timestamp: Utc::now(),
                line: "[ERROR] Database connection failed".to_string(),
                labels: HashMap::new(),
            },
        ];

        let filtered = filter_actual_errors(entries);

        // Should have filtered out the 2 false positives
        assert_eq!(filtered.len(), 2);

        // Verify the actual errors are retained
        assert!(filtered.iter().any(|e| e.line.contains("error[E0382]")));
        assert!(filtered.iter().any(|e| e.line.contains("[ERROR]")));

        // Verify false positives are removed
        assert!(!filtered
            .iter()
            .any(|e| e.line.contains("command extension")));
        assert!(!filtered.iter().any(|e| e.line.contains("errorMessages")));
    }

    #[test]
    fn test_filter_actual_errors_preserves_all_when_no_false_positives() {
        use chrono::Utc;

        let entries = vec![
            LogEntry {
                timestamp: Utc::now(),
                line: "[ERROR] Connection refused".to_string(),
                labels: HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: "level=error msg=\"timeout\"".to_string(),
                labels: HashMap::new(),
            },
        ];

        let filtered = filter_actual_errors(entries);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_is_false_positive_structured_info_level() {
        // Go-style structured logs with level=info should be filtered as false positives
        // These are the exact patterns seen in ArgoCD logs
        assert!(is_false_positive(
            r#"time="2026-01-02T11:49:25Z" level=info msg="manifest cache hit: &ApplicationSource{RepoURL:https://argoproj.github.io/argo-helm"#
        ));
        assert!(is_false_positive(r#"level=info msg="processing request""#));
        assert!(is_false_positive(r#"level=INFO msg="started""#));

        // JSON-style log levels
        assert!(is_false_positive(
            r#"{"level": "info", "msg": "cache hit"}"#
        ));
        assert!(is_false_positive(
            r#"{"level": "debug", "msg": "trace data"}"#
        ));
    }

    #[test]
    fn test_is_false_positive_manifest_cache_hit() {
        // ArgoCD manifest cache hits are informational
        assert!(is_false_positive("manifest cache hit: application data"));
        assert!(is_false_positive("Manifest cache hit for repo"));
    }

    #[test]
    fn test_filter_argocd_info_logs() {
        use chrono::Utc;

        // These are the exact error samples from the task
        let entries = vec![
            LogEntry {
                timestamp: Utc::now(),
                line: r#"F time="2026-01-02T11:49:25Z" level=info msg="manifest cache hit: &ApplicationSource{RepoURL:https://argoproj.github.io/argo-helm,Path:,TargetRevision:0.45.21,Helm:&ApplicationSourceHelm{ValueFiles:[]..."#.to_string(),
                labels: HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: r#"F time="2026-01-02T11:49:26Z" level=info msg="manifest cache hit: &ApplicationSource{RepoURL:https://argoproj.github.io/argo-helm,Path:,TargetRevision:0.45.21,Helm:&ApplicationSourceHelm{ValueFiles:[]..."#.to_string(),
                labels: HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: r#"F time="2026-01-02T11:49:39Z" level=info msg="manifest cache hit: &ApplicationSource{RepoURL:https://prometheus-community.github.io/helm-charts,Path:,TargetRevision:1.29.0,Helm:&ApplicationSourceHelm{..."#.to_string(),
                labels: HashMap::new(),
            },
            LogEntry {
                timestamp: Utc::now(),
                line: r#"F time="2026-01-02T11:50:02Z" level=info msg="manifest cache hit: &ApplicationSource{RepoURL:https://fluent.github.io/helm-charts,Path:,TargetRevision:0.47.7,Helm:&ApplicationSourceHelm{ValueFiles:[],..."#.to_string(),
                labels: HashMap::new(),
            },
            // This should be retained as an actual error
            LogEntry {
                timestamp: Utc::now(),
                line: "[ERROR] ArgoCD sync failed".to_string(),
                labels: HashMap::new(),
            },
        ];

        let filtered = filter_actual_errors(entries);

        // Should have filtered out all 4 INFO-level manifest cache hits
        assert_eq!(filtered.len(), 1);

        // Only the actual error should remain
        assert!(filtered.iter().any(|e| e.line.contains("[ERROR]")));
        assert!(!filtered
            .iter()
            .any(|e| e.line.contains("manifest cache hit")));
    }
}
