//! Tenant controller - reconciliation logic
//!
//! Watches Tenant resources and reconciles desired state.

use std::sync::Arc;

use futures::StreamExt;
use kube::{
    api::{Api, Patch, PatchParams},
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
        watcher::Config,
    },
    Client, ResourceExt,
};
use tracing::{error, info, instrument, warn};

use crate::{
    crd::{Tenant, TenantPhase, TenantStatus},
    error::{Error, Result},
    resources,
};

/// Finalizer name for cleanup
const TENANT_FINALIZER: &str = "tenants.cto.5dlabs.ai/finalizer";

/// Controller context
pub struct Context {
    pub client: Client,
}

/// Run the tenant controller
pub async fn run_controller() -> Result<()> {
    let client = Client::try_default().await?;
    let tenants: Api<Tenant> = Api::all(client.clone());

    let context = Arc::new(Context { client });

    info!("Starting tenant operator controller");

    Controller::new(tenants, Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|result| async {
            match result {
                Ok((obj, _action)) => {
                    info!(tenant = %obj.name, "Reconciled successfully");
                }
                Err(e) => {
                    error!(error = %e, "Reconciliation error");
                }
            }
        })
        .await;

    Ok(())
}

/// Main reconciliation function
#[instrument(skip(tenant, ctx), fields(tenant = %tenant.name_any()))]
async fn reconcile(tenant: Arc<Tenant>, ctx: Arc<Context>) -> Result<Action> {
    let tenants: Api<Tenant> = Api::all(ctx.client.clone());

    finalizer(&tenants, TENANT_FINALIZER, tenant, |event| async {
        match event {
            FinalizerEvent::Apply(tenant) => apply(tenant, &ctx).await,
            FinalizerEvent::Cleanup(tenant) => cleanup(tenant, &ctx).await,
        }
    })
    .await
    .map_err(|e| Error::Finalizer(Box::new(e)))
}

/// Apply (create/update) tenant resources
#[instrument(skip(tenant, ctx), fields(tenant = %tenant.name_any()))]
async fn apply(tenant: Arc<Tenant>, ctx: &Context) -> Result<Action> {
    let name = tenant.name_any();
    let namespace = tenant.namespace_name();

    info!(tenant = %name, namespace = %namespace, "Reconciling tenant");

    // Update status to Provisioning
    update_status(&ctx.client, &name, TenantPhase::Provisioning, None).await?;

    // 1. Create namespace
    info!("Creating namespace: {}", namespace);
    resources::create_namespace(&ctx.client, &tenant).await?;

    // 2. Create RBAC
    info!("Setting up RBAC for namespace: {}", namespace);
    resources::create_rbac(&ctx.client, &tenant).await?;

    // 3. Create ExternalSecret
    info!("Creating ExternalSecret for tenant secrets");
    resources::create_external_secret(&ctx.client, &tenant).await?;

    // 4. Create ArgoCD Application
    info!("Creating ArgoCD Application for tenant agents");
    resources::create_argocd_app(&ctx.client, &tenant).await?;

    // Update status to Ready
    let status = TenantStatus {
        phase: TenantPhase::Ready,
        namespace: Some(namespace.clone()),
        argocd_app: Some(tenant.argocd_app_name()),
        external_secret_name: Some(tenant.external_secret_name()),
        conditions: vec![],
        observed_generation: tenant.metadata.generation,
    };
    update_status_full(&ctx.client, &name, status).await?;

    info!(tenant = %name, "Tenant provisioned successfully");

    // Requeue after 5 minutes for periodic reconciliation
    Ok(Action::requeue(std::time::Duration::from_secs(300)))
}

/// Cleanup tenant resources on deletion
#[instrument(skip(tenant, ctx), fields(tenant = %tenant.name_any()))]
async fn cleanup(tenant: Arc<Tenant>, ctx: &Context) -> Result<Action> {
    let name = tenant.name_any();
    let namespace = tenant.namespace_name();

    info!(tenant = %name, "Cleaning up tenant resources");

    // Delete in reverse order
    // 1. Delete ArgoCD Application
    if let Err(e) = resources::delete_argocd_app(&ctx.client, &tenant).await {
        warn!(error = %e, "Failed to delete ArgoCD app, continuing cleanup");
    }

    // 2. Delete ExternalSecret
    if let Err(e) = resources::delete_external_secret(&ctx.client, &tenant).await {
        warn!(error = %e, "Failed to delete ExternalSecret, continuing cleanup");
    }

    // 3. Delete namespace (this will cascade delete RBAC)
    if let Err(e) = resources::delete_namespace(&ctx.client, &namespace).await {
        warn!(error = %e, "Failed to delete namespace, continuing cleanup");
    }

    info!(tenant = %name, "Tenant cleanup completed");
    Ok(Action::await_change())
}

/// Update tenant status phase
async fn update_status(
    client: &Client,
    name: &str,
    phase: TenantPhase,
    message: Option<String>,
) -> Result<()> {
    let tenants: Api<Tenant> = Api::all(client.clone());

    let conditions = message.map(|m| {
        vec![serde_json::json!({
            "type": "Ready",
            "status": if phase == TenantPhase::Ready { "True" } else { "False" },
            "message": m,
        })]
    }).unwrap_or_default();

    let status = serde_json::json!({
        "status": {
            "phase": phase,
            "conditions": conditions
        }
    });

    tenants
        .patch_status(
            name,
            &PatchParams::apply("tenant-operator"),
            &Patch::Merge(&status),
        )
        .await?;

    Ok(())
}

/// Update tenant status with full status object
async fn update_status_full(client: &Client, name: &str, status: TenantStatus) -> Result<()> {
    let tenants: Api<Tenant> = Api::all(client.clone());

    let patch = serde_json::json!({ "status": status });

    tenants
        .patch_status(
            name,
            &PatchParams::apply("tenant-operator"),
            &Patch::Merge(&patch),
        )
        .await?;

    Ok(())
}

/// Error policy for the controller
fn error_policy(tenant: Arc<Tenant>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        tenant = %tenant.name_any(),
        error = %error,
        "Reconciliation failed"
    );

    // Exponential backoff on errors
    Action::requeue(std::time::Duration::from_secs(30))
}
