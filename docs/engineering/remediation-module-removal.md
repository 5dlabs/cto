# Remediation Module Removal Guide

This document describes how to safely remove the `remediation/` module from the controller crate. The module was scaffolding for a planned "Agent Remediation Loop" feature that was never fully connected and will not be implemented.

## Overview

The remediation module was designed for a QA feedback loop where:
- Tess (QA agent) would post "🔴 Required Changes" comments on PRs
- The controller would parse those structured feedback comments
- The `RemediationStateManager` would track fix iterations
- Rex would iteratively fix issues based on feedback

**This feature was never hooked up and is not being used in production.**

## Files to Delete

Delete the entire `crates/controller/src/remediation/` directory:

```
crates/controller/src/remediation/
├── auth.rs           # Author validation for feedback comments
├── error.rs          # Error types for parsing
├── markdown.rs       # Markdown checkbox parsing
├── mod.rs            # Module entry point and re-exports
├── parser.rs         # Main feedback comment parser
├── patterns.rs       # Regex pattern extraction
├── state.rs          # RemediationStateManager (ConfigMap-based state)
├── types.rs          # StructuredFeedback, IssueType, Severity types
└── tests/
    ├── fixtures.rs   # Test fixtures
    └── mod.rs        # Test module
```

**Command:**
```bash
rm -rf crates/controller/src/remediation/
```

## Files to Update

### 1. `crates/controller/src/lib.rs`

**Remove:**
```rust
pub mod remediation;
```

**Before:**
```rust
pub mod cli;
pub mod crds;
pub mod remediation;
pub mod tasks;
```

**After:**
```rust
pub mod cli;
pub mod crds;
pub mod tasks;
```

---

### 2. `crates/controller/src/bin/agent_controller.rs`

**Changes needed:**

1. Remove import:
```rust
// DELETE this line:
use controller::remediation::RemediationStateManager;
```

2. Remove from `AppState` struct:
```rust
// BEFORE:
struct AppState {
    client: kube::Client,
    namespace: String,
    config: Arc<ControllerConfig>,
    remediation_state_manager: Arc<RemediationStateManager>,
}

// AFTER:
struct AppState {
    client: kube::Client,
    namespace: String,
    config: Arc<ControllerConfig>,
}
```

3. Remove state manager creation in `main()`:
```rust
// DELETE these lines:
let task_context = TaskContext {
    client: client.clone(),
    namespace: namespace.clone(),
    config: controller_config.clone(),
};
let remediation_state_manager = Arc::new(RemediationStateManager::new(&task_context));
```

4. Update `AppState` initialization in `main()`:
```rust
// BEFORE:
let state = AppState {
    client: client.clone(),
    namespace: namespace.clone(),
    config: controller_config.clone(),
    remediation_state_manager,
};

// AFTER:
let state = AppState {
    client: client.clone(),
    namespace: namespace.clone(),
    config: controller_config.clone(),
};
```

5. Update `webhook_handler` function - remove all state manager usage:
```rust
// DELETE the LabelOrchestrator state_manager parameter:
// BEFORE:
let mut orchestrator = LabelOrchestrator::new(
    label_client,
    state.remediation_state_manager.clone(),
    override_detector,
);

// AFTER:
let mut orchestrator = LabelOrchestrator::new(label_client, override_detector);

// DELETE the entire state initialization block:
match state
    .remediation_state_manager
    .load_state(u32::try_from(pr_number).unwrap_or(0), &task_id)
    .await
{
    // ... entire match block ...
}
```

6. Update the `State` parameter in `webhook_handler` to be unused:
```rust
// BEFORE:
async fn webhook_handler(
    State(state): State<AppState>,
    ...

// AFTER:
async fn webhook_handler(
    State(_state): State<AppState>,
    ...
```

---

### 3. `crates/controller/src/tasks/label/orchestrator.rs`

**Changes needed:**

1. Remove import:
```rust
// DELETE:
use crate::remediation::RemediationStateManager;
use std::sync::Arc;
```

2. Remove from struct:
```rust
// BEFORE:
pub struct LabelOrchestrator {
    label_client: GitHubLabelClient,
    label_schema: LabelSchema,
    #[allow(dead_code)]
    state_manager: Arc<RemediationStateManager>,
    override_detector: OverrideDetector,
}

// AFTER:
pub struct LabelOrchestrator {
    label_client: GitHubLabelClient,
    label_schema: LabelSchema,
    override_detector: OverrideDetector,
}
```

3. Update constructor:
```rust
// BEFORE:
pub fn new(
    label_client: GitHubLabelClient,
    state_manager: Arc<RemediationStateManager>,
    override_detector: OverrideDetector,
) -> Self {
    Self {
        label_client,
        label_schema: LabelSchema::default(),
        state_manager,
        override_detector,
    }
}

// AFTER:
pub fn new(label_client: GitHubLabelClient, override_detector: OverrideDetector) -> Self {
    Self {
        label_client,
        label_schema: LabelSchema::default(),
        override_detector,
    }
}
```

---

### 4. `crates/controller/src/tasks/label/cleanup.rs`

**Changes needed:**

1. Remove import:
```rust
// DELETE:
use crate::remediation::RemediationStateManager;
use std::sync::Arc;
```

2. Remove from struct:
```rust
// BEFORE:
pub struct LabelCleanupManager {
    label_client: GitHubLabelClient,
    state_manager: Arc<RemediationStateManager>,
}

// AFTER:
pub struct LabelCleanupManager {
    label_client: GitHubLabelClient,
}
```

3. Update constructor:
```rust
// BEFORE:
pub fn new(
    label_client: GitHubLabelClient,
    state_manager: Arc<RemediationStateManager>,
) -> Self {
    Self {
        label_client,
        state_manager,
    }
}

// AFTER:
pub fn new(label_client: GitHubLabelClient) -> Self {
    Self { label_client }
}
```

---

### 5. `crates/controller/src/tasks/cancel/aware.rs`

**Changes needed:**

1. Remove imports:
```rust
// DELETE:
use crate::remediation::{RemediationState, RemediationStateManager};
```

2. Remove from struct:
```rust
// BEFORE:
pub struct StateAwareCancellation {
    client: Client,
    namespace: String,
    state_manager: RemediationStateManager,
    lock_manager: DistributedLock,
    ...
}

// AFTER:
pub struct StateAwareCancellation {
    client: Client,
    namespace: String,
    lock_manager: DistributedLock,
    ...
}
```

3. Update constructor to remove `state_manager` parameter:
```rust
// BEFORE:
pub fn new(client: Client, namespace: &str, state_manager: RemediationStateManager) -> Self {
    ...
    Self {
        client,
        namespace: namespace.to_string(),
        state_manager,
        lock_manager,
        ...
    }
}

// AFTER:
pub fn new(client: Client, namespace: &str) -> Self {
    ...
    Self {
        client,
        namespace: namespace.to_string(),
        lock_manager,
        ...
    }
}
```

4. In `cancel_agents_with_state_check`, remove the remediation state check block:
```rust
// DELETE entire block checking RemediationStatus::InProgress:
let state_result = self.state_manager.load_state(pr_number_u32, task_id).await;
match state_result {
    Ok(Some(state)) => {
        if matches!(state.status, crate::remediation::RemediationStatus::InProgress) {
            // ... entire block ...
        }
        // ...
    }
    // ...
}
```

5. Update `agents_completed` method signature:
```rust
// BEFORE:
async fn agents_completed(&self, state: &RemediationState) -> Result<bool, CancellationError>

// AFTER:
async fn agents_completed(&self, task_id: &str) -> Result<bool, CancellationError>
```

---

### 6. `crates/controller/src/tasks/security/mod.rs`

**Changes needed:**

Remove the unused import:
```rust
// DELETE:
use crate::remediation::{RemediationState, StructuredFeedback};
```

---

### 7. `crates/controller/src/tasks/security/validation.rs`

**Changes needed:**

Delete the entire `validate_structured_feedback` method (lines ~169-232) which references:
- `crate::remediation::StructuredFeedback`
- `crate::remediation::IssueType`
- `crate::remediation::Severity`

```rust
// DELETE entire method:
pub async fn validate_structured_feedback(
    &self,
    feedback: &crate::remediation::StructuredFeedback,
) -> ValidationResult<InputValidationResult> {
    // ... entire implementation ...
}
```

---

## Verification Steps

After making all changes, run:

```bash
# 1. Check compilation
cargo check -p controller

# 2. Run clippy with pedantic warnings
cargo clippy -p controller -- -D warnings

# 3. Run tests
cargo test -p controller

# 4. Format check
cargo fmt -p controller -- --check
```

## Dependency Graph

```
lib.rs
  └── pub mod remediation  ──────────────────────────────────┐
                                                              │
agent_controller.rs                                          │
  └── use controller::remediation::RemediationStateManager ──┤
                                                              │
tasks/cancel/aware.rs                                        │
  └── use crate::remediation::{RemediationState, ...}  ──────┤
                                                              │
tasks/label/orchestrator.rs                                  │
  └── use crate::remediation::RemediationStateManager  ──────┤
                                                              │
tasks/label/cleanup.rs                                       │
  └── use crate::remediation::RemediationStateManager  ──────┤
                                                              │
tasks/security/mod.rs                                        │
  └── use crate::remediation::{RemediationState, ...}  ──────┤
                                                              │
tasks/security/validation.rs                                 │
  └── crate::remediation::{StructuredFeedback, ...}   ───────┘
```

## Summary

| Action | File Count |
|--------|------------|
| Delete files | 10 files (entire remediation/ directory) |
| Update files | 7 files |
| Total changes | ~200 lines removed, ~50 lines modified |

## Notes

- The `LabelOrchestrator.state_manager` field was already marked `#[allow(dead_code)]` - it was stored but never used
- The `validate_structured_feedback` function in security/validation.rs was never called
- The `parse_feedback_comment` function was only used in tests
- No CRDs or Helm charts reference the remediation module
- No workflows (Play, Intake) use the remediation module




