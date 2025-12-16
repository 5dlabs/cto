//! GitHub Actions workflow failure sensor.
//!
//! Polls GitHub Actions for workflow failures and triggers CI remediation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::ci::{self, types::CiFailure, CiRouter, CodeRunSpawner};

/// Configuration for the GitHub Actions sensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    /// Repositories to monitor (e.g., "5dlabs/cto")
    pub repositories: Vec<String>,
    /// Poll interval in seconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    /// How far back to look on first poll (minutes)
    #[serde(default = "default_lookback")]
    pub lookback_mins: u64,
    /// Whether to create GitHub issues for failures
    #[serde(default = "default_create_issues")]
    pub create_issues: bool,
    /// Labels to add to created issues
    #[serde(default = "default_labels")]
    pub issue_labels: Vec<String>,
    /// Branches to monitor (empty = all)
    #[serde(default)]
    pub branches: Vec<String>,
    /// Workflows to exclude
    #[serde(default)]
    pub excluded_workflows: Vec<String>,
    /// Maximum failures to process per poll
    #[serde(default = "default_max_per_poll")]
    pub max_per_poll: usize,
    /// Kubernetes namespace for `CodeRuns`
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

fn default_poll_interval() -> u64 {
    300
}
fn default_lookback() -> u64 {
    60
}
fn default_create_issues() -> bool {
    true
}
fn default_labels() -> Vec<String> {
    vec!["healer".to_string(), "ci-failure".to_string()]
}
fn default_max_per_poll() -> usize {
    10
}
fn default_namespace() -> String {
    "cto".to_string()
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            repositories: vec!["5dlabs/cto".to_string()],
            poll_interval_secs: default_poll_interval(),
            lookback_mins: default_lookback(),
            create_issues: default_create_issues(),
            issue_labels: default_labels(),
            branches: vec![],
            excluded_workflows: vec![],
            max_per_poll: default_max_per_poll(),
            namespace: default_namespace(),
        }
    }
}

/// A detected workflow failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailure {
    /// Workflow run ID
    pub run_id: u64,
    /// Workflow name
    pub workflow_name: String,
    /// Job name (if available)
    pub job_name: Option<String>,
    /// Job ID (if available)
    pub job_id: Option<u64>,
    /// Branch name
    pub branch: String,
    /// Head commit SHA
    pub head_sha: String,
    /// Commit message
    pub commit_message: String,
    /// Repository (owner/repo)
    pub repository: String,
    /// URL to the workflow run
    pub html_url: String,
    /// URL to the specific job (if available)
    pub job_url: Option<String>,
    /// Actor who triggered the workflow
    pub actor: String,
    /// When the run started
    pub run_started_at: DateTime<Utc>,
    /// When we detected this failure
    pub detected_at: DateTime<Utc>,
    /// Conclusion (failure, cancelled, etc.)
    pub conclusion: String,
}

impl WorkflowFailure {
    /// Convert to [`CiFailure`] for the remediation pipeline.
    #[must_use]
    pub fn to_ci_failure(&self) -> CiFailure {
        CiFailure {
            workflow_run_id: self.run_id,
            workflow_name: self.workflow_name.clone(),
            job_name: self.job_name.clone(),
            conclusion: self.conclusion.clone(),
            branch: self.branch.clone(),
            head_sha: self.head_sha.clone(),
            commit_message: self.commit_message.clone(),
            html_url: self.html_url.clone(),
            repository: self.repository.clone(),
            sender: self.actor.clone(),
            detected_at: self.detected_at,
            raw_event: None,
        }
    }
}

/// GitHub Actions workflow failure sensor.
pub struct GitHubActionsSensor {
    config: SensorConfig,
    remediation_config: ci::types::RemediationConfig,
    /// Track processed run IDs to avoid duplicates
    processed_runs: HashSet<u64>,
    /// Last poll timestamp
    last_poll: Option<DateTime<Utc>>,
}

impl GitHubActionsSensor {
    /// Create a new sensor with the given configuration.
    #[must_use]
    pub fn new(config: SensorConfig, remediation_config: ci::types::RemediationConfig) -> Self {
        Self {
            config,
            remediation_config,
            processed_runs: HashSet::new(),
            last_poll: None,
        }
    }

    /// Run the sensor in a continuous loop.
    ///
    /// # Errors
    ///
    /// This function runs indefinitely and only returns an error if
    /// an unrecoverable failure occurs during polling.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Err(e) = self.poll_once() {
                error!("Sensor poll failed: {e}");
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.config.poll_interval_secs,
            ))
            .await;
        }
    }

    /// Perform a single poll cycle.
    ///
    /// # Errors
    ///
    /// Returns an error if polling repositories or processing failures fails.
    pub fn poll_once(&mut self) -> Result<Vec<WorkflowFailure>> {
        let mut all_failures = Vec::new();

        for repo in &self.config.repositories.clone() {
            match self.poll_repository(repo) {
                Ok(failures) => {
                    if !failures.is_empty() {
                        info!("Found {} new failure(s) in {}", failures.len(), repo);
                    }
                    all_failures.extend(failures);
                }
                Err(e) => {
                    error!("Error polling {}: {}", repo, e);
                }
            }
        }

        // Process failures (up to max_per_poll)
        let mut processed = Vec::new();
        for failure in all_failures.into_iter().take(self.config.max_per_poll) {
            info!(
                "Processing failure: {} (run {})",
                failure.workflow_name, failure.run_id
            );
            match self.process_failure(&failure) {
                Ok(()) => processed.push(failure),
                Err(e) => {
                    error!(
                        "Failed to process failure {} ({}): {}",
                        failure.workflow_name, failure.run_id, e
                    );
                }
            }
        }

        self.last_poll = Some(Utc::now());
        Ok(processed)
    }

    /// Poll a single repository for failures.
    fn poll_repository(&mut self, repository: &str) -> Result<Vec<WorkflowFailure>> {
        debug!("Polling repository: {}", repository);

        // Use `gh` CLI to list workflow runs
        // Note: actor field is not available in `gh run list`, we'll fetch it separately
        let output = Command::new("gh")
            .args([
                "run",
                "list",
                "--repo",
                repository,
                "--status",
                "failure",
                "--json",
                "databaseId,name,headBranch,headSha,url,conclusion,createdAt,workflowName,event",
                "--limit",
                "50",
            ])
            .output()
            .context("Failed to execute gh run list")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh run list failed: {stderr}");
        }

        let runs: Vec<serde_json::Value> =
            serde_json::from_slice(&output.stdout).context("Failed to parse gh output")?;

        let mut failures = Vec::new();
        let lookback_cutoff = Utc::now()
            - chrono::Duration::minutes(self.config.lookback_mins.try_into().unwrap_or(60));

        for run in runs {
            let run_id = run["databaseId"].as_u64().unwrap_or(0);
            if run_id == 0 || self.processed_runs.contains(&run_id) {
                continue;
            }

            // Parse created_at timestamp
            let created_at_str = run["createdAt"].as_str().unwrap_or("");
            let created_at = DateTime::parse_from_rfc3339(created_at_str)
                .ok()
                .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));

            // Skip old runs (unless this is first poll with last_poll=None)
            if self.last_poll.is_some() && created_at < lookback_cutoff {
                continue;
            }

            // Get branch and check filters
            let branch = run["headBranch"].as_str().unwrap_or("").to_string();
            if !self.config.branches.is_empty()
                && !self.config.branches.iter().any(|b| b == &branch)
            {
                continue;
            }

            // Get workflow name and check exclusions
            let workflow_name = run["workflowName"].as_str().unwrap_or("").to_string();
            if self
                .config
                .excluded_workflows
                .iter()
                .any(|w| workflow_name.contains(w))
            {
                continue;
            }

            // Mark as processed
            self.processed_runs.insert(run_id);

            // Actor is not available in run list, will be fetched via run view
            let failure = WorkflowFailure {
                run_id,
                workflow_name,
                job_name: None, // Will be filled in later if we fetch job details
                job_id: None,
                branch,
                head_sha: run["headSha"].as_str().unwrap_or("").to_string(),
                commit_message: String::new(), // Not in this API response
                repository: repository.to_string(),
                html_url: run["url"].as_str().unwrap_or("").to_string(),
                job_url: None,
                actor: "unknown".to_string(), // Will be fetched via run view
                run_started_at: created_at,
                detected_at: Utc::now(),
                conclusion: run["conclusion"].as_str().unwrap_or("failure").to_string(),
            };

            failures.push(failure);
        }

        Ok(failures)
    }

    /// Process a detected failure.
    fn process_failure(&self, failure: &WorkflowFailure) -> Result<()> {
        // Fetch additional details (job info, actor)
        let (job_name, job_id, job_url, actor) = Self::fetch_failed_job_details(failure)?;
        let mut failure = failure.clone();
        failure.job_name = job_name;
        failure.job_id = job_id;
        failure.job_url = job_url;
        if let Some(actor_name) = actor {
            failure.actor = actor_name;
        }

        // Convert to CiFailure for routing
        let ci_failure = failure.to_ci_failure();

        // Fetch logs for classification
        let logs = Self::fetch_workflow_logs(&failure).unwrap_or_default();

        // Classify and route
        let router = CiRouter::new();
        let failure_type = router.classify_failure(&ci_failure, &logs);

        // Build remediation context for routing
        let routing_ctx = ci::types::RemediationContext {
            failure: Some(ci_failure.clone()),
            security_alert: None,
            failure_type: Some(failure_type.clone()),
            workflow_logs: logs.clone(),
            pr: None,
            changed_files: vec![],
            argocd_status: None,
            recent_logs: String::new(),
            pod_state: None,
            error_rate: None,
            historical: None,
            previous_attempts: vec![],
            agent_failure_output: None,
            changes_made_so_far: vec![],
        };
        let agent = router.route(&routing_ctx);
        info!("Classified as {:?}, routing to {:?}", failure_type, agent);

        // Create GitHub issue if configured
        if self.config.create_issues {
            if let Err(e) = self.create_github_issue(&failure, &failure_type, &logs) {
                warn!("Failed to create GitHub issue: {}", e);
            }
        }

        // Spawn CodeRun for remediation
        let spawner = CodeRunSpawner::new(
            self.remediation_config.clone(),
            &self.config.namespace,
            &failure.repository,
        )?;

        spawner.spawn(agent, &routing_ctx)?;

        Ok(())
    }

    /// Fetch failed job details for a workflow run.
    /// Returns (`job_name`, `job_id`, `job_url`, `actor`)
    #[allow(clippy::type_complexity)]
    fn fetch_failed_job_details(
        failure: &WorkflowFailure,
    ) -> Result<(Option<String>, Option<u64>, Option<String>, Option<String>)> {
        let output = Command::new("gh")
            .args([
                "run",
                "view",
                &failure.run_id.to_string(),
                "--repo",
                &failure.repository,
                "--json",
                "jobs,actor",
            ])
            .output()
            .context("Failed to execute gh run view")?;

        if !output.status.success() {
            return Ok((None, None, None, None));
        }

        let data: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);

        // Extract actor
        let actor = data["actor"]
            .get("login")
            .and_then(|v| v.as_str())
            .map(String::from);

        let jobs = data["jobs"].as_array();

        if let Some(jobs) = jobs {
            // Find the first failed job
            for job in jobs {
                if job["conclusion"].as_str() == Some("failure") {
                    let name = job["name"].as_str().map(String::from);
                    let id = job["databaseId"].as_u64();
                    let url = job["url"].as_str().map(String::from);
                    return Ok((name, id, url, actor));
                }
            }
        }

        Ok((None, None, None, actor))
    }

    /// Fetch workflow logs.
    fn fetch_workflow_logs(failure: &WorkflowFailure) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "run",
                "view",
                &failure.run_id.to_string(),
                "--repo",
                &failure.repository,
                "--log-failed",
            ])
            .output()
            .context("Failed to fetch workflow logs")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Try fetching all logs if --log-failed doesn't work
            let output = Command::new("gh")
                .args([
                    "run",
                    "view",
                    &failure.run_id.to_string(),
                    "--repo",
                    &failure.repository,
                    "--log",
                ])
                .output()
                .context("Failed to fetch workflow logs")?;

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }

    /// Create a GitHub issue for the failure.
    fn create_github_issue(
        &self,
        failure: &WorkflowFailure,
        failure_type: &ci::types::CiFailureType,
        logs: &str,
    ) -> Result<String> {
        let title = format!(
            "[CI Failure] {} - {} ({})",
            failure.workflow_name,
            failure.branch,
            failure.head_sha.chars().take(7).collect::<String>()
        );

        let body = Self::generate_issue_body(failure, failure_type, logs);

        let labels = self.config.issue_labels.join(",");

        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                &failure.repository,
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                &labels,
            ])
            .output()
            .context("Failed to create GitHub issue")?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!("Created GitHub issue: {}", url);
            Ok(url)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh issue create failed: {stderr}");
        }
    }

    /// Generate issue body with failure details.
    fn generate_issue_body(
        failure: &WorkflowFailure,
        failure_type: &ci::types::CiFailureType,
        logs: &str,
    ) -> String {
        let log_excerpt = if logs.len() > 3000 {
            format!("{}...\n\n(truncated)", &logs[logs.len() - 3000..])
        } else {
            logs.to_string()
        };

        format!(
            r"## CI Failure Detected

| Field | Value |
|-------|-------|
| **Workflow** | {} |
| **Job** | {} |
| **Branch** | {} |
| **Commit** | {} |
| **Actor** | {} |
| **Run URL** | {} |
| **Failure Type** | {:?} |
| **Detected At** | {} |

### Log Excerpt

<details>
<summary>Click to expand logs</summary>

```
{}
```

</details>

### Remediation

- [ ] A CodeRun has been spawned to investigate and fix this issue
- [ ] Review the proposed changes when the PR is created
- [ ] Merge or close based on the fix quality

---
*This issue was automatically created by Healer CI Sensor*
",
            failure.workflow_name,
            failure.job_name.as_deref().unwrap_or("N/A"),
            failure.branch,
            failure.head_sha,
            failure.actor,
            failure.html_url,
            failure_type,
            failure.detected_at.format("%Y-%m-%d %H:%M:%S UTC"),
            log_excerpt
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_config_default() {
        let config = SensorConfig::default();
        assert_eq!(config.poll_interval_secs, 300);
        assert_eq!(config.lookback_mins, 60);
        assert!(config.create_issues);
        assert_eq!(config.repositories, vec!["5dlabs/cto"]);
    }

    #[test]
    fn test_workflow_failure_to_ci_failure() {
        let failure = WorkflowFailure {
            run_id: 12345,
            workflow_name: "CI".to_string(),
            job_name: Some("build".to_string()),
            job_id: Some(67890),
            branch: "main".to_string(),
            head_sha: "abc123".to_string(), // pragma: allowlist secret
            commit_message: "Test commit".to_string(),
            repository: "5dlabs/cto".to_string(),
            html_url: "https://github.com/5dlabs/cto/actions/runs/12345".to_string(),
            job_url: Some(
                "https://github.com/5dlabs/cto/actions/runs/12345/jobs/67890".to_string(),
            ),
            actor: "testuser".to_string(),
            run_started_at: Utc::now(),
            detected_at: Utc::now(),
            conclusion: "failure".to_string(),
        };

        let ci_failure = failure.to_ci_failure();
        assert_eq!(ci_failure.workflow_run_id, 12345);
        assert_eq!(ci_failure.workflow_name, "CI");
        assert_eq!(ci_failure.branch, "main");
    }

    #[test]
    fn test_issue_body_generation() {
        let failure = WorkflowFailure {
            run_id: 12345,
            workflow_name: "CI".to_string(),
            job_name: Some("build".to_string()),
            job_id: Some(67890),
            branch: "main".to_string(),
            head_sha: "abc123def456".to_string(), // pragma: allowlist secret
            commit_message: "Test commit".to_string(),
            repository: "5dlabs/cto".to_string(),
            html_url: "https://github.com/5dlabs/cto/actions/runs/12345".to_string(),
            job_url: None,
            actor: "testuser".to_string(),
            run_started_at: Utc::now(),
            detected_at: Utc::now(),
            conclusion: "failure".to_string(),
        };

        let body = GitHubActionsSensor::generate_issue_body(
            &failure,
            &ci::types::CiFailureType::RustClippy,
            "error: test error\n",
        );

        assert!(body.contains("CI Failure Detected"));
        assert!(body.contains("CI"));
        assert!(body.contains("build"));
        assert!(body.contains("main"));
        assert!(body.contains("testuser"));
    }
}
