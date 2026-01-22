//! Watcher CodeRun management for dual-model execution pattern.
//!
//! When an executor CodeRun has `watcherConfig.enabled = true`, a paired watcher
//! CodeRun is created that monitors progress and reports issues via a coordination
//! file (ConfigMap).

use crate::cli::types::CLIType;
use crate::crds::coderun::{CLIConfig, CodeRunSpec, WatcherConfig};
use crate::crds::CodeRun;
use crate::tasks::types::{Context, Error, Result};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Patch, PatchParams, PostParams};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Watcher job name prefix.
const WATCHER_JOB_PREFIX: &str = "watcher-";

/// Hash a string to an 8-character hex string.
fn hash_string(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:08x}", hasher.finish() & 0xFFFF_FFFF)
}

/// Generate watcher CodeRun name from executor name.
#[must_use]
pub fn watcher_coderun_name(executor_name: &str) -> String {
    // Kubernetes names are limited to 63 characters
    const MAX_K8S_NAME_LENGTH: usize = 63;
    let prefix_len = WATCHER_JOB_PREFIX.len();
    let available = MAX_K8S_NAME_LENGTH.saturating_sub(prefix_len);

    if executor_name.len() <= available {
        format!("{WATCHER_JOB_PREFIX}{executor_name}")
    } else {
        // Truncate and add hash suffix for uniqueness
        let truncated = &executor_name[..available.saturating_sub(9)];
        let hash = hash_string(executor_name);
        format!("{WATCHER_JOB_PREFIX}{truncated}-{hash}")
    }
}

/// Generate coordination ConfigMap name.
#[must_use]
pub fn coordination_configmap_name(executor_name: &str) -> String {
    const MAX_K8S_NAME_LENGTH: usize = 63;
    const PREFIX: &str = "coord-";
    let available = MAX_K8S_NAME_LENGTH.saturating_sub(PREFIX.len());

    if executor_name.len() <= available {
        format!("{PREFIX}{executor_name}")
    } else {
        let truncated = &executor_name[..available.saturating_sub(9)];
        let hash = hash_string(executor_name);
        format!("{PREFIX}{truncated}-{hash}")
    }
}

/// Check if a CodeRun is a watcher CodeRun.
#[must_use]
pub fn is_watcher_coderun(code_run: &CodeRun) -> bool {
    code_run.spec.watcher_for.is_some() || code_run.spec.run_type == "watcher"
}

/// Check if watcher mode is enabled for a CodeRun.
#[must_use]
pub fn should_spawn_watcher(code_run: &CodeRun) -> bool {
    // Don't spawn watcher for a watcher
    if is_watcher_coderun(code_run) {
        return false;
    }

    code_run
        .spec
        .watcher_config
        .as_ref()
        .is_some_and(|config| config.enabled)
}

/// Create the coordination ConfigMap for executor/watcher communication.
///
/// The ConfigMap contains:
/// - `coordination.json`: Shared state between executor and watcher
///
/// # Errors
/// Returns error if ConfigMap creation fails.
#[allow(clippy::too_many_lines)]
pub async fn create_coordination_configmap(
    client: &kube::Client,
    namespace: &str,
    executor: &CodeRun,
    watcher_name: &str,
) -> Result<String> {
    let executor_name = executor.name_any();
    let configmap_name = coordination_configmap_name(&executor_name);

    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);

    // Check if already exists
    if configmaps.get(&configmap_name).await.is_ok() {
        debug!("Coordination ConfigMap {} already exists", configmap_name);
        return Ok(configmap_name);
    }

    let default_watcher_config = WatcherConfig::default();
    let watcher_config = executor
        .spec
        .watcher_config
        .as_ref()
        .unwrap_or(&default_watcher_config);

    let coordination_json = json!({
        "executor": {
            "status": "running",
            "currentStep": "starting",
            "stepNumber": 0,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "lastError": null,
            "attemptCount": 1,
            "coderunName": executor_name
        },
        "watcher": {
            "status": "starting",
            "lastCheck": null,
            "checkCount": 0,
            "coderunName": watcher_name
        },
        "issueQueue": [],
        "circuitBreaker": {
            "state": "closed",
            "failureCount": 0,
            "sameStepFailures": 0,
            "lastFailedStep": null,
            "lastError": null,
            "openedAt": null,
            "threshold": watcher_config.circuit_breaker_threshold
        },
        "session": {
            "id": executor.metadata.uid.as_deref().unwrap_or("unknown"),
            "startedAt": chrono::Utc::now().to_rfc3339(),
            "lastActivity": chrono::Utc::now().to_rfc3339()
        },
        "config": {
            "checkIntervalSecs": watcher_config.check_interval_secs,
            "service": executor.spec.service,
            "taskId": executor.spec.task_id,
            "repository": executor.spec.repository_url
        }
    });

    let mut data = BTreeMap::new();
    data.insert(
        "coordination.json".to_string(),
        serde_json::to_string_pretty(&coordination_json).map_err(|e| {
            Error::ConfigError(format!("Failed to serialize coordination JSON: {e}"))
        })?,
    );

    // Create owner reference to executor CodeRun
    let owner_ref = OwnerReference {
        api_version: "agents.platform/v1".to_string(),
        kind: "CodeRun".to_string(),
        name: executor_name.clone(),
        uid: executor.metadata.uid.clone().unwrap_or_default(),
        controller: Some(true),
        block_owner_deletion: Some(true),
    };

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            name: Some(configmap_name.clone()),
            namespace: Some(namespace.to_string()),
            owner_references: Some(vec![owner_ref]),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert(
                    "agents.platform/type".to_string(),
                    "coordination".to_string(),
                );
                labels.insert("executor-coderun".to_string(), executor_name.clone());
                labels.insert("watcher-coderun".to_string(), watcher_name.to_string());
                labels
            }),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    match configmaps.create(&PostParams::default(), &configmap).await {
        Ok(_) => {
            info!(
                "Created coordination ConfigMap {} for executor {}",
                configmap_name, executor_name
            );
            Ok(configmap_name)
        }
        Err(kube::Error::Api(api_err)) if api_err.code == 409 => {
            debug!(
                "Coordination ConfigMap {} already exists (conflict)",
                configmap_name
            );
            Ok(configmap_name)
        }
        Err(e) => Err(e.into()),
    }
}

/// Create a watcher CodeRun for the given executor CodeRun.
///
/// The watcher CodeRun:
/// - Uses the CLI and model from `watcherConfig`
/// - References the executor via `watcherFor`
/// - Shares the workspace PVC with the executor
/// - Has owner reference to executor for cleanup
///
/// # Errors
/// Returns error if CodeRun creation fails.
#[allow(clippy::too_many_lines)]
pub async fn create_watcher_coderun(
    client: &kube::Client,
    namespace: &str,
    executor: &CodeRun,
    coordination_configmap: &str,
) -> Result<String> {
    let executor_name = executor.name_any();
    let watcher_name = watcher_coderun_name(&executor_name);

    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

    // Check if already exists
    if coderuns.get(&watcher_name).await.is_ok() {
        debug!("Watcher CodeRun {} already exists", watcher_name);
        return Ok(watcher_name);
    }

    let watcher_config = executor
        .spec
        .watcher_config
        .as_ref()
        .ok_or_else(|| Error::ConfigError("Watcher config is required".to_string()))?;

    // Determine CLI type
    let cli_type = watcher_config
        .cli
        .as_ref()
        .and_then(|cli| CLIType::from_str_ci(cli))
        .unwrap_or(CLIType::Factory);

    // Build CLI config
    let cli_config = CLIConfig {
        cli_type,
        model: watcher_config
            .model
            .clone()
            .unwrap_or_else(|| "glm-4-plus".to_string()),
        settings: {
            let mut settings = HashMap::new();
            settings.insert(
                "template".to_string(),
                json!(watcher_config
                    .template
                    .clone()
                    .unwrap_or_else(|| "watcher/base".to_string())),
            );
            settings.insert(
                "checkIntervalSecs".to_string(),
                json!(watcher_config.check_interval_secs),
            );
            settings.insert(
                "circuitBreakerThreshold".to_string(),
                json!(watcher_config.circuit_breaker_threshold),
            );
            settings
        },
        max_tokens: None,
        temperature: None,
        model_rotation: None,
    };

    // Build environment variables
    let mut env = executor.spec.env.clone();
    env.insert(
        "COORDINATION_FILE".to_string(),
        "/workspace/coordination.json".to_string(),
    );
    env.insert("EXECUTOR_CODERUN".to_string(), executor_name.clone());
    env.insert(
        "CHECK_INTERVAL_SECS".to_string(),
        watcher_config.check_interval_secs.to_string(),
    );
    env.insert(
        "CIRCUIT_BREAKER_THRESHOLD".to_string(),
        watcher_config.circuit_breaker_threshold.to_string(),
    );

    // Build labels
    let mut labels = BTreeMap::new();
    labels.insert("agents.platform/type".to_string(), "watcher".to_string());
    labels.insert("executor-coderun".to_string(), executor_name.clone());
    labels.insert(
        "coordination-configmap".to_string(),
        coordination_configmap.to_string(),
    );
    if let Some(task_id) = executor.spec.task_id {
        labels.insert("task-id".to_string(), task_id.to_string());
    }
    if let Some(executor_labels) = &executor.metadata.labels {
        // Copy service and workflow-related labels
        for key in ["service", "workflow-stage", "pr-number"] {
            if let Some(value) = executor_labels.get(key) {
                labels.insert(key.to_string(), value.clone());
            }
        }
    }

    // Create owner reference to executor CodeRun
    let owner_ref = OwnerReference {
        api_version: "agents.platform/v1".to_string(),
        kind: "CodeRun".to_string(),
        name: executor_name.clone(),
        uid: executor.metadata.uid.clone().unwrap_or_default(),
        controller: Some(true),
        block_owner_deletion: Some(true),
    };

    let watcher_spec = CodeRunSpec {
        run_type: "watcher".to_string(),
        task_id: executor.spec.task_id,
        service: executor.spec.service.clone(),
        repository_url: executor.spec.repository_url.clone(),
        docs_repository_url: executor.spec.docs_repository_url.clone(),
        docs_project_directory: executor.spec.docs_project_directory.clone(),
        working_directory: executor.spec.working_directory.clone(),
        model: cli_config.model.clone(),
        prompt_style: None,
        github_user: executor.spec.github_user.clone(),
        github_app: executor.spec.github_app.clone(),
        context_version: executor.spec.context_version,
        docs_branch: executor.spec.docs_branch.clone(),
        continue_session: false,
        overwrite_memory: false,
        env,
        env_from_secrets: executor.spec.env_from_secrets.clone(),
        enable_docker: false, // Watcher doesn't need Docker
        task_requirements: None,
        service_account_name: executor.spec.service_account_name.clone(),
        cli_config: Some(cli_config),
        linear_integration: None, // Watcher doesn't need Linear
        prompt_modification: None,
        acceptance_criteria: None,
        remote_tools: None, // Watcher uses minimal tools
        local_tools: None,
        fresh_workspace: Some(false), // Share workspace with executor
        subtasks: None,
        watcher_config: None, // Watcher doesn't spawn another watcher
        watcher_for: Some(executor_name.clone()),
    };

    let watcher = CodeRun {
        metadata: ObjectMeta {
            name: Some(watcher_name.clone()),
            namespace: Some(namespace.to_string()),
            owner_references: Some(vec![owner_ref]),
            labels: Some(labels),
            ..Default::default()
        },
        spec: watcher_spec,
        status: None,
    };

    match coderuns.create(&PostParams::default(), &watcher).await {
        Ok(_) => {
            info!(
                "Created watcher CodeRun {} for executor {}",
                watcher_name, executor_name
            );
            Ok(watcher_name)
        }
        Err(kube::Error::Api(api_err)) if api_err.code == 409 => {
            debug!("Watcher CodeRun {} already exists (conflict)", watcher_name);
            Ok(watcher_name)
        }
        Err(e) => Err(e.into()),
    }
}

/// Spawn a watcher CodeRun if watcher mode is enabled for the executor.
///
/// This function:
/// 1. Creates the coordination ConfigMap
/// 2. Creates the watcher CodeRun
/// 3. Updates executor status with watcher info
///
/// # Errors
/// Returns error if any step fails.
pub async fn spawn_watcher_if_enabled(
    ctx: &Arc<Context>,
    executor: &CodeRun,
) -> Result<Option<String>> {
    if !should_spawn_watcher(executor) {
        return Ok(None);
    }

    let executor_name = executor.name_any();
    let watcher_name = watcher_coderun_name(&executor_name);

    info!(
        "Spawning watcher {} for executor {}",
        watcher_name, executor_name
    );

    // Create coordination ConfigMap
    let coordination_configmap =
        create_coordination_configmap(&ctx.client, &ctx.namespace, executor, &watcher_name).await?;

    // Create watcher CodeRun
    let watcher_name = create_watcher_coderun(
        &ctx.client,
        &ctx.namespace,
        executor,
        &coordination_configmap,
    )
    .await?;

    // Update executor status with watcher info
    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let status_patch = json!({
        "status": {
            "watcherCoderun": watcher_name,
            "coordinationConfigMap": coordination_configmap
        }
    });

    if let Err(e) = coderuns
        .patch_status(
            &executor_name,
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await
    {
        warn!("Failed to update executor status with watcher info: {}", e);
        // Don't fail - watcher is still created
    }

    Ok(Some(watcher_name))
}

/// Clean up watcher CodeRun when executor completes or fails.
///
/// Since watcher has owner reference to executor, Kubernetes will handle
/// deletion automatically. This function is for explicit cleanup if needed.
///
/// # Errors
/// Returns error if deletion fails (other than 404).
pub async fn cleanup_watcher(
    client: &kube::Client,
    namespace: &str,
    executor: &CodeRun,
) -> Result<()> {
    let executor_name = executor.name_any();
    let watcher_name = watcher_coderun_name(&executor_name);

    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

    match coderuns
        .delete(&watcher_name, &kube::api::DeleteParams::default())
        .await
    {
        Ok(_) => {
            info!(
                "Deleted watcher CodeRun {} for executor {}",
                watcher_name, executor_name
            );
            Ok(())
        }
        Err(kube::Error::Api(api_err)) if api_err.code == 404 => {
            debug!(
                "Watcher CodeRun {} already deleted or never created",
                watcher_name
            );
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_coderun_name() {
        assert_eq!(watcher_coderun_name("my-executor"), "watcher-my-executor");
    }

    #[test]
    fn test_watcher_coderun_name_truncation() {
        let long_name = "a".repeat(70);
        let watcher_name = watcher_coderun_name(&long_name);
        assert!(watcher_name.len() <= 63);
        assert!(watcher_name.starts_with("watcher-"));
    }

    #[test]
    fn test_coordination_configmap_name() {
        assert_eq!(
            coordination_configmap_name("my-executor"),
            "coord-my-executor"
        );
    }

    #[test]
    fn test_is_watcher_coderun() {
        let mut executor = CodeRun {
            metadata: ObjectMeta::default(),
            spec: CodeRunSpec::default(),
            status: None,
        };

        // Not a watcher
        assert!(!is_watcher_coderun(&executor));

        // Is a watcher via watcher_for
        executor.spec.watcher_for = Some("executor".to_string());
        assert!(is_watcher_coderun(&executor));

        // Is a watcher via run_type
        executor.spec.watcher_for = None;
        executor.spec.run_type = "watcher".to_string();
        assert!(is_watcher_coderun(&executor));
    }

    #[test]
    fn test_should_spawn_watcher() {
        let mut executor = CodeRun {
            metadata: ObjectMeta::default(),
            spec: CodeRunSpec::default(),
            status: None,
        };

        // No watcher config
        assert!(!should_spawn_watcher(&executor));

        // Watcher disabled
        executor.spec.watcher_config = Some(WatcherConfig {
            enabled: false,
            ..Default::default()
        });
        assert!(!should_spawn_watcher(&executor));

        // Watcher enabled
        executor.spec.watcher_config = Some(WatcherConfig {
            enabled: true,
            ..Default::default()
        });
        assert!(should_spawn_watcher(&executor));

        // Watcher should not spawn another watcher
        executor.spec.watcher_for = Some("parent-executor".to_string());
        assert!(!should_spawn_watcher(&executor));
    }
}
