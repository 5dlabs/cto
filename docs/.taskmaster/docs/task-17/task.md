# Task 17: Setup Multi-Task Processing



## Overview
Extend the workflow system to handle multiple tasks in sequence with proper state management, progress tracking, and checkpointing. This enables complex workflows that span multiple related tasks while maintaining resilience and observability.

## Technical Implementation



### Architecture
The multi-task processing system implements:
- **Task Sequencing**: Execute tasks in defined order with dependency management
- **State Management**: Preserve workflow state between task executions
- **Progress Tracking**: Monitor and report progress across the entire workflow
- **Checkpointing**: Save intermediate results for failure recovery
- **Memoization**: Cache completed steps to avoid redundant work

### Implementation Components

#### 1. Multi-Task Workflow Definition

**File**: `workflows/multi-task-processing.yaml`




```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: multi-task-processor
  namespace: taskmaster
spec:
  entrypoint: process-task-sequence

  arguments:
    parameters:
    - name: task-list
      description: "JSON array of task IDs to process"
    - name: task-range
      description: "Range specification (e.g., '1-5', '10,12,14')"
    - name: continue-on-error
      value: "false"
      description: "Continue processing remaining tasks on individual failures"
    - name: checkpoint-interval
      value: "1"
      description: "Save checkpoint after every N tasks"

  templates:
  - name: process-task-sequence
    steps:
    - - name: parse-task-list
        template: parse-tasks
        arguments:
          parameters:
          - name: task-list
            value: "{{workflow.parameters.task-list}}"
          - name: task-range
            value: "{{workflow.parameters.task-range}}"

    - - name: initialize-workflow-state
        template: init-state
        arguments:
          parameters:
          - name: total-tasks
            value: "{{steps.parse-task-list.outputs.parameters.task-count}}"

    - - name: process-tasks-loop
        template: task-processing-loop
        arguments:
          parameters:
          - name: task-ids
            value: "{{steps.parse-task-list.outputs.parameters.task-ids}}"
          - name: workflow-state
            value: "{{steps.initialize-workflow-state.outputs.parameters.state}}"

  - name: parse-tasks
    inputs:
      parameters:
      - name: task-list
      - name: task-range
    outputs:
      parameters:
      - name: task-ids
        valueFrom:
          path: /tmp/task-ids.json
      - name: task-count
        valueFrom:
          path: /tmp/task-count.txt
    script:
      image: python:3.9-alpine
      command: [python]
      source: |
        import json
        import sys

        def parse_task_range(range_str):
            """Parse range specification like '1-5' or '10,12,14'"""
            if not range_str:
                return []

            tasks = []
            for part in range_str.split(','):
                if '-' in part:
                    start, end = map(int, part.split('-'))
                    tasks.extend(range(start, end + 1))
                else:
                    tasks.append(int(part))
            return tasks

        def parse_task_list(list_str):
            """Parse JSON array of task IDs"""
            if not list_str:
                return []
            return json.loads(list_str)

        # Parse inputs
        task_list = "{{inputs.parameters.task-list}}"
        task_range = "{{inputs.parameters.task-range}}"

        # Determine task IDs
        if task_list:
            task_ids = parse_task_list(task_list)
        elif task_range:
            task_ids = parse_task_range(task_range)
        else:
            task_ids = []

        # Output results
        with open('/tmp/task-ids.json', 'w') as f:
            json.dump(task_ids, f)

        with open('/tmp/task-count.txt', 'w') as f:
            f.write(str(len(task_ids)))

        print(f"Parsed {len(task_ids)} tasks: {task_ids}")

  - name: init-state
    inputs:
      parameters:
      - name: total-tasks
    outputs:
      parameters:
      - name: state
        valueFrom:
          path: /tmp/workflow-state.json
    script:
      image: python:3.9-alpine
      command: [python]
      source: |
        import json
        from datetime import datetime, timezone

        state = {
            "workflow_id": "{{workflow.uid}}",
            "start_time": datetime.now(timezone.utc).isoformat(),
            "total_tasks": int("{{inputs.parameters.total-tasks}}"),
            "completed_tasks": 0,
            "failed_tasks": 0,
            "current_task_index": 0,
            "checkpoints": [],
            "task_results": {},
            "last_checkpoint": None,
            "status": "initialized"
        }

        with open('/tmp/workflow-state.json', 'w') as f:
            json.dump(state, f, indent=2)

  - name: task-processing-loop
    inputs:
      parameters:
      - name: task-ids
      - name: workflow-state
    dag:
      tasks:
      - name: execute-task-sequence
        template: process-single-task
        arguments:
          parameters:
          - name: task-id
            value: "{{item}}"
          - name: workflow-state
            value: "{{inputs.parameters.workflow-state}}"
          - name: task-index
            value: "{{item-index}}"
        withParam: "{{inputs.parameters.task-ids}}"

      - name: create-checkpoint
        dependencies: [execute-task-sequence]
        template: checkpoint-workflow
        when: "{{workflow.parameters.checkpoint-interval}} > 0"
        arguments:
          parameters:
          - name: workflow-state
            value: "{{tasks.execute-task-sequence.outputs.parameters.updated-state}}"

  - name: process-single-task
    inputs:
      parameters:
      - name: task-id
      - name: workflow-state
      - name: task-index
    outputs:
      parameters:
      - name: updated-state
        valueFrom:
          path: /tmp/updated-state.json
      - name: task-result
        valueFrom:
          path: /tmp/task-result.json
    retryStrategy:
      limit: 3
      retryPolicy: "OnFailure"
      backoff:
        duration: "30s"
        factor: 2
        maxDuration: "5m"
    script:
      image: taskmaster/controller:latest
      command: [bash]
      source: |
        set -euo pipefail

        TASK_ID="{{inputs.parameters.task-id}}"
        TASK_INDEX="{{inputs.parameters.task-index}}"
        WORKFLOW_STATE='{{inputs.parameters.workflow-state}}'

        echo "Processing task $TASK_ID (index: $TASK_INDEX)"

        # Load current workflow state
        echo "$WORKFLOW_STATE" > /tmp/current-state.json

        # Check if task was already completed (memoization)
        if jq -e ".task_results[\"$TASK_ID\"]" /tmp/current-state.json > /dev/null; then
            echo "Task $TASK_ID already completed, using cached result"
            CACHED_RESULT=$(jq ".task_results[\"$TASK_ID\"]" /tmp/current-state.json)
            echo "$CACHED_RESULT" > /tmp/task-result.json
            cp /tmp/current-state.json /tmp/updated-state.json
            exit 0
        fi

        # Execute the actual task
        start_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # Call task execution service
        TASK_RESULT=$(curl -s -X POST http://taskmaster-controller/api/tasks \
            -H "Content-Type: application/json" \
            -d "{\"task_id\": \"$TASK_ID\"}")

        end_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # Process task result
        TASK_STATUS=$(echo "$TASK_RESULT" | jq -r '.status')

        if [ "$TASK_STATUS" = "completed" ]; then
            echo "Task $TASK_ID completed successfully"

            # Update workflow state
            jq --arg task_id "$TASK_ID" \


               --arg result "$TASK_RESULT" \


               --arg start_time "$start_time" \


               --arg end_time "$end_time" \


               --argjson index "$TASK_INDEX" '
               .completed_tasks += 1 |
               .current_task_index = $index |
               .task_results[$task_id] = ($result | fromjson) |
               .task_results[$task_id].start_time = $start_time |
               .task_results[$task_id].end_time = $end_time |
               .last_updated = now | todate
               ' /tmp/current-state.json > /tmp/updated-state.json

        else
            echo "Task $TASK_ID failed: $TASK_STATUS"

            # Update workflow state with failure
            jq --arg task_id "$TASK_ID" \


               --arg error "$TASK_RESULT" \


               --argjson index "$TASK_INDEX" '
               .failed_tasks += 1 |
               .current_task_index = $index |
               .task_results[$task_id] = {"status": "failed", "error": $error} |
               .last_updated = now | todate
               ' /tmp/current-state.json > /tmp/updated-state.json

            # Check continue-on-error policy
            if [ "{{workflow.parameters.continue-on-error}}" != "true" ]; then
                echo "Failing workflow due to task failure and continue-on-error=false"
                exit 1
            fi
        fi

        # Output task result
        echo "$TASK_RESULT" > /tmp/task-result.json

  - name: checkpoint-workflow
    inputs:
      parameters:
      - name: workflow-state
    script:
      image: python:3.9-alpine
      command: [python]
      source: |
        import json
        from datetime import datetime, timezone

        # Load current state
        state = json.loads('{{inputs.parameters.workflow-state}}')

        # Create checkpoint
        checkpoint = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "completed_tasks": state["completed_tasks"],
            "failed_tasks": state["failed_tasks"],
            "current_task_index": state["current_task_index"],
            "progress_percentage": (state["completed_tasks"] / state["total_tasks"]) * 100
        }

        # Add checkpoint to state
        state["checkpoints"].append(checkpoint)
        state["last_checkpoint"] = checkpoint["timestamp"]

        # Store checkpoint in persistent storage
        checkpoint_data = {
            "workflow_id": state["workflow_id"],
            "checkpoint": checkpoint,
            "state_snapshot": state
        }

        # Save to checkpoint storage (Redis/database)
        print(f"Created checkpoint at {checkpoint['timestamp']}")
        print(f"Progress: {checkpoint['progress_percentage']:.1f}%")






```

#### 2. State Management Service

**File**: `controller/src/workflow/state.rs`




```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub current_task_index: usize,
    pub task_results: HashMap<String, TaskResult>,
    pub checkpoints: Vec<Checkpoint>,
    pub last_checkpoint: Option<DateTime<Utc>>,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Initialized,
    Running,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub timestamp: DateTime<Utc>,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub current_task_index: usize,
    pub progress_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

impl WorkflowState {
    pub fn new(workflow_id: Uuid, total_tasks: usize) -> Self {
        Self {
            workflow_id,
            start_time: Utc::now(),
            total_tasks,
            completed_tasks: 0,
            failed_tasks: 0,
            current_task_index: 0,
            task_results: HashMap::new(),
            checkpoints: Vec::new(),
            last_checkpoint: None,
            status: WorkflowStatus::Initialized,
        }
    }

    pub fn update_task_result(&mut self, task_id: &str, result: TaskResult) {
        self.task_results.insert(task_id.to_string(), result);

        match result.status.as_str() {
            "completed" => self.completed_tasks += 1,
            "failed" => self.failed_tasks += 1,
            _ => {}
        }

        self.current_task_index += 1;

        // Update overall status
        if self.completed_tasks + self.failed_tasks >= self.total_tasks {
            self.status = if self.failed_tasks > 0 {
                WorkflowStatus::Failed
            } else {
                WorkflowStatus::Completed
            };
        }
    }

    pub fn create_checkpoint(&mut self) -> Checkpoint {
        let checkpoint = Checkpoint {
            timestamp: Utc::now(),
            completed_tasks: self.completed_tasks,
            failed_tasks: self.failed_tasks,
            current_task_index: self.current_task_index,
            progress_percentage: (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0,
        };

        self.checkpoints.push(checkpoint.clone());
        self.last_checkpoint = Some(checkpoint.timestamp);

        checkpoint
    }

    pub fn get_progress(&self) -> f64 {
        (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
    }

    pub fn is_task_completed(&self, task_id: &str) -> bool {
        self.task_results
            .get(task_id)
            .map(|result| result.status == "completed")
            .unwrap_or(false)
    }
}

// State persistence service
pub struct StateManager {
    storage: Box<dyn StateStorage>,
}

#[async_trait::async_trait]
pub trait StateStorage: Send + Sync {
    async fn save_state(&self, state: &WorkflowState) -> Result<(), StateError>;
    async fn load_state(&self, workflow_id: &Uuid) -> Result<Option<WorkflowState>, StateError>;
    async fn delete_state(&self, workflow_id: &Uuid) -> Result<(), StateError>;
}



#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Storage error: {0}")]
    Storage(String),
}






```

#### 3. Progress Tracking and Reporting

**File**: `controller/src/workflow/progress.rs`




```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    pub workflow_id: Uuid,
    pub current_task: Option<String>,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub failed_tasks: usize,
    pub progress_percentage: f64,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
    pub current_stage: String,
    pub task_breakdown: Vec<TaskProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub status: TaskStatus,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub struct ProgressTracker {
    reports: Arc<RwLock<std::collections::HashMap<Uuid, ProgressReport>>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            reports: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn update_progress(&self, workflow_id: Uuid, state: &WorkflowState) {
        let mut reports = self.reports.write().await;

        let report = ProgressReport {
            workflow_id,
            current_task: self.get_current_task(state),
            completed_tasks: state.completed_tasks,
            total_tasks: state.total_tasks,
            failed_tasks: state.failed_tasks,
            progress_percentage: state.get_progress(),
            estimated_completion: self.estimate_completion(state),
            current_stage: self.determine_stage(state),
            task_breakdown: self.create_task_breakdown(state),
        };

        reports.insert(workflow_id, report);
    }

    pub async fn get_progress(&self, workflow_id: &Uuid) -> Option<ProgressReport> {
        let reports = self.reports.read().await;
        reports.get(workflow_id).cloned()
    }

    fn estimate_completion(&self, state: &WorkflowState) -> Option<chrono::DateTime<chrono::Utc>> {
        if state.completed_tasks == 0 {
            return None;
        }

        let elapsed = chrono::Utc::now() - state.start_time;
        let avg_task_time = elapsed / state.completed_tasks as i32;
        let remaining_tasks = state.total_tasks - state.completed_tasks;

        Some(chrono::Utc::now() + avg_task_time * remaining_tasks as i32)
    }
}






```

### Integration with Argo Workflows

#### 1. Workflow Parameters and Memoization




```yaml
# Enable memoization for expensive operations
spec:
  templates:
  - name: process-single-task
    memoize:
      key: "task-{{inputs.parameters.task-id}}"
      cache:
        configMap:
          name: workflow-cache
          key: "{{inputs.parameters.task-id}}"
    # ... rest of template






```

#### 2. Retry and Error Handling




```yaml
# Comprehensive retry strategy
retryStrategy:
  limit: 3
  retryPolicy: "OnFailure"
  backoff:
    duration: "30s"
    factor: 2
    maxDuration: "5m"
  expression: "lastRetry.exitCode == 1"

# Failure handling
onExit: workflow-cleanup






```

## Testing Strategy

### Unit Tests



```rust


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_initialization() {
        let workflow_id = Uuid::new_v4();
        let state = WorkflowState::new(workflow_id, 5);

        assert_eq!(state.total_tasks, 5);
        assert_eq!(state.completed_tasks, 0);
        assert_eq!(state.status, WorkflowStatus::Initialized);
    }

    #[test]
    fn test_progress_calculation() {
        let mut state = WorkflowState::new(Uuid::new_v4(), 10);
        state.completed_tasks = 3;

        assert_eq!(state.get_progress(), 30.0);
    }

    #[tokio::test]
    async fn test_checkpoint_creation() {
        let mut state = WorkflowState::new(Uuid::new_v4(), 5);
        state.completed_tasks = 2;

        let checkpoint = state.create_checkpoint();

        assert_eq!(checkpoint.completed_tasks, 2);
        assert_eq!(checkpoint.progress_percentage, 40.0);
        assert_eq!(state.checkpoints.len(), 1);
    }
}






```

### Integration Tests



```rust


#[tokio::test]
async fn test_multi_task_workflow_execution() {
    let workflow_def = MultiTaskWorkflow::new(vec!["task-1", "task-2", "task-3"]);
    let result = workflow_def.execute().await.unwrap();

    assert_eq!(result.completed_tasks, 3);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.task_results.contains_key("task-1"));
}






```

## Performance Considerations

1. **State Serialization**: Use efficient serialization formats (bincode vs JSON)
2. **Checkpoint Frequency**: Balance between recovery capability and performance overhead
3. **Memory Management**: Clean up completed workflow states periodically
4. **Parallel Execution**: Support concurrent task execution where dependencies allow

## Monitoring and Observability

### Metrics Collection



```rust
// Prometheus metrics
lazy_static! {
    static ref WORKFLOW_DURATION: HistogramVec = register_histogram_vec!(
        "workflow_duration_seconds",
        "Workflow execution duration",
        &["workflow_type", "status"]
    ).unwrap();

    static ref TASK_COUNTER: CounterVec = register_counter_vec!(
        "tasks_total",
        "Total number of tasks processed",
        &["status"]
    ).unwrap();
}






```



### Progress Dashboard


- Real-time workflow progress visualization


- Task execution timeline


- Resource utilization monitoring


- Error rate tracking


- Estimated completion times
