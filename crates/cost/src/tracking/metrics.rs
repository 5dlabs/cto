//! Efficiency metrics for cost optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::models::{AgentId, ProjectId, TaskId, TrackedCall};

/// Aggregated metrics for a set of tracked calls.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Total number of API calls.
    pub total_calls: u64,
    /// Number of successful calls.
    pub successful_calls: u64,
    /// Number of failed calls.
    pub failed_calls: u64,
    /// Total input tokens.
    pub total_input_tokens: i64,
    /// Total output tokens.
    pub total_output_tokens: i64,
    /// Total cached tokens.
    pub total_cached_tokens: i64,
    /// Total estimated cost in USD.
    pub total_cost_usd: f64,
    /// Average tokens per call.
    pub avg_tokens_per_call: f64,
    /// Average cost per call in USD.
    pub avg_cost_per_call: f64,
    /// Cache hit rate (cached tokens / total input tokens).
    pub cache_hit_rate: f64,
    /// Success rate (successful calls / total calls).
    pub success_rate: f64,
    /// Average duration in milliseconds.
    pub avg_duration_ms: Option<f64>,
}

impl AggregatedMetrics {
    /// Calculate metrics from a set of tracked calls.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Expected for statistical calculations
    pub fn from_calls(calls: &[TrackedCall]) -> Self {
        if calls.is_empty() {
            return Self::default();
        }

        let total_calls = calls.len() as u64;
        let successful_calls = calls.iter().filter(|c| c.success).count() as u64;
        let failed_calls = total_calls - successful_calls;

        let total_input_tokens: i64 = calls.iter().map(|c| c.input_tokens).sum();
        let total_output_tokens: i64 = calls.iter().map(|c| c.output_tokens).sum();
        let total_cached_tokens: i64 = calls.iter().map(|c| c.cached_tokens).sum();
        let total_cost_usd: f64 = calls.iter().map(|c| c.estimated_cost_usd).sum();

        let total_tokens = total_input_tokens + total_output_tokens;
        let avg_tokens_per_call = total_tokens as f64 / total_calls as f64;
        let avg_cost_per_call = total_cost_usd / total_calls as f64;

        let cache_hit_rate = if total_input_tokens > 0 {
            total_cached_tokens as f64 / total_input_tokens as f64
        } else {
            0.0
        };

        let success_rate = successful_calls as f64 / total_calls as f64;

        let durations: Vec<u64> = calls.iter().filter_map(|c| c.duration_ms).collect();
        let avg_duration_ms = if durations.is_empty() {
            None
        } else {
            Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
        };

        Self {
            total_calls,
            successful_calls,
            failed_calls,
            total_input_tokens,
            total_output_tokens,
            total_cached_tokens,
            total_cost_usd,
            avg_tokens_per_call,
            avg_cost_per_call,
            cache_hit_rate,
            success_rate,
            avg_duration_ms,
        }
    }
}

/// Task-level efficiency metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Task ID.
    pub task_id: TaskId,
    /// Number of iterations/attempts.
    pub iterations: u32,
    /// Total API calls for this task.
    pub total_calls: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Total tokens used.
    pub total_tokens: i64,
    /// Cost per iteration.
    pub cost_per_iteration: f64,
    /// Tokens per iteration.
    pub tokens_per_iteration: f64,
    /// Whether the task completed successfully.
    pub completed: bool,
    /// Aggregated metrics for all calls.
    pub metrics: AggregatedMetrics,
}

impl TaskMetrics {
    /// Calculate task metrics from tracked calls.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Expected for statistical calculations
    pub fn from_calls(task_id: TaskId, calls: &[TrackedCall]) -> Self {
        let metrics = AggregatedMetrics::from_calls(calls);

        // Count unique iterations
        let iterations: u32 = calls
            .iter()
            .filter_map(|c| c.context.iteration)
            .max()
            .unwrap_or(1);

        let total_tokens = metrics.total_input_tokens + metrics.total_output_tokens;
        let iterations_f64 = f64::from(iterations);

        Self {
            task_id,
            iterations,
            total_calls: metrics.total_calls,
            total_cost_usd: metrics.total_cost_usd,
            total_tokens,
            cost_per_iteration: metrics.total_cost_usd / iterations_f64,
            tokens_per_iteration: total_tokens as f64 / iterations_f64,
            completed: calls.iter().any(|c| c.success),
            metrics,
        }
    }
}

/// Agent-level efficiency metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent ID.
    pub agent_id: AgentId,
    /// Total tasks worked on.
    pub tasks_count: u64,
    /// Total API calls.
    pub total_calls: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Average iterations per task.
    pub avg_iterations_per_task: f64,
    /// Average cost per task.
    pub avg_cost_per_task: f64,
    /// Task completion rate.
    pub task_completion_rate: f64,
    /// Aggregated metrics for all calls.
    pub metrics: AggregatedMetrics,
}

impl AgentMetrics {
    /// Calculate agent metrics from tracked calls grouped by task.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Expected for statistical calculations
    pub fn from_calls(agent_id: AgentId, calls: &[TrackedCall]) -> Self {
        let metrics = AggregatedMetrics::from_calls(calls);

        // Group calls by task
        let mut tasks: HashMap<TaskId, Vec<&TrackedCall>> = HashMap::new();
        for call in calls {
            if let Some(ref task_id) = call.context.task_id {
                tasks.entry(task_id.clone()).or_default().push(call);
            }
        }

        let tasks_count = tasks.len() as u64;
        let completed_tasks = tasks
            .values()
            .filter(|task_calls| task_calls.iter().any(|c| c.success))
            .count() as u64;

        let total_iterations: u32 = tasks
            .values()
            .map(|task_calls| {
                task_calls
                    .iter()
                    .filter_map(|c| c.context.iteration)
                    .max()
                    .unwrap_or(1)
            })
            .sum();

        let tasks_count_f64 = if tasks_count > 0 {
            tasks_count as f64
        } else {
            1.0
        };

        Self {
            agent_id,
            tasks_count,
            total_calls: metrics.total_calls,
            total_cost_usd: metrics.total_cost_usd,
            avg_iterations_per_task: f64::from(total_iterations) / tasks_count_f64,
            avg_cost_per_task: metrics.total_cost_usd / tasks_count_f64,
            task_completion_rate: completed_tasks as f64 / tasks_count_f64,
            metrics,
        }
    }
}

/// Project-level metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Project ID.
    pub project_id: ProjectId,
    /// Total tasks in this project.
    pub tasks_count: u64,
    /// Total API calls.
    pub total_calls: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Cost breakdown by agent.
    pub cost_by_agent: HashMap<AgentId, f64>,
    /// Cost breakdown by model.
    pub cost_by_model: HashMap<String, f64>,
    /// Aggregated metrics.
    pub metrics: AggregatedMetrics,
}

impl ProjectMetrics {
    /// Calculate project metrics from tracked calls.
    #[must_use]
    pub fn from_calls(project_id: ProjectId, calls: &[TrackedCall]) -> Self {
        let metrics = AggregatedMetrics::from_calls(calls);

        // Count unique tasks
        let tasks_count = calls
            .iter()
            .filter_map(|c| c.context.task_id.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        // Cost by agent
        let mut cost_by_agent: HashMap<AgentId, f64> = HashMap::new();
        for call in calls {
            if let Some(ref agent_id) = call.context.agent_id {
                *cost_by_agent.entry(agent_id.clone()).or_default() += call.estimated_cost_usd;
            }
        }

        // Cost by model
        let mut cost_by_model: HashMap<String, f64> = HashMap::new();
        for call in calls {
            *cost_by_model.entry(call.model.clone()).or_default() += call.estimated_cost_usd;
        }

        Self {
            project_id,
            tasks_count,
            total_calls: metrics.total_calls,
            total_cost_usd: metrics.total_cost_usd,
            cost_by_agent,
            cost_by_model,
            metrics,
        }
    }
}

/// Comparison between agents for optimization insights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComparison {
    /// Agent metrics being compared.
    pub agents: Vec<AgentMetrics>,
    /// Most cost-efficient agent (lowest cost per task).
    pub most_efficient_agent: Option<AgentId>,
    /// Least cost-efficient agent.
    pub least_efficient_agent: Option<AgentId>,
    /// Agent with best completion rate.
    pub best_completion_rate_agent: Option<AgentId>,
    /// Agent with fewest iterations per task.
    pub fewest_iterations_agent: Option<AgentId>,
}

impl AgentComparison {
    /// Compare a list of agent metrics.
    #[must_use]
    pub fn from_metrics(agents: Vec<AgentMetrics>) -> Self {
        let most_efficient_agent = agents
            .iter()
            .filter(|a| a.tasks_count > 0)
            .min_by(|a, b| {
                a.avg_cost_per_task
                    .partial_cmp(&b.avg_cost_per_task)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|a| a.agent_id.clone());

        let least_efficient_agent = agents
            .iter()
            .filter(|a| a.tasks_count > 0)
            .max_by(|a, b| {
                a.avg_cost_per_task
                    .partial_cmp(&b.avg_cost_per_task)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|a| a.agent_id.clone());

        let best_completion_rate_agent = agents
            .iter()
            .filter(|a| a.tasks_count > 0)
            .max_by(|a, b| {
                a.task_completion_rate
                    .partial_cmp(&b.task_completion_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|a| a.agent_id.clone());

        let fewest_iterations_agent = agents
            .iter()
            .filter(|a| a.tasks_count > 0)
            .min_by(|a, b| {
                a.avg_iterations_per_task
                    .partial_cmp(&b.avg_iterations_per_task)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|a| a.agent_id.clone());

        Self {
            agents,
            most_efficient_agent,
            least_efficient_agent,
            best_completion_rate_agent,
            fewest_iterations_agent,
        }
    }
}















