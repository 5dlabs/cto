//! Cost tracker for recording and querying API calls.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::metrics::{
    AgentComparison, AgentMetrics, AggregatedMetrics, ProjectMetrics, TaskMetrics,
};
use super::models::{
    AgentId, ProjectId, SessionId, TaskId, TrackedCall, TrackingContext, TrackingFilter,
};

/// A thread-safe cost tracker for recording API calls with context.
#[derive(Debug, Clone)]
pub struct CostTracker {
    calls: Arc<RwLock<Vec<TrackedCall>>>,
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CostTracker {
    /// Create a new cost tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            calls: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a new API call.
    pub fn record(&self, call: TrackedCall) {
        if let Ok(mut calls) = self.calls.write() {
            calls.push(call);
        }
    }

    /// Create a builder for recording a new call.
    #[must_use]
    pub fn builder(&self) -> TrackedCallBuilder {
        TrackedCallBuilder::new(self.clone())
    }

    /// Query tracked calls with a filter.
    #[must_use]
    pub fn query(&self, filter: &TrackingFilter) -> Vec<TrackedCall> {
        self.calls
            .read()
            .map(|calls| {
                calls
                    .iter()
                    .filter(|c| filter.matches(c))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all tracked calls.
    #[must_use]
    pub fn all(&self) -> Vec<TrackedCall> {
        self.calls.read().map(|c| c.clone()).unwrap_or_default()
    }

    /// Get aggregated metrics for calls matching a filter.
    #[must_use]
    pub fn metrics(&self, filter: &TrackingFilter) -> AggregatedMetrics {
        let calls = self.query(filter);
        AggregatedMetrics::from_calls(&calls)
    }

    /// Get metrics for a specific task.
    #[must_use]
    pub fn task_metrics(&self, task_id: impl Into<TaskId>) -> TaskMetrics {
        let task_id = task_id.into();
        let filter = TrackingFilter::new().with_task(task_id.clone());
        let calls = self.query(&filter);
        TaskMetrics::from_calls(task_id, &calls)
    }

    /// Get metrics for a specific agent.
    #[must_use]
    pub fn agent_metrics(&self, agent_id: impl Into<AgentId>) -> AgentMetrics {
        let agent_id = agent_id.into();
        let filter = TrackingFilter::new().with_agent(agent_id.clone());
        let calls = self.query(&filter);
        AgentMetrics::from_calls(agent_id, &calls)
    }

    /// Get metrics for a specific project.
    #[must_use]
    pub fn project_metrics(&self, project_id: impl Into<ProjectId>) -> ProjectMetrics {
        let project_id = project_id.into();
        let filter = TrackingFilter::new().with_project(project_id.clone());
        let calls = self.query(&filter);
        ProjectMetrics::from_calls(project_id, &calls)
    }

    /// Compare all agents' efficiency.
    #[must_use]
    pub fn compare_agents(&self) -> AgentComparison {
        let calls = self.all();

        // Group by agent
        let mut by_agent: HashMap<AgentId, Vec<TrackedCall>> = HashMap::new();
        for call in calls {
            if let Some(ref agent_id) = call.context.agent_id {
                by_agent.entry(agent_id.clone()).or_default().push(call);
            }
        }

        let agent_metrics: Vec<AgentMetrics> = by_agent
            .into_iter()
            .map(|(agent_id, calls)| AgentMetrics::from_calls(agent_id, &calls))
            .collect();

        AgentComparison::from_metrics(agent_metrics)
    }

    /// Get all unique task IDs.
    #[must_use]
    pub fn task_ids(&self) -> Vec<TaskId> {
        self.calls
            .read()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|c| c.context.task_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all unique agent IDs.
    #[must_use]
    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.calls
            .read()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|c| c.context.agent_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all unique project IDs.
    #[must_use]
    pub fn project_ids(&self) -> Vec<ProjectId> {
        self.calls
            .read()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|c| c.context.project_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get total cost in USD.
    #[must_use]
    pub fn total_cost(&self) -> f64 {
        self.calls
            .read()
            .map(|calls| calls.iter().map(|c| c.estimated_cost_usd).sum())
            .unwrap_or(0.0)
    }

    /// Clear all tracked calls.
    pub fn clear(&self) {
        if let Ok(mut calls) = self.calls.write() {
            calls.clear();
        }
    }

    /// Get count of tracked calls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.calls.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if tracker is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Export all calls to JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let calls = self.all();
        serde_json::to_string_pretty(&calls)
    }

    /// Import calls from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn import_json(&self, json: &str) -> Result<(), serde_json::Error> {
        let calls: Vec<TrackedCall> = serde_json::from_str(json)?;
        if let Ok(mut existing) = self.calls.write() {
            existing.extend(calls);
        }
        Ok(())
    }
}

/// Builder for creating tracked calls.
pub struct TrackedCallBuilder {
    tracker: CostTracker,
    context: TrackingContext,
    provider: String,
    model: String,
    input_tokens: i64,
    output_tokens: i64,
    cached_tokens: i64,
    estimated_cost_usd: f64,
    duration_ms: Option<u64>,
    success: bool,
    error: Option<String>,
}

impl TrackedCallBuilder {
    fn new(tracker: CostTracker) -> Self {
        Self {
            tracker,
            context: TrackingContext::new(),
            provider: String::new(),
            model: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            estimated_cost_usd: 0.0,
            duration_ms: None,
            success: true,
            error: None,
        }
    }

    /// Set the tracking context.
    #[must_use]
    pub fn context(mut self, context: TrackingContext) -> Self {
        self.context = context;
        self
    }

    /// Set the project ID.
    #[must_use]
    pub fn project(mut self, project_id: impl Into<ProjectId>) -> Self {
        self.context.project_id = Some(project_id.into());
        self
    }

    /// Set the task ID.
    #[must_use]
    pub fn task(mut self, task_id: impl Into<TaskId>) -> Self {
        self.context.task_id = Some(task_id.into());
        self
    }

    /// Set the agent ID.
    #[must_use]
    pub fn agent(mut self, agent_id: impl Into<AgentId>) -> Self {
        self.context.agent_id = Some(agent_id.into());
        self
    }

    /// Set the session ID.
    #[must_use]
    pub fn session(mut self, session_id: impl Into<SessionId>) -> Self {
        self.context.session_id = Some(session_id.into());
        self
    }

    /// Set the iteration number.
    #[must_use]
    pub fn iteration(mut self, iteration: u32) -> Self {
        self.context.iteration = Some(iteration);
        self
    }

    /// Set the provider name.
    #[must_use]
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set the model name.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the input tokens.
    #[must_use]
    pub fn input_tokens(mut self, tokens: i64) -> Self {
        self.input_tokens = tokens;
        self
    }

    /// Set the output tokens.
    #[must_use]
    pub fn output_tokens(mut self, tokens: i64) -> Self {
        self.output_tokens = tokens;
        self
    }

    /// Set the cached tokens.
    #[must_use]
    pub fn cached_tokens(mut self, tokens: i64) -> Self {
        self.cached_tokens = tokens;
        self
    }

    /// Set the estimated cost in USD.
    #[must_use]
    pub fn cost(mut self, cost_usd: f64) -> Self {
        self.estimated_cost_usd = cost_usd;
        self
    }

    /// Set the duration in milliseconds.
    #[must_use]
    pub fn duration_ms(mut self, duration: u64) -> Self {
        self.duration_ms = Some(duration);
        self
    }

    /// Mark the call as successful.
    #[must_use]
    pub fn success(mut self) -> Self {
        self.success = true;
        self.error = None;
        self
    }

    /// Mark the call as failed with an error.
    #[must_use]
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }

    /// Record the call and return the tracked call.
    #[must_use]
    pub fn record(self) -> TrackedCall {
        let call = TrackedCall {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            context: self.context,
            provider: self.provider,
            model: self.model,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cached_tokens: self.cached_tokens,
            estimated_cost_usd: self.estimated_cost_usd,
            duration_ms: self.duration_ms,
            success: self.success,
            error: self.error,
        };
        self.tracker.record(call.clone());
        call
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_record_and_query() {
        let tracker = CostTracker::new();

        let _ = tracker
            .builder()
            .project("proj-1")
            .task("task-1")
            .agent("claude")
            .provider("anthropic")
            .model("claude-3-opus")
            .input_tokens(1000)
            .output_tokens(500)
            .cost(0.05)
            .record();

        let _ = tracker
            .builder()
            .project("proj-1")
            .task("task-2")
            .agent("gpt-4")
            .provider("openai")
            .model("gpt-4-turbo")
            .input_tokens(800)
            .output_tokens(400)
            .cost(0.03)
            .record();

        assert_eq!(tracker.len(), 2);

        // Query by task
        let filter = TrackingFilter::new().with_task("task-1");
        let results = tracker.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].model, "claude-3-opus");

        // Query by agent
        let filter = TrackingFilter::new().with_agent("gpt-4");
        let results = tracker.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].context.task_id.as_ref().unwrap().0, "task-2");
    }

    #[test]
    fn test_task_metrics() {
        let tracker = CostTracker::new();

        // Record multiple iterations for a task
        for i in 1..=3 {
            let _ = tracker
                .builder()
                .task("task-1")
                .agent("claude")
                .iteration(i)
                .input_tokens(1000)
                .output_tokens(500)
                .cost(0.05)
                .record();
        }

        let metrics = tracker.task_metrics("task-1");
        assert_eq!(metrics.iterations, 3);
        assert_eq!(metrics.total_calls, 3);
        assert!((metrics.total_cost_usd - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_agent_comparison() {
        let tracker = CostTracker::new();

        // Agent 1: efficient (1 iteration, low cost)
        let _ = tracker
            .builder()
            .task("task-1")
            .agent("efficient-agent")
            .iteration(1)
            .cost(0.02)
            .success()
            .record();

        // Agent 2: less efficient (3 iterations, high cost)
        for i in 1..=3 {
            let _ = tracker
                .builder()
                .task("task-2")
                .agent("slow-agent")
                .iteration(i)
                .cost(0.05)
                .success()
                .record();
        }

        let comparison = tracker.compare_agents();
        assert_eq!(comparison.agents.len(), 2);
        assert_eq!(
            comparison.most_efficient_agent.as_ref().unwrap().0,
            "efficient-agent"
        );
    }

    #[test]
    fn test_export_import_json() {
        let tracker = CostTracker::new();

        let _ = tracker
            .builder()
            .task("task-1")
            .agent("claude")
            .cost(0.05)
            .record();

        let json = tracker.export_json().unwrap();

        let tracker2 = CostTracker::new();
        tracker2.import_json(&json).unwrap();

        assert_eq!(tracker2.len(), 1);
    }
}


















