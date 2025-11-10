use crate::crds::{CodeRun, DocsRun};
use futures::StreamExt;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher::Config;
use kube::{Api, Client, ResourceExt};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, Instrument};

pub mod cancel;
pub mod code;
pub mod config;
pub mod docs;
pub mod github;
pub mod label;
pub mod play;
pub mod template_paths;
pub mod tool_catalog;
pub mod types;
pub mod workflow;

// Re-export commonly used items
pub use code::reconcile_code_run;
pub use config::ControllerConfig;
pub use docs::reconcile_docs_run;
pub use types::{Error, Result};

// Context is crate-internal only
use types::Context;

/// Main entry point for the separated task controllers
#[instrument(skip(client), fields(namespace = %namespace))]
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    info!(
        "Starting separated task controllers in namespace: {}",
        namespace
    );

    debug!("Loading controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            debug!("Successfully loaded controller configuration");
            debug!("Configuration cleanup enabled = {}", cfg.cleanup.enabled);

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!("Configuration validation failed: {}", validation_error);
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            debug!("Configuration validation passed");
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration, using defaults: {}", e);
            debug!("Creating default configuration...");
            let default_config = ControllerConfig::default();

            // Validate default configuration
            if let Err(validation_error) = default_config.validate() {
                error!("Default configuration is invalid: {}", validation_error);
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            debug!("Default configuration validation passed");
            default_config
        }
    };

    debug!("Creating controller context...");

    // Create shared context
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    debug!("Controller context created successfully");

    // Startup visibility: list existing CodeRuns so we can see what the controller should observe
    {
        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), &namespace);
        match code_api.list(&ListParams::default()).await {
            Ok(list) => {
                info!(
                    "Controller startup: found {} CodeRun(s) in namespace {}",
                    list.items.len(),
                    namespace
                );
                for cr in list.items {
                    let name = cr.name_any();
                    let app = cr
                        .spec
                        .github_app
                        .clone()
                        .unwrap_or_else(|| "(none)".to_string());
                    let phase = cr
                        .status
                        .as_ref()
                        .map(|s| s.phase.clone())
                        .unwrap_or_else(String::new);
                    info!(
                        "Existing CodeRun: name={}, githubApp={}, phase='{}'",
                        name, app, phase
                    );
                }
            }
            Err(e) => {
                error!("Failed to list CodeRuns at startup: {}", e);
            }
        }
    }

    // NOTE: Periodic resync loop disabled due to performance regression
    // The thundering herd of reconciliations every 120s was causing excessive
    // CPU and memory usage. The kube-rs controller runtime already handles
    // requeues and watches properly, so this forced resync is unnecessary.
    //
    // If you need to re-enable for debugging stuck resources, consider:
    // 1. Much longer interval (e.g., 30+ minutes)
    // 2. Rate limiting the patches
    // 3. Only patching resources stuck for >N minutes

    // Run both controllers concurrently
    info!("Starting DocsRun and CodeRun controllers...");

    let docs_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move { run_docs_controller(client, namespace, context).await }
    });

    let code_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move { run_code_controller(client, namespace, context).await }
    });

    debug!("Both controllers started, waiting for completion...");

    // Wait for both controllers to complete (they should run indefinitely)
    match tokio::try_join!(docs_controller_handle, code_controller_handle) {
        Ok((docs_result, code_result)) => {
            if let Err(e) = docs_result {
                error!("DocsRun controller failed: {:?}", e);
            }
            if let Err(e) = code_result {
                error!("CodeRun controller failed: {:?}", e);
            }
        }
        Err(e) => {
            error!("Controller task join error: {:?}", e);
        }
    }

    info!("Task controller shutting down");
    Ok(())
}

/// Run the DocsRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_docs_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    info!("Starting DocsRun controller");

    let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), &namespace);
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(docs_api, watcher_config.clone())
        .owns(jobs_api, watcher_config)
        .run(reconcile_docs_run, error_policy_docs, context)
        .for_each(|reconciliation_result| {
            let docs_span = tracing::info_span!("docs_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(docs_run_resource) => {
                        info!(
                            resource = ?docs_run_resource,
                            "DocsRun reconciliation successful"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "DocsRun reconciliation error"
                        );
                    }
                }
            }
            .instrument(docs_span)
        })
        .await;

    info!("DocsRun controller shutting down");
    Ok(())
}

/// Run the CodeRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_code_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    info!("Starting CodeRun controller");

    let code_api: Api<CodeRun> = Api::namespaced(client.clone(), &namespace);
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(code_api, watcher_config.clone())
        .owns(jobs_api, watcher_config)
        .run(reconcile_code_run, error_policy_code, context)
        .for_each(|reconciliation_result| {
            let code_span = tracing::info_span!("code_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(code_run_resource) => {
                        info!(
                            resource = ?code_run_resource,
                            "CodeRun reconciliation successful"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "CodeRun reconciliation error"
                        );
                    }
                }
            }
            .instrument(code_span)
        })
        .await;

    info!("CodeRun controller shutting down");
    Ok(())
}

/// Error policy for DocsRun controller - limit to single retry
#[instrument(skip(_ctx), fields(docs_run_name = %_docs_run.name_any(), namespace = %_ctx.namespace))]
fn error_policy_docs(_docs_run: Arc<DocsRun>, err: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?err,
        docs_run_name = %_docs_run.name_any(),
        "DocsRun reconciliation failed - no retries, stopping"
    );
    // Don't retry - just stop on first failure
    Action::await_change()
}

/// Error policy for CodeRun controller - limit to single retry
#[instrument(skip(_ctx), fields(code_run_name = %_code_run.name_any(), namespace = %_ctx.namespace))]
fn error_policy_code(_code_run: Arc<CodeRun>, err: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?err,
        code_run_name = %_code_run.name_any(),
        "CodeRun reconciliation failed - no retries, stopping"
    );
    // Don't retry - just stop on first failure
    Action::await_change()
}
