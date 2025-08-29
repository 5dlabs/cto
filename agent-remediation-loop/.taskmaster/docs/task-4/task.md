# Task 4: Implement State Management System

## Overview
Build a comprehensive ConfigMap-based state tracking system for remediation iterations and feedback history in the Rust controller. This system provides persistent state management for the Agent Remediation Loop, enabling atomic operations, feedback history storage, and robust recovery mechanisms.

## Technical Context
The remediation system requires stateful tracking of iteration counts, feedback history, and operational metadata across controller restarts. Using Kubernetes ConfigMaps as the storage backend provides native Kubernetes integration, atomic operations through server-side apply, and consistent state management across multiple controller replicas.

### Architecture Integration
- **Controller Location**: Enhances existing Rust controller at `controller/src/`
- **State Module**: New module at `controller/src/remediation/state.rs`
- **Storage Backend**: Kubernetes ConfigMaps in controller namespace
- **Serialization**: JSON-based data storage using serde
- **Atomic Operations**: Server-side apply for concurrent safety

## Implementation Guide

### Step 1: Design ConfigMap Schema and Data Structure

#### 1.1 ConfigMap Naming Convention
- Pattern: `task-{id}-state`
- Example: `task-42-state`
- Namespace: Same as controller deployment

#### 1.2 Data Structure Design
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    pub task_id: String,
    pub iteration: u32,
    pub status: RemediationStatus,
    pub feedback_history: Vec<FeedbackEntry>,
    pub last_update: DateTime<Utc>,
    pub error_messages: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub version: String, // Schema version for evolution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationStatus {
    Initialized,
    InProgress,
    Completed,
    Failed,
    MaxIterationsReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub severity: FeedbackSeverity,
    pub issue_type: IssueType,
    pub description: String,
    pub resolved: bool,
    pub pr_comment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    Bug,
    Enhancement,
    Documentation,
    Performance,
    Security,
}
```

### Step 2: Implement Rust StateManager in Controller

#### 2.1 StateManager Structure
```rust
use kube::{Api, Client, Error as KubeError};
use k8s_openapi::api::core::v1::ConfigMap;
use serde_json;
use std::collections::BTreeMap;
use tracing::{info, warn, error};

const MAX_ITERATIONS: u32 = 10;
const STATE_VERSION: &str = "v1";

#[derive(Clone)]
pub struct StateManager {
    client: Client,
    namespace: String,
    configmap_api: Api<ConfigMap>,
}

impl StateManager {
    pub fn new(client: Client, namespace: String) -> Self {
        let configmap_api = Api::namespaced(client.clone(), &namespace);
        
        Self {
            client,
            namespace,
            configmap_api,
        }
    }

    fn configmap_name(&self, task_id: &str) -> String {
        format!("task-{}-state", task_id)
    }
}
```

#### 2.2 CRUD Operations Implementation
```rust
impl StateManager {
    pub async fn get_state(&self, task_id: &str) -> Result<Option<RemediationState>, StateError> {
        let cm_name = self.configmap_name(task_id);
        
        match self.configmap_api.get(&cm_name).await {
            Ok(cm) => {
                if let Some(data) = cm.data.get("state.json") {
                    match serde_json::from_str::<RemediationState>(data) {
                        Ok(state) => {
                            info!("Retrieved state for task {}", task_id);
                            Ok(Some(state))
                        }
                        Err(e) => {
                            error!("Failed to deserialize state: {}", e);
                            Err(StateError::DeserializationError(e.to_string()))
                        }
                    }
                } else {
                    warn!("ConfigMap {} exists but has no state data", cm_name);
                    Ok(None)
                }
            }
            Err(KubeError::Api(kube::api::ErrorResponse { code: 404, .. })) => Ok(None),
            Err(e) => {
                error!("Failed to get ConfigMap {}: {}", cm_name, e);
                Err(StateError::KubernetesError(e.to_string()))
            }
        }
    }

    pub async fn create_or_update_state(
        &self,
        state: &RemediationState,
    ) -> Result<(), StateError> {
        let cm_name = self.configmap_name(&state.task_id);
        let state_json = serde_json::to_string_pretty(state)
            .map_err(|e| StateError::SerializationError(e.to_string()))?;

        let mut data = BTreeMap::new();
        data.insert("state.json".to_string(), state_json);
        data.insert("last_update".to_string(), Utc::now().to_rfc3339());
        data.insert("task_id".to_string(), state.task_id.clone());
        data.insert("iteration".to_string(), state.iteration.to_string());

        let configmap = ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some(cm_name.clone()),
                namespace: Some(self.namespace.clone()),
                labels: Some(self.get_labels(&state.task_id)),
                annotations: Some(self.get_annotations()),
                ..Default::default()
            },
            data: Some(data),
            ..Default::default()
        };

        // Use server-side apply for atomic operations
        let params = kube::api::PatchParams::apply("state-manager");
        match self.configmap_api.patch(&cm_name, &params, &kube::api::Patch::Apply(&configmap)).await {
            Ok(_) => {
                info!("Successfully updated state for task {}", state.task_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update ConfigMap {}: {}", cm_name, e);
                Err(StateError::KubernetesError(e.to_string()))
            }
        }
    }

    fn get_labels(&self, task_id: &str) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "remediation-state".to_string());
        labels.insert("task-id".to_string(), task_id.to_string());
        labels.insert("managed-by".to_string(), "agent-remediation-controller".to_string());
        labels
    }

    fn get_annotations(&self) -> BTreeMap<String, String> {
        let mut annotations = BTreeMap::new();
        annotations.insert("kubectl.kubernetes.io/last-applied-configuration".to_string(), "managed-by-controller".to_string());
        annotations.insert("remediation.5dlabs.com/version".to_string(), STATE_VERSION.to_string());
        annotations
    }
}
```

### Step 3: Build Atomic Iteration Counter

#### 3.1 Atomic Increment Implementation
```rust
impl StateManager {
    pub async fn increment_iteration(&self, task_id: &str) -> Result<u32, StateError> {
        // Get current state or create new one
        let mut state = self.get_state(task_id).await?
            .unwrap_or_else(|| RemediationState {
                task_id: task_id.to_string(),
                iteration: 0,
                status: RemediationStatus::Initialized,
                feedback_history: Vec::new(),
                last_update: Utc::now(),
                error_messages: Vec::new(),
                metadata: HashMap::new(),
                version: STATE_VERSION.to_string(),
            });

        // Check iteration limit
        if state.iteration >= MAX_ITERATIONS {
            warn!("Task {} has reached maximum iterations ({})", task_id, MAX_ITERATIONS);
            state.status = RemediationStatus::MaxIterationsReached;
            self.create_or_update_state(&state).await?;
            return Err(StateError::MaxIterationsReached(state.iteration));
        }

        // Atomic increment
        state.iteration += 1;
        state.last_update = Utc::now();
        state.status = RemediationStatus::InProgress;

        // Add iteration metadata
        state.metadata.insert(
            format!("iteration_{}_started", state.iteration),
            Utc::now().to_rfc3339(),
        );

        self.create_or_update_state(&state).await?;
        
        info!("Incremented iteration for task {} to {}", task_id, state.iteration);
        Ok(state.iteration)
    }

    pub async fn get_current_iteration(&self, task_id: &str) -> Result<u32, StateError> {
        match self.get_state(task_id).await? {
            Some(state) => Ok(state.iteration),
            None => Ok(0),
        }
    }
}
```

### Step 4: Implement Feedback History with Serde JSON

#### 4.1 Feedback Management
```rust
impl StateManager {
    pub async fn append_feedback(
        &self,
        task_id: &str,
        feedback: FeedbackEntry,
    ) -> Result<(), StateError> {
        let mut state = self.get_state(task_id).await?
            .ok_or_else(|| StateError::StateNotFound(task_id.to_string()))?;

        // Add feedback to history
        state.feedback_history.push(feedback.clone());
        state.last_update = Utc::now();

        // Check ConfigMap size limit (1MB)
        let serialized = serde_json::to_string(&state)
            .map_err(|e| StateError::SerializationError(e.to_string()))?;
        
        if serialized.len() > 950_000 { // Leave buffer for other data
            warn!("State approaching ConfigMap size limit, compressing feedback history");
            self.compress_feedback_history(&mut state).await?;
        }

        self.create_or_update_state(&state).await?;
        info!("Added feedback from {} to task {}", feedback.author, task_id);
        Ok(())
    }

    pub async fn get_feedback_history(
        &self,
        task_id: &str,
    ) -> Result<Vec<FeedbackEntry>, StateError> {
        match self.get_state(task_id).await? {
            Some(state) => Ok(state.feedback_history),
            None => Ok(Vec::new()),
        }
    }

    async fn compress_feedback_history(
        &self,
        state: &mut RemediationState,
    ) -> Result<(), StateError> {
        // Keep only most recent 20 entries and resolved high-priority items
        let mut compressed = Vec::new();
        
        // Keep resolved high/critical items
        for entry in &state.feedback_history {
            if entry.resolved && matches!(entry.severity, FeedbackSeverity::High | FeedbackSeverity::Critical) {
                compressed.push(entry.clone());
            }
        }
        
        // Keep most recent 15 entries
        let recent_start = state.feedback_history.len().saturating_sub(15);
        for entry in state.feedback_history.iter().skip(recent_start) {
            if !compressed.contains(entry) {
                compressed.push(entry.clone());
            }
        }
        
        // Sort by timestamp
        compressed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Add compression metadata
        state.metadata.insert(
            "feedback_compressed_at".to_string(),
            Utc::now().to_rfc3339(),
        );
        state.metadata.insert(
            "original_feedback_count".to_string(),
            state.feedback_history.len().to_string(),
        );
        
        state.feedback_history = compressed;
        Ok(())
    }
}
```

### Step 5: Create State Recovery System

#### 5.1 Recovery Implementation
```rust
impl StateManager {
    pub async fn recover_state(&self, task_id: &str) -> Result<RemediationState, StateError> {
        info!("Recovering state for task {}", task_id);
        
        match self.get_state(task_id).await? {
            Some(mut state) => {
                // Validate state consistency
                self.validate_state(&mut state).await?;
                
                // Update recovery metadata
                state.metadata.insert(
                    "last_recovery".to_string(),
                    Utc::now().to_rfc3339(),
                );
                
                self.create_or_update_state(&state).await?;
                info!("Successfully recovered state for task {}", task_id);
                Ok(state)
            }
            None => {
                // Create new state for recovery
                let state = RemediationState {
                    task_id: task_id.to_string(),
                    iteration: 0,
                    status: RemediationStatus::Initialized,
                    feedback_history: Vec::new(),
                    last_update: Utc::now(),
                    error_messages: Vec::new(),
                    metadata: HashMap::new(),
                    version: STATE_VERSION.to_string(),
                };
                
                self.create_or_update_state(&state).await?;
                info!("Created new state during recovery for task {}", task_id);
                Ok(state)
            }
        }
    }

    async fn validate_state(&self, state: &mut RemediationState) -> Result<(), StateError> {
        let mut repairs_made = false;

        // Check version compatibility
        if state.version != STATE_VERSION {
            warn!("State version mismatch: {} vs {}", state.version, STATE_VERSION);
            state.version = STATE_VERSION.to_string();
            repairs_made = true;
        }

        // Validate iteration consistency
        if state.iteration > MAX_ITERATIONS {
            warn!("Invalid iteration count: {}, capping at {}", state.iteration, MAX_ITERATIONS);
            state.iteration = MAX_ITERATIONS;
            state.status = RemediationStatus::MaxIterationsReached;
            repairs_made = true;
        }

        // Remove duplicate feedback entries
        state.feedback_history.dedup_by(|a, b| {
            a.pr_comment_id == b.pr_comment_id && a.timestamp == b.timestamp
        });

        if repairs_made {
            state.metadata.insert(
                "state_repaired_at".to_string(),
                Utc::now().to_rfc3339(),
            );
            info!("Repaired state inconsistencies for task {}", state.task_id);
        }

        Ok(())
    }
}
```

### Step 6: Implement TTL Cleanup with Tokio Tasks

#### 6.1 Cleanup System Implementation
```rust
use tokio::time::{interval, Duration};
use chrono::Duration as ChronoDuration;

impl StateManager {
    pub fn start_cleanup_task(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(6 * 3600)); // Every 6 hours
            
            loop {
                cleanup_interval.tick().await;
                
                if let Err(e) = self.cleanup_old_states().await {
                    error!("Cleanup task failed: {}", e);
                } else {
                    info!("Cleanup task completed successfully");
                }
            }
        })
    }

    async fn cleanup_old_states(&self) -> Result<(), StateError> {
        let retention_period = ChronoDuration::days(7); // Configurable
        let cutoff_time = Utc::now() - retention_period;

        // List all state ConfigMaps
        let list_params = kube::api::ListParams::default()
            .labels("app=remediation-state");
        
        let configmaps = self.configmap_api.list(&list_params).await
            .map_err(|e| StateError::KubernetesError(e.to_string()))?;

        let mut cleaned_count = 0;

        for cm in configmaps.items {
            if let Some(name) = cm.metadata.name {
                if let Some(data) = cm.data {
                    if let Some(last_update_str) = data.get("last_update") {
                        if let Ok(last_update) = DateTime::parse_from_rfc3339(last_update_str) {
                            let last_update_utc = last_update.with_timezone(&Utc);
                            
                            if last_update_utc < cutoff_time {
                                // Check if task is still active before deleting
                                if let Some(state_json) = data.get("state.json") {
                                    if let Ok(state) = serde_json::from_str::<RemediationState>(state_json) {
                                        if matches!(state.status, RemediationStatus::InProgress) {
                                            continue; // Don't delete active tasks
                                        }
                                    }
                                }
                                
                                // Soft delete with grace period
                                info!("Cleaning up old state ConfigMap: {}", name);
                                match self.configmap_api.delete(&name, &kube::api::DeleteParams::default()).await {
                                    Ok(_) => {
                                        cleaned_count += 1;
                                        info!("Successfully deleted state ConfigMap: {}", name);
                                    }
                                    Err(e) => {
                                        warn!("Failed to delete ConfigMap {}: {}", name, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Cleanup completed, removed {} old state ConfigMaps", cleaned_count);
        Ok(())
    }
}
```

### Step 7: Error Handling and Monitoring

#### 7.1 Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Kubernetes API error: {0}")]
    KubernetesError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("State not found for task: {0}")]
    StateNotFound(String),
    
    #[error("Maximum iterations reached: {0}")]
    MaxIterationsReached(u32),
    
    #[error("State validation failed: {0}")]
    ValidationError(String),
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_state_creation_and_retrieval() {
        // Test state CRUD operations
    }

    #[tokio::test]
    async fn test_iteration_increment() {
        // Test atomic iteration counter
    }

    #[tokio::test]
    async fn test_feedback_history_management() {
        // Test feedback append and compression
    }

    #[tokio::test]
    async fn test_max_iteration_limit() {
        // Test iteration limit enforcement
    }

    #[tokio::test]
    async fn test_state_recovery() {
        // Test recovery and validation logic
    }
}
```

### Integration Tests
1. **ConfigMap Operations**: Test with real Kubernetes cluster
2. **Concurrent Access**: Test multiple controller instances
3. **Recovery Scenarios**: Test after controller restart
4. **Cleanup Operations**: Test TTL-based cleanup
5. **Error Handling**: Test various failure scenarios

## Performance Considerations

### Optimizations
- **Batch Operations**: Group multiple state updates when possible
- **Compression**: Automatic feedback history compression
- **Caching**: In-memory caching for frequently accessed states
- **Connection Pooling**: Reuse Kubernetes client connections

### Resource Limits
- ConfigMap size limit: 1MB (with compression handling)
- Memory usage: Monitor for large feedback histories
- API rate limits: Implement exponential backoff

## Security Considerations

### Access Control
- Use service account with minimal required permissions
- Validate all input data before storage
- Sanitize user-provided feedback content

### Data Protection
- No sensitive data in ConfigMap storage
- Audit logging for all state modifications
- Secure cleanup of deleted states

## Monitoring and Alerting

### Metrics
- State operation success/failure rates
- ConfigMap size distribution
- Cleanup operation metrics
- Recovery operation frequency

### Alerts
- Failed state operations
- ConfigMap approaching size limits
- Cleanup operation failures
- Frequent recovery operations

## Success Criteria
- StateManager successfully manages remediation state across restarts
- Atomic iteration counter prevents race conditions
- Feedback history stored with automatic compression
- TTL cleanup maintains reasonable storage usage
- Integration with controller reconciliation loop
- Comprehensive error handling and recovery