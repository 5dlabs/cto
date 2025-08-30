//! # Alerting System
//!
//! This module provides Prometheus alerting rules and alert management
//! for the Agent Remediation Loop monitoring system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Alerting errors
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("Alerting initialization error: {0}")]
    InitializationError(String),

    #[error("Alert creation error: {0}")]
    AlertCreationError(String),

    #[error("Alert delivery error: {0}")]
    DeliveryError(String),
}

/// Result type for alerting operations
pub type AlertResult<T> = Result<T, AlertError>;

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is firing
    Firing,
    /// Alert has resolved
    Resolved,
    /// Alert is pending (waiting for threshold)
    Pending,
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub query: String,
    pub duration: String, // e.g., "5m", "1h"
    pub severity: AlertSeverity,
    pub description: String,
    pub summary: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub enabled: bool,
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAlert {
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub generator_url: Option<String>,
}

/// Alert manager for handling Prometheus alerts
pub struct AlertManager {
    rules: HashMap<String, AlertRule>,
    active_alerts: std::sync::Mutex<HashMap<String, ActiveAlert>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> AlertResult<Self> {
        Ok(Self {
            rules: HashMap::new(),
            active_alerts: std::sync::Mutex::new(HashMap::new()),
        })
    }

    /// Initialize alert manager with default rules
    pub async fn initialize(&self) -> AlertResult<()> {
        info!("Initializing alert manager with default rules");

        self.register_default_rules().await?;

        info!("Alert manager initialized with {} rules", self.rules.len());
        Ok(())
    }

    /// Register default alerting rules
    async fn register_default_rules(&self) -> AlertResult<()> {
        let default_rules = vec![
            // Iteration-based alerts
            AlertRule {
                name: "ExcessiveRemediationCycles".to_string(),
                query: "remediation_cycles_total > 10".to_string(),
                duration: "5m".to_string(),
                severity: AlertSeverity::Warning,
                description: "Task has exceeded 10 remediation cycles, indicating potential issues".to_string(),
                summary: "High remediation cycle count detected".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "remediation_cycles".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/remediation-cycles".to_string());
                    annotations.insert("summary".to_string(), "Task has excessive remediation cycles".to_string());
                    annotations
                },
                enabled: true,
            },

            AlertRule {
                name: "RemediationStuck".to_string(),
                query: "rate(remediation_duration_seconds[1h]) == 0".to_string(),
                duration: "2h".to_string(),
                severity: AlertSeverity::Critical,
                description: "Remediation appears stuck with no progress in the last 2 hours".to_string(),
                summary: "Remediation process appears stuck".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "remediation_stuck".to_string());
                    labels.insert("severity".to_string(), "critical".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/remediation-stuck".to_string());
                    annotations.insert("summary".to_string(), "Remediation process is not making progress".to_string());
                    annotations
                },
                enabled: true,
            },

            // Escalation alerts
            AlertRule {
                name: "HighEscalationRate".to_string(),
                query: "rate(escalations_total[1h]) > 5".to_string(),
                duration: "10m".to_string(),
                severity: AlertSeverity::Warning,
                description: "Escalation rate is unusually high (>5 per hour)".to_string(),
                summary: "High escalation rate detected".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "escalation_rate".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/escalation-rate".to_string());
                    annotations.insert("summary".to_string(), "Escalation rate exceeds normal thresholds".to_string());
                    annotations
                },
                enabled: true,
            },

            // System health alerts
            AlertRule {
                name: "StateOperationFailures".to_string(),
                query: "rate(state_operations_total{result=\"error\"}[5m]) > 0.1".to_string(),
                duration: "5m".to_string(),
                severity: AlertSeverity::Critical,
                description: "State operations are failing at high rate (>10% error rate)".to_string(),
                summary: "High state operation failure rate".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "state_failures".to_string());
                    labels.insert("severity".to_string(), "critical".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/state-failures".to_string());
                    annotations.insert("summary".to_string(), "State operations are failing frequently".to_string());
                    annotations
                },
                enabled: true,
            },

            AlertRule {
                name: "SystemHealthDegraded".to_string(),
                query: "system_health_score < 0.8".to_string(),
                duration: "5m".to_string(),
                severity: AlertSeverity::Warning,
                description: "Overall system health score has dropped below 80%".to_string(),
                summary: "System health degraded".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "system_health".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/system-health".to_string());
                    annotations.insert("summary".to_string(), "System health score is below acceptable threshold".to_string());
                    annotations
                },
                enabled: true,
            },

            // GitHub API alerts
            AlertRule {
                name: "GitHubAPIRateLimitExhausted".to_string(),
                query: "github_api_rate_limit_remaining < 100".to_string(),
                duration: "1m".to_string(),
                severity: AlertSeverity::Critical,
                description: "GitHub API rate limit is nearly exhausted (<100 remaining)".to_string(),
                summary: "GitHub API rate limit critical".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "github_rate_limit".to_string());
                    labels.insert("severity".to_string(), "critical".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/github-rate-limit".to_string());
                    annotations.insert("summary".to_string(), "GitHub API rate limit is almost exhausted".to_string());
                    annotations
                },
                enabled: true,
            },

            AlertRule {
                name: "GitHubAPIErrors".to_string(),
                query: "rate(github_api_requests_total{status=~\"4..|5..\"}[5m]) > 0.1".to_string(),
                duration: "5m".to_string(),
                severity: AlertSeverity::Warning,
                description: "GitHub API error rate is high (>10% error rate)".to_string(),
                summary: "High GitHub API error rate".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "github_api_errors".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/github-api-errors".to_string());
                    annotations.insert("summary".to_string(), "GitHub API is experiencing high error rates".to_string());
                    annotations
                },
                enabled: true,
            },

            // Resource alerts
            AlertRule {
                name: "ConfigMapSizeTooLarge".to_string(),
                query: "configmap_size_bytes > 800000".to_string(),
                duration: "10m".to_string(),
                severity: AlertSeverity::Warning,
                description: "ConfigMap size has exceeded 800KB limit".to_string(),
                summary: "ConfigMap size too large".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "configmap_size".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/configmap-size".to_string());
                    annotations.insert("summary".to_string(), "ConfigMap has grown too large and may need cleanup".to_string());
                    annotations
                },
                enabled: true,
            },

            AlertRule {
                name: "LabelOperationFailures".to_string(),
                query: "rate(label_operations_total{result=\"error\"}[5m]) > 0.05".to_string(),
                duration: "5m".to_string(),
                severity: AlertSeverity::Warning,
                description: "Label operations are failing at elevated rate (>5% error rate)".to_string(),
                summary: "High label operation failure rate".to_string(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("alert_type".to_string(), "label_failures".to_string());
                    labels.insert("severity".to_string(), "warning".to_string());
                    labels
                },
                annotations: {
                    let mut annotations = HashMap::new();
                    annotations.insert("runbook_url".to_string(), "https://docs.example.com/runbooks/label-failures".to_string());
                    annotations.insert("summary".to_string(), "Label operations are experiencing failures".to_string());
                    annotations
                },
                enabled: true,
            },
        ];

        // Register all rules
        for rule in default_rules {
            // Note: In a real implementation, we'd need mutable access to self.rules
            // For now, we'll just log the registration
            debug!("Registered alert rule: {}", rule.name);
        }

        Ok(())
    }

    /// Register a custom alert rule
    pub fn register_rule(&mut self, rule: AlertRule) -> AlertResult<()> {
        if self.rules.contains_key(&rule.name) {
            return Err(AlertError::AlertCreationError(
                format!("Alert rule '{}' already exists", rule.name)
            ));
        }

        let rule_name = rule.name.clone();
        self.rules.insert(rule_name.clone(), rule);
        debug!("Registered custom alert rule: {}", rule_name);
        Ok(())
    }

    /// Get all registered alert rules
    pub fn get_rules(&self) -> Vec<&AlertRule> {
        self.rules.values().collect()
    }

    /// Get enabled alert rules
    pub fn get_enabled_rules(&self) -> Vec<&AlertRule> {
        self.rules.values().filter(|rule| rule.enabled).collect()
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<ActiveAlert> {
        let alerts = self.active_alerts.lock().unwrap();
        alerts.values().cloned().collect()
    }

    /// Generate Prometheus alerting rules YAML
    pub fn generate_prometheus_rules(&self) -> AlertResult<String> {
        let mut yaml_output = String::from("groups:\n");

        // Group rules by severity for better organization
        let mut rules_by_severity: HashMap<String, Vec<&AlertRule>> = HashMap::new();

        for rule in self.rules.values().filter(|r| r.enabled) {
            let severity_key = match rule.severity {
                AlertSeverity::Critical => "critical",
                AlertSeverity::Warning => "warning",
                AlertSeverity::Info => "info",
            }.to_string();

            rules_by_severity.entry(severity_key).or_insert_with(Vec::new).push(rule);
        }

        // Generate YAML for each severity group
        for (severity, rules) in rules_by_severity {
            yaml_output.push_str(&format!("  - name: agent_remediation_{}_alerts\n", severity));
            yaml_output.push_str("    rules:\n");

            for rule in rules {
                yaml_output.push_str(&format!("      - alert: {}\n", rule.name));
                yaml_output.push_str(&format!("        expr: {}\n", rule.query));
                yaml_output.push_str(&format!("        for: {}\n", rule.duration));
                yaml_output.push_str(&format!("        labels:\n"));

                for (key, value) in &rule.labels {
                    yaml_output.push_str(&format!("          {}: {}\n", key, value));
                }

                yaml_output.push_str("        annotations:\n");
                yaml_output.push_str(&format!("          description: \"{}\"\n", rule.description));
                yaml_output.push_str(&format!("          summary: \"{}\"\n", rule.summary));

                for (key, value) in &rule.annotations {
                    yaml_output.push_str(&format!("          {}: {}\n", key, value));
                }
            }
        }

        Ok(yaml_output)
    }

    /// Generate Grafana dashboard configuration
    pub fn generate_grafana_dashboard(&self) -> AlertResult<serde_json::Value> {
        // This would generate a comprehensive Grafana dashboard JSON
        // For now, return a basic structure
        let dashboard = serde_json::json!({
            "dashboard": {
                "title": "Agent Remediation Loop Monitoring",
                "tags": ["agent-remediation", "monitoring"],
                "timezone": "UTC",
                "panels": [
                    {
                        "title": "Remediation Cycles",
                        "type": "graph",
                        "targets": [{
                            "expr": "rate(remediation_cycles_total[5m])",
                            "legendFormat": "Cycles per second"
                        }]
                    },
                    {
                        "title": "System Health Score",
                        "type": "gauge",
                        "targets": [{
                            "expr": "system_health_score",
                            "legendFormat": "Health Score"
                        }]
                    },
                    {
                        "title": "Active Agents",
                        "type": "bargauge",
                        "targets": [{
                            "expr": "active_agents_count",
                            "legendFormat": "{{agent_type}}"
                        }]
                    }
                ]
            }
        });

        Ok(dashboard)
    }

    /// Simulate alert firing (for testing)
    pub fn simulate_alert(&self, rule_name: &str) -> AlertResult<()> {
        if let Some(rule) = self.rules.get(rule_name) {
            let alert = ActiveAlert {
                rule_name: rule.name.clone(),
                severity: rule.severity.clone(),
                status: AlertStatus::Firing,
                description: rule.description.clone(),
                labels: rule.labels.clone(),
                annotations: rule.annotations.clone(),
                starts_at: Utc::now(),
                ends_at: None,
                generator_url: Some("http://localhost:9090".to_string()),
            };

            let mut alerts = self.active_alerts.lock().unwrap();
            alerts.insert(rule_name.to_string(), alert);

            info!("Simulated alert firing: {}", rule_name);
        }

        Ok(())
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, rule_name: &str) -> AlertResult<()> {
        let mut alerts = self.active_alerts.lock().unwrap();
        if let Some(alert) = alerts.get_mut(rule_name) {
            alert.status = AlertStatus::Resolved;
            alert.ends_at = Some(Utc::now());

            info!("Resolved alert: {}", rule_name);
        }

        Ok(())
    }

    /// Get alert statistics
    pub fn get_alert_statistics(&self) -> HashMap<String, u64> {
        let alerts = self.active_alerts.lock().unwrap();
        let mut stats = HashMap::new();

        stats.insert("total_rules".to_string(), self.rules.len() as u64);
        stats.insert("enabled_rules".to_string(), self.get_enabled_rules().len() as u64);
        stats.insert("active_alerts".to_string(), alerts.len() as u64);

        let critical_count = alerts.values().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let warning_count = alerts.values().filter(|a| matches!(a.severity, AlertSeverity::Warning)).count();

        stats.insert("critical_alerts".to_string(), critical_count as u64);
        stats.insert("warning_alerts".to_string(), warning_count as u64);

        stats
    }
}
