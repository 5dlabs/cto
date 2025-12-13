//! Platform stack deployment for bare metal clusters.
//!
//! This module provides functions to deploy the CTO platform stack components
//! using Helm charts and kubectl.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn write_temp_yaml(prefix: &str, yaml: &str) -> Result<std::path::PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("{prefix}-{nanos}.yaml");
    let path = std::env::temp_dir().join(filename);
    fs::write(&path, yaml)
        .with_context(|| format!("Failed to write temp YAML to {}", path.display()))?;
    Ok(path)
}

fn kubectl_apply_yaml(kubeconfig: &Path, yaml: &str) -> Result<()> {
    let path = write_temp_yaml("cto-metal", yaml)?;
    let res = kubectl(
        kubeconfig,
        &["apply", "-f", path.to_string_lossy().as_ref()],
    )
    .map(|_| ());
    // Best-effort cleanup.
    let _ = fs::remove_file(&path);
    res
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

/// Deploy `OpenEBS` Replicated PV Mayastor via Helm.
///
/// Uses the Mayastor helm repository: `https://openebs.github.io/mayastor-extensions/`.
///
/// # Errors
///
/// Returns an error if helm commands fail.
pub fn deploy_mayastor(kubeconfig: &Path, namespace: &str, chart_version: &str) -> Result<()> {
    println!("   Deploying OpenEBS Mayastor...");

    // Add Mayastor repo
    let _ = helm(
        kubeconfig,
        &[
            "repo",
            "add",
            "mayastor",
            "https://openebs.github.io/mayastor-extensions/",
        ],
    );
    helm(kubeconfig, &["repo", "update"])?;

    // Install Mayastor
    helm(
        kubeconfig,
        &[
            "upgrade",
            "--install",
            "mayastor",
            "mayastor/mayastor",
            "--namespace",
            namespace,
            "--create-namespace",
            "--version",
            chart_version,
            "--wait",
        ],
    )?;

    println!("   ✅ Mayastor deployed");
    Ok(())
}

/// Create a Mayastor `DiskPool` on a specific node.
///
/// `disk_uri` should be something like `aio:///dev/disk/by-id/<id>` or `aio:///dev/nvme0n1`.
///
/// # Errors
///
/// Returns an error if applying the CR fails.
pub fn create_mayastor_diskpool(
    kubeconfig: &Path,
    namespace: &str,
    pool_name: &str,
    node_name: &str,
    disk_uri: &str,
) -> Result<()> {
    let yaml = format!(
        r#"apiVersion: "openebs.io/v1beta3"
kind: DiskPool
metadata:
  name: {pool_name}
  namespace: {namespace}
spec:
  node: {node_name}
  disks: ["{disk_uri}"]
"#
    );

    kubectl_apply_yaml(kubeconfig, &yaml)?;
    Ok(())
}

/// Create a Mayastor `StorageClass`.
///
/// # Errors
///
/// Returns an error if applying the `StorageClass` fails.
pub fn create_mayastor_storage_class(
    kubeconfig: &Path,
    name: &str,
    repl: u8,
    make_default: bool,
) -> Result<()> {
    let default_annotation = if make_default {
        r#"
  annotations:
    storageclass.kubernetes.io/is-default-class: "true""#
    } else {
        ""
    };

    let yaml = format!(
        r#"apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: {name}{default_annotation}
provisioner: io.openebs.csi-mayastor
parameters:
  protocol: nvmf
  repl: "{repl}"
"#
    );

    kubectl_apply_yaml(kubeconfig, &yaml)?;
    Ok(())
}

/// Run an fio benchmark Job against a PVC created from the given `StorageClass`.
///
/// Writes fio output to logs for retrieval via `kubectl logs job/<job_name>`.
///
/// # Errors
///
/// Returns an error if kubectl commands fail.
pub fn run_fio_benchmark_job(
    kubeconfig: &Path,
    namespace: &str,
    job_name: &str,
    storage_class: &str,
    pvc_size: &str,
    runtime_seconds: u32,
) -> Result<String> {
    // Namespace for the benchmark
    let _ = kubectl(kubeconfig, &["create", "ns", namespace]);

    let yaml = format!(
        r#"apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {job_name}-pvc
  namespace: {namespace}
spec:
  accessModes: ["ReadWriteOnce"]
  resources:
    requests:
      storage: {pvc_size}
  storageClassName: {storage_class}
---
apiVersion: batch/v1
kind: Job
metadata:
  name: {job_name}
  namespace: {namespace}
spec:
  backoffLimit: 0
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: fio
          image: alpine/fio
          args:
            - "--name=benchtest"
            - "--filename=/volume/testfile"
            - "--direct=1"
            - "--rw=randrw"
            - "--rwmixread=75"
            - "--bs=4k"
            - "--iodepth=64"
            - "--numjobs=1"
            - "--time_based"
            - "--runtime={runtime_seconds}"
            - "--group_reporting"
          volumeMounts:
            - name: vol
              mountPath: /volume
      volumes:
        - name: vol
          persistentVolumeClaim:
            claimName: {job_name}-pvc
"#
    );

    kubectl_apply_yaml(kubeconfig, &yaml)?;

    // Wait for job completion (up to 30m by default)
    let wait_timeout = "1800s";
    let _ = kubectl(
        kubeconfig,
        &[
            "wait",
            "--for=condition=complete",
            &format!("job/{job_name}"),
            "-n",
            namespace,
            "--timeout",
            wait_timeout,
        ],
    )?;

    // Fetch logs
    let logs = kubectl(
        kubeconfig,
        &["logs", &format!("job/{job_name}"), "-n", namespace],
    )?;
    Ok(logs)
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

/// Deploy Cilium CNI with `ClusterMesh` support for Talos Linux.
///
/// Cilium is installed with kube-proxy replacement, `WireGuard` encryption,
/// and Hubble observability. Each cluster needs a unique `cluster_name` and
/// `cluster_id` (1-255) for `ClusterMesh` connectivity.
///
/// **Important**: This uses Talos-specific capability settings that drop
/// `SYS_MODULE` (which Talos doesn't allow for Kubernetes workloads) and
/// configures cgroup mounts correctly for Talos.
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

    // Install Cilium with ClusterMesh-ready configuration and Talos-specific settings
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
            "1.15.6",
            // IPAM mode
            "--set",
            "ipam.mode=kubernetes",
            // Kube-proxy replacement
            "--set",
            "kubeProxyReplacement=true",
            "--set",
            "k8sServiceHost=localhost",
            "--set",
            "k8sServicePort=7445", // KubePrism port for Talos
            // Talos-specific: capabilities (drops SYS_MODULE which Talos blocks)
            "--set",
            "securityContext.capabilities.ciliumAgent={CHOWN,KILL,NET_ADMIN,NET_RAW,IPC_LOCK,SYS_ADMIN,SYS_RESOURCE,DAC_OVERRIDE,FOWNER,SETGID,SETUID}",
            "--set",
            "securityContext.capabilities.cleanCiliumState={NET_ADMIN,SYS_ADMIN,SYS_RESOURCE}",
            // Talos-specific: cgroup configuration (Talos pre-mounts these)
            "--set",
            "cgroup.autoMount.enabled=false",
            "--set",
            "cgroup.hostRoot=/sys/fs/cgroup",
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
