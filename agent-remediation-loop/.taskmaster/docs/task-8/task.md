# Task 8: Implement Escalation and Termination Logic

## Overview
Build a comprehensive escalation system for maximum iterations and implement various termination conditions. This system provides automated escalation when remediation limits are reached, handles timeout scenarios, detects success criteria, and manages manual intervention requests with graceful termination procedures.

## Technical Context
The Agent Remediation Loop requires intelligent termination logic to prevent infinite loops, escalate complex issues, and recognize successful completion. This system acts as the safety net and success detector for the entire remediation process, ensuring resources aren't wasted and human intervention occurs when needed.

### Escalation Triggers
The system monitors multiple conditions that require escalation:
- **Iteration Limits**: Maximum 10 remediation cycles
- **Timeout Protection**: 4-hour maximum duration per task
- **Critical Errors**: System failures requiring immediate attention
- **Manual Override**: Human intervention requests
- **Success Detection**: Automatic recognition of completion

## Implementation Guide

### Step 1: Implement Iteration Limit Checking

#### 1.1 Core Iteration Management
```rust
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{error, info, warn};

const MAX_ITERATIONS: u8 = 10;
const ITERATION_WARNING_THRESHOLD: u8 = 7;

#[derive(Debug, Clone)]
pub struct EscalationManager {
    max_iterations: u8,
    timeout_duration: Duration,
    warning_threshold: u8,
    notification_channels: Vec<NotificationChannel>,
    github_client: GitHubClient,
}

impl EscalationManager {
    pub fn new(
        github_client: GitHubClient,
        notification_channels: Vec<NotificationChannel>,
    ) -> Self {
        Self {
            max_iterations: MAX_ITERATIONS,
            timeout_duration: Duration::hours(4),
            warning_threshold: ITERATION_WARNING_THRESHOLD,
            notification_channels,
            github_client,
        }
    }

    pub async fn check_iteration_limit(
        &self,
        state: &RemediationState,
    ) -> Result<IterationStatus, EscalationError> {
        info!("Checking iteration limit for task {}", state.task_id);
        
        if state.iteration >= self.max_iterations {
            warn!(
                "Task {} has reached maximum iterations: {}/{}",
                state.task_id, state.iteration, self.max_iterations
            );
            
            self.escalate_max_iterations(state).await?;
            return Ok(IterationStatus::MaxReached);
        }

        if state.iteration >= self.warning_threshold {
            warn!(
                "Task {} approaching maximum iterations: {}/{}",
                state.task_id, state.iteration, self.max_iterations
            );
            
            self.send_warning_notification(state).await?;
            return Ok(IterationStatus::Warning);
        }

        info!(
            "Task {} iteration check passed: {}/{}",
            state.task_id, state.iteration, self.max_iterations
        );
        
        Ok(IterationStatus::Normal)
    }

    async fn escalate_max_iterations(
        &self,
        state: &RemediationState,
    ) -> Result<(), EscalationError> {
        let escalation_data = EscalationData {
            task_id: state.task_id.clone(),
            pr_number: state.pr_number,
            reason: EscalationReason::MaxIterations,
            current_iteration: state.iteration,
            max_iterations: self.max_iterations,
            duration: Utc::now() - state.start_time,
            feedback_summary: self.summarize_feedback(&state.feedback_history),
        };

        // Post escalation comment to PR
        self.post_escalation_comment(&escalation_data).await?;
        
        // Send notifications
        self.send_escalation_notifications(&escalation_data).await?;
        
        info!("Max iterations escalation completed for task {}", state.task_id);
        Ok(())
    }

    async fn send_warning_notification(
        &self,
        state: &RemediationState,
    ) -> Result<(), EscalationError> {
        let warning_data = WarningData {
            task_id: state.task_id.clone(),
            pr_number: state.pr_number,
            current_iteration: state.iteration,
            max_iterations: self.max_iterations,
            remaining_iterations: self.max_iterations - state.iteration,
        };

        for channel in &self.notification_channels {
            if let Err(e) = channel.send_warning(&warning_data).await {
                error!("Failed to send warning notification: {}", e);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IterationStatus {
    Normal,
    Warning,
    MaxReached,
}

#[derive(Debug, Clone)]
pub struct WarningData {
    pub task_id: String,
    pub pr_number: u32,
    pub current_iteration: u8,
    pub max_iterations: u8,
    pub remaining_iterations: u8,
}
```

#### 1.2 Iteration History Tracking
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationHistory {
    pub iteration_number: u8,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: IterationOutcome,
    pub feedback_received: bool,
    pub changes_made: u32,
    pub error_count: u32,
    pub duration_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IterationOutcome {
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Escalated,
}

impl EscalationManager {
    pub async fn track_iteration_start(
        &self,
        state: &mut RemediationState,
        iteration: u8,
    ) -> Result<(), EscalationError> {
        let history_entry = IterationHistory {
            iteration_number: iteration,
            started_at: Utc::now(),
            completed_at: None,
            status: IterationOutcome::InProgress,
            feedback_received: false,
            changes_made: 0,
            error_count: 0,
            duration_minutes: None,
        };

        state.iteration_history.push(history_entry);
        info!("Started tracking iteration {} for task {}", iteration, state.task_id);
        
        Ok(())
    }

    pub async fn track_iteration_completion(
        &self,
        state: &mut RemediationState,
        iteration: u8,
        outcome: IterationOutcome,
        changes_made: u32,
    ) -> Result<(), EscalationError> {
        if let Some(history) = state.iteration_history
            .iter_mut()
            .find(|h| h.iteration_number == iteration)
        {
            let now = Utc::now();
            history.completed_at = Some(now);
            history.status = outcome;
            history.changes_made = changes_made;
            history.duration_minutes = Some((now - history.started_at).num_minutes());
            
            info!(
                "Completed iteration {} for task {} in {} minutes",
                iteration, state.task_id, history.duration_minutes.unwrap_or(0)
            );
        }

        Ok(())
    }
}
```

### Step 2: Implement Timeout Detection System

#### 2.1 Timeout Monitoring
```rust
impl EscalationManager {
    pub async fn check_timeout(
        &self,
        state: &RemediationState,
    ) -> Result<TimeoutStatus, EscalationError> {
        let elapsed = Utc::now() - state.start_time;
        
        if elapsed > self.timeout_duration {
            warn!(
                "Task {} has exceeded timeout: {} hours",
                state.task_id,
                elapsed.num_hours()
            );
            
            self.escalate_timeout(state, elapsed).await?;
            return Ok(TimeoutStatus::Exceeded);
        }

        // Warning at 75% of timeout duration
        let warning_threshold = self.timeout_duration * 3 / 4;
        if elapsed > warning_threshold {
            warn!(
                "Task {} approaching timeout: {} hours remaining",
                state.task_id,
                (self.timeout_duration - elapsed).num_hours()
            );
            
            self.send_timeout_warning(state, elapsed).await?;
            return Ok(TimeoutStatus::Warning);
        }

        info!(
            "Task {} timeout check passed: {} hours elapsed",
            state.task_id,
            elapsed.num_hours()
        );

        Ok(TimeoutStatus::Normal)
    }

    async fn escalate_timeout(
        &self,
        state: &RemediationState,
        elapsed_duration: Duration,
    ) -> Result<(), EscalationError> {
        let escalation_data = EscalationData {
            task_id: state.task_id.clone(),
            pr_number: state.pr_number,
            reason: EscalationReason::Timeout,
            current_iteration: state.iteration,
            max_iterations: self.max_iterations,
            duration: elapsed_duration,
            feedback_summary: self.summarize_feedback(&state.feedback_history),
        };

        self.post_escalation_comment(&escalation_data).await?;
        self.send_escalation_notifications(&escalation_data).await?;
        
        info!("Timeout escalation completed for task {}", state.task_id);
        Ok(())
    }

    async fn send_timeout_warning(
        &self,
        state: &RemediationState,
        elapsed: Duration,
    ) -> Result<(), EscalationError> {
        let remaining = self.timeout_duration - elapsed;
        
        for channel in &self.notification_channels {
            let warning = TimeoutWarning {
                task_id: state.task_id.clone(),
                pr_number: state.pr_number,
                elapsed_hours: elapsed.num_hours(),
                remaining_hours: remaining.num_hours(),
                current_iteration: state.iteration,
            };
            
            if let Err(e) = channel.send_timeout_warning(&warning).await {
                error!("Failed to send timeout warning: {}", e);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeoutStatus {
    Normal,
    Warning,
    Exceeded,
}

#[derive(Debug, Clone)]
pub struct TimeoutWarning {
    pub task_id: String,
    pub pr_number: u32,
    pub elapsed_hours: i64,
    pub remaining_hours: i64,
    pub current_iteration: u8,
}
```

### Step 3: Build Critical Error Detection

#### 3.1 Error Classification System
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriticalErrorType {
    SystemFailure,
    AuthenticationError,
    RateLimitExceeded,
    RepositoryAccessDenied,
    InfrastructureFailure,
    DataCorruption,
    InvalidConfiguration,
    ExternalServiceFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalError {
    pub error_type: CriticalErrorType,
    pub message: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub context: ErrorContext,
    pub retry_attempted: bool,
    pub escalation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub task_id: String,
    pub pr_number: u32,
    pub iteration: u8,
    pub operation: String,
    pub component: String,
}

impl EscalationManager {
    pub async fn handle_error(
        &self,
        error: &dyn std::error::Error,
        context: ErrorContext,
    ) -> Result<ErrorHandlingAction, EscalationError> {
        let critical_error = self.classify_error(error, context)?;
        
        if critical_error.escalation_required {
            warn!(
                "Critical error detected for task {}: {:?}",
                critical_error.context.task_id, critical_error.error_type
            );
            
            self.escalate_critical_error(&critical_error).await?;
            return Ok(ErrorHandlingAction::Escalate);
        }

        if self.should_retry(&critical_error) {
            info!(
                "Retryable error for task {}: {}",
                critical_error.context.task_id, critical_error.message
            );
            return Ok(ErrorHandlingAction::Retry);
        }

        Ok(ErrorHandlingAction::Continue)
    }

    fn classify_error(
        &self,
        error: &dyn std::error::Error,
        context: ErrorContext,
    ) -> Result<CriticalError, EscalationError> {
        let error_message = error.to_string().to_lowercase();
        
        let (error_type, escalation_required) = match error_message {
            msg if msg.contains("authentication") || msg.contains("unauthorized") => {
                (CriticalErrorType::AuthenticationError, true)
            }
            msg if msg.contains("rate limit") || msg.contains("rate-limit") => {
                (CriticalErrorType::RateLimitExceeded, false)
            }
            msg if msg.contains("permission denied") || msg.contains("access denied") => {
                (CriticalErrorType::RepositoryAccessDenied, true)
            }
            msg if msg.contains("kubernetes") || msg.contains("cluster") => {
                (CriticalErrorType::InfrastructureFailure, true)
            }
            msg if msg.contains("corrupt") || msg.contains("invalid data") => {
                (CriticalErrorType::DataCorruption, true)
            }
            msg if msg.contains("config") || msg.contains("configuration") => {
                (CriticalErrorType::InvalidConfiguration, true)
            }
            msg if msg.contains("timeout") || msg.contains("connection refused") => {
                (CriticalErrorType::ExternalServiceFailure, false)
            }
            _ => (CriticalErrorType::SystemFailure, true)
        };

        Ok(CriticalError {
            error_type,
            message: error.to_string(),
            details: serde_json::json!({
                "error_chain": format!("{:?}", error),
                "component": context.component,
                "operation": context.operation,
            }),
            timestamp: Utc::now(),
            context,
            retry_attempted: false,
            escalation_required,
        })
    }

    async fn escalate_critical_error(
        &self,
        error: &CriticalError,
    ) -> Result<(), EscalationError> {
        let escalation_data = EscalationData {
            task_id: error.context.task_id.clone(),
            pr_number: error.context.pr_number,
            reason: EscalationReason::CriticalError(error.error_type.clone()),
            current_iteration: error.context.iteration,
            max_iterations: self.max_iterations,
            duration: Utc::now() - error.timestamp, // This should be calculated properly
            feedback_summary: String::new(), // Would need to fetch from state
        };

        self.post_error_escalation_comment(error, &escalation_data).await?;
        self.send_escalation_notifications(&escalation_data).await?;
        
        info!(
            "Critical error escalation completed for task {} - {:?}",
            error.context.task_id, error.error_type
        );
        
        Ok(())
    }

    fn should_retry(&self, error: &CriticalError) -> bool {
        matches!(
            error.error_type,
            CriticalErrorType::RateLimitExceeded
                | CriticalErrorType::ExternalServiceFailure
        ) && !error.retry_attempted
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandlingAction {
    Continue,
    Retry,
    Escalate,
}
```

### Step 4: Implement Manual Override Detection

#### 4.1 Override Detection System
```rust
impl EscalationManager {
    pub async fn check_manual_override(
        &self,
        pr_number: u32,
    ) -> Result<OverrideStatus, EscalationError> {
        let labels = self.github_client.get_pr_labels(pr_number).await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;

        let overrides = self.detect_override_labels(&labels);
        
        if !overrides.is_empty() {
            info!(
                "Manual override detected on PR {}: {:?}",
                pr_number, overrides
            );
            
            return Ok(OverrideStatus::Active(overrides));
        }

        // Check for manual intervention comments
        let comments = self.github_client.get_pr_comments(pr_number).await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;

        let intervention_requests = self.detect_intervention_comments(&comments);
        
        if !intervention_requests.is_empty() {
            info!(
                "Manual intervention requested on PR {}: {} requests",
                pr_number, intervention_requests.len()
            );
            
            return Ok(OverrideStatus::InterventionRequested(intervention_requests));
        }

        Ok(OverrideStatus::None)
    }

    fn detect_override_labels(&self, labels: &[String]) -> Vec<OverrideType> {
        let mut overrides = Vec::new();
        
        for label in labels {
            match label.as_str() {
                "skip-automation" => overrides.push(OverrideType::SkipAutomation),
                "manual-review-required" => overrides.push(OverrideType::ManualReviewRequired),
                "pause-remediation" => overrides.push(OverrideType::PauseRemediation),
                "emergency-stop" => overrides.push(OverrideType::EmergencyStop),
                _ => {}
            }
        }
        
        overrides
    }

    fn detect_intervention_comments(
        &self,
        comments: &[github::Comment],
    ) -> Vec<InterventionRequest> {
        let mut requests = Vec::new();
        
        for comment in comments {
            // Look for specific intervention patterns
            let body = comment.body.to_lowercase();
            
            if body.contains("@platform-team") && body.contains("help") {
                requests.push(InterventionRequest {
                    comment_id: comment.id,
                    author: comment.user.login.clone(),
                    timestamp: comment.created_at,
                    request_type: InterventionType::PlatformTeamHelp,
                    message: comment.body.clone(),
                });
            }
            
            if body.contains("manual intervention") || body.contains("human review") {
                requests.push(InterventionRequest {
                    comment_id: comment.id,
                    author: comment.user.login.clone(),
                    timestamp: comment.created_at,
                    request_type: InterventionType::ManualReview,
                    message: comment.body.clone(),
                });
            }
        }
        
        requests
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverrideStatus {
    None,
    Active(Vec<OverrideType>),
    InterventionRequested(Vec<InterventionRequest>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverrideType {
    SkipAutomation,
    ManualReviewRequired,
    PauseRemediation,
    EmergencyStop,
}

#[derive(Debug, Clone)]
pub struct InterventionRequest {
    pub comment_id: u64,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub request_type: InterventionType,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterventionType {
    PlatformTeamHelp,
    ManualReview,
    EmergencyStop,
    GeneralSupport,
}
```

### Step 5: Create Escalation Notification System

#### 5.1 Multi-Channel Notification System
```rust
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    GitHub,
    Slack,
    Email,
    PagerDuty,
}

#[derive(Debug, Clone)]
pub struct EscalationData {
    pub task_id: String,
    pub pr_number: u32,
    pub reason: EscalationReason,
    pub current_iteration: u8,
    pub max_iterations: u8,
    pub duration: Duration,
    pub feedback_summary: String,
}

#[derive(Debug, Clone)]
pub enum EscalationReason {
    MaxIterations,
    Timeout,
    CriticalError(CriticalErrorType),
    ManualRequest,
}

impl EscalationManager {
    pub async fn post_escalation_comment(
        &self,
        escalation_data: &EscalationData,
    ) -> Result<(), EscalationError> {
        let comment_body = self.format_escalation_comment(escalation_data);
        
        self.github_client
            .post_pr_comment(escalation_data.pr_number, &comment_body)
            .await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;
        
        info!(
            "Posted escalation comment for task {} on PR {}",
            escalation_data.task_id, escalation_data.pr_number
        );
        
        Ok(())
    }

    fn format_escalation_comment(&self, data: &EscalationData) -> String {
        let reason_text = match &data.reason {
            EscalationReason::MaxIterations => {
                format!(
                    "**Maximum iterations reached** ({}/{})",
                    data.current_iteration, data.max_iterations
                )
            }
            EscalationReason::Timeout => {
                format!("**Timeout exceeded** ({} hours)", data.duration.num_hours())
            }
            EscalationReason::CriticalError(error_type) => {
                format!("**Critical error encountered**: {:?}", error_type)
            }
            EscalationReason::ManualRequest => "**Manual escalation requested**".to_string(),
        };

        format!(
            r#"## 🚨 Remediation Escalation

{reason}

**Task ID**: {task_id}  
**Duration**: {duration} hours  
**Iterations Attempted**: {current_iteration}  

### Summary of Attempts
{feedback_summary}

### Next Steps
- [ ] Manual review by @platform-team or @cto
- [ ] Assess if requirements need clarification  
- [ ] Consider if task complexity exceeds automation capabilities
- [ ] Manual implementation or guidance may be required

**Automated remediation has been suspended for this task.**

*Escalated at: {timestamp}*

cc: @platform-team @cto"#,
            reason = reason_text,
            task_id = data.task_id,
            duration = data.duration.num_hours(),
            current_iteration = data.current_iteration,
            feedback_summary = data.feedback_summary,
            timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        )
    }

    async fn post_error_escalation_comment(
        &self,
        error: &CriticalError,
        escalation_data: &EscalationData,
    ) -> Result<(), EscalationError> {
        let comment_body = format!(
            r#"## 🚨 Critical Error Escalation

**Error Type**: {:?}  
**Task ID**: {}  
**Component**: {}  
**Operation**: {}

### Error Details
```
{}
```

### Context
- **Iteration**: {}
- **Timestamp**: {}
- **Retry Attempted**: {}

### Immediate Actions Required
- [ ] Investigate error cause
- [ ] Check system health and connectivity
- [ ] Verify configuration and permissions
- [ ] Manual intervention may be required

cc: @platform-team @cto"#,
            error.error_type,
            error.context.task_id,
            error.context.component,
            error.context.operation,
            error.message,
            error.context.iteration,
            error.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            error.retry_attempted,
        );

        self.github_client
            .post_pr_comment(escalation_data.pr_number, &comment_body)
            .await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;

        Ok(())
    }

    pub async fn send_escalation_notifications(
        &self,
        escalation_data: &EscalationData,
    ) -> Result<(), EscalationError> {
        for channel in &self.notification_channels {
            if let Err(e) = self.send_notification(channel, escalation_data).await {
                error!(
                    "Failed to send escalation notification via {:?}: {}",
                    channel, e
                );
            }
        }
        Ok(())
    }

    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        data: &EscalationData,
    ) -> Result<(), EscalationError> {
        match channel {
            NotificationChannel::GitHub => {
                // GitHub notification already handled by comment posting
                Ok(())
            }
            NotificationChannel::Slack => {
                self.send_slack_notification(data).await
            }
            NotificationChannel::Email => {
                self.send_email_notification(data).await
            }
            NotificationChannel::PagerDuty => {
                self.send_pagerduty_notification(data).await
            }
        }
    }

    async fn send_slack_notification(
        &self,
        data: &EscalationData,
    ) -> Result<(), EscalationError> {
        // Implementation would integrate with Slack API
        info!("Sending Slack notification for task {}", data.task_id);
        Ok(())
    }

    async fn send_email_notification(
        &self,
        data: &EscalationData,
    ) -> Result<(), EscalationError> {
        // Implementation would integrate with email service
        info!("Sending email notification for task {}", data.task_id);
        Ok(())
    }

    async fn send_pagerduty_notification(
        &self,
        data: &EscalationData,
    ) -> Result<(), EscalationError> {
        // Implementation would integrate with PagerDuty API
        info!("Sending PagerDuty notification for task {}", data.task_id);
        Ok(())
    }

    fn summarize_feedback(&self, feedback_history: &[FeedbackEntry]) -> String {
        if feedback_history.is_empty() {
            return "No feedback received during remediation attempts.".to_string();
        }

        let mut summary = String::new();
        for (idx, feedback) in feedback_history.iter().enumerate() {
            summary.push_str(&format!(
                "**Iteration {}**: {} ({})\n",
                idx + 1,
                feedback.description.chars().take(100).collect::<String>(),
                feedback.severity
            ));
        }
        
        summary
    }
}
```

### Step 6: Implement PR Comment Posting for Escalations

#### 6.1 GitHub Integration
```rust
use serde_json::json;

pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub async fn post_pr_comment(
        &self,
        pr_number: u32,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/repos/{}/issues/{}/comments",
            self.base_url,
            std::env::var("GITHUB_REPOSITORY")?,
            pr_number
        );

        let payload = json!({
            "body": body
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "agent-remediation-loop")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error: {} - {}",
                response.status(),
                response.text().await?
            )
            .into());
        }

        Ok(())
    }

    pub async fn get_pr_labels(&self, pr_number: u32) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/repos/{}/pulls/{}",
            self.base_url,
            std::env::var("GITHUB_REPOSITORY")?,
            pr_number
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "agent-remediation-loop")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()).into());
        }

        let pr_data: serde_json::Value = response.json().await?;
        let labels = pr_data["labels"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|label| label["name"].as_str().map(String::from))
            .collect();

        Ok(labels)
    }

    pub async fn get_pr_comments(&self, pr_number: u32) -> Result<Vec<github::Comment>, Box<dyn std::error::Error>> {
        // Implementation would fetch and parse PR comments
        Ok(Vec::new())
    }
}

pub mod github {
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone)]
    pub struct Comment {
        pub id: u64,
        pub body: String,
        pub user: User,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone)]
    pub struct User {
        pub login: String,
    }
}
```

### Step 7: Implement Success Criteria Detection

#### 7.1 Success Detection System
```rust
impl EscalationManager {
    pub async fn check_success_criteria(
        &self,
        state: &RemediationState,
        pr_number: u32,
    ) -> Result<SuccessStatus, EscalationError> {
        info!("Checking success criteria for task {}", state.task_id);

        // Check if all feedback items are resolved
        let unresolved_feedback = state
            .feedback_history
            .iter()
            .filter(|f| !f.resolved)
            .count();

        if unresolved_feedback > 0 {
            return Ok(SuccessStatus::PendingFeedback(unresolved_feedback));
        }

        // Check PR approval status
        let approval_status = self.check_pr_approvals(pr_number).await?;
        if !approval_status.approved {
            return Ok(SuccessStatus::PendingApproval);
        }

        // Check CI status
        let ci_status = self.check_ci_status(pr_number).await?;
        if !ci_status.all_passed {
            return Ok(SuccessStatus::PendingCI(ci_status.failing_checks));
        }

        // Check for explicit success signals
        let success_signals = self.detect_success_signals(pr_number).await?;
        if !success_signals.is_empty() {
            info!("Success criteria met for task {}", state.task_id);
            return Ok(SuccessStatus::Success(success_signals));
        }

        // If no explicit success but all checks pass, consider successful
        if approval_status.approved && ci_status.all_passed && unresolved_feedback == 0 {
            info!("Implicit success criteria met for task {}", state.task_id);
            return Ok(SuccessStatus::ImplicitSuccess);
        }

        Ok(SuccessStatus::InProgress)
    }

    async fn check_pr_approvals(&self, pr_number: u32) -> Result<ApprovalStatus, EscalationError> {
        // Implementation would check GitHub PR reviews
        Ok(ApprovalStatus {
            approved: false,
            approvers: Vec::new(),
            required_approvals: 1,
            current_approvals: 0,
        })
    }

    async fn check_ci_status(&self, pr_number: u32) -> Result<CIStatus, EscalationError> {
        // Implementation would check GitHub status checks
        Ok(CIStatus {
            all_passed: false,
            failing_checks: Vec::new(),
            pending_checks: Vec::new(),
            required_checks_passed: false,
        })
    }

    async fn detect_success_signals(&self, pr_number: u32) -> Result<Vec<SuccessSignal>, EscalationError> {
        let comments = self.github_client.get_pr_comments(pr_number).await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;

        let mut signals = Vec::new();

        for comment in comments {
            let body = comment.body.to_lowercase();
            
            // Look for explicit approval from Tess
            if comment.user.login.contains("tess") && 
               (body.contains("approved") || body.contains("lgtm") || body.contains("looks good")) {
                signals.push(SuccessSignal::TessApproval {
                    comment_id: comment.id,
                    timestamp: comment.created_at,
                });
            }

            // Look for "all issues resolved" type comments
            if body.contains("all issues") && body.contains("resolved") {
                signals.push(SuccessSignal::IssuesResolved {
                    comment_id: comment.id,
                    author: comment.user.login,
                });
            }

            // Look for explicit completion markers
            if body.contains("task complete") || body.contains("ready to merge") {
                signals.push(SuccessSignal::ExplicitCompletion {
                    comment_id: comment.id,
                    author: comment.user.login,
                });
            }
        }

        Ok(signals)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuccessStatus {
    InProgress,
    PendingFeedback(usize),
    PendingApproval,
    PendingCI(Vec<String>),
    Success(Vec<SuccessSignal>),
    ImplicitSuccess,
}

#[derive(Debug, Clone)]
pub struct ApprovalStatus {
    pub approved: bool,
    pub approvers: Vec<String>,
    pub required_approvals: u32,
    pub current_approvals: u32,
}

#[derive(Debug, Clone)]
pub struct CIStatus {
    pub all_passed: bool,
    pub failing_checks: Vec<String>,
    pub pending_checks: Vec<String>,
    pub required_checks_passed: bool,
}

#[derive(Debug, Clone)]
pub enum SuccessSignal {
    TessApproval {
        comment_id: u64,
        timestamp: DateTime<Utc>,
    },
    IssuesResolved {
        comment_id: u64,
        author: String,
    },
    ExplicitCompletion {
        comment_id: u64,
        author: String,
    },
}
```

### Step 8: Build Graceful Termination Procedures

#### 8.1 Termination Manager
```rust
impl EscalationManager {
    pub async fn terminate_remediation(
        &self,
        state: &mut RemediationState,
        reason: TerminationReason,
        pr_number: u32,
    ) -> Result<TerminationResult, EscalationError> {
        info!("Initiating graceful termination for task {}", state.task_id);

        let termination_data = TerminationData {
            task_id: state.task_id.clone(),
            pr_number,
            reason: reason.clone(),
            final_iteration: state.iteration,
            duration: Utc::now() - state.start_time,
            feedback_count: state.feedback_history.len(),
            timestamp: Utc::now(),
        };

        // Update state to mark termination
        state.status = match reason {
            TerminationReason::Success => RemediationStatus::Completed,
            TerminationReason::MaxIterations | TerminationReason::Timeout => RemediationStatus::MaxIterationsReached,
            TerminationReason::CriticalError => RemediationStatus::Failed,
            TerminationReason::ManualIntervention => RemediationStatus::Failed,
        };

        state.last_update = Utc::now();
        state.error_messages.push(format!("Terminated: {:?}", reason));

        // Perform cleanup operations
        let cleanup_result = self.perform_cleanup(&termination_data).await?;

        // Post final status comment
        self.post_termination_comment(&termination_data).await?;

        // Update PR labels
        self.update_final_labels(pr_number, &reason).await?;

        // Record metrics
        self.record_termination_metrics(&termination_data).await?;

        // Send final notifications
        self.send_termination_notifications(&termination_data).await?;

        info!(
            "Graceful termination completed for task {} - {:?}",
            state.task_id, reason
        );

        Ok(TerminationResult {
            task_id: state.task_id.clone(),
            reason,
            cleanup_successful: cleanup_result.successful,
            cleanup_errors: cleanup_result.errors,
            final_state: state.status.clone(),
        })
    }

    async fn perform_cleanup(
        &self,
        termination_data: &TerminationData,
    ) -> Result<CleanupResult, EscalationError> {
        let mut cleanup_result = CleanupResult {
            successful: true,
            errors: Vec::new(),
        };

        // Clean up any running CodeRun resources
        if let Err(e) = self.cleanup_coderun_resources(&termination_data.task_id).await {
            cleanup_result.successful = false;
            cleanup_result.errors.push(format!("CodeRun cleanup failed: {}", e));
        }

        // Archive state data
        if let Err(e) = self.archive_state_data(termination_data).await {
            cleanup_result.errors.push(format!("State archival failed: {}", e));
            // Not considered critical failure
        }

        // Clean up temporary resources
        if let Err(e) = self.cleanup_temporary_resources(&termination_data.task_id).await {
            cleanup_result.errors.push(format!("Temporary resource cleanup failed: {}", e));
        }

        Ok(cleanup_result)
    }

    async fn cleanup_coderun_resources(&self, task_id: &str) -> Result<(), EscalationError> {
        // Implementation would clean up any running CodeRun resources
        info!("Cleaning up CodeRun resources for task {}", task_id);
        Ok(())
    }

    async fn archive_state_data(&self, termination_data: &TerminationData) -> Result<(), EscalationError> {
        // Implementation would archive state data for historical analysis
        info!("Archiving state data for task {}", termination_data.task_id);
        Ok(())
    }

    async fn cleanup_temporary_resources(&self, task_id: &str) -> Result<(), EscalationError> {
        // Implementation would clean up temporary files, caches, etc.
        info!("Cleaning up temporary resources for task {}", task_id);
        Ok(())
    }

    async fn post_termination_comment(&self, data: &TerminationData) -> Result<(), EscalationError> {
        let comment_body = self.format_termination_comment(data);
        
        self.github_client
            .post_pr_comment(data.pr_number, &comment_body)
            .await
            .map_err(|e| EscalationError::GitHubApi(e.to_string()))?;

        Ok(())
    }

    fn format_termination_comment(&self, data: &TerminationData) -> String {
        let status_emoji = match data.reason {
            TerminationReason::Success => "✅",
            TerminationReason::MaxIterations => "🚨",
            TerminationReason::Timeout => "⏰",
            TerminationReason::CriticalError => "💥",
            TerminationReason::ManualIntervention => "👤",
        };

        let reason_text = match &data.reason {
            TerminationReason::Success => "**Remediation completed successfully**",
            TerminationReason::MaxIterations => "**Maximum iterations reached**",
            TerminationReason::Timeout => "**Timeout exceeded**",
            TerminationReason::CriticalError => "**Critical error encountered**",
            TerminationReason::ManualIntervention => "**Manual intervention requested**",
        };

        format!(
            r#"{emoji} ## Remediation Terminated

{reason}

**Task ID**: {task_id}  
**Final Iteration**: {final_iteration}  
**Total Duration**: {duration} hours  
**Feedback Items**: {feedback_count}

### Final Status
Automated remediation has been terminated. State has been preserved for analysis.

*Terminated at: {timestamp}*"#,
            emoji = status_emoji,
            reason = reason_text,
            task_id = data.task_id,
            final_iteration = data.final_iteration,
            duration = data.duration.num_hours(),
            feedback_count = data.feedback_count,
            timestamp = data.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        )
    }

    async fn update_final_labels(&self, pr_number: u32, reason: &TerminationReason) -> Result<(), EscalationError> {
        // Implementation would update PR labels based on termination reason
        info!("Updating final labels for PR {} - {:?}", pr_number, reason);
        Ok(())
    }

    async fn record_termination_metrics(&self, data: &TerminationData) -> Result<(), EscalationError> {
        // Implementation would record metrics for monitoring and analysis
        info!("Recording termination metrics for task {}", data.task_id);
        Ok(())
    }

    async fn send_termination_notifications(&self, data: &TerminationData) -> Result<(), EscalationError> {
        // Implementation would send notifications about termination
        info!("Sending termination notifications for task {}", data.task_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TerminationReason {
    Success,
    MaxIterations,
    Timeout,
    CriticalError,
    ManualIntervention,
}

#[derive(Debug, Clone)]
pub struct TerminationData {
    pub task_id: String,
    pub pr_number: u32,
    pub reason: TerminationReason,
    pub final_iteration: u8,
    pub duration: Duration,
    pub feedback_count: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TerminationResult {
    pub task_id: String,
    pub reason: TerminationReason,
    pub cleanup_successful: bool,
    pub cleanup_errors: Vec<String>,
    pub final_state: RemediationStatus,
}

#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub successful: bool,
    pub errors: Vec<String>,
}
```

### Step 9: Create Comprehensive Termination Path Testing

#### 9.1 Test Framework
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_iteration_limit_enforcement() {
        let escalation_manager = create_test_escalation_manager().await;
        
        let mut state = create_test_remediation_state();
        state.iteration = MAX_ITERATIONS;
        
        let result = escalation_manager.check_iteration_limit(&state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IterationStatus::MaxReached);
    }

    #[tokio::test]
    async fn test_iteration_warning_threshold() {
        let escalation_manager = create_test_escalation_manager().await;
        
        let mut state = create_test_remediation_state();
        state.iteration = ITERATION_WARNING_THRESHOLD;
        
        let result = escalation_manager.check_iteration_limit(&state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IterationStatus::Warning);
    }

    #[tokio::test]
    async fn test_timeout_detection() {
        let escalation_manager = create_test_escalation_manager().await;
        
        let mut state = create_test_remediation_state();
        state.start_time = Utc::now() - Duration::hours(5); // Exceed 4-hour timeout
        
        let result = escalation_manager.check_timeout(&state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TimeoutStatus::Exceeded);
    }

    #[tokio::test]
    async fn test_critical_error_classification() {
        let escalation_manager = create_test_escalation_manager().await;
        let context = create_test_error_context();
        
        let auth_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Authentication failed");
        let result = escalation_manager.handle_error(&auth_error, context).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ErrorHandlingAction::Escalate);
    }

    #[tokio::test]
    async fn test_success_criteria_detection() {
        let escalation_manager = create_test_escalation_manager().await;
        
        let mut state = create_test_remediation_state();
        // Mark all feedback as resolved
        for feedback in &mut state.feedback_history {
            feedback.resolved = true;
        }
        
        let result = escalation_manager.check_success_criteria(&state, 123).await;
        assert!(result.is_ok());
        
        // Should be pending approval since we didn't mock the approval status
        match result.unwrap() {
            SuccessStatus::PendingApproval => assert!(true),
            _ => assert!(false, "Expected PendingApproval status"),
        }
    }

    #[tokio::test]
    async fn test_graceful_termination() {
        let escalation_manager = create_test_escalation_manager().await;
        
        let mut state = create_test_remediation_state();
        let result = escalation_manager
            .terminate_remediation(&mut state, TerminationReason::Success, 123)
            .await;
        
        assert!(result.is_ok());
        let termination_result = result.unwrap();
        assert_eq!(termination_result.reason, TerminationReason::Success);
        assert!(termination_result.cleanup_successful);
    }

    // Helper functions
    async fn create_test_escalation_manager() -> EscalationManager {
        let github_client = GitHubClient::new("test-token".to_string());
        let notification_channels = vec![NotificationChannel::GitHub];
        EscalationManager::new(github_client, notification_channels)
    }

    fn create_test_remediation_state() -> RemediationState {
        RemediationState {
            task_id: "test-123".to_string(),
            pr_number: 456,
            iteration: 1,
            status: RemediationStatus::InProgress,
            start_time: Utc::now() - Duration::hours(1),
            last_update: Utc::now(),
            feedback_history: vec![create_test_feedback_entry()],
            iteration_history: Vec::new(),
            error_messages: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    fn create_test_feedback_entry() -> FeedbackEntry {
        FeedbackEntry {
            timestamp: Utc::now(),
            author: "test-user".to_string(),
            severity: FeedbackSeverity::Medium,
            issue_type: IssueType::Bug,
            description: "Test feedback".to_string(),
            resolved: false,
            pr_comment_id: "123".to_string(),
        }
    }

    fn create_test_error_context() -> ErrorContext {
        ErrorContext {
            task_id: "test-123".to_string(),
            pr_number: 456,
            iteration: 1,
            operation: "test_operation".to_string(),
            component: "test_component".to_string(),
        }
    }
}
```

## Error Handling and Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EscalationError {
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    
    #[error("State management error: {0}")]
    StateManagement(String),
    
    #[error("Notification error: {0}")]
    Notification(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Critical system error: {0}")]
    Critical(String),
}

// Re-export types from other modules that this task depends on
pub use crate::state::{RemediationState, RemediationStatus, FeedbackEntry, FeedbackSeverity, IssueType};
```

## Integration Points

### State Management Integration
- Closely integrates with Task 4's state management for iteration tracking
- Updates remediation state during escalation events
- Maintains consistency between escalation decisions and stored state

### Label Management Integration
- Works with Task 7's label orchestration for final label updates
- Updates PR labels based on termination reason
- Coordinates with override detection systems

### Notification Integration
- Multi-channel notification system for escalation events
- Integration with existing monitoring and alerting infrastructure
- Customizable notification templates and channels

## Success Criteria
- Iteration limits enforced with proper escalation at 10 cycles
- Timeout detection prevents runaway processes beyond 4 hours
- Critical error classification triggers appropriate responses
- Manual override detection respects human intervention
- Success criteria detection recognizes completion automatically
- Graceful termination preserves state and cleans up resources
- Multi-channel notifications ensure visibility of escalations
- Comprehensive testing validates all termination scenarios