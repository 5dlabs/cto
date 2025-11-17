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
//! - **`DistributedLock`**: Kubernetes lease-based concurrency control
//! - **`StateAwareCancellation`**: Integration with remediation state management
//! - **`AtomicLabelManager`**: GitHub API optimistic concurrency control
//! - **`CancellationCoordinator`**: Queue-based concurrent operation management
//! - **`RecoveryManager`**: Automated failure detection and repair
//!
//! ## Usage
//!
//! ### Basic Distributed Locking
//!
//! ```rust,ignore
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a Kubernetes client
//! let client = kube::Client::try_default().await?;
//!
//! let lock = DistributedLock::new(&client, "agent-platform", "cancel-task-42", "controller-1");
//! let lease = lock.try_acquire("controller-1").await?;
//!
//! // Critical section - only one process can execute this
//! // perform_cancellation().await?;
//!
//! lease.release().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### State-Aware Cancellation
//!
//! ```rust,ignore
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create dependencies (normally injected)
//! // let client = kube::Client::try_default().await?;
//! // let state_manager = RemediationStateManager::new(&client);
//! // let lock_manager = DistributedLockManager::new(&client);
//!
//! // let cancellation = StateAwareCancellation::new(client, state_manager, lock_manager);
//! // cancellation.cancel_agents_with_state_check("controller-1", "task-42", 123).await?;
//! # Ok(())
//! # }
//! ```

pub mod aware;
pub mod coordinator;
pub mod labels;
pub mod lock;
pub mod recovery;

pub use aware::{CancellationRequest, StateAwareCancellation};
pub use coordinator::{CancellationCoordinator, CancellationStatus};
pub use labels::{AtomicLabelManager, ConcurrentModificationError, LabelTransition};
pub use lock::{ActiveLease, DistributedLock, LeaseError};
pub use recovery::RecoveryManager;
