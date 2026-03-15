//! Installation flow commands
//!
//! Handles the full installation:
//! 1. Check/install binaries (kind, kubectl, helm)
//! 2. Create Kind cluster
//! 3. Pull Docker images
//! 4. Deploy via Helm

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use tauri::{Emitter, State};

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::keychain::{self, CredentialKey};
use crate::runtime::{self as runtime, ContainerRuntime};

/// Installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStatus {
    pub step: InstallStep,
    pub message: String,
    pub progress: u8, // 0-100
    pub error: Option<String>,
}

/// Installation steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallStep {
    CheckingPrerequisites,
    InstallingBinaries,
    CreatingCluster,
    PullingImages,
    DeployingServices,
    ConfiguringIngress,
    Complete,
    Failed,
}

/// Binary check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryCheck {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Required binaries for CTO
const REQUIRED_BINARIES: &[&str] = &["docker", "kind", "kubectl", "helm", "argo"];

/// Binaries we can auto-install via brew
const BREW_INSTALLABLE: &[(&str, &str, bool)] = &[
    ("docker", "docker", true),
    ("kind", "kind", false),
    ("kubectl", "kubernetes-cli", false),
    ("helm", "helm", false),
    ("argo", "argo", false),
];

/// kind v0.31.0 default Kubernetes node image.
/// Keep this pinned to the release digest for reproducible local clusters.
const KIND_NODE_IMAGE: &str =
    "kindest/node:v1.35.0@sha256:452d707d4862f52530247495d180205e029056831160e22870e37e3f6c1ac31f";

/// Images to pull for initial deployment
const CORE_IMAGES: &[&str] = &[KIND_NODE_IMAGE];

/// Path to our Helm chart (relative to repo root)
const CHART_PATH: &str = "infra/charts/cto-lite";
const CTO_SUPPORT_CHART_PATH: &str = "infra/charts/cto";
const MORGAN_CHART_PATH: &str = "infra/charts/openclaw-agent";
const MORGAN_VALUES_PATH: &str = "infra/gitops/agents/morgan-values.yaml";
const CTO_SUPPORT_VALUES_PATH: &str =
    "crates/cto-lite/tauri/resources/helm/cto-local-support-values.yaml";
const REMOTE_RUNTIME_IMAGE_REPOSITORY: &str = "registry.5dlabs.ai/5dlabs/runtime";
const REMOTE_RUNTIME_IMAGE_TAG: &str = "full";
const LOCAL_AGENT_BASE_DOCKERFILE_PATH: &str = "infra/images/agents/Dockerfile";
const LOCAL_AGENT_BASE_IMAGE_REPOSITORY: &str = "cto/agents";
const LOCAL_AGENT_BASE_IMAGE_TAG_PREFIX: &str = "latest";
const LOCAL_RUNTIME_DOCKERFILE_PATH: &str = "infra/images/runtime/Dockerfile";
const LOCAL_RUNTIME_IMAGE_REPOSITORY: &str = "cto/runtime";
const LOCAL_RUNTIME_IMAGE_TAG_PREFIX: &str = "full-local";
const LOCAL_RUNTIME_TARGET: &str = "production";
const LOCAL_OPENCLAW_DOCKERFILE_PATH: &str = "infra/images/openclaw-platform/Dockerfile";
const LOCAL_OPENCLAW_IMAGE_REPOSITORY: &str = "cto/openclaw-platform";
const LOCAL_OPENCLAW_IMAGE_TAG_PREFIX: &str = "beta-local";
const LOCAL_OPENCLAW_CHANNEL: &str = "beta";
const KIND_CONTEXT: &str = "kind-cto-lite";
const LOCAL_PATH_PROVISIONER_URL: &str =
    "https://raw.githubusercontent.com/rancher/local-path-provisioner/v0.0.28/deploy/local-path-storage.yaml";
const KIND_INGRESS_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.14.3/deploy/static/provider/kind/deploy.yaml";
const MORGAN_INGRESS_MANIFEST_PATH: &str =
    "crates/cto-lite/tauri/resources/manifests/openclaw-morgan-ingress.yaml";
const LOCAL_NATS_MANIFEST_PATH: &str = "crates/cto-lite/tauri/resources/manifests/local-nats.yaml";

#[derive(Debug, Clone)]
struct LocalImageRef {
    repository: String,
    tag: String,
}

impl LocalImageRef {
    fn new(repository: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            tag: tag.into(),
        }
    }

    fn as_ref(&self) -> String {
        format!("{}:{}", self.repository, self.tag)
    }
}

/// Check for required binaries
#[tauri::command]
pub async fn check_prerequisites() -> Result<Vec<BinaryCheck>, AppError> {
    let mut results = Vec::new();

    for name in REQUIRED_BINARIES {
        let (found, path, version) = if *name == "docker" {
            let found = runtime::is_docker_available();
            let path = runtime::get_runtime_path(ContainerRuntime::Docker);
            let version = runtime::get_runtime_version(ContainerRuntime::Docker);
            (found, path, version)
        } else {
            let found = which::which(name).is_ok();
            let path = which::which(name)
                .ok()
                .map(|p| p.to_string_lossy().to_string());
            let version = if found {
                get_binary_version(name)
            } else {
                None
            };
            (found, path, version)
        };

        results.push(BinaryCheck {
            name: name.to_string(),
            found,
            path,
            version,
        });
    }

    Ok(results)
}

/// Check if Homebrew is installed
fn is_brew_installed() -> bool {
    which::which("brew").is_ok()
}

fn prepend_docker_path(command: &mut Command) {
    if let Some(docker_path) = runtime::get_runtime_path(ContainerRuntime::Docker) {
        if let Some(parent) = std::path::Path::new(&docker_path).parent() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let parent_path = parent.to_string_lossy();
            if !current_path.split(':').any(|entry| entry == parent_path) {
                command.env("PATH", format!("{}:{}", parent_path, current_path));
            }
        }
    }
}

fn kind_command() -> Command {
    let mut command = Command::new("kind");
    prepend_docker_path(&mut command);
    command
}

fn docker_command() -> Command {
    if let Some(path) = runtime::get_runtime_path(ContainerRuntime::Docker) {
        return Command::new(path);
    }

    let mut command = Command::new("docker");
    prepend_docker_path(&mut command);
    command
}

fn run_kubectl_kind(args: &[&str]) -> AppResult<std::process::Output> {
    Command::new("kubectl")
        .args(args)
        .args(["--context", KIND_CONTEXT])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))
}

fn ensure_namespace(name: &str) -> AppResult<()> {
    let exists = run_kubectl_kind(&["get", "namespace", name])
        .map(|output| output.status.success())
        .unwrap_or(false);

    if exists {
        return Ok(());
    }

    let output = run_kubectl_kind(&["create", "namespace", name])?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("already exists") {
        Ok(())
    } else {
        Err(AppError::CommandFailed(format!(
            "Failed to create namespace {}: {}",
            name, stderr
        )))
    }
}

fn apply_remote_manifest(url: &str) -> AppResult<()> {
    let output = run_kubectl_kind(&["apply", "-f", url])?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Failed to apply manifest {}: {}",
            url, stderr
        )))
    }
}

fn wait_for_deployment(namespace: &str, name: &str, timeout: &str) -> AppResult<()> {
    let output = run_kubectl_kind(&[
        "rollout",
        "status",
        &format!("deployment/{}", name),
        "-n",
        namespace,
        "--timeout",
        timeout,
    ])?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Timed out waiting for {} in {}: {}",
            name, namespace, stderr
        )))
    }
}

fn deployment_ready_replicas(namespace: &str, name: &str) -> AppResult<u32> {
    let output = run_kubectl_kind(&[
        "get",
        "deployment",
        name,
        "-n",
        namespace,
        "-o",
        "jsonpath={.status.readyReplicas}",
    ])?;

    if !output.status.success() {
        return Ok(0);
    }

    let ready = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(ready.parse::<u32>().unwrap_or(0))
}

fn ingress_host(namespace: &str, name: &str) -> AppResult<Option<String>> {
    let output = run_kubectl_kind(&[
        "get",
        "ingress",
        name,
        "-n",
        namespace,
        "-o",
        "jsonpath={.spec.rules[0].host}",
    ])?;

    if !output.status.success() {
        return Ok(None);
    }

    let host = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if host.is_empty() {
        Ok(None)
    } else {
        Ok(Some(host))
    }
}

fn existing_local_installation_ready() -> AppResult<bool> {
    if !kind_cluster_exists("cto-lite")? {
        return Ok(false);
    }

    let morgan_ready = deployment_ready_replicas("openclaw", "openclaw-morgan")? > 0;
    let tools_ready = deployment_ready_replicas("cto", "cto-tools")? > 0;
    let openmemory_ready = deployment_ready_replicas("cto", "cto-openmemory")? > 0;
    let nats_ready = deployment_ready_replicas("messaging", "nats")? > 0;
    let ingress = ingress_host("openclaw", "openclaw-morgan")?;

    Ok(morgan_ready
        && tools_ready
        && openmemory_ready
        && nats_ready
        && ingress.as_deref() == Some("morgan.localhost"))
}

fn persist_installation_complete(
    db: &Database,
    runtime_image: Option<&LocalImageRef>,
    openclaw_image: Option<&LocalImageRef>,
) -> AppResult<()> {
    if let Some(openclaw_image_ref) = openclaw_image {
        db.set_config("local_openclaw_image", &openclaw_image_ref.as_ref())?;
    }

    if let Some(runtime_image_ref) = runtime_image {
        db.set_config("local_runtime_image", &runtime_image_ref.as_ref())?;
    }

    db.set_config("installation_complete", "true")?;
    let _ = db.set_setup_progress(6);
    let _ = db.mark_setup_complete();

    Ok(())
}

fn install_local_path_provisioner() -> AppResult<()> {
    tracing::info!("Installing local-path provisioner for Kind");
    apply_remote_manifest(LOCAL_PATH_PROVISIONER_URL)?;

    let _ = run_kubectl_kind(&[
        "patch",
        "storageclass",
        "local-path",
        "-p",
        r#"{"metadata":{"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}"#,
    ]);

    Ok(())
}

fn install_kind_ingress_controller() -> AppResult<()> {
    tracing::info!("Installing ingress-nginx for Kind");
    apply_remote_manifest(KIND_INGRESS_MANIFEST_URL)?;
    wait_for_deployment("ingress-nginx", "ingress-nginx-controller", "240s")
}

fn get_repo_relative_path(relative_path: &str) -> AppResult<String> {
    let repo_path = std::path::Path::new(relative_path);
    if repo_path.exists() {
        return Ok(relative_path.to_string());
    }

    if let Ok(cwd) = std::env::current_dir() {
        let direct = cwd.join(relative_path);
        if direct.exists() {
            return Ok(direct.to_string_lossy().to_string());
        }

        let mut parent = cwd.parent();
        while let Some(path) = parent {
            let candidate = path.join(relative_path);
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().to_string());
            }
            parent = path.parent();
        }
    }

    Err(AppError::ConfigError(format!(
        "Could not find {} from the current working directory",
        relative_path
    )))
}

fn find_repo_root() -> AppResult<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;

    for candidate in cwd.ancestors() {
        if candidate.join(".git").exists() || candidate.join("infra").exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(AppError::ConfigError(
        "Could not determine the repository root for local image builds.".to_string(),
    ))
}

fn local_openclaw_image() -> LocalImageRef {
    LocalImageRef::new(
        LOCAL_OPENCLAW_IMAGE_REPOSITORY,
        format!(
            "{}-{}",
            LOCAL_OPENCLAW_IMAGE_TAG_PREFIX,
            local_container_arch_tag()
        ),
    )
}

fn local_runtime_image() -> LocalImageRef {
    LocalImageRef::new(
        LOCAL_RUNTIME_IMAGE_REPOSITORY,
        format!(
            "{}-{}",
            LOCAL_RUNTIME_IMAGE_TAG_PREFIX,
            local_container_arch_tag()
        ),
    )
}

fn remote_runtime_image() -> LocalImageRef {
    LocalImageRef::new(REMOTE_RUNTIME_IMAGE_REPOSITORY, REMOTE_RUNTIME_IMAGE_TAG)
}

fn local_agent_base_image() -> LocalImageRef {
    LocalImageRef::new(
        LOCAL_AGENT_BASE_IMAGE_REPOSITORY,
        format!(
            "{}-{}",
            LOCAL_AGENT_BASE_IMAGE_TAG_PREFIX,
            local_container_arch_tag()
        ),
    )
}

fn local_container_arch_tag() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" | "arm64" => "arm64",
        "x86_64" | "amd64" => "amd64",
        other => other,
    }
}

fn local_container_platform() -> &'static str {
    match local_container_arch_tag() {
        "arm64" => "linux/arm64",
        "amd64" => "linux/amd64",
        _ => "linux/amd64",
    }
}

fn docker_image_exists(image: &str) -> bool {
    docker_command()
        .args(["image", "inspect", image])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn remove_docker_image_if_exists(image: &str) -> AppResult<()> {
    if !docker_image_exists(image) {
        return Ok(());
    }

    let output = docker_command()
        .args(["image", "rm", "-f", image])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to remove image {}: {}", image, e)))?;

    if output.status.success() {
        tracing::info!("Removed obsolete image {}", image);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Failed to remove image {}: {}",
            image, stderr
        )))
    }
}

fn cleanup_legacy_local_images() -> AppResult<()> {
    let obsolete_images = match local_container_arch_tag() {
        "arm64" => vec![
            "cto/runtime:full-local".to_string(),
            "cto/runtime:full-local-amd64".to_string(),
            "cto/openclaw-platform:beta-local".to_string(),
            "cto/openclaw-platform:beta-local-amd64".to_string(),
        ],
        "amd64" => vec![
            "cto/runtime:full-local".to_string(),
            "cto/runtime:full-local-arm64".to_string(),
            "cto/openclaw-platform:beta-local".to_string(),
            "cto/openclaw-platform:beta-local-arm64".to_string(),
        ],
        _ => vec![
            "cto/runtime:full-local".to_string(),
            "cto/openclaw-platform:beta-local".to_string(),
        ],
    };

    for image in obsolete_images {
        remove_docker_image_if_exists(&image)?;
    }

    Ok(())
}

fn build_local_runtime_image(force_rebuild: bool) -> AppResult<LocalImageRef> {
    let image = local_runtime_image();
    let image_ref = image.as_ref();

    cleanup_legacy_local_images()?;

    if !force_rebuild && docker_image_exists(&image_ref) {
        tracing::info!("Reusing cached local runtime image {}", image_ref);
        return Ok(image);
    }

    let dockerfile_path = get_repo_relative_path(LOCAL_RUNTIME_DOCKERFILE_PATH)?;
    let repo_root = find_repo_root()?;
    let platform = local_container_platform();

    tracing::info!(
        "Building local runtime image {} from {} for {}",
        image_ref,
        dockerfile_path,
        platform
    );

    let repo_root_arg = repo_root.to_string_lossy().to_string();

    let output = docker_command()
        .args([
            "build",
            "--pull",
            "--platform",
            platform,
            "--target",
            LOCAL_RUNTIME_TARGET,
            "-f",
            &dockerfile_path,
            "-t",
            &image_ref,
            &repo_root_arg,
        ])
        .output()
        .map_err(|e| {
            AppError::CommandFailed(format!("Failed to build local runtime image: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Local runtime image build failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "Failed to build the local runtime image: {}",
            stderr
        )));
    }

    Ok(image)
}

fn build_local_agent_base_image(force_rebuild: bool) -> AppResult<LocalImageRef> {
    let image = local_agent_base_image();
    let image_ref = image.as_ref();

    if !force_rebuild && docker_image_exists(&image_ref) {
        tracing::info!("Reusing cached local agent base image {}", image_ref);
        return Ok(image);
    }

    let dockerfile_path = get_repo_relative_path(LOCAL_AGENT_BASE_DOCKERFILE_PATH)?;
    let repo_root = find_repo_root()?;
    let platform = local_container_platform();
    let runtime_image = remote_runtime_image();

    pull_image_required(&runtime_image.as_ref()).map_err(|error| {
        AppError::CommandFailed(format!(
            "Failed to pull the runtime image {} required for the local agent build: {}",
            runtime_image.as_ref(),
            error
        ))
    })?;

    tracing::info!(
        "Building local agent base image {} from {} for {}",
        image_ref,
        dockerfile_path,
        platform
    );

    let base_image_arg = format!("BASE_IMAGE={}", runtime_image.as_ref());
    let repo_root_arg = repo_root.to_string_lossy().to_string();

    let output = docker_command()
        .args([
            "build",
            "--pull",
            "--platform",
            platform,
            "-f",
            &dockerfile_path,
            "-t",
            &image_ref,
            "--build-arg",
            &base_image_arg,
            &repo_root_arg,
        ])
        .output()
        .map_err(|e| {
            AppError::CommandFailed(format!("Failed to build local agent base image: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Local agent base image build failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "Failed to build the local agent base image: {}",
            stderr
        )));
    }

    Ok(image)
}

fn pull_image_required(image: &str) -> AppResult<()> {
    tracing::info!("Pulling required image: {}", image);

    let output = docker_command()
        .args(["pull", image])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to pull image: {}", e)))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(AppError::CommandFailed(format!(
        "Failed to pull required image {}: {}",
        image, stderr
    )))
}

fn prepare_bootstrap_images() -> AppResult<(LocalImageRef, LocalImageRef)> {
    cleanup_legacy_local_images()?;

    let base_image = build_local_agent_base_image(false)?;
    let local_openclaw_image = build_local_openclaw_image(&base_image, false)?;
    load_image_to_kind(&base_image.as_ref(), "cto-lite")?;
    load_image_to_kind(&local_openclaw_image.as_ref(), "cto-lite")?;
    Ok((base_image, local_openclaw_image))
}

fn build_local_openclaw_image(
    base_image: &LocalImageRef,
    force_rebuild: bool,
) -> AppResult<LocalImageRef> {
    let image = local_openclaw_image();
    let image_ref = image.as_ref();

    if !force_rebuild && docker_image_exists(&image_ref) {
        tracing::info!("Reusing cached local OpenClaw image {}", image_ref);
        return Ok(image);
    }

    let dockerfile_path = get_repo_relative_path(LOCAL_OPENCLAW_DOCKERFILE_PATH)?;
    let repo_root = find_repo_root()?;
    let platform = local_container_platform();

    tracing::info!(
        "Building local OpenClaw image {} from {} for {}",
        image_ref,
        dockerfile_path,
        platform
    );

    let base_image_arg = format!("BASE_IMAGE={}", base_image.as_ref());
    let openclaw_version_arg = format!("OPENCLAW_VERSION={}", LOCAL_OPENCLAW_CHANNEL);
    let repo_root_arg = repo_root.to_string_lossy().to_string();

    let output = docker_command()
        .args([
            "build",
            "--pull",
            "--platform",
            platform,
            "-f",
            &dockerfile_path,
            "-t",
            &image_ref,
            "--build-arg",
            &base_image_arg,
            "--build-arg",
            &openclaw_version_arg,
            &repo_root_arg,
        ])
        .output()
        .map_err(|e| {
            AppError::CommandFailed(format!("Failed to build local OpenClaw image: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Local OpenClaw image build failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "Failed to build the local OpenClaw image: {}",
            stderr
        )));
    }

    Ok(image)
}

fn apply_morgan_gateway_ingress() -> AppResult<()> {
    ensure_namespace("openclaw")?;

    let manifest_path = get_repo_relative_path(MORGAN_INGRESS_MANIFEST_PATH)?;
    let output = run_kubectl_kind(&["apply", "-f", &manifest_path])?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Failed to apply Morgan ingress: {}",
            stderr
        )))
    }
}

fn apply_local_nats_manifest() -> AppResult<()> {
    ensure_namespace("messaging")?;

    let manifest_path = get_repo_relative_path(LOCAL_NATS_MANIFEST_PATH)?;
    let output = run_kubectl_kind(&["apply", "-f", &manifest_path])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "Failed to apply local NATS manifest: {}",
            stderr
        )));
    }

    wait_for_deployment("messaging", "nats", "180s")
}

fn apply_secret_manifest(namespace: &str, name: &str, data: &[(&str, String)]) -> AppResult<()> {
    let mut string_data = serde_json::Map::new();
    for (key, value) in data {
        string_data.insert((*key).to_string(), serde_json::Value::String(value.clone()));
    }

    let manifest = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": name,
            "namespace": namespace,
        },
        "type": "Opaque",
        "stringData": serde_json::Value::Object(string_data),
    });

    let mut child = Command::new("kubectl")
        .args(["apply", "--context", KIND_CONTEXT, "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to apply secret {}: {}", name, e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(manifest.to_string().as_bytes())?;
    }

    let output = child.wait_with_output().map_err(|e| {
        AppError::CommandFailed(format!("Failed to finish applying {}: {}", name, e))
    })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Failed to apply secret {}: {}",
            name, stderr
        )))
    }
}

fn apply_dockerconfig_secret(
    namespace: &str,
    name: &str,
    docker_config_json: &str,
) -> AppResult<()> {
    let manifest = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": name,
            "namespace": namespace,
        },
        "type": "kubernetes.io/dockerconfigjson",
        "stringData": {
            ".dockerconfigjson": docker_config_json,
        },
    });

    let mut child = Command::new("kubectl")
        .args(["apply", "--context", KIND_CONTEXT, "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::CommandFailed(format!(
                "Failed to apply docker config secret {}: {}",
                name, e
            ))
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(manifest.to_string().as_bytes())?;
    }

    let output = child.wait_with_output().map_err(|e| {
        AppError::CommandFailed(format!("Failed to finish applying {}: {}", name, e))
    })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::CommandFailed(format!(
            "Failed to apply docker config secret {}: {}",
            name, stderr
        )))
    }
}

fn read_secret_value(namespace: &str, name: &str, key: &str) -> AppResult<Option<String>> {
    let output = run_kubectl_kind(&["get", "secret", name, "-n", namespace, "-o", "json"])?;
    if !output.status.success() {
        return Ok(None);
    }

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        AppError::CommandFailed(format!(
            "Failed to parse secret {} in {}: {}",
            name, namespace, e
        ))
    })?;

    let Some(encoded) = payload
        .get("data")
        .and_then(|data| data.get(key))
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| {
            AppError::CommandFailed(format!("Failed to decode secret key {}: {}", key, e))
        })?;

    let value = String::from_utf8(decoded).map_err(|e| {
        AppError::CommandFailed(format!("Secret key {} was not valid UTF-8: {}", key, e))
    })?;

    Ok(Some(value))
}

fn sync_local_openclaw_secrets() -> AppResult<()> {
    ensure_namespace("openclaw")?;

    let openai = keychain::get_credential(CredentialKey::OpenAiApiKey)?;
    let anthropic = keychain::get_credential(CredentialKey::AnthropicApiKey)?;
    let github = keychain::get_credential(CredentialKey::GithubAccessToken)?;

    let mut api_key_data = Vec::new();
    if let Some(value) = openai {
        api_key_data.push(("openai-api-key", value));
    }
    if let Some(value) = anthropic {
        api_key_data.push(("anthropic-api-key", value));
    }
    if let Some(value) = github {
        api_key_data.push(("github-pat", value));
    }

    if api_key_data.is_empty() {
        tracing::warn!(
            "No local OpenAI or Anthropic API key was found; continuing bootstrap without provider credentials"
        );
    } else {
        apply_secret_manifest("openclaw", "openclaw-api-keys", &api_key_data)?;
    }

    apply_secret_manifest(
        "openclaw",
        "openclaw-discord-tokens",
        &[("morgan", "disabled".to_string())],
    )?;

    let docker_config = dirs::home_dir()
        .map(|home| home.join(".docker/config.json"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .filter(|content| !content.trim().is_empty())
        .unwrap_or_else(|| r#"{"auths":{}}"#.to_string());
    apply_dockerconfig_secret("openclaw", "ghcr-secret", &docker_config)?;

    Ok(())
}

fn sync_local_support_secrets() -> AppResult<()> {
    ensure_namespace("cto")?;

    let docker_config = dirs::home_dir()
        .map(|home| home.join(".docker/config.json"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .filter(|content| !content.trim().is_empty())
        .unwrap_or_else(|| r#"{"auths":{}}"#.to_string());
    apply_dockerconfig_secret("cto", "ghcr-secret", &docker_config)?;

    let github = read_secret_value("openclaw", "openclaw-api-keys", "github-pat")?
        .or(keychain::get_credential(CredentialKey::GithubAccessToken)?);
    if let Some(value) = github {
        apply_secret_manifest(
            "cto",
            "tools-github-secrets",
            &[("GITHUB_PERSONAL_ACCESS_TOKEN", value)],
        )?;
    }

    if let Some(value) = read_secret_value("openclaw", "openclaw-api-keys", "firecrawl-api-key")? {
        apply_secret_manifest(
            "cto",
            "tools-firecrawl-secrets",
            &[("FIRECRAWL_API_KEY", value)],
        )?;
    }

    let mut mirrored = Vec::new();
    for key in [
        "anthropic-api-key",
        "openai-api-key",
        "github-pat",
        "firecrawl-api-key",
        "brave-api-key",
        "factory-api-key",
        "op-service-account-token",
    ] {
        if let Some(value) = read_secret_value("openclaw", "openclaw-api-keys", key)? {
            mirrored.push((key, value));
        }
    }

    if !mirrored.is_empty() {
        apply_secret_manifest("cto", "openclaw-api-keys", &mirrored)?;
    }

    Ok(())
}

/// Install a binary via Homebrew
fn brew_install(formula: &str, cask: bool) -> AppResult<()> {
    tracing::info!(
        "Installing {} via Homebrew{}...",
        formula,
        if cask { " cask" } else { "" }
    );

    let mut args = vec!["install"];
    if cask {
        args.push("--cask");
    }
    args.push(formula);

    let output = Command::new("brew")
        .args(&args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run brew: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if already installed
        if stderr.contains("already installed") {
            return Ok(());
        }
        return Err(AppError::CommandFailed(format!(
            "brew install {}{} failed: {}",
            if cask { "--cask " } else { "" },
            formula,
            stderr
        )));
    }

    Ok(())
}

/// Install missing dependencies automatically
fn install_missing_dependencies(window: &tauri::Window) -> AppResult<()> {
    let emit = |msg: &str| {
        let _ = window.emit(
            "install-progress",
            InstallStatus {
                step: InstallStep::InstallingBinaries,
                message: msg.to_string(),
                progress: 10,
                error: None,
            },
        );
    };

    // Check what's missing
    let missing: Vec<_> = BREW_INSTALLABLE
        .iter()
        .filter(|(bin, _, _)| {
            if *bin == "docker" {
                !runtime::is_docker_available()
            } else {
                which::which(bin).is_err()
            }
        })
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    // Check if we can use Homebrew
    if !is_brew_installed() {
        // On macOS without brew, we could try other methods
        // For now, return an error with helpful message
        #[cfg(target_os = "macos")]
        {
            return Err(AppError::CommandFailed(
                "Homebrew is required to install dependencies. Install it from https://brew.sh"
                    .to_string(),
            ));
        }

        #[cfg(not(target_os = "macos"))]
        {
            let names: Vec<_> = missing.iter().map(|(bin, _)| *bin).collect();
            return Err(AppError::CommandFailed(format!(
                "Missing required tools: {}. Please install them manually.",
                names.join(", ")
            )));
        }
    }

    // Install each missing dependency
    for (bin, formula, cask) in missing {
        emit(&format!("Installing {}...", bin));
        brew_install(formula, *cask)?;
        tracing::info!("Successfully installed {}", bin);
    }

    Ok(())
}

/// Get version of a binary
fn get_binary_version(name: &str) -> Option<String> {
    let args: &[&str] = match name {
        "docker" => &["--version"],
        "kind" => &["version"],
        "kubectl" => &["version", "--client", "--short"],
        "helm" => &["version", "--short"],
        _ => &["--version"],
    };

    Command::new(name)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

/// Run the full installation
#[tauri::command]
pub async fn run_installation(
    db: State<'_, Database>,
    window: tauri::Window,
) -> Result<(), AppError> {
    // Helper to emit progress
    let emit_progress = |step: InstallStep, message: &str, progress: u8| {
        let _ = window.emit(
            "install-progress",
            InstallStatus {
                step,
                message: message.to_string(),
                progress,
                error: None,
            },
        );
    };

    if existing_local_installation_ready()? {
        tracing::info!("Existing local Kind + Morgan installation is already healthy");
        persist_installation_complete(&db, None, None)?;
        emit_progress(InstallStep::Complete, "Launching CTO...", 100);
        return Ok(());
    }

    // Step 1: Check prerequisites
    emit_progress(
        InstallStep::CheckingPrerequisites,
        "Preparing your local environment...",
        5,
    );

    let prereqs = check_prerequisites().await?;
    let missing: Vec<_> = prereqs.iter().filter(|b| !b.found).collect();

    // Auto-install missing dependencies, including the container runtime on supported platforms.
    if !missing.is_empty() {
        let missing_names: Vec<_> = missing.iter().map(|b| b.name.as_str()).collect();

        if !missing_names.is_empty() {
            emit_progress(
                InstallStep::InstallingBinaries,
                "Installing dependencies...",
                10,
            );
            install_missing_dependencies(&window)?;
        }
    }

    emit_progress(
        InstallStep::CheckingPrerequisites,
        "Preparing your local environment...",
        18,
    );
    let runtime_env = runtime::fully_auto_runtime()?;
    if !runtime_env.docker_available {
        return Err(AppError::CommandFailed(
            "A Docker-compatible runtime is still unavailable after bootstrap.".to_string(),
        ));
    }

    // Step 2: Create Kind cluster
    emit_progress(
        InstallStep::CreatingCluster,
        "Starting local services...",
        20,
    );

    if !kind_cluster_exists("cto-lite")? {
        create_kind_cluster()?;
    } else {
        tracing::info!("Kind cluster 'cto-lite' already exists");
    }

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Configuring the local runtime...",
        28,
    );
    install_local_path_provisioner()?;

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Configuring the local runtime...",
        32,
    );
    install_kind_ingress_controller()?;

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Configuring the local runtime...",
        36,
    );
    sync_local_openclaw_secrets()?;

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Configuring the local runtime...",
        40,
    );
    sync_local_support_secrets()?;

    emit_progress(
        InstallStep::PullingImages,
        "Preparing the local engine...",
        44,
    );

    for (i, image) in CORE_IMAGES.iter().enumerate() {
        let progress = 44 + ((i as u8 + 1) * 6 / CORE_IMAGES.len() as u8);
        emit_progress(
            InstallStep::PullingImages,
            "Preparing the local engine...",
            progress,
        );
        pull_image(image)?;
    }

    emit_progress(
        InstallStep::PullingImages,
        "Preparing the local engine...",
        50,
    );
    let (runtime_image, openclaw_image) = prepare_bootstrap_images()?;

    emit_progress(
        InstallStep::DeployingServices,
        "Starting local services...",
        56,
    );
    apply_local_nats_manifest()?;

    emit_progress(
        InstallStep::DeployingServices,
        "Starting local services...",
        62,
    );
    let cto_support_chart_path = get_cto_support_chart_path()?;
    let cto_support_values_path = get_cto_support_values_path()?;
    helm_install_cto_support(&cto_support_chart_path, &cto_support_values_path)?;

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Configuring the local runtime...",
        68,
    );
    apply_morgan_gateway_ingress()?;

    // Step 4: Add Helm repos and update dependencies
    emit_progress(
        InstallStep::PullingImages,
        "Preparing the local environment...",
        72,
    );
    add_argo_helm_repo()?;

    // Find chart path
    let chart_path = get_chart_path()?;
    tracing::info!("Using chart at: {}", chart_path);

    emit_progress(
        InstallStep::PullingImages,
        "Preparing the local environment...",
        76,
    );
    update_helm_dependencies(&chart_path)?;

    emit_progress(
        InstallStep::DeployingServices,
        "Starting local services...",
        82,
    );
    let morgan_chart_path = get_morgan_chart_path()?;
    let morgan_values_path = get_morgan_values_path()?;
    helm_install_morgan(&morgan_chart_path, &morgan_values_path, &openclaw_image)?;

    emit_progress(
        InstallStep::ConfiguringIngress,
        "Starting local services...",
        88,
    );
    wait_for_pods("openclaw")?;

    // Step 5: Deploy services via Helm
    emit_progress(InstallStep::DeployingServices, "Finalizing setup...", 92);
    helm_install(&chart_path, "cto-lite")?;

    // Step 6: Wait for services to be ready
    emit_progress(InstallStep::ConfiguringIngress, "Finalizing setup...", 96);
    wait_for_pods("cto-lite")?;

    // Complete
    emit_progress(InstallStep::Complete, "Launching CTO...", 100);

    // Mark installation done in DB
    persist_installation_complete(&db, Some(&runtime_image), Some(&openclaw_image))?;

    Ok(())
}

/// Check if Kind cluster exists
fn kind_cluster_exists(name: &str) -> AppResult<bool> {
    let output = kind_command()
        .args(["get", "clusters"])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to list Kind clusters: {}", e)))?;

    if output.status.success() {
        let clusters = String::from_utf8_lossy(&output.stdout);
        Ok(clusters.lines().any(|line| line.trim() == name))
    } else {
        Ok(false)
    }
}

/// Create Kind cluster with CTO config
fn create_kind_cluster() -> AppResult<()> {
    tracing::info!("Creating Kind cluster 'cto-lite'");

    let config = format!(
        r#"
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  image: {kind_node_image}
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
  - containerPort: 8080
    hostPort: 8080
    protocol: TCP
"#,
        kind_node_image = KIND_NODE_IMAGE
    );

    // Write config to temp file
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("cto-lite-kind-config.yaml");
    std::fs::write(&config_path, config)?;

    let output = kind_command()
        .args([
            "create",
            "cluster",
            "--name",
            "cto-lite",
            "--config",
            config_path.to_str().unwrap(),
            "--wait",
            "120s",
        ])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create cluster: {}", e)))?;

    // Clean up
    let _ = std::fs::remove_file(&config_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::ClusterError(format!(
            "Failed to create cluster: {}",
            stderr
        )));
    }

    tracing::info!("Kind cluster 'cto-lite' created successfully");
    Ok(())
}

/// Pull a Docker image
fn pull_image(image: &str) -> AppResult<()> {
    tracing::info!("Pulling image: {}", image);

    let output = docker_command()
        .args(["pull", image])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to pull image: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if image doesn't exist - it might not be published yet
        tracing::warn!("Failed to pull {}: {}", image, stderr);
    }

    Ok(())
}

/// Load image into Kind cluster
#[allow(dead_code)]
fn load_image_to_kind(image: &str, cluster: &str) -> AppResult<()> {
    tracing::info!("Loading image {} into cluster {}", image, cluster);

    let output = kind_command()
        .args(["load", "docker-image", image, "--name", cluster])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to load image: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "Failed to load {} into Kind: {}",
            image, stderr
        )));
    }

    Ok(())
}

/// Create Kubernetes namespace
#[allow(dead_code)]
fn create_namespace(name: &str) -> AppResult<()> {
    let output = Command::new("kubectl")
        .args(["create", "namespace", name, "--context", "kind-cto-lite"])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create namespace: {}", e)))?;

    // Ignore "already exists" errors
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            return Err(AppError::CommandFailed(format!(
                "Failed to create namespace: {}",
                stderr
            )));
        }
    }

    Ok(())
}

/// Add Argo Helm repository
fn add_argo_helm_repo() -> AppResult<()> {
    tracing::info!("Adding Argo Helm repository");

    let output = Command::new("helm")
        .args([
            "repo",
            "add",
            "argo",
            "https://argoproj.github.io/argo-helm",
        ])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to add Argo repo: {}", e)))?;

    // Ignore "already exists" errors
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            tracing::warn!("Failed to add Argo repo: {}", stderr);
        }
    }

    // Update repos
    let _ = Command::new("helm").args(["repo", "update"]).output();

    Ok(())
}

/// Update Helm chart dependencies
fn update_helm_dependencies(chart_path: &str) -> AppResult<()> {
    tracing::info!("Updating Helm dependencies for {}", chart_path);

    let output = Command::new("helm")
        .args(["dependency", "update", chart_path])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to update dependencies: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "Failed to update Helm dependencies: {}",
            stderr
        )));
    }

    Ok(())
}

/// Install CTO via Helm
fn helm_install(chart_path: &str, namespace: &str) -> AppResult<()> {
    tracing::info!("Installing CTO chart from {}", chart_path);

    // Check if release already exists
    let check = Command::new("helm")
        .args([
            "status",
            "cto-lite",
            "--namespace",
            namespace,
            "--kube-context",
            "kind-cto-lite",
        ])
        .output();

    let release_exists = check.map(|o| o.status.success()).unwrap_or(false);

    let mut args = vec![
        if release_exists { "upgrade" } else { "install" },
        "cto-lite",
        chart_path,
        "--namespace",
        namespace,
        "--create-namespace",
        "--kube-context",
        "kind-cto-lite",
        "--wait",
        "--timeout",
        "5m",
    ];

    if release_exists {
        args.push("--reuse-values");
    }

    let output = Command::new("helm")
        .args(&args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run helm: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Helm install failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "Helm install failed: {}",
            stderr
        )));
    }

    tracing::info!("CTO installed successfully");
    Ok(())
}

fn write_morgan_override_values(image: &LocalImageRef) -> AppResult<std::path::PathBuf> {
    let override_values = format!(
        r#"
namespace: openclaw
createNamespace: false
imagePullSecrets:
  - name: ghcr-secret
nodeSelector: null
tolerations:
  - key: "node-role.kubernetes.io/control-plane"
    operator: "Exists"
    effect: "NoSchedule"
  - key: "node-role.kubernetes.io/master"
    operator: "Exists"
    effect: "NoSchedule"
storage:
  storageClass: local-path
image:
  repository: {repository}
  tag: {tag}
  pullPolicy: IfNotPresent
discord:
  tokenSecretKey: henry
telemetry:
  enabled: false
cloudProviders:
  bedrock:
    enabled: false
"#,
        repository = image.repository,
        tag = image.tag
    );

    let path = std::env::temp_dir().join("cto-lite-morgan-kind-values.yaml");
    std::fs::write(&path, override_values.trim_start())?;
    Ok(path)
}

fn helm_install_morgan(
    chart_path: &str,
    values_path: &str,
    image: &LocalImageRef,
) -> AppResult<()> {
    tracing::info!("Installing local Morgan chart from {}", chart_path);

    let release = "openclaw-morgan";
    let namespace = "openclaw";
    let override_path = write_morgan_override_values(image)?;

    let check = Command::new("helm")
        .args([
            "status",
            release,
            "--namespace",
            namespace,
            "--kube-context",
            KIND_CONTEXT,
        ])
        .output();

    let release_exists = check.map(|output| output.status.success()).unwrap_or(false);

    let mut args = vec![
        if release_exists { "upgrade" } else { "install" },
        release,
        chart_path,
        "--namespace",
        namespace,
        "--create-namespace",
        "--kube-context",
        KIND_CONTEXT,
        "--wait",
        "--timeout",
        "8m",
        "-f",
        values_path,
        "-f",
        override_path.to_str().unwrap_or_default(),
    ];

    if release_exists {
        args.push("--reuse-values");
    }

    let output = Command::new("helm")
        .args(&args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run helm for Morgan: {}", e)))?;

    let _ = std::fs::remove_file(&override_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "Morgan helm install failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "Morgan helm install failed: {}",
            stderr
        )));
    }

    tracing::info!("Morgan deployed successfully");
    Ok(())
}

fn helm_install_cto_support(chart_path: &str, values_path: &str) -> AppResult<()> {
    tracing::info!("Installing local CTO support services from {}", chart_path);

    let release = "cto-support";
    let namespace = "cto";

    let check = Command::new("helm")
        .args([
            "status",
            release,
            "--namespace",
            namespace,
            "--kube-context",
            KIND_CONTEXT,
        ])
        .output();

    let release_exists = check.map(|output| output.status.success()).unwrap_or(false);

    let mut args = vec![
        if release_exists { "upgrade" } else { "install" },
        release,
        chart_path,
        "--namespace",
        namespace,
        "--create-namespace",
        "--kube-context",
        KIND_CONTEXT,
        "--wait",
        "--timeout",
        "12m",
        "-f",
        values_path,
    ];

    if release_exists {
        args.push("--reuse-values");
    }

    let output = Command::new("helm").args(&args).output().map_err(|e| {
        AppError::CommandFailed(format!(
            "Failed to run helm for local support services: {}",
            e
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "CTO support helm install failed:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
        return Err(AppError::CommandFailed(format!(
            "CTO support helm install failed: {}",
            stderr
        )));
    }

    tracing::info!("Local CTO support services deployed successfully");
    Ok(())
}

/// Wait for pods to be ready
fn wait_for_pods(namespace: &str) -> AppResult<()> {
    tracing::info!("Waiting for pods in namespace {} to be ready", namespace);

    // Wait up to 3 minutes for pods to be ready
    for _ in 0..36 {
        let output = Command::new("kubectl")
            .args([
                "get",
                "pods",
                "--namespace",
                namespace,
                "--context",
                "kind-cto-lite",
                "-o",
                "jsonpath={.items[*].status.phase}",
            ])
            .output();

        if let Ok(o) = output {
            if o.status.success() {
                let phases = String::from_utf8_lossy(&o.stdout);
                let all_running = phases
                    .split_whitespace()
                    .all(|p| p == "Running" || p == "Succeeded");

                if all_running && !phases.is_empty() {
                    tracing::info!("All pods are running");
                    return Ok(());
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // Don't fail - some pods might still be starting
    tracing::warn!("Timeout waiting for all pods, but continuing...");
    Ok(())
}

/// Get the chart path (handles both dev and production)
fn get_chart_path() -> AppResult<String> {
    // In development, use relative path from repo root
    // In production, chart is bundled in app resources

    // Try repo-relative path first
    let repo_path = std::path::Path::new(CHART_PATH);
    if repo_path.exists() {
        return Ok(CHART_PATH.to_string());
    }

    // Try from current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_path = cwd.join(CHART_PATH);
        if cwd_path.exists() {
            return Ok(cwd_path.to_string_lossy().to_string());
        }

        // Try parent directories (in case we're in a subdirectory)
        let mut parent = cwd.parent();
        while let Some(p) = parent {
            let chart_path = p.join(CHART_PATH);
            if chart_path.exists() {
                return Ok(chart_path.to_string_lossy().to_string());
            }
            parent = p.parent();
        }
    }

    Err(AppError::ConfigError(format!(
        "Could not find Helm chart at {}. Make sure you're running from the repository root.",
        CHART_PATH
    )))
}

fn get_morgan_chart_path() -> AppResult<String> {
    get_repo_relative_path(MORGAN_CHART_PATH)
}

fn get_morgan_values_path() -> AppResult<String> {
    get_repo_relative_path(MORGAN_VALUES_PATH)
}

fn get_cto_support_chart_path() -> AppResult<String> {
    get_repo_relative_path(CTO_SUPPORT_CHART_PATH)
}

fn get_cto_support_values_path() -> AppResult<String> {
    get_repo_relative_path(CTO_SUPPORT_VALUES_PATH)
}

/// Get installation status
#[tauri::command]
pub async fn get_install_status(db: State<'_, Database>) -> Result<bool, AppError> {
    let complete = db.get_config("installation_complete")?;
    if complete.as_deref() == Some("true") {
        return Ok(true);
    }

    if existing_local_installation_ready()? {
        tracing::info!("Recovered installation state from local Kind + Morgan health");
        persist_installation_complete(&db, None, None)?;
        return Ok(true);
    }

    Ok(false)
}

/// Delete installation (for testing/reset)
#[tauri::command]
pub async fn reset_installation(db: State<'_, Database>) -> Result<(), AppError> {
    tracing::info!("Resetting installation");

    // Delete Kind cluster
    let _ = kind_command()
        .args(["delete", "cluster", "--name", "cto-lite"])
        .output();

    // Reset DB flag
    db.set_config("installation_complete", "false")?;

    Ok(())
}
