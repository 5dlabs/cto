//! BOLT-003: Credential Injection via ExternalSecret
//!
//! Creates Kubernetes resources for BoltRun tasks:
//! - ExternalSecret pointing to tenant's credential path in OpenBao
//! - Kubernetes Secret created with provider credentials
//! - Job with installer binary and proper environment

use crate::crds::{BoltRun, BoltTaskType, ClusterSize};
use crate::tasks::types::{Context, Error, Result};

use k8s_openapi::api::batch::v1::{Job, JobSpec};
use k8s_openapi::api::core::v1::{
    Container, EnvFromSource, EnvVar, PodSpec, PodTemplateSpec, ResourceRequirements,
    SecretEnvSource,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Api, DynamicObject, GroupVersionKind, PostParams};
use kube::core::ObjectMeta;
use kube::Client;
use serde_json::json;
use std::collections::BTreeMap;
use tracing::{debug, info};

/// BOLT-003: Create ExternalSecret for credential injection
///
/// ExternalSecret created pointing to tenant's credential path
/// Kubernetes Secret created with provider credentials
/// Secret mounted as env vars in Bolt pod (never disk)
/// Pod ServiceAccount has minimal required permissions
/// Credentials unavailable after pod terminates
pub async fn create_external_secret(
    client: &Client,
    namespace: &str,
    name: &str,
    credential_path: &str,
    tenant_ref: &str,
) -> Result<()> {
    info!(
        "Creating ExternalSecret {} for path {} in namespace {}",
        name, credential_path, namespace
    );

    // ExternalSecret CRD structure (external-secrets.io/v1beta1)
    let external_secret = json!({
        "apiVersion": "external-secrets.io/v1beta1",
        "kind": "ExternalSecret",
        "metadata": {
            "name": name,
            "namespace": namespace,
            "labels": {
                "cto.5dlabs.ai/tenant": tenant_ref,
                "cto.5dlabs.ai/managed-by": "bolt-controller"
            }
        },
        "spec": {
            "refreshInterval": "1h",
            "secretStoreRef": {
                "name": "openbao-cluster-store",
                "kind": "ClusterSecretStore"
            },
            "target": {
                "name": name,
                "creationPolicy": "Owner",
                "deletionPolicy": "Delete"
            },
            "data": [
                {
                    "secretKey": "PROVIDER_API_KEY", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("secret/data/{}", credential_path),
                        "property": "api_key"
                    }
                },
                {
                    "secretKey": "PROVIDER_TYPE", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("secret/data/{}", credential_path),
                        "property": "provider"
                    }
                },
                {
                    "secretKey": "PROVIDER_REGION", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("secret/data/{}", credential_path),
                        "property": "region"
                    }
                }
            ]
        }
    });

    // Use dynamic API to create ExternalSecret
    let gvk = GroupVersionKind::gvk("external-secrets.io", "v1beta1", "ExternalSecret");
    let api_resource = kube::api::ApiResource::from_gvk(&gvk);
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &api_resource);

    let external_secret_obj: DynamicObject = serde_json::from_value(external_secret)?;

    match api
        .create(&PostParams::default(), &external_secret_obj)
        .await
    {
        Ok(_) => {
            info!("ExternalSecret {} created successfully", name);
            Ok(())
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            debug!("ExternalSecret {} already exists", name);
            Ok(())
        }
        Err(e) => Err(Error::KubeError(e)),
    }
}

/// Create the Bolt agent Job for a BoltRun
#[allow(clippy::too_many_lines)]
pub async fn create_bolt_job(
    jobs_api: &Api<Job>,
    bolt_run: &BoltRun,
    _ctx: &Context,
) -> Result<()> {
    let job_name = bolt_run.job_name();
    let namespace = bolt_run
        .metadata
        .namespace
        .as_deref()
        .unwrap_or("cto-admin");

    info!("Creating Job {} in namespace {}", job_name, namespace);

    // Get cluster configuration
    let (node_count, cp_count, worker_count, plan) = match bolt_run.spec.provision.as_ref() {
        Some(provision) => match provision.cluster_size {
            ClusterSize::Small => (2, 1, 1, "c2-small-x86"),
            ClusterSize::Medium => (4, 1, 3, "c2-medium-x86"),
            ClusterSize::Large => (8, 3, 5, "c2-large-x86"),
        },
        None => (4, 1, 3, "c2-medium-x86"), // Default to medium
    };

    let provision = bolt_run.spec.provision.as_ref();
    let region = provision.map_or("DAL", |p| p.region.as_str());
    let talos_version = provision.map_or("v1.9.0", |p| p.talos_version.as_str());

    // Build installer command based on task type
    let command = match bolt_run.spec.task_type {
        BoltTaskType::Provision => {
            vec![
                "/usr/local/bin/installer".to_string(),
                "install".to_string(),
                "--cluster-name".to_string(),
                bolt_run.cluster_name(),
                "--region".to_string(),
                region.to_string(),
                "--cp-plan".to_string(),
                plan.to_string(),
                "--worker-plan".to_string(),
                plan.to_string(),
                "--nodes".to_string(),
                (cp_count + worker_count).to_string(),
                "--talos-version".to_string(),
                talos_version.to_string(),
                "--enable-vlan".to_string(),
                "--verbose".to_string(),
            ]
        }
        BoltTaskType::Debug => {
            vec![
                "/usr/local/bin/installer".to_string(),
                "debug".to_string(),
                "--cluster-name".to_string(),
                bolt_run.cluster_name(),
                "--verbose".to_string(),
            ]
        }
        BoltTaskType::Upgrade => {
            vec![
                "/usr/local/bin/installer".to_string(),
                "upgrade".to_string(),
                "--cluster-name".to_string(),
                bolt_run.cluster_name(),
                "--verbose".to_string(),
            ]
        }
        BoltTaskType::Destroy => {
            vec![
                "/usr/local/bin/installer".to_string(),
                "destroy".to_string(),
                "--cluster-name".to_string(),
                bolt_run.cluster_name(),
                "--force".to_string(),
                "--verbose".to_string(),
            ]
        }
    };

    // Build environment variables
    let mut env_vars = vec![
        EnvVar {
            name: "TENANT_ID".to_string(),
            value: Some(bolt_run.spec.tenant_ref.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "CLUSTER_NAME".to_string(),
            value: Some(bolt_run.cluster_name()),
            ..Default::default()
        },
        EnvVar {
            name: "NODE_COUNT".to_string(),
            value: Some(node_count.to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "BOLTRUN_NAME".to_string(),
            value: bolt_run.metadata.name.clone(),
            ..Default::default()
        },
        EnvVar {
            name: "BOLTRUN_NAMESPACE".to_string(),
            value: Some(namespace.to_string()),
            ..Default::default()
        },
    ];

    // Add model configuration
    env_vars.push(EnvVar {
        name: "BOLT_MODEL".to_string(),
        value: Some(bolt_run.spec.execution.model.clone()),
        ..Default::default()
    });

    // Build resource requirements
    let mut limits = BTreeMap::new();
    limits.insert("cpu".to_string(), Quantity("2".to_string()));
    limits.insert("memory".to_string(), Quantity("4Gi".to_string()));

    let mut requests = BTreeMap::new();
    requests.insert("cpu".to_string(), Quantity("500m".to_string()));
    requests.insert("memory".to_string(), Quantity("1Gi".to_string()));

    // Create labels
    let mut labels = BTreeMap::new();
    labels.insert(
        "cto.5dlabs.ai/tenant".to_string(),
        bolt_run.spec.tenant_ref.clone(),
    );
    labels.insert(
        "cto.5dlabs.ai/managed-by".to_string(),
        "bolt-controller".to_string(),
    );
    labels.insert(
        "cto.5dlabs.ai/task-type".to_string(),
        format!("{:?}", bolt_run.spec.task_type).to_lowercase(),
    );

    let retry_limit = i32::try_from(bolt_run.spec.execution.retry_limit).unwrap_or(i32::MAX);

    // Create the Job spec
    let job = Job {
        metadata: ObjectMeta {
            name: Some(job_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(labels.clone()),
            owner_references: Some(vec![OwnerReference {
                api_version: "cto.5dlabs.ai/v1alpha1".to_string(),
                kind: "BoltRun".to_string(),
                name: bolt_run.metadata.name.clone().unwrap_or_default(),
                uid: bolt_run.metadata.uid.clone().unwrap_or_default(),
                controller: Some(true),
                block_owner_deletion: Some(true),
            }]),
            ..Default::default()
        },
        spec: Some(JobSpec {
            backoff_limit: Some(retry_limit),
            ttl_seconds_after_finished: Some(3600), // Clean up after 1 hour
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    service_account_name: Some("bolt-agent".to_string()),
                    restart_policy: Some("Never".to_string()),
                    containers: vec![Container {
                        name: "bolt".to_string(),
                        // Use runtime:dev which includes the installer binary
                        // TODO: Change to runtime:latest once installer is in production images
                        image: Some("registry.5dlabs.ai/5dlabs/runtime:dev".to_string()),
                        command: Some(command),
                        env: Some(env_vars),
                        // BOLT-003: Inject credentials from ExternalSecret as env vars
                        env_from: Some(vec![EnvFromSource {
                            secret_ref: Some(SecretEnvSource {
                                name: bolt_run.external_secret_name(),
                                optional: Some(false),
                            }),
                            ..Default::default()
                        }]),
                        resources: Some(ResourceRequirements {
                            limits: Some(limits),
                            requests: Some(requests),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the Job
    match jobs_api.create(&PostParams::default(), &job).await {
        Ok(_) => {
            info!("Job {} created successfully", job_name);
            Ok(())
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            debug!("Job {} already exists", job_name);
            Ok(())
        }
        Err(e) => Err(Error::KubeError(e)),
    }
}
