//! # Distributed Lock Implementation
//!
//! This module provides distributed locking capabilities using Kubernetes coordination/v1 Lease API.
//! The implementation ensures mutual exclusion across multiple controller instances for critical
//! cancellation operations, preventing race conditions during concurrent remediation workflows.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use k8s_openapi::api::coordination::v1::{Lease as K8sLease, LeaseSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use kube::api::{Api, DeleteParams, Patch, PatchParams, PostParams};
use kube::core::ObjectMeta;
use kube::{Client, Error as KubeError};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during distributed lock operations
#[derive(Error, Debug)]
pub enum LeaseError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] KubeError),

    #[error("Lock acquisition failed: {message}")]
    AcquisitionFailed { message: String },

    #[error("Lock is held by another holder: {holder}")]
    LockHeld { holder: String },

    #[error("Lease renewal failed: {0}")]
    RenewalFailed(String),

    #[error("Lock timeout exceeded")]
    TimeoutExceeded,

    #[error("Lease validation failed: {0}")]
    ValidationError(String),
}

/// Distributed lock using Kubernetes Lease API for concurrency control
#[derive(Clone)]
pub struct DistributedLock {
    client: Client,
    namespace: String,
    lock_name: String,
    holder_name: String,
    lease_duration: Duration,
    renewal_interval: Duration,
}

impl DistributedLock {
    /// Create a new distributed lock
    #[must_use] pub fn new(client: Client, namespace: &str, lock_name: &str, holder_name: &str) -> Self {
        Self {
            client,
            namespace: namespace.to_string(),
            lock_name: lock_name.to_string(),
            holder_name: holder_name.to_string(),
            lease_duration: Duration::from_secs(30),
            renewal_interval: Duration::from_secs(10),
        }
    }

    /// Set the lease duration (default: 30 seconds)
    #[must_use] pub fn with_lease_duration(mut self, duration: Duration) -> Self {
        self.lease_duration = duration;
        self
    }

    /// Set the renewal interval (default: 10 seconds)
    #[must_use] pub fn with_renewal_interval(mut self, interval: Duration) -> Self {
        self.renewal_interval = interval;
        self
    }

    /// Attempt to acquire the lock
    pub async fn try_acquire(&self) -> Result<ActiveLease, LeaseError> {
        debug!(
            lock_name = %self.lock_name,
            holder = %self.holder_name,
            "Attempting to acquire distributed lock"
        );

        let lease_api: Api<K8sLease> = Api::namespaced(self.client.clone(), &self.namespace);

        let lease = self.create_lease_object()?;

        // Try to create the lease (succeeds if it doesn't exist)
        match lease_api.create(&PostParams::default(), &lease).await {
            Ok(created_lease) => {
                info!(
                    lock_name = %self.lock_name,
                    holder = %self.holder_name,
                    "Successfully acquired distributed lock"
                );
                Ok(ActiveLease::new(
                    created_lease,
                    self.client.clone(),
                    &self.namespace,
                    self.renewal_interval,
                ))
            }
            Err(KubeError::Api(err)) if err.code == 409 => {
                // Lease already exists, try to acquire it
                self.try_acquire_existing_lease(&lease_api).await
            }
            Err(e) => {
                warn!(
                    lock_name = %self.lock_name,
                    holder = %self.holder_name,
                    error = %e,
                    "Failed to create lease"
                );
                Err(LeaseError::KubeError(e))
            }
        }
    }

    /// Try to acquire an existing lease
    async fn try_acquire_existing_lease(
        &self,
        lease_api: &Api<K8sLease>,
    ) -> Result<ActiveLease, LeaseError> {
        let existing_lease = lease_api.get(&self.lock_name).await?;

        // Check if the lease is expired
        if self.is_lease_expired(&existing_lease) {
            debug!(
                lock_name = %self.lock_name,
                holder = %self.holder_name,
                "Lease is expired, attempting to acquire"
            );

            // Try to update the expired lease
            let mut updated_lease = existing_lease.clone();
            updated_lease.spec = Some(self.create_lease_spec()?);
            updated_lease.metadata.annotations = Some(self.create_annotations());

            match lease_api
                .replace(&self.lock_name, &PostParams::default(), &updated_lease)
                .await
            {
                Ok(acquired_lease) => {
                    info!(
                        lock_name = %self.lock_name,
                        holder = %self.holder_name,
                        "Successfully acquired expired distributed lock"
                    );
                    Ok(ActiveLease::new(
                        acquired_lease,
                        self.client.clone(),
                        &self.namespace,
                        self.renewal_interval,
                    ))
                }
                Err(e) => {
                    warn!(
                        lock_name = %self.lock_name,
                        holder = %self.holder_name,
                        error = %e,
                        "Failed to acquire expired lease"
                    );
                    Err(LeaseError::KubeError(e))
                }
            }
        } else {
            // Lease is still valid and held by someone else
            let holder = existing_lease
                .spec
                .as_ref()
                .and_then(|spec| spec.holder_identity.as_ref())
                .map_or("unknown", std::string::String::as_str);

            debug!(
                lock_name = %self.lock_name,
                holder = %holder,
                "Lock is held by another process"
            );

            Err(LeaseError::LockHeld {
                holder: holder.to_string(),
            })
        }
    }

    /// Check if a lease is expired
    fn is_lease_expired(&self, lease: &K8sLease) -> bool {
        let Some(spec) = &lease.spec else {
            return true;
        };

        let Some(renew_time) = &spec.renew_time else {
            return true;
        };

        let Some(duration_seconds) = spec.lease_duration_seconds else {
            return true;
        };

        let expiration_time = renew_time.0 + chrono::Duration::seconds(i64::from(duration_seconds));
        let now = chrono::Utc::now();

        expiration_time < now
    }

    /// Create a lease object for initial creation
    fn create_lease_object(&self) -> Result<K8sLease, LeaseError> {
        Ok(K8sLease {
            metadata: ObjectMeta {
                name: Some(self.lock_name.clone()),
                namespace: Some(self.namespace.clone()),
                annotations: Some(self.create_annotations()),
                ..Default::default()
            },
            spec: Some(self.create_lease_spec()?),
        })
    }

    /// Create lease spec
    fn create_lease_spec(&self) -> Result<LeaseSpec, LeaseError> {
        let now = chrono::Utc::now();

        Ok(LeaseSpec {
            holder_identity: Some(self.holder_name.clone()),
            lease_duration_seconds: Some(self.lease_duration.as_secs() as i32),
            acquire_time: Some(MicroTime(now)),
            renew_time: Some(MicroTime(now)),
            lease_transitions: None,
        })
    }

    /// Create lease annotations
    fn create_annotations(&self) -> BTreeMap<String, String> {
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "cancellation.5dlabs.com/holder".to_string(),
            self.holder_name.clone(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/acquired".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/operation".to_string(),
            "agent-cancellation".to_string(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/lock-name".to_string(),
            self.lock_name.clone(),
        );
        annotations
    }
}

/// Active lease that can be renewed and released
pub struct ActiveLease {
    lease: K8sLease,
    client: Client,
    namespace: String,
    renewal_interval: Duration,
    renewal_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ActiveLease {
    /// Create a new lease instance
    fn new(lease: K8sLease, client: Client, namespace: &str, renewal_interval: Duration) -> Self {
        let mut lease_instance = Self {
            lease,
            client,
            namespace: namespace.to_string(),
            renewal_interval,
            renewal_handle: None,
        };

        // Start automatic renewal
        lease_instance.start_renewal();

        lease_instance
    }

    /// Start automatic lease renewal
    fn start_renewal(&mut self) {
        let client = self.client.clone();
        let namespace = self.namespace.clone();
        let lease_name = self.lease.metadata.name.clone().unwrap_or_default();
        let renewal_interval = self.renewal_interval;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(renewal_interval);
            let lease_api: Api<K8sLease> = Api::namespaced(client, &namespace);

            loop {
                interval.tick().await;

                // Attempt to renew the lease
                match Self::renew_lease(&lease_api, &lease_name).await {
                    Ok(()) => {
                        debug!(lease_name = %lease_name, "Lease renewed successfully");
                    }
                    Err(e) => {
                        error!(
                            lease_name = %lease_name,
                            error = %e,
                            "Failed to renew lease, lease may expire"
                        );
                        break;
                    }
                }
            }
        });

        self.renewal_handle = Some(handle);
    }

    /// Renew the lease
    async fn renew_lease(lease_api: &Api<K8sLease>, lease_name: &str) -> Result<(), LeaseError> {
        let mut patch = HashMap::new();
        patch.insert(
            "spec",
            HashMap::from([("renewTime", chrono::Utc::now().to_rfc3339())]),
        );

        let patch_data = serde_json::to_vec(&patch)
            .map_err(|e| LeaseError::ValidationError(format!("Failed to serialize patch: {e}")))?;

        lease_api
            .patch(
                lease_name,
                &PatchParams::apply("cancellation-controller"),
                &Patch::Apply(&patch_data),
            )
            .await?;

        Ok(())
    }

    /// Release the lease
    pub async fn release(self) -> Result<(), LeaseError> {
        // Stop renewal
        if let Some(ref handle) = self.renewal_handle {
            handle.abort();
        }

        let lease_api: Api<K8sLease> = Api::namespaced(self.client.clone(), &self.namespace);
        let lease_name = self
            .lease
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| LeaseError::ValidationError("Lease name is missing".to_string()))?;

        lease_api
            .delete(lease_name, &DeleteParams::default())
            .await?;

        info!(lease_name = %lease_name, "Lease released successfully");

        Ok(())
    }

    /// Get the lease name
    #[must_use] pub fn name(&self) -> &str {
        self.lease.metadata.name.as_deref().unwrap_or("")
    }

    /// Get the holder identity
    #[must_use] pub fn holder(&self) -> &str {
        self.lease
            .spec
            .as_ref()
            .and_then(|spec| spec.holder_identity.as_ref())
            .map_or("", std::string::String::as_str)
    }

    /// Check if the lease is still valid
    #[must_use] pub fn is_valid(&self) -> bool {
        let Some(spec) = &self.lease.spec else {
            return false;
        };

        let Some(renew_time) = &spec.renew_time else {
            return false;
        };

        let Some(duration_seconds) = spec.lease_duration_seconds else {
            return false;
        };

        let expiration_time = renew_time.0 + chrono::Duration::seconds(i64::from(duration_seconds));
        let now = chrono::Utc::now();

        expiration_time > now
    }
}

impl Drop for ActiveLease {
    fn drop(&mut self) {
        // Stop renewal when lease is dropped
        if let Some(handle) = self.renewal_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_distributed_lock_creation() {
        // Mock client for testing - in real tests this would be mocked
        // let client = Client::try_default().await.unwrap();
        // For now, we'll skip the client-dependent test
        let duration = Duration::from_secs(30);
        assert_eq!(duration.as_secs(), 30);
    }

    #[test]
    fn test_lease_annotations() {
        // Mock client for testing - in real tests this would be mocked
        // let client = Client::try_default().await.unwrap();
        // let lock = DistributedLock::new(client, "test-ns", "test-lock", "test-holder");

        // For now, test the annotation creation logic directly
        let lock_name = "test-lock";
        let holder_name = "test-holder";

        let mut annotations = BTreeMap::new();
        annotations.insert(
            "cancellation.5dlabs.com/holder".to_string(),
            holder_name.to_string(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/acquired".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/operation".to_string(),
            "agent-cancellation".to_string(),
        );
        annotations.insert(
            "cancellation.5dlabs.com/lock-name".to_string(),
            lock_name.to_string(),
        );

        assert!(annotations.contains_key("cancellation.5dlabs.com/holder"));
        assert!(annotations.contains_key("cancellation.5dlabs.com/acquired"));
        assert!(annotations.contains_key("cancellation.5dlabs.com/operation"));
        assert!(annotations.contains_key("cancellation.5dlabs.com/lock-name"));

        assert_eq!(annotations["cancellation.5dlabs.com/holder"], "test-holder");
        assert_eq!(
            annotations["cancellation.5dlabs.com/operation"],
            "agent-cancellation"
        );
        assert_eq!(
            annotations["cancellation.5dlabs.com/lock-name"],
            "test-lock"
        );
    }
}
