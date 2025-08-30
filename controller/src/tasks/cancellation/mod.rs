//! # Cancellation System Module
//!
//! This module provides enhanced agent cancellation capabilities with distributed locking,
//! state awareness, and atomic operations to prevent race conditions during concurrent
//! remediation workflows.
//!
//! ## Architecture
//!
//! The cancellation system consists of several key components:
//!
//! - **DistributedLock**: Kubernetes lease-based concurrency control
//! - **StateAwareCancellation**: Integration with remediation state management
//! - **AtomicLabelManager**: GitHub API optimistic concurrency control
//! - **CancellationCoordinator**: Queue-based concurrent operation management
//! - **RecoveryManager**: Automated failure detection and repair
//!
//! ## Usage
//!
//! ### Basic Distributed Locking
//!
//! ```rust
//! use cto::tasks::cancellation::{DistributedLock, Lease};
//!
//! let lock = DistributedLock::new(client, "agent-platform", "cancel-task-42", "controller-1");
//! let lease = lock.try_acquire(context).await?;
//!
//! // Critical section - only one process can execute this
//! perform_cancellation().await?;
//!
//! lease.release().await?;
//! ```
//!
//! ### State-Aware Cancellation
//!
//! ```rust
//! use cto::tasks::cancellation::StateAwareCancellation;
//!
//! let cancellation = StateAwareCancellation::new(client, state_manager, lock_manager);
//! cancellation.cancel_agents_with_state_check(context, "task-42", 123).await?;
//! ```

pub mod distributed_lock;
pub mod state_aware;
pub mod atomic_labels;
pub mod coordinator;
pub mod recovery;

pub use distributed_lock::{DistributedLock, Lease, LeaseError};
pub use state_aware::{StateAwareCancellation, CancellationRequest};
pub use atomic_labels::{AtomicLabelManager, LabelTransition, ConcurrentModificationError};
pub use coordinator::{CancellationCoordinator, CancellationStatus};
pub use recovery::RecoveryManager;
