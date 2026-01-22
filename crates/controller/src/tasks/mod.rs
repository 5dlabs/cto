use crate::crds::{BoltRun, CodeRun};
use futures::StreamExt;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher::Config;
use kube::{Api, Client, ResourceExt};
use notify::Notifier;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn, Instrument};

pub mod bolt;
pub mod cancel;
pub mod cleanup;
pub mod code;
pub mod config;
pub mod github;
pub mod heal;
pub mod label;
pub mod play;
pub mod security;
pub mod template_paths;
pub mod tool_catalog;
pub mod tool_inventory;
pub mod types;
pub mod workflow;

// Re-export commonly used items
pub use bolt::reconcile_bolt_run;
pub use code::reconcile_code_run;
pub use config::ControllerConfig;
pub use types::{Error, Result};

// Context is crate-internal only
use types::Context;

/// Main entry point for the separated task controllers
#[instrument(skip(client), fields(namespace = %namespace))]
#[allow(clippy::too_many_lines)] // Complex controller startup flow not easily split
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    info!(
        "Starting separated task controllers in namespace: {}",
        namespace
    );

    // Load controller configuration from mounted file or env var
    let config_path = std::env::var("CONTROLLER_CONFIG_PATH")
        .unwrap_or_else(|_| "/config/config.yaml".to_string());
    debug!("Loading controller configuration from: {}", config_path);

    let config = match ControllerConfig::from_mounted_file(&config_path) {
        Ok(cfg) => {
            debug!(
                "Successfully loaded controller configuration from {}",
                config_path
            );
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

    // Initialize notification system
    let notifier = Notifier::from_env();
    if notifier.has_channels() {
        info!(
            "Notification system enabled with {} channel(s)",
            notifier.channel_count()
        );
    } else {
        debug!("Notification system disabled (no channels configured)");
    }

    // Create shared context
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
        notifier: Arc::new(notifier),
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
                        .map_or_else(String::new, |s| s.phase.clone());
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

    // BOLT-002: Startup visibility for BoltRuns
    {
        let bolt_api: Api<BoltRun> = Api::namespaced(client.clone(), &namespace);
        match bolt_api.list(&ListParams::default()).await {
            Ok(list) => {
                info!(
                    "Controller startup: found {} BoltRun(s) in namespace {}",
                    list.items.len(),
                    namespace
                );
                for br in list.items {
                    let name = br.name_any();
                    let tenant = br.spec.tenant_ref.clone();
                    let phase = br
                        .status
                        .as_ref()
                        .map_or_else(String::new, |s| format!("{:?}", s.phase));
                    info!(
                        "Existing BoltRun: name={}, tenant={}, phase='{}'",
                        name, tenant, phase
                    );
                }
            }
            Err(e) => {
                // BoltRun CRD may not be installed yet, this is not fatal
                warn!(
                    "Failed to list BoltRuns at startup (CRD may not be installed): {}",
                    e
                );
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

    // Run both CodeRun and BoltRun controllers concurrently
    info!("Starting CodeRun and BoltRun controllers...");

    let code_context = context.clone();
    let bolt_context = context.clone();
    let code_client = client.clone();
    let bolt_client = client.clone();
    let code_namespace = namespace.clone();
    let bolt_namespace = namespace.clone();

    // Use tokio::select! to run both controllers concurrently
    // Either controller completing or failing will stop both
    tokio::select! {
        result = run_code_controller(code_client, code_namespace, code_context) => {
            match result {
                Ok(()) => info!("CodeRun controller completed"),
                Err(e) => error!("CodeRun controller failed: {:?}", e),
            }
        }
        result = run_bolt_controller(bolt_client, bolt_namespace, bolt_context) => {
            match result {
                Ok(()) => info!("BoltRun controller completed"),
                Err(e) => error!("BoltRun controller failed: {:?}", e),
            }
        }
    }

    info!("Task controller shutting down");
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

/// Error policy for CodeRun controller - limit to single retry
#[instrument(skip(_ctx), fields(code_run_name = %_code_run.name_any(), namespace = %_ctx.namespace))]
#[allow(clippy::used_underscore_binding)]
fn error_policy_code(_code_run: Arc<CodeRun>, err: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?err,
        code_run_name = %_code_run.name_any(),
        "CodeRun reconciliation failed - no retries, stopping"
    );
    // Don't retry - just stop on first failure
    Action::await_change()
}

/// BOLT-002: Run the BoltRun controller
///
/// Controller watches BoltRun resources in cto-admin namespace
/// On create: spawns Bolt agent pod with installer binary
/// Passes tenant config and credential ref to pod
/// Updates BoltRun status as pod progresses
/// Handles pod completion/failure appropriately
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_bolt_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    info!("Starting BoltRun controller for admin provisioning tasks");

    let bolt_api: Api<BoltRun> = Api::namespaced(client.clone(), &namespace);
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(bolt_api, watcher_config.clone())
        .owns(jobs_api, watcher_config)
        .run(reconcile_bolt_run, error_policy_bolt, context)
        .for_each(|reconciliation_result| {
            let bolt_span = tracing::info_span!("bolt_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(bolt_run_resource) => {
                        info!(
                            resource = ?bolt_run_resource,
                            "BoltRun reconciliation successful"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "BoltRun reconciliation error"
                        );
                    }
                }
            }
            .instrument(bolt_span)
        })
        .await;

    info!("BoltRun controller shutting down");
    Ok(())
}

/// Error policy for BoltRun controller
#[instrument(skip(_ctx), fields(bolt_run_name = %_bolt_run.name_any(), namespace = %_ctx.namespace))]
#[allow(clippy::used_underscore_binding)]
fn error_policy_bolt(_bolt_run: Arc<BoltRun>, err: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?err,
        bolt_run_name = %_bolt_run.name_any(),
        "BoltRun reconciliation failed - will retry"
    );
    // Retry after a delay for BoltRun - provisioning tasks may have transient failures
    Action::requeue(std::time::Duration::from_secs(30))
}
