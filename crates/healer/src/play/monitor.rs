//! Real-time play monitoring with log streaming and anomaly detection.
//!
//! This module provides continuous monitoring of running play workflows by:
//! - Detecting active plays via `CodeRun` CRDs
//! - Streaming logs from all agent pods
//! - Analyzing logs against expected behaviors
//! - Creating GitHub issues when anomalies are detected
//! - Running probe-based evaluations for acceptance criteria (context engineering)

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::github::GitHubClient;
use crate::loki::{LokiClient, LokiConfig};

use super::behavior::{AgentType, BehaviorAnalyzer, DetectionType, LogAnalysis};
use super::evaluator::{EvaluatorConfig, ProbeEvaluator};
use super::types::{ArtifactTrail, EvaluationProbe, EvaluationResults, ProbeResult, ProbeType};

/// Configuration for play monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Namespace to monitor for `CodeRuns`
    pub namespace: String,
    /// Poll interval for log queries (seconds)
    pub poll_interval_secs: u64,
    /// Minimum severity to report (critical, high, medium, low)
    pub min_severity: String,
    /// GitHub repository for issue creation
    pub repository: Option<String>,
    /// Whether to create issues automatically
    pub auto_create_issues: bool,
    /// Maximum issues to create per play
    pub max_issues_per_play: usize,
    /// Cooldown between issues for same pattern (minutes)
    pub issue_cooldown_mins: i64,
    /// Time window for log queries (minutes)
    pub log_window_mins: i64,
    /// Configuration for probe-based evaluation (context engineering)
    pub evaluator_config: EvaluatorConfig,
    /// Whether to use LLM for probe evaluation (false = offline mode)
    pub use_llm_evaluation: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            namespace: "cto".to_string(),
            poll_interval_secs: 30,
            min_severity: "medium".to_string(),
            repository: None,
            auto_create_issues: true,
            max_issues_per_play: 5,
            issue_cooldown_mins: 10,
            log_window_mins: 5,
            evaluator_config: EvaluatorConfig::default(),
            use_llm_evaluation: true,
        }
    }
}

/// A running play being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredPlay {
    /// Play identifier (usually task-id)
    pub play_id: String,
    /// Service identifier
    pub service: Option<String>,
    /// When monitoring started
    pub started_at: DateTime<Utc>,
    /// Active `CodeRuns` in this play
    pub active_coderuns: Vec<ActiveCodeRun>,
    /// Issues created for this play
    pub issues_created: Vec<String>,
    /// Last time logs were checked
    pub last_log_check: Option<DateTime<Utc>>,
    /// Detected anomalies
    pub anomalies: Vec<DetectedAnomaly>,
    /// Probe-based evaluation results (context engineering)
    pub evaluation_results: Option<EvaluationResults>,
    /// Artifact trail from agent session
    pub artifact_trail: Option<ArtifactTrail>,
}

/// An active `CodeRun` being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCodeRun {
    /// `CodeRun` name
    pub name: String,
    /// Agent type
    pub agent: AgentType,
    /// Pod name if running
    pub pod_name: Option<String>,
    /// Current phase
    pub phase: String,
    /// When started
    pub started_at: Option<DateTime<Utc>>,
}

/// A detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAnomaly {
    /// When detected
    pub detected_at: DateTime<Utc>,
    /// The log analysis result
    pub analysis: LogAnalysis,
    /// `CodeRun` that produced this
    pub coderun_name: String,
    /// Whether an issue was created
    pub issue_created: Option<String>,
    /// Hash of pattern+coderun for deduplication
    pub fingerprint: String,
}

/// Event emitted by the monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    /// Event type
    pub event_type: MonitorEventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Play ID if applicable
    pub play_id: Option<String>,
    /// Event details
    pub details: serde_json::Value,
}

/// Types of monitor events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorEventType {
    /// New play detected
    PlayDetected,
    /// Play completed
    PlayCompleted,
    /// `CodeRun` started
    CodeRunStarted,
    /// `CodeRun` completed
    CodeRunCompleted,
    /// Anomaly detected
    AnomalyDetected,
    /// Issue created
    IssueCreated,
    /// Success pattern matched
    SuccessDetected,
    /// Probe-based evaluation completed (context engineering)
    EvaluationCompleted,
    /// Monitor error
    Error,
}

/// Play monitor for real-time log analysis
pub struct PlayMonitor {
    config: MonitorConfig,
    loki: LokiClient,
    analyzer: BehaviorAnalyzer,
    github: Option<GitHubClient>,
    /// Probe-based evaluator for context engineering quality assessment
    evaluator: ProbeEvaluator,
    /// Currently monitored plays
    plays: HashMap<String, MonitoredPlay>,
    /// Fingerprints of recently reported anomalies (for deduplication)
    recent_fingerprints: HashSet<String>,
    /// Event sender
    event_tx: Option<mpsc::Sender<MonitorEvent>>,
}

impl PlayMonitor {
    /// Create a new play monitor
    #[must_use]
    pub fn new(config: MonitorConfig) -> Self {
        let loki = LokiClient::new(LokiConfig::default());
        let github = config.repository.as_ref().map(|repo| {
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() == 2 {
                GitHubClient::new(parts[0], parts[1])
            } else {
                GitHubClient::new("5dlabs", repo)
            }
        });

        Self {
            config,
            loki,
            analyzer: BehaviorAnalyzer::new(),
            github,
            plays: HashMap::new(),
            recent_fingerprints: HashSet::new(),
            event_tx: None,
        }
    }

    /// Set the event sender for emitting monitor events
    pub fn set_event_sender(&mut self, tx: mpsc::Sender<MonitorEvent>) {
        self.event_tx = Some(tx);
    }

    /// Run the monitor loop
    ///
    /// # Errors
    /// Returns an error if the monitoring loop encounters a fatal error.
    pub async fn run(&mut self) -> Result<()> {
        info!(
            namespace = %self.config.namespace,
            poll_interval = %self.config.poll_interval_secs,
            "Starting play monitor"
        );

        loop {
            if let Err(e) = self.poll_once().await {
                error!(error = %e, "Monitor poll failed");
                self.emit_event(
                    MonitorEventType::Error,
                    None,
                    serde_json::json!({
                        "error": e.to_string()
                    }),
                )
                .await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.config.poll_interval_secs,
            ))
            .await;
        }
    }

    /// Poll once for active plays and check logs
    ///
    /// # Errors
    /// Returns an error if kubectl or Loki queries fail.
    pub async fn poll_once(&mut self) -> Result<()> {
        // 1. Discover active CodeRuns
        let active_coderuns = self.discover_active_coderuns()?;

        // 2. Group by play_id (task-id label)
        let mut plays_by_id: HashMap<String, Vec<ActiveCodeRun>> = HashMap::new();
        for coderun in active_coderuns {
            let play_id = coderun
                .name
                .split('-')
                .take(3)
                .collect::<Vec<_>>()
                .join("-");
            plays_by_id.entry(play_id).or_default().push(coderun);
        }

        // 3. Update tracked plays
        for (play_id, coderuns) in plays_by_id {
            if self.plays.contains_key(&play_id) {
                // Update existing play
                if let Some(play) = self.plays.get_mut(&play_id) {
                    play.active_coderuns = coderuns;
                }
            } else {
                // New play detected
                info!(play_id = %play_id, coderuns = %coderuns.len(), "New play detected");
                self.emit_event(
                    MonitorEventType::PlayDetected,
                    Some(&play_id),
                    serde_json::json!({
                        "coderuns": coderuns.iter().map(|c| &c.name).collect::<Vec<_>>()
                    }),
                )
                .await;

                self.plays.insert(
                    play_id.clone(),
                    MonitoredPlay {
                        play_id: play_id.clone(),
                        service: None,
                        started_at: Utc::now(),
                        active_coderuns: coderuns.clone(),
                        issues_created: Vec::new(),
                        last_log_check: None,
                        anomalies: Vec::new(),
                        evaluation_results: None,
                        artifact_trail: None,
                    },
                );
            }
        }

        // 4. Check logs for each active play
        let play_ids: Vec<_> = self.plays.keys().cloned().collect();
        for play_id in play_ids {
            if let Err(e) = self.check_play_logs(&play_id).await {
                warn!(play_id = %play_id, error = %e, "Failed to check play logs");
            }
        }

        // 5. Clean up completed plays
        self.cleanup_completed_plays().await;

        Ok(())
    }

    /// Discover active `CodeRuns` in the namespace
    fn discover_active_coderuns(&self) -> Result<Vec<ActiveCodeRun>> {
        let output = Command::new("kubectl")
            .args([
                "get",
                "coderuns",
                "-n",
                &self.config.namespace,
                "-o",
                "json",
            ])
            .output()
            .context("Failed to run kubectl get coderuns")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl failed: {stderr}");
        }

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).context("Failed to parse kubectl output")?;

        let mut coderuns = Vec::new();

        if let Some(items) = json["items"].as_array() {
            for item in items {
                let name = item["metadata"]["name"].as_str().unwrap_or_default();
                let phase = item["status"]["phase"].as_str().unwrap_or("Unknown");

                // Only track Running or Pending CodeRuns
                if phase != "Running" && phase != "Pending" {
                    continue;
                }

                let labels = item["metadata"]["labels"].as_object();
                let agent_str = labels
                    .and_then(|l| l.get("agents.platform/agent").or(l.get("healer/agent")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let pod_name = item["status"]["podName"].as_str().map(String::from);

                let started_at = item["status"]["startedAt"]
                    .as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                coderuns.push(ActiveCodeRun {
                    name: name.to_string(),
                    agent: agent_str.parse().unwrap_or(AgentType::Unknown),
                    pod_name,
                    phase: phase.to_string(),
                    started_at,
                });
            }
        }

        debug!(count = %coderuns.len(), "Discovered active CodeRuns");
        Ok(coderuns)
    }

    /// Check logs for a specific play
    async fn check_play_logs(&mut self, play_id: &str) -> Result<()> {
        let play = match self.plays.get(play_id) {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let now = Utc::now();
        let window_start = play
            .last_log_check
            .unwrap_or_else(|| now - Duration::minutes(self.config.log_window_mins));

        // Check logs for each active CodeRun
        for coderun in &play.active_coderuns {
            if let Some(pod_name) = &coderun.pod_name {
                if let Err(e) = self
                    .analyze_pod_logs(
                        play_id,
                        &coderun.name,
                        pod_name,
                        coderun.agent,
                        window_start,
                        now,
                    )
                    .await
                {
                    warn!(
                        play_id = %play_id,
                        pod = %pod_name,
                        error = %e,
                        "Failed to analyze pod logs"
                    );
                }
            }
        }

        // Update last check time
        if let Some(play) = self.plays.get_mut(play_id) {
            play.last_log_check = Some(now);
        }

        Ok(())
    }

    /// Analyze logs from a specific pod
    async fn analyze_pod_logs(
        &mut self,
        play_id: &str,
        coderun_name: &str,
        pod_name: &str,
        agent: AgentType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<()> {
        // Query logs from Loki
        let entries = self.loki
            .query_pod_logs(&self.config.namespace, pod_name, start, end, 1000)
            .await
            .unwrap_or_else(|e| {
                debug!(pod = %pod_name, error = %e, "Could not query Loki, falling back to kubectl");
                Vec::new()
            });

        // If Loki didn't return logs, try kubectl
        let log_lines: Vec<String> = if entries.is_empty() {
            self.get_kubectl_logs(pod_name, 100).unwrap_or_default()
        } else {
            entries.iter().map(|e| e.line.clone()).collect()
        };

        // Analyze each line
        for line in &log_lines {
            let analysis = self.analyzer.analyze_line(line, agent);

            match analysis.detection_type {
                DetectionType::Failure | DetectionType::Anomaly => {
                    self.handle_anomaly(play_id, coderun_name, analysis).await?;
                }
                DetectionType::Success => {
                    debug!(
                        play_id = %play_id,
                        pattern = %analysis.matched_pattern,
                        "Success pattern detected"
                    );
                    self.emit_event(
                        MonitorEventType::SuccessDetected,
                        Some(play_id),
                        serde_json::json!({
                            "pattern": analysis.matched_pattern,
                            "agent": format!("{:?}", agent),
                            "line": analysis.line,
                        }),
                    )
                    .await;
                }
                DetectionType::Normal => {}
            }
        }

        Ok(())
    }

    /// Get logs via kubectl as fallback
    fn get_kubectl_logs(&self, pod_name: &str, tail: u32) -> Result<Vec<String>> {
        let output = Command::new("kubectl")
            .args([
                "logs",
                pod_name,
                "-n",
                &self.config.namespace,
                "--tail",
                &tail.to_string(),
            ])
            .output()
            .context("Failed to run kubectl logs")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl logs failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(String::from).collect())
    }

    /// Handle a detected anomaly
    async fn handle_anomaly(
        &mut self,
        play_id: &str,
        coderun_name: &str,
        analysis: LogAnalysis,
    ) -> Result<()> {
        // Check severity threshold
        let severity_order = |s: &str| match s {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        };

        if severity_order(&analysis.severity) > severity_order(&self.config.min_severity) {
            return Ok(());
        }

        // Create fingerprint for deduplication
        let fingerprint = format!(
            "{}:{}:{}",
            coderun_name,
            analysis.matched_pattern,
            &analysis.line.chars().take(50).collect::<String>()
        );

        if self.recent_fingerprints.contains(&fingerprint) {
            debug!(fingerprint = %fingerprint, "Skipping duplicate anomaly");
            return Ok(());
        }

        info!(
            play_id = %play_id,
            coderun = %coderun_name,
            severity = %analysis.severity,
            pattern = %analysis.matched_pattern,
            "Anomaly detected"
        );

        // Emit event
        self.emit_event(
            MonitorEventType::AnomalyDetected,
            Some(play_id),
            serde_json::json!({
                "coderun": coderun_name,
                "severity": analysis.severity,
                "pattern": analysis.matched_pattern,
                "line": analysis.line,
                "detection_type": format!("{:?}", analysis.detection_type),
            }),
        )
        .await;

        // Record the anomaly
        let anomaly = DetectedAnomaly {
            detected_at: Utc::now(),
            analysis: analysis.clone(),
            coderun_name: coderun_name.to_string(),
            issue_created: None,
            fingerprint: fingerprint.clone(),
        };

        if let Some(play) = self.plays.get_mut(play_id) {
            play.anomalies.push(anomaly);
        }

        self.recent_fingerprints.insert(fingerprint);

        // Create GitHub issue if enabled
        if self.config.auto_create_issues {
            if let Some(play) = self.plays.get(play_id) {
                if play.issues_created.len() < self.config.max_issues_per_play {
                    self.create_anomaly_issue(play_id, coderun_name, &analysis)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Create a GitHub issue for an anomaly
    async fn create_anomaly_issue(
        &mut self,
        play_id: &str,
        coderun_name: &str,
        analysis: &LogAnalysis,
    ) -> Result<()> {
        let Some(github) = &self.github else {
            debug!("GitHub client not configured, skipping issue creation");
            return Ok(());
        };

        let title = format!(
            "[HEAL-MONITOR] {} detected in {} ({})",
            analysis.matched_pattern,
            coderun_name,
            analysis.severity.to_uppercase()
        );

        let body = format!(
            r"## Anomaly Detection

**Play ID:** `{play_id}`
**CodeRun:** `{coderun_name}`
**Agent:** `{agent:?}`
**Severity:** {severity}
**Pattern:** {pattern}

### Detected Log Line

```
{line}
```

### Detection Details

- **Type:** {detection_type:?}
- **Detected At:** {detected_at}

### Recommended Actions

1. Check the CodeRun logs for full context
2. Review if this is expected behavior for this agent
3. If this is a real issue, investigate and fix
4. If this is a false positive, consider updating the behavior patterns

---
*This issue was auto-created by the Healer Play Monitor*
",
            play_id = play_id,
            coderun_name = coderun_name,
            agent = analysis.agent,
            severity = analysis.severity,
            pattern = analysis.matched_pattern,
            line = analysis.line,
            detection_type = analysis.detection_type,
            detected_at = Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        );

        match github.create_issue(&title, &body, &["heal", "monitor", &analysis.severity]) {
            Ok(issue_url) => {
                info!(issue = %issue_url, "Created GitHub issue for anomaly");

                if let Some(play) = self.plays.get_mut(play_id) {
                    play.issues_created.push(issue_url.clone());
                }

                self.emit_event(
                    MonitorEventType::IssueCreated,
                    Some(play_id),
                    serde_json::json!({
                        "issue_url": issue_url,
                        "coderun": coderun_name,
                        "severity": analysis.severity,
                    }),
                )
                .await;
            }
            Err(e) => {
                warn!(error = %e, "Failed to create GitHub issue");
            }
        }

        Ok(())
    }

    /// Clean up completed plays
    async fn cleanup_completed_plays(&mut self) {
        let completed: Vec<_> = self
            .plays
            .iter()
            .filter(|(_, play)| {
                play.active_coderuns.is_empty()
                    || play
                        .active_coderuns
                        .iter()
                        .all(|c| c.phase != "Running" && c.phase != "Pending")
            })
            .map(|(id, _)| id.clone())
            .collect();

        for play_id in completed {
            if let Some(play) = self.plays.remove(&play_id) {
                info!(
                    play_id = %play_id,
                    anomalies = %play.anomalies.len(),
                    issues = %play.issues_created.len(),
                    "Play completed, removing from monitoring"
                );

                self.emit_event(
                    MonitorEventType::PlayCompleted,
                    Some(&play_id),
                    serde_json::json!({
                        "anomalies_detected": play.anomalies.len(),
                        "issues_created": play.issues_created,
                        "duration_mins": (Utc::now() - play.started_at).num_minutes(),
                    }),
                )
                .await;
            }
        }
    }

    /// Emit a monitor event
    async fn emit_event(
        &self,
        event_type: MonitorEventType,
        play_id: Option<&str>,
        details: serde_json::Value,
    ) {
        let event = MonitorEvent {
            event_type,
            timestamp: Utc::now(),
            play_id: play_id.map(String::from),
            details,
        };

        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event).await;
        }
    }

    /// Get current monitoring status
    #[must_use]
    pub fn get_status(&self) -> MonitorStatus {
        MonitorStatus {
            active_plays: self.plays.len(),
            total_coderuns: self.plays.values().map(|p| p.active_coderuns.len()).sum(),
            total_anomalies: self.plays.values().map(|p| p.anomalies.len()).sum(),
            total_issues: self.plays.values().map(|p| p.issues_created.len()).sum(),
            plays: self.plays.values().cloned().collect(),
        }
    }

    // =========================================================================
    // Probe-Based Evaluation (Context Engineering)
    // =========================================================================

    /// Load artifact trail from a CodeRun's workspace.
    ///
    /// The artifact trail is persisted by the sidecar to `/workspace/artifact-trail.json`.
    async fn load_artifact_trail(&self, pod_name: &str) -> Option<ArtifactTrail> {
        let output = Command::new("kubectl")
            .args([
                "exec",
                pod_name,
                "-n",
                &self.config.namespace,
                "--",
                "cat",
                "/workspace/artifact-trail.json",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            debug!(pod = %pod_name, "No artifact trail found");
            return None;
        }

        serde_json::from_slice(&output.stdout).ok()
    }

    /// Generate evaluation probes for a play based on its context.
    ///
    /// Uses the artifact trail and anomaly history to create targeted probes
    /// that test whether the agent retained critical information.
    #[must_use]
    pub fn generate_probes(&self, play: &MonitoredPlay) -> Vec<EvaluationProbe> {
        let mut probes = Vec::new();

        // Artifact probes - test file tracking
        if let Some(trail) = &play.artifact_trail {
            if !trail.files_modified.is_empty() {
                let files: Vec<_> = trail.files_modified.keys().cloned().collect();
                probes.push(
                    EvaluationProbe::new(
                        ProbeType::Artifact,
                        "Which files have been modified in this session?",
                    )
                    .with_keywords(files),
                );
            }

            if !trail.files_created.is_empty() {
                probes.push(
                    EvaluationProbe::new(
                        ProbeType::Artifact,
                        "What new files were created?",
                    )
                    .with_keywords(trail.files_created.clone()),
                );
            }

            if !trail.decisions_made.is_empty() {
                probes.push(
                    EvaluationProbe::new(
                        ProbeType::Decision,
                        "What key decisions were made during this task?",
                    )
                    .with_keywords(
                        trail
                            .decisions_made
                            .iter()
                            .flat_map(|d| d.split_whitespace().take(3).map(String::from))
                            .collect(),
                    ),
                );
            }
        }

        // Recall probes - test error retention if anomalies occurred
        if !play.anomalies.is_empty() {
            let error_keywords: Vec<_> = play
                .anomalies
                .iter()
                .flat_map(|a| {
                    a.analysis
                        .line
                        .split_whitespace()
                        .filter(|w| w.len() > 4)
                        .take(3)
                        .map(String::from)
                })
                .collect();

            if !error_keywords.is_empty() {
                probes.push(
                    EvaluationProbe::new(
                        ProbeType::Recall,
                        "What errors or issues were encountered?",
                    )
                    .with_keywords(error_keywords),
                );
            }
        }

        // Continuation probe - always useful
        probes.push(EvaluationProbe::new(
            ProbeType::Continuation,
            "What is the next step to complete this task?",
        ));

        // Acceptance probe - critical for Play workflow
        probes.push(
            EvaluationProbe::new(
                ProbeType::Acceptance,
                "Have all acceptance criteria been met?",
            )
            .with_keywords(vec![
                "complete".to_string(),
                "pass".to_string(),
                "success".to_string(),
            ]),
        );

        probes
    }

    /// Run evaluation probes for a play.
    ///
    /// This would typically call an LLM to answer the probe questions,
    /// but for now we just set up the framework and log the probes.
    pub async fn run_evaluation(&mut self, play_id: &str) -> Result<EvaluationResults> {
        let play = self
            .plays
            .get(play_id)
            .ok_or_else(|| anyhow::anyhow!("Play not found: {play_id}"))?
            .clone();

        // Load artifact trail if not already loaded
        let artifact_trail = if play.artifact_trail.is_some() {
            play.artifact_trail.clone()
        } else {
            // Try to load from the first running pod
            let trail = if let Some(coderun) = play.active_coderuns.first() {
                if let Some(pod) = &coderun.pod_name {
                    self.load_artifact_trail(pod).await
                } else {
                    None
                }
            } else {
                None
            };

            // Store it in the play
            if let Some(play_mut) = self.plays.get_mut(play_id) {
                play_mut.artifact_trail = trail.clone();
            }

            trail
        };

        // Generate probes
        let play_with_trail = MonitoredPlay {
            artifact_trail: artifact_trail.clone(),
            ..play.clone()
        };
        let probes = self.generate_probes(&play_with_trail);

        info!(
            play_id = %play_id,
            probe_count = %probes.len(),
            "Generated evaluation probes"
        );

        // For now, create placeholder results
        // In a full implementation, this would call an LLM to answer each probe
        let probe_results: Vec<ProbeResult> = probes
            .into_iter()
            .map(|probe| ProbeResult {
                score: 0.5, // Placeholder - would be computed from LLM response
                passed: false, // Placeholder
                response: String::new(),
                notes: Some("Evaluation pending - requires LLM integration".to_string()),
                probe,
            })
            .collect();

        let mut results = EvaluationResults::from_probes(probe_results);
        if let Some(trail) = artifact_trail {
            results = results.with_artifact_trail(trail);
        }

        // Store results
        if let Some(play_mut) = self.plays.get_mut(play_id) {
            play_mut.evaluation_results = Some(results.clone());
        }

        // Emit event
        self.emit_event(
            MonitorEventType::EvaluationCompleted,
            Some(play_id),
            serde_json::json!({
                "overall_score": results.overall_score,
                "passed": results.passed,
                "probe_count": results.probes.len(),
                "threshold": results.threshold,
            }),
        )
        .await;

        Ok(results)
    }

    /// Check if a play passes acceptance criteria using probe-based evaluation.
    ///
    /// This is the main entry point for context engineering evaluation.
    pub async fn verify_acceptance(&mut self, play_id: &str, threshold: f32) -> Result<bool> {
        let results = self.run_evaluation(play_id).await?;

        if results.overall_score >= threshold {
            info!(
                play_id = %play_id,
                score = %results.overall_score,
                threshold = %threshold,
                "Play passed acceptance criteria"
            );
            Ok(true)
        } else {
            warn!(
                play_id = %play_id,
                score = %results.overall_score,
                threshold = %threshold,
                "Play failed acceptance criteria"
            );
            Ok(false)
        }
    }
}

/// Current monitoring status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    /// Number of active plays being monitored
    pub active_plays: usize,
    /// Total active `CodeRuns` across all plays
    pub total_coderuns: usize,
    /// Total anomalies detected
    pub total_anomalies: usize,
    /// Total issues created
    pub total_issues: usize,
    /// Details of each play
    pub plays: Vec<MonitoredPlay>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.namespace, "cto");
        assert_eq!(config.poll_interval_secs, 30);
        assert!(config.auto_create_issues);
    }

    #[test]
    fn test_severity_filtering() {
        let config = MonitorConfig {
            min_severity: "high".to_string(),
            ..Default::default()
        };

        // High severity should pass
        let severity_order = |s: &str| match s {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        };

        assert!(severity_order("high") <= severity_order(&config.min_severity));
        assert!(severity_order("critical") <= severity_order(&config.min_severity));
        assert!(severity_order("medium") > severity_order(&config.min_severity));
    }
}
