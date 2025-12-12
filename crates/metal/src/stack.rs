//! Platform stack deployment for bare metal clusters.
//!
//! This module provides functions to deploy the CTO platform stack components
//! using Helm charts and kubectl.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// `OpenBao` initialization response containing unseal keys and root token.
#[derive(Debug, Clone)]
pub struct OpenBaoInit {
    pub unseal_keys: Vec<String>,
    pub root_token: String,
}

/// Run a kubectl command with the given kubeconfig.
fn kubectl(kubeconfig: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("kubectl")
        .arg("--kubeconfig")
        .arg(kubeconfig)
        .args(args)
        .output()
        .context("Failed to execute kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kubectl failed: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run a helm command with the given kubeconfig.
fn helm(kubeconfig: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("helm")
        .env("KUBECONFIG", kubeconfig)
        .args(args)
        .output()
        .context("Failed to execute helm")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("helm failed: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Deploy local-path-provisioner for bare metal PVC support.
///
/// # Errors
///
/// Returns an error if kubectl commands fail.
pub fn deploy_local_path_provisioner(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying local-path-provisioner...");

    kubectl(
        kubeconfig,
        &[
            "apply",
            "-f",
            "https://raw.githubusercontent.com/rancher/local-path-provisioner/v0.0.26/deploy/local-path-storage.yaml",
        ],
    )?;

    // Set as default storage class
    kubectl(
        kubeconfig,
        &[
            "patch",
            "storageclass",
            "local-path",
            "-p",
            r#"{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}"#,
        ],
    )?;

    println!("   ✅ local-path-provisioner deployed");
    Ok(())
}

/// Deploy cert-manager for TLS certificate management.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_cert_manager(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying cert-manager...");

    // Add jetstack repo
    let _ = helm(
        kubeconfig,
        &["repo", "add", "jetstack", "https://charts.jetstack.io"],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install cert-manager
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "cert-manager",
            "jetstack/cert-manager",
            "--namespace",
            "cert-manager",
            "--create-namespace",
            "--set",
            "installCRDs=true",
            "--wait",
        ],
    )?;

    println!("   ✅ cert-manager deployed");
    Ok(())
}

/// Deploy `ArgoCD` for `GitOps`.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_argocd(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying ArgoCD...");

    // Add argo repo
    let _ = helm(
        kubeconfig,
        &[
            "repo",
            "add",
            "argo",
            "https://argoproj.github.io/argo-helm",
        ],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install ArgoCD
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "argocd",
            "argo/argo-cd",
            "--namespace",
            "argocd",
            "--create-namespace",
            "--wait",
        ],
    )?;

    println!("   ✅ ArgoCD deployed");
    Ok(())
}

/// Deploy `OpenBao` for secrets management.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_openbao(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying OpenBao...");

    // Add openbao repo
    let _ = helm(
        kubeconfig,
        &[
            "repo",
            "add",
            "openbao",
            "https://openbao.github.io/openbao-helm",
        ],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install OpenBao
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "openbao",
            "openbao/openbao",
            "--namespace",
            "openbao",
            "--create-namespace",
            "--set",
            "server.standalone.enabled=true",
            "--wait",
        ],
    )?;

    println!("   ✅ OpenBao deployed");
    Ok(())
}

/// Deploy ingress-nginx for ingress controller.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_ingress_nginx(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying ingress-nginx...");

    // Add ingress-nginx repo
    let _ = helm(
        kubeconfig,
        &[
            "repo",
            "add",
            "ingress-nginx",
            "https://kubernetes.github.io/ingress-nginx",
        ],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install ingress-nginx
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "ingress-nginx",
            "ingress-nginx/ingress-nginx",
            "--namespace",
            "ingress-nginx",
            "--create-namespace",
            "--wait",
        ],
    )?;

    println!("   ✅ ingress-nginx deployed");
    Ok(())
}

/// Deploy Argo Workflows for workflow automation.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_argo_workflows(kubeconfig: &Path) -> Result<()> {
    println!("   Deploying Argo Workflows...");

    // Add argo repo (already added by argocd, but safe to re-add)
    let _ = helm(
        kubeconfig,
        &[
            "repo",
            "add",
            "argo",
            "https://argoproj.github.io/argo-helm",
        ],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install Argo Workflows
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "argo-workflows",
            "argo/argo-workflows",
            "--namespace",
            "argo",
            "--create-namespace",
            "--wait",
        ],
    )?;

    println!("   ✅ Argo Workflows deployed");
    Ok(())
}

/// Initialize `OpenBao` and return unseal keys and root token.
///
/// # Errors
///
/// Returns an error if kubectl commands fail or JSON parsing fails.
pub fn init_openbao(kubeconfig: &Path) -> Result<OpenBaoInit> {
    println!("   Initializing OpenBao...");

    let output = kubectl(
        kubeconfig,
        &[
            "exec",
            "-n",
            "openbao",
            "openbao-0",
            "--",
            "bao",
            "operator",
            "init",
            "-key-shares=1",
            "-key-threshold=1",
            "-format=json",
        ],
    )?;

    // Parse JSON output
    let init: serde_json::Value =
        serde_json::from_str(&output).context("Failed to parse OpenBao init output")?;

    let unseal_keys = init["unseal_keys_b64"]
        .as_array()
        .context("Missing unseal_keys_b64")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let root_token = init["root_token"]
        .as_str()
        .context("Missing root_token")?
        .to_string();

    println!("   ✅ OpenBao initialized");
    Ok(OpenBaoInit {
        unseal_keys,
        root_token,
    })
}

/// Unseal `OpenBao` with the given key.
///
/// # Errors
///
/// Returns an error if kubectl commands fail.
pub fn unseal_openbao(kubeconfig: &Path, unseal_key: &str) -> Result<()> {
    println!("   Unsealing OpenBao...");

    kubectl(
        kubeconfig,
        &[
            "exec",
            "-n",
            "openbao",
            "openbao-0",
            "--",
            "bao",
            "operator",
            "unseal",
            unseal_key,
        ],
    )?;

    println!("   ✅ OpenBao unsealed");
    Ok(())
}

/// Get the `ArgoCD` admin password.
///
/// # Errors
///
/// Returns an error if kubectl commands fail.
pub fn get_argocd_password(kubeconfig: &Path) -> Result<String> {
    // Get the password directly decoded using kubectl's go-template
    let output = kubectl(
        kubeconfig,
        &[
            "get",
            "secret",
            "-n",
            "argocd",
            "argocd-initial-admin-secret",
            "-o",
            r"go-template={{.data.password | base64decode}}",
        ],
    )?;

    Ok(output.trim().to_string())
}

/// Deploy Cilium CNI with `ClusterMesh` support.
///
/// Cilium is installed with kube-proxy replacement, `WireGuard` encryption,
/// and Hubble observability. Each cluster needs a unique `cluster_name` and
/// `cluster_id` (1-255) for `ClusterMesh` connectivity.
///
/// # Arguments
///
/// * `kubeconfig` - Path to the kubeconfig file
/// * `cluster_name` - Unique name for this cluster in `ClusterMesh`
/// * `cluster_id` - Unique ID (1-255) for this cluster in `ClusterMesh`
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_cilium(kubeconfig: &Path, cluster_name: &str, cluster_id: u8) -> Result<()> {
    println!("   Deploying Cilium CNI (cluster: {cluster_name}, id: {cluster_id})...");

    // Add Cilium repo
    let _ = helm(
        kubeconfig,
        &["repo", "add", "cilium", "https://helm.cilium.io/"],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install Cilium with ClusterMesh-ready configuration
    let cluster_name_arg = format!("cluster.name={cluster_name}");
    let cluster_id_arg = format!("cluster.id={cluster_id}");

    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "cilium",
            "cilium/cilium",
            "--namespace",
            "kube-system",
            "--version",
            "1.16.4",
            // Kube-proxy replacement
            "--set",
            "kubeProxyReplacement=true",
            "--set",
            "k8sServiceHost=localhost",
            "--set",
            "k8sServicePort=7445", // KubePrism port for Talos
            // Cluster identity for ClusterMesh
            "--set",
            &cluster_name_arg,
            "--set",
            &cluster_id_arg,
            // WireGuard encryption
            "--set",
            "encryption.enabled=true",
            "--set",
            "encryption.type=wireguard",
            // Hubble observability
            "--set",
            "hubble.enabled=true",
            "--set",
            "hubble.relay.enabled=true",
            "--set",
            "hubble.ui.enabled=true",
            "--wait",
        ],
    )?;

    println!("   Cilium deployed successfully");
    Ok(())
}

/// Enable Cilium `ClusterMesh` for multi-cluster connectivity.
///
/// This deploys the clustermesh-apiserver and enables the cluster
/// to participate in `ClusterMesh` connections. Requires Cilium CLI.
///
/// # Errors
///
/// Returns an error if the cilium CLI fails.
pub fn enable_clustermesh(kubeconfig: &Path) -> Result<()> {
    use std::process::Command;

    println!("   Enabling Cilium ClusterMesh...");

    let output = Command::new("cilium")
        .env("KUBECONFIG", kubeconfig)
        .args(["clustermesh", "enable"])
        .output()
        .context("Failed to run cilium clustermesh enable")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if already enabled
        if stderr.contains("already enabled") {
            println!("   ClusterMesh is already enabled");
            return Ok(());
        }
        anyhow::bail!("Failed to enable ClusterMesh: {stderr}");
    }

    println!("   ClusterMesh enabled successfully");
    Ok(())
}
