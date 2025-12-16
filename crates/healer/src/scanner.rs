//! Log scanner for periodic error detection and remediation triggering.
//!
//! Scans Loki logs for errors and warnings across platform namespaces,
//! analyzing patterns to determine if automated remediation should be triggered.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::loki::{LogEntry, LokiClient};

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
            error_threshold: 5,
            warn_threshold: 20,
            include_info: false,
        }
    }
}

/// A service with detected issues
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
    #[allow(clippy::cast_sign_loss)]
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
            let error_query =
                format!(r#"{{namespace="{namespace}"}} |~ "(?i)(error|fatal|panic)""#);
            let error_entries = self
                .loki
                .query_logs(&error_query, start, end, 1000)
                .await
                .unwrap_or_else(|e| {
                    warn!("Failed to query errors for {}: {}", namespace, e);
                    Vec::new()
                });

            // Query for warning-level logs
            let warn_query = format!(r#"{{namespace="{namespace}"}} |~ "(?i)(warn|warning)""#);
            let warn_entries = self
                .loki
                .query_logs(&warn_query, start, end, 500)
                .await
                .unwrap_or_else(|e| {
                    warn!("Failed to query warnings for {}: {}", namespace, e);
                    Vec::new()
                });

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
            // Extract pod name from labels
            let pod = entry
                .labels
                .get("pod")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            // Extract service name (strip random suffix from pod name)
            let service = extract_service_name(&pod);
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
        let critical_services = ["controller", "healer", "pm", "argo-workflows"];
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
fn extract_service_name(pod_name: &str) -> String {
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
fn determine_agent_for_service(service: &str, sample_errors: &[String]) -> String {
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
        assert_eq!(config.error_threshold, 5);
        assert!(config.namespaces.contains(&"cto".to_string()));
    }
}
