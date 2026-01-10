//! Resource creation and management for tenants
//!
//! Creates Kubernetes resources for each tenant.

use k8s_openapi::api::core::v1::{Namespace, ServiceAccount};
use k8s_openapi::api::rbac::v1::{RoleBinding, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, DeleteParams, Patch, PatchParams, PostParams};
use kube::Client;
use serde_json::json;
use tracing::{info, warn};

use crate::crd::Tenant;
use crate::error::{Error, Result};

/// Create the tenant namespace
///
/// # Errors
/// Returns an error if namespace creation fails.
pub async fn create_namespace(client: &Client, tenant: &Tenant) -> Result<()> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let namespace_name = tenant.namespace_name();
    let tenant_name = tenant
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::InvalidConfig("Tenant must have a name".to_string()))?;

    let ns = Namespace {
        metadata: ObjectMeta {
            name: Some(namespace_name.clone()),
            labels: Some(
                [
                    (
                        "app.kubernetes.io/managed-by".to_string(),
                        "tenant-operator".to_string(),
                    ),
                    ("cto.5dlabs.ai/tenant".to_string(), tenant_name.clone()),
                    (
                        "cto.5dlabs.ai/tier".to_string(),
                        format!("{:?}", tenant.spec.tier).to_lowercase(),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    match namespaces.create(&PostParams::default(), &ns).await {
        Ok(_) => {
            info!(namespace = %namespace_name, "Created namespace");
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 409 => {
            info!(namespace = %namespace_name, "Namespace already exists");
            Ok(())
        }
        Err(e) => Err(Error::NamespaceCreation(e.to_string())),
    }
}

/// Delete the tenant namespace
///
/// # Errors
/// Returns an error if namespace deletion fails.
pub async fn delete_namespace(client: &Client, namespace: &str) -> Result<()> {
    let namespaces: Api<Namespace> = Api::all(client.clone());

    match namespaces.delete(namespace, &DeleteParams::default()).await {
        Ok(_) => {
            info!(namespace = %namespace, "Deleted namespace");
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!(namespace = %namespace, "Namespace already deleted");
            Ok(())
        }
        Err(e) => Err(Error::NamespaceCreation(e.to_string())),
    }
}

/// Create RBAC resources for the tenant
///
/// # Errors
/// Returns an error if RBAC setup fails.
pub async fn create_rbac(client: &Client, tenant: &Tenant) -> Result<()> {
    let namespace = tenant.namespace_name();
    let tenant_name = tenant
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::InvalidConfig("Tenant must have a name".to_string()))?;

    // Create ServiceAccount
    let service_accounts: Api<ServiceAccount> = Api::namespaced(client.clone(), &namespace);
    let sa = ServiceAccount {
        metadata: ObjectMeta {
            name: Some("tenant-agent".to_string()),
            namespace: Some(namespace.clone()),
            labels: Some(
                [
                    (
                        "app.kubernetes.io/managed-by".to_string(),
                        "tenant-operator".to_string(),
                    ),
                    ("cto.5dlabs.ai/tenant".to_string(), tenant_name.clone()),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    match service_accounts.create(&PostParams::default(), &sa).await {
        Ok(_) => info!(namespace = %namespace, "Created ServiceAccount"),
        Err(kube::Error::Api(e)) if e.code == 409 => {
            info!(namespace = %namespace, "ServiceAccount already exists");
        }
        Err(e) => return Err(Error::RbacSetup(e.to_string())),
    }

    // Create RoleBinding to allow agent operations
    let role_bindings: Api<RoleBinding> = Api::namespaced(client.clone(), &namespace);
    let rb = RoleBinding {
        metadata: ObjectMeta {
            name: Some("tenant-agent-binding".to_string()),
            namespace: Some(namespace.clone()),
            labels: Some(
                [
                    (
                        "app.kubernetes.io/managed-by".to_string(),
                        "tenant-operator".to_string(),
                    ),
                    ("cto.5dlabs.ai/tenant".to_string(), tenant_name.clone()),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: "edit".to_string(), // Standard edit role
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: "tenant-agent".to_string(),
            namespace: Some(namespace.clone()),
            ..Default::default()
        }]),
    };

    match role_bindings.create(&PostParams::default(), &rb).await {
        Ok(_) => info!(namespace = %namespace, "Created RoleBinding"),
        Err(kube::Error::Api(e)) if e.code == 409 => {
            info!(namespace = %namespace, "RoleBinding already exists");
        }
        Err(e) => return Err(Error::RbacSetup(e.to_string())),
    }

    Ok(())
}

/// Create `ExternalSecret` for the tenant's API keys
///
/// # Errors
/// Returns an error if the tenant has no name.
pub async fn create_external_secret(client: &Client, tenant: &Tenant) -> Result<()> {
    let namespace = tenant.namespace_name();
    let secret_name = tenant.external_secret_name();
    let tenant_name = tenant
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::InvalidConfig("Tenant must have a name".to_string()))?;

    // ExternalSecret CRD - using dynamic API
    let external_secrets = Api::<kube::core::DynamicObject>::namespaced_with(
        client.clone(),
        &namespace,
        &kube::discovery::ApiResource {
            group: "external-secrets.io".to_string(),
            version: "v1beta1".to_string(),
            kind: "ExternalSecret".to_string(),
            api_version: "external-secrets.io/v1beta1".to_string(),
            plural: "externalsecrets".to_string(),
        },
    );

    let external_secret = json!({
        "apiVersion": "external-secrets.io/v1beta1",
        "kind": "ExternalSecret",
        "metadata": {
            "name": secret_name,
            "namespace": namespace,
            "labels": {
                "app.kubernetes.io/managed-by": "tenant-operator",
                "cto.5dlabs.ai/tenant": tenant_name
            }
        },
        "spec": {
            "refreshInterval": "1h",
            "secretStoreRef": {
                "name": "openbao",
                "kind": "ClusterSecretStore"
            },
            "target": {
                "name": format!("{}-api-keys", tenant_name),
                "creationPolicy": "Owner"
            },
            "data": [
                {
                    "secretKey": "ANTHROPIC_API_KEY", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("tenants/{}/api-keys", tenant_name),
                        "property": "ANTHROPIC_API_KEY"
                    }
                },
                {
                    "secretKey": "OPENAI_API_KEY", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("tenants/{}/api-keys", tenant_name),
                        "property": "OPENAI_API_KEY"
                    }
                },
                {
                    "secretKey": "GEMINI_API_KEY", // pragma: allowlist secret
                    "remoteRef": {
                        "key": format!("tenants/{}/api-keys", tenant_name),
                        "property": "GEMINI_API_KEY"
                    }
                }
            ]
        }
    });

    match external_secrets
        .patch(
            &secret_name,
            &PatchParams::apply("tenant-operator"),
            &Patch::Apply(&external_secret),
        )
        .await
    {
        Ok(_) => {
            info!(namespace = %namespace, name = %secret_name, "Created ExternalSecret");
            Ok(())
        }
        Err(e) => {
            warn!(error = %e, "Failed to create ExternalSecret - this may be expected if ESO is not installed");
            // Don't fail the reconciliation if ESO is not installed
            Ok(())
        }
    }
}

/// Delete `ExternalSecret`
///
/// # Errors
/// Returns an error if `ExternalSecret` deletion fails.
pub async fn delete_external_secret(client: &Client, tenant: &Tenant) -> Result<()> {
    let namespace = tenant.namespace_name();
    let secret_name = tenant.external_secret_name();

    let external_secrets = Api::<kube::core::DynamicObject>::namespaced_with(
        client.clone(),
        &namespace,
        &kube::discovery::ApiResource {
            group: "external-secrets.io".to_string(),
            version: "v1beta1".to_string(),
            kind: "ExternalSecret".to_string(),
            api_version: "external-secrets.io/v1beta1".to_string(),
            plural: "externalsecrets".to_string(),
        },
    );

    match external_secrets
        .delete(&secret_name, &DeleteParams::default())
        .await
    {
        Ok(_) => {
            info!(namespace = %namespace, name = %secret_name, "Deleted ExternalSecret");
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!(namespace = %namespace, "ExternalSecret already deleted");
            Ok(())
        }
        Err(e) => Err(Error::ExternalSecret(e.to_string())),
    }
}

/// Create `ArgoCD` Application for tenant agents
///
/// # Errors
/// Returns an error if Application creation fails.
#[allow(clippy::too_many_lines)]
pub async fn create_argocd_app(client: &Client, tenant: &Tenant) -> Result<()> {
    let app_name = tenant.argocd_app_name();
    let namespace = tenant.namespace_name();
    let tenant_name = tenant
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::InvalidConfig("Tenant must have a name".to_string()))?;

    // Get enabled agents or use defaults
    let enabled_agents = tenant
        .spec
        .agents
        .as_ref()
        .map(|a| &a.enabled)
        .filter(|e| !e.is_empty())
        .map_or_else(
            || vec!["morgan".to_string(), "cleo".to_string(), "tess".to_string()],
            |agents| {
                agents
                    .iter()
                    .map(|a| format!("{a:?}").to_lowercase())
                    .collect::<Vec<_>>()
            },
        );

    // ArgoCD Application CRD - using dynamic API
    let applications = Api::<kube::core::DynamicObject>::namespaced_with(
        client.clone(),
        "argocd",
        &kube::discovery::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            kind: "Application".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            plural: "applications".to_string(),
        },
    );

    // Build agent config
    let agent_config: serde_json::Value = enabled_agents
        .iter()
        .map(|agent| (agent.clone(), json!({ "enabled": true })))
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into();

    let application = json!({
        "apiVersion": "argoproj.io/v1alpha1",
        "kind": "Application",
        "metadata": {
            "name": app_name,
            "namespace": "argocd",
            "labels": {
                "app.kubernetes.io/managed-by": "tenant-operator",
                "cto.5dlabs.ai/tenant": tenant_name
            },
            "finalizers": ["resources-finalizer.argocd.argoproj.io"]
        },
        "spec": {
            "project": "tenants",
            "source": {
                "repoURL": "https://github.com/5dlabs/cto",
                "targetRevision": "main",
                "path": "infra/charts/tenant-agents",
                "helm": {
                    "values": serde_yaml::to_string(&json!({
                        "tenant": {
                            "id": tenant_name,
                            "namespace": namespace
                        },
                        "agents": agent_config,
                        "secrets": {
                            "externalSecret": tenant.external_secret_name()
                        }
                    })).unwrap_or_default()
                }
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": namespace
            },
            "syncPolicy": {
                "automated": {
                    "prune": true,
                    "selfHeal": true
                },
                "syncOptions": ["CreateNamespace=false"]
            }
        }
    });

    match applications
        .patch(
            &app_name,
            &PatchParams::apply("tenant-operator"),
            &Patch::Apply(&application),
        )
        .await
    {
        Ok(_) => {
            info!(app = %app_name, "Created ArgoCD Application");
            Ok(())
        }
        Err(e) => {
            warn!(error = %e, "Failed to create ArgoCD Application");
            Err(Error::ArgoApp(e.to_string()))
        }
    }
}

/// Delete `ArgoCD` Application
///
/// # Errors
/// Returns an error if Application deletion fails.
pub async fn delete_argocd_app(client: &Client, tenant: &Tenant) -> Result<()> {
    let app_name = tenant.argocd_app_name();

    let applications = Api::<kube::core::DynamicObject>::namespaced_with(
        client.clone(),
        "argocd",
        &kube::discovery::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            kind: "Application".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            plural: "applications".to_string(),
        },
    );

    match applications
        .delete(&app_name, &DeleteParams::default())
        .await
    {
        Ok(_) => {
            info!(app = %app_name, "Deleted ArgoCD Application");
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!(app = %app_name, "ArgoCD Application already deleted");
            Ok(())
        }
        Err(e) => Err(Error::ArgoApp(e.to_string())),
    }
}
