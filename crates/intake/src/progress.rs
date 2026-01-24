//! Progress event types for Linear agent dialog integration.
//!
//! This module provides structured progress events that the intake workflow
//! emits to a JSON lines file. The Linear sidecar reads these events and
//! updates the agent dialog accordingly.
//!
//! # Event Types
//!
//! - `Config`: Initial configuration summary
//! - `Step`: Workflow step progress (1-4)
//! - `Retry`: Retry attempt notifications
//! - `TaskProgress`: Task generation progress
//! - `Complete`: Final completion summary
//!
//! # Usage
//!
//! ```rust,ignore
//! let writer = ProgressWriter::from_env()?;
//! writer.emit(ProgressEvent::Config { ... })?;
//! ```

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// Step status for tracking workflow progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Step has not started yet.
    Pending,
    /// Step is currently in progress.
    InProgress,
    /// Step completed successfully.
    Completed,
    /// Step was skipped.
    Skipped,
    /// Step failed.
    Failed,
}

/// Progress events emitted during intake workflow.
///
/// These events are written as JSON lines to a progress file,
/// which the Linear sidecar watches and maps to agent activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// Initial configuration summary.
    Config {
        /// AI model being used.
        model: String,
        /// CLI type (claude, cursor, etc.).
        cli: String,
        /// Target number of tasks to generate.
        target_tasks: u32,
        /// Acceptance threshold percentage.
        acceptance: u32,
    },

    /// Workflow step progress update.
    Step {
        /// Step number (1-4).
        step: u8,
        /// Total number of steps.
        total: u8,
        /// Human-readable step name.
        name: String,
        /// Current status of the step.
        status: StepStatus,
        /// Optional details (e.g., task count).
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// Retry attempt notification.
    Retry {
        /// Which step is being retried.
        step: u8,
        /// Current attempt number (1-based).
        attempt: u8,
        /// Maximum retry attempts.
        max: u8,
        /// Reason for retry.
        reason: String,
    },

    /// Task generation progress.
    TaskProgress {
        /// Number of tasks generated so far.
        generated: u32,
        /// Target number of tasks.
        target: u32,
    },

    /// Workflow completion summary.
    Complete {
        /// Total tasks generated.
        tasks: u32,
        /// Total duration in seconds.
        duration_secs: f64,
        /// Whether the workflow succeeded.
        success: bool,
        /// Optional error message if failed.
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

impl ProgressEvent {
    /// Create a config event.
    #[must_use]
    pub fn config(model: &str, cli: &str, target_tasks: u32, acceptance: u32) -> Self {
        Self::Config {
            model: model.to_string(),
            cli: cli.to_string(),
            target_tasks,
            acceptance,
        }
    }

    /// Create a step started event.
    #[must_use]
    pub fn step_started(step: u8, name: &str) -> Self {
        Self::Step {
            step,
            total: 4,
            name: name.to_string(),
            status: StepStatus::InProgress,
            details: None,
        }
    }

    /// Create a step completed event.
    #[must_use]
    pub fn step_completed(step: u8, name: &str, details: Option<&str>) -> Self {
        Self::Step {
            step,
            total: 4,
            name: name.to_string(),
            status: StepStatus::Completed,
            details: details.map(String::from),
        }
    }

    /// Create a step skipped event.
    #[must_use]
    pub fn step_skipped(step: u8, name: &str, reason: &str) -> Self {
        Self::Step {
            step,
            total: 4,
            name: name.to_string(),
            status: StepStatus::Skipped,
            details: Some(reason.to_string()),
        }
    }

    /// Create a step failed event.
    #[must_use]
    pub fn step_failed(step: u8, name: &str, error: &str) -> Self {
        Self::Step {
            step,
            total: 4,
            name: name.to_string(),
            status: StepStatus::Failed,
            details: Some(error.to_string()),
        }
    }

    /// Create a retry event.
    #[must_use]
    pub fn retry(step: u8, attempt: u8, max: u8, reason: &str) -> Self {
        Self::Retry {
            step,
            attempt,
            max,
            reason: reason.to_string(),
        }
    }

    /// Create a task progress event.
    #[must_use]
    pub fn task_progress(generated: u32, target: u32) -> Self {
        Self::TaskProgress { generated, target }
    }

    /// Create a completion event.
    #[must_use]
    pub fn complete(tasks: u32, duration_secs: f64, success: bool, error: Option<&str>) -> Self {
        Self::Complete {
            tasks,
            duration_secs,
            success,
            error: error.map(String::from),
        }
    }
}

/// Writer for emitting progress events to a JSON lines file.
///
/// Thread-safe via internal mutex. Events are written immediately
/// and flushed to disk for real-time sidecar consumption.
pub struct ProgressWriter {
    file: Mutex<Option<BufWriter<File>>>,
    path: PathBuf,
}

impl ProgressWriter {
    /// Create a new progress writer for the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn new(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        Ok(Self {
            file: Mutex::new(Some(BufWriter::new(file))),
            path,
        })
    }

    /// Create a progress writer from environment variable.
    ///
    /// Reads `PROGRESS_FILE` env var. Returns `None` if not set.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let path = std::env::var("PROGRESS_FILE").ok()?;
        Self::new(path).ok()
    }

    /// Get the path this writer is writing to.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Emit a progress event.
    ///
    /// The event is serialized as JSON and written as a single line,
    /// then immediately flushed to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    pub fn emit(&self, event: &ProgressEvent) -> std::io::Result<()> {
        let mut guard = self.file.lock().unwrap();
        if let Some(ref mut writer) = *guard {
            let json = serde_json::to_string(event)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            writeln!(writer, "{json}")?;
            writer.flush()?;
        }
        Ok(())
    }

    /// Close the writer and flush any remaining data.
    pub fn close(&self) {
        let mut guard = self.file.lock().unwrap();
        if let Some(writer) = guard.take() {
            drop(writer);
        }
    }
}

impl Drop for ProgressWriter {
    fn drop(&mut self) {
        self.close();
    }
}

/// Global progress writer instance.
///
/// Initialized lazily when first accessed. Uses `PROGRESS_FILE` env var.
static PROGRESS_WRITER: std::sync::OnceLock<Option<ProgressWriter>> = std::sync::OnceLock::new();

/// Get the global progress writer.
///
/// Returns `None` if `PROGRESS_FILE` env var is not set.
#[must_use]
pub fn progress_writer() -> Option<&'static ProgressWriter> {
    PROGRESS_WRITER
        .get_or_init(ProgressWriter::from_env)
        .as_ref()
}

/// Emit a progress event to the global writer.
///
/// No-op if `PROGRESS_FILE` is not set.
pub fn emit_progress(event: &ProgressEvent) {
    if let Some(writer) = progress_writer() {
        if let Err(e) = writer.emit(event) {
            tracing::warn!(error = %e, "Failed to emit progress event");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_event_serialization() {
        let event = ProgressEvent::config("claude-opus-4-5", "claude", 50, 90);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"config""#));
        assert!(json.contains(r#""model":"claude-opus-4-5""#));
        assert!(json.contains(r#""target_tasks":50"#));
    }

    #[test]
    fn test_step_event_serialization() {
        let event = ProgressEvent::step_started(1, "Parse PRD");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"step""#));
        assert!(json.contains(r#""step":1"#));
        assert!(json.contains(r#""status":"in_progress""#));
    }

    #[test]
    fn test_retry_event_serialization() {
        let event = ProgressEvent::retry(1, 2, 3, "Extended thinking disabled");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"retry""#));
        assert!(json.contains(r#""attempt":2"#));
        assert!(json.contains(r#""max":3"#));
    }

    #[test]
    fn test_progress_writer() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        let writer = ProgressWriter::new(&path).unwrap();
        writer
            .emit(&ProgressEvent::config("model", "cli", 10, 90))
            .unwrap();
        writer
            .emit(&ProgressEvent::step_started(1, "Test"))
            .unwrap();
        writer.close();

        // Read back and verify
        let file = std::fs::File::open(&path).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(r#""type":"config""#));
        assert!(lines[1].contains(r#""type":"step""#));
    }
}
