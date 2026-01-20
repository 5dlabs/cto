//! Complexity filter - determines if a task is worth learning.

use crate::models::TaskRecord;
use tracing::debug;

/// Complexity filter for determining task learnability.
#[derive(Debug, Clone)]
pub struct ComplexityFilter {
    /// Minimum number of tool calls required.
    pub min_tool_calls: usize,

    /// Minimum duration in seconds.
    pub min_duration_secs: u64,

    /// Minimum complexity score threshold (0.0 - 1.0).
    pub complexity_threshold: f32,

    /// Minimum unique tools used.
    pub min_unique_tools: usize,
}

impl Default for ComplexityFilter {
    fn default() -> Self {
        Self {
            min_tool_calls: 3,
            min_duration_secs: 300, // 5 minutes
            complexity_threshold: 0.5,
            min_unique_tools: 2,
        }
    }
}

impl ComplexityFilter {
    /// Create a new complexity filter with custom settings.
    #[must_use]
    pub fn new(min_tool_calls: usize, min_duration_secs: u64, complexity_threshold: f32) -> Self {
        Self {
            min_tool_calls,
            min_duration_secs,
            complexity_threshold,
            min_unique_tools: 2,
        }
    }

    /// Check if a task passes the complexity filter.
    #[must_use]
    pub fn should_learn(&self, task: &TaskRecord) -> bool {
        // Must be successful
        if !task.is_learnable(self.min_tool_calls, self.min_duration_secs) {
            debug!(
                task_id = %task.id,
                "Task not learnable: basic criteria not met"
            );
            return false;
        }

        // Check unique tools
        let unique_tools = task.unique_tools();
        if unique_tools.len() < self.min_unique_tools {
            debug!(
                task_id = %task.id,
                unique_tools = unique_tools.len(),
                min_required = self.min_unique_tools,
                "Task not learnable: not enough unique tools"
            );
            return false;
        }

        // Calculate and check complexity score
        let score = self.calculate_complexity(task);
        if score < self.complexity_threshold {
            debug!(
                task_id = %task.id,
                score,
                threshold = self.complexity_threshold,
                "Task not learnable: complexity score too low"
            );
            return false;
        }

        debug!(
            task_id = %task.id,
            score,
            "Task passes complexity filter"
        );
        true
    }

    /// Calculate complexity score for a task (0.0 - 1.0).
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for score calculation
    pub fn calculate_complexity(&self, task: &TaskRecord) -> f32 {
        let mut score = 0.0f32;

        // Factor 1: Number of tool calls (normalized)
        let tool_call_score = (task.tool_calls.len() as f32 / 20.0).min(1.0);
        score += tool_call_score * 0.3;

        // Factor 2: Unique tools (normalized)
        let unique_tools = task.unique_tools().len();
        let unique_score = (unique_tools as f32 / 10.0).min(1.0);
        score += unique_score * 0.3;

        // Factor 3: Duration (normalized to 30 min max)
        if let Some(duration) = task.duration() {
            let duration_secs = duration.num_seconds().max(0) as f32;
            let duration_score = (duration_secs / 1800.0).min(1.0);
            score += duration_score * 0.2;
        }

        // Factor 4: Has user preferences
        if !task.user_preferences.is_empty() {
            score += 0.1;
        }

        // Factor 5: Has progress updates (indicates iterative work)
        let progress_score = (task.progresses.len() as f32 / 10.0).min(1.0);
        score += progress_score * 0.1;

        score.min(1.0)
    }

    /// Get detailed complexity breakdown for debugging.
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Duration seconds are always positive after max(0)
    pub fn complexity_breakdown(&self, task: &TaskRecord) -> ComplexityBreakdown {
        let unique_tools = task.unique_tools();
        let duration_secs = task.duration().map_or(0, |d| d.num_seconds().max(0) as u64);

        ComplexityBreakdown {
            tool_calls: task.tool_calls.len(),
            unique_tools: unique_tools.len(),
            duration_secs,
            progress_count: task.progresses.len(),
            preference_count: task.user_preferences.len(),
            total_score: self.calculate_complexity(task),
            passes_filter: self.should_learn(task),
        }
    }
}

/// Breakdown of complexity factors for debugging.
#[derive(Debug, Clone)]
pub struct ComplexityBreakdown {
    /// Number of tool calls.
    pub tool_calls: usize,
    /// Number of unique tools.
    pub unique_tools: usize,
    /// Duration in seconds.
    pub duration_secs: u64,
    /// Number of progress updates.
    pub progress_count: usize,
    /// Number of user preferences.
    pub preference_count: usize,
    /// Total complexity score.
    pub total_score: f32,
    /// Whether it passes the filter.
    pub passes_filter: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskStatus, ToolCallRecord};

    fn create_complex_task() -> TaskRecord {
        let mut task = TaskRecord::new("1", "Complex implementation task");
        task.start();

        // Add varied tool calls
        for i in 0..10 {
            let tool = match i % 5 {
                0 => "read_file",
                1 => "write_file",
                2 => "search_files",
                3 => "git_status",
                _ => "list_directory",
            };
            task.add_tool_call(ToolCallRecord::new(format!("call-{i}"), tool, "{}"));
        }

        task.add_progress("Read existing code");
        task.add_progress("Analyzed patterns");
        task.add_progress("Implemented changes");

        task.add_preference("Use async/await");
        task.add_preference("Follow existing patterns");

        task.complete(true);
        task
    }

    #[test]
    fn test_default_filter() {
        let filter = ComplexityFilter::default();
        assert_eq!(filter.min_tool_calls, 3);
        assert_eq!(filter.min_duration_secs, 300);
    }

    #[test]
    fn test_simple_task_rejected() {
        let filter = ComplexityFilter {
            min_duration_secs: 0, // Override for test
            ..Default::default()
        };

        let mut task = TaskRecord::new("1", "Simple task");
        task.start();
        task.add_tool_call(ToolCallRecord::new("1", "read_file", "{}"));
        task.complete(true);

        assert!(!filter.should_learn(&task));
    }

    #[test]
    fn test_complex_task_accepted() {
        let filter = ComplexityFilter {
            min_duration_secs: 0,
            complexity_threshold: 0.3,
            ..Default::default()
        };

        let task = create_complex_task();
        assert!(filter.should_learn(&task));
    }

    #[test]
    fn test_complexity_score() {
        let filter = ComplexityFilter::default();
        let task = create_complex_task();

        let score = filter.calculate_complexity(&task);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_failed_task_rejected() {
        let filter = ComplexityFilter {
            min_duration_secs: 0,
            ..Default::default()
        };

        let mut task = create_complex_task();
        task.status = TaskStatus::Failed;

        assert!(!filter.should_learn(&task));
    }

    #[test]
    fn test_complexity_breakdown() {
        let filter = ComplexityFilter {
            min_duration_secs: 0,
            complexity_threshold: 0.3,
            ..Default::default()
        };

        let task = create_complex_task();
        let breakdown = filter.complexity_breakdown(&task);

        assert_eq!(breakdown.tool_calls, 10);
        assert_eq!(breakdown.unique_tools, 5);
        assert_eq!(breakdown.progress_count, 3);
        assert_eq!(breakdown.preference_count, 2);
        assert!(breakdown.passes_filter);
    }
}
