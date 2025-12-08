//! Talos cluster bootstrapping utilities.
//!
//! This module provides functions for bootstrapping a Talos cluster,
//! including waiting for maintenance mode, generating configs, applying
//! configs, and bootstrapping the control plane.

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use tracing::{debug, info, warn};

/// Default Talos API port.
pub const TALOS_API_PORT: u16 = 50000;

/// Default Kubernetes API port.
pub const K8S_API_PORT: u16 = 6443;

/// Talos bootstrap configuration.
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Cluster name.
    pub cluster_name: String,
    /// Node IP address.
    pub node_ip: String,
    /// Install disk (e.g., `/dev/sda`).
    pub install_disk: String,
    /// Output directory for generated configs.
    pub output_dir: PathBuf,
    /// Talos version string (e.g., "v1.9.0").
    pub talos_version: String,
    /// Optional config patches (YAML) to apply.
    pub config_patches: Vec<String>,
}

impl BootstrapConfig {
    /// Create a new bootstrap configuration.
    #[must_use]
    pub fn new(cluster_name: impl Into<String>, node_ip: impl Into<String>) -> Self {
        Self {
            cluster_name: cluster_name.into(),
            node_ip: node_ip.into(),
            install_disk: "/dev/sda".to_string(),
            output_dir: PathBuf::from("/tmp/talos-bootstrap"),
            talos_version: "v1.9.0".to_string(),
            config_patches: Vec::new(),
        }
    }

    /// Set the install disk.
    #[must_use]
    pub fn with_install_disk(mut self, disk: impl Into<String>) -> Self {
        self.install_disk = disk.into();
        self
    }

    /// Set the output directory.
    #[must_use]
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set the Talos version.
    #[must_use]
    pub fn with_talos_version(mut self, version: impl Into<String>) -> Self {
        self.talos_version = version.into();
        self
    }

    /// Add a config patch (inline YAML).
    #[must_use]
    pub fn with_config_patch(mut self, patch: impl Into<String>) -> Self {
        self.config_patches.push(patch.into());
        self
    }

    /// Configure kube-proxy mode (iptables or ipvs).
    /// Use "disabled" to disable kube-proxy entirely (for Cilium replacement).
    #[must_use]
    pub fn with_kube_proxy_mode(self, mode: &str) -> Self {
        let patch = match mode {
            "disabled" => "cluster:\n  proxy:\n    disabled: true\n".to_string(),
            "ipvs" => "cluster:\n  proxy:\n    mode: ipvs\n".to_string(),
            _ => "cluster:\n  proxy:\n    mode: iptables\n".to_string(),
        };
        self.with_config_patch(patch)
    }
}

/// Wait for Talos to be reachable in maintenance mode.
///
/// Polls the Talos API port until it responds or timeout is reached.
///
/// # Errors
///
/// Returns an error if the timeout is reached before Talos responds.
pub fn wait_for_talos(ip: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    let addr = format!("{ip}:{TALOS_API_PORT}");

    info!(
        "Waiting for Talos at {addr} (timeout: {}s)...",
        timeout.as_secs()
    );

    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for Talos at {addr}");
        }

        match TcpStream::connect_timeout(
            &addr.parse().context("Invalid address")?,
            Duration::from_secs(5),
        ) {
            Ok(_) => {
                info!("✅ Talos is responding at {addr}");
                return Ok(());
            }
            Err(e) => {
                debug!("Connection attempt failed: {e}");
                std::thread::sleep(Duration::from_secs(10));
            }
        }
    }
}

/// Check if talosctl is installed.
///
/// # Errors
///
/// Returns an error if talosctl is not installed or not working.
pub fn check_talosctl() -> Result<()> {
    let output = Command::new("talosctl")
        .arg("version")
        .arg("--client")
        .output()
        .context("Failed to run talosctl - is it installed?")?;

    if !output.status.success() {
        bail!("talosctl is not working properly");
    }

    let version = String::from_utf8_lossy(&output.stdout);
    debug!("talosctl version: {}", version.trim());
    Ok(())
}

/// Generate Talos secrets.
///
/// Creates a secrets.yaml file that contains cluster secrets (certs, keys, etc).
///
/// # Errors
///
/// Returns an error if the output directory cannot be created or talosctl fails.
pub fn generate_secrets(output_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    let secrets_path = output_dir.join("secrets.yaml");

    info!("Generating Talos secrets...");
    let output = Command::new("talosctl")
        .args(["gen", "secrets", "-o"])
        .arg(&secrets_path)
        .output()
        .context("Failed to run talosctl gen secrets")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to generate secrets: {stderr}");
    }

    info!("✅ Secrets generated: {}", secrets_path.display());
    Ok(secrets_path)
}

/// Generate Talos machine configuration.
///
/// Creates controlplane.yaml, worker.yaml, and talosconfig files.
///
/// # Errors
///
/// Returns an error if secrets are missing or talosctl fails.
pub fn generate_config(config: &BootstrapConfig) -> Result<GeneratedConfigs> {
    let secrets_path = config.output_dir.join("secrets.yaml");
    if !secrets_path.exists() {
        bail!("Secrets file not found. Run generate_secrets first.");
    }

    let endpoint = format!("https://{}:{K8S_API_PORT}", config.node_ip);

    info!(
        "Generating Talos config for cluster '{}'...",
        config.cluster_name
    );

    let mut cmd = Command::new("talosctl");
    cmd.args([
        "gen",
        "config",
        &config.cluster_name,
        &endpoint,
        "--with-secrets",
    ])
    .arg(&secrets_path)
    .args(["--output-dir"])
    .arg(&config.output_dir)
    .args(["--install-disk", &config.install_disk]);

    // Add any config patches
    for patch in &config.config_patches {
        cmd.args(["--config-patch", patch]);
    }

    let output = cmd.output().context("Failed to run talosctl gen config")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to generate config: {stderr}");
    }

    let configs = GeneratedConfigs {
        controlplane: config.output_dir.join("controlplane.yaml"),
        worker: config.output_dir.join("worker.yaml"),
        talosconfig: config.output_dir.join("talosconfig"),
    };

    info!("✅ Configs generated:");
    info!("   - {}", configs.controlplane.display());
    info!("   - {}", configs.worker.display());
    info!("   - {}", configs.talosconfig.display());

    Ok(configs)
}

/// Paths to generated Talos configuration files.
#[derive(Debug, Clone)]
pub struct GeneratedConfigs {
    /// Path to controlplane.yaml.
    pub controlplane: PathBuf,
    /// Path to worker.yaml.
    pub worker: PathBuf,
    /// Path to talosconfig.
    pub talosconfig: PathBuf,
}

/// Apply Talos machine configuration to a node.
///
/// This triggers the installation of Talos to disk and a reboot.
///
/// # Errors
///
/// Returns an error if talosctl fails to apply the config.
pub fn apply_config(node_ip: &str, config_path: &Path) -> Result<()> {
    info!("Applying Talos config to {node_ip}...");
    info!("  Config: {}", config_path.display());

    let output = Command::new("talosctl")
        .args(["apply-config", "--insecure", "--nodes", node_ip, "--file"])
        .arg(config_path)
        .output()
        .context("Failed to run talosctl apply-config")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to apply config: {stderr}");
    }

    info!("✅ Config applied! Node will install and reboot.");
    Ok(())
}

/// Wait for Talos to be ready after installation.
///
/// After applying config, the node reboots and installs Talos to disk.
/// This waits for it to come back up in normal (non-maintenance) mode.
///
/// # Errors
///
/// Returns an error if the timeout is reached before Talos is healthy.
pub fn wait_for_install(node_ip: &str, talosconfig: &Path, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    let addr = format!("{node_ip}:{TALOS_API_PORT}");

    info!(
        "Waiting for Talos installation to complete (timeout: {}s)...",
        timeout.as_secs()
    );

    // First wait for the node to go down (reboot)
    info!("Waiting for node to reboot...");
    std::thread::sleep(Duration::from_secs(30));

    // Then wait for it to come back up
    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for Talos installation at {addr}");
        }

        // Try to connect
        if TcpStream::connect_timeout(
            &addr.parse().context("Invalid address")?,
            Duration::from_secs(5),
        )
        .is_ok()
        {
            // Port is open, try talosctl version (health requires bootstrap)
            // Note: We use -e (endpoint) because talosconfig may have empty endpoints
            let output = Command::new("talosctl")
                .args(["--talosconfig"])
                .arg(talosconfig)
                .args(["-e", node_ip, "-n", node_ip, "version"])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if stdout.contains("Server:") {
                        info!("✅ Talos is responding after install!");
                        return Ok(());
                    }
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    debug!("Version check not ready: {stderr}");
                }
                Err(e) => {
                    debug!("Version check failed: {e}");
                }
            }
        }

        debug!("Node not ready yet, waiting...");
        std::thread::sleep(Duration::from_secs(15));
    }
}

/// Bootstrap the Talos cluster.
///
/// This initializes etcd and the Kubernetes control plane on the first node.
///
/// # Errors
///
/// Returns an error if talosctl bootstrap fails (unless already bootstrapped).
pub fn bootstrap_cluster(node_ip: &str, talosconfig: &Path) -> Result<()> {
    info!("Bootstrapping Talos cluster on {node_ip}...");

    // Note: We use -e (endpoint) because talosconfig may have empty endpoints
    let output = Command::new("talosctl")
        .args(["--talosconfig"])
        .arg(talosconfig)
        .args(["-e", node_ip, "-n", node_ip, "bootstrap"])
        .output()
        .context("Failed to run talosctl bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if already bootstrapped
        if stderr.contains("already bootstrapped") || stderr.contains("etcd is already running") {
            warn!("Cluster appears to already be bootstrapped");
            return Ok(());
        }
        bail!("Failed to bootstrap: {stderr}");
    }

    info!("✅ Cluster bootstrapped!");
    Ok(())
}

/// Wait for Kubernetes API to be ready.
///
/// # Errors
///
/// Returns an error if the timeout is reached before Kubernetes is ready.
pub fn wait_for_kubernetes(node_ip: &str, talosconfig: &Path, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    let k8s_addr = format!("{node_ip}:{K8S_API_PORT}");

    info!(
        "Waiting for Kubernetes API to be ready (timeout: {}s)...",
        timeout.as_secs()
    );

    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for Kubernetes API");
        }

        // First check if K8s API port is open
        if TcpStream::connect_timeout(
            &k8s_addr.parse().context("Invalid address")?,
            Duration::from_secs(5),
        )
        .is_ok()
        {
            // Port is open, try talosctl health
            // Note: We use -e (endpoint) because talosconfig may have empty endpoints
            let output = Command::new("talosctl")
                .args(["--talosconfig"])
                .arg(talosconfig)
                .args([
                    "-e",
                    node_ip,
                    "-n",
                    node_ip,
                    "health",
                    "--wait-timeout",
                    "30s",
                ])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    info!("✅ Kubernetes API is ready!");
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&out.stderr);
                debug!("Health check: {stderr}");
            }
        }

        debug!("Kubernetes not ready yet...");
        std::thread::sleep(Duration::from_secs(10));
    }
}

/// Get kubeconfig from the Talos cluster.
///
/// # Errors
///
/// Returns an error if talosctl fails to retrieve the kubeconfig.
pub fn get_kubeconfig(node_ip: &str, talosconfig: &Path, output_path: &Path) -> Result<()> {
    info!("Fetching kubeconfig...");

    // Note: We need to specify -e (endpoint) because talosconfig may have empty endpoints
    let output = Command::new("talosctl")
        .args(["--talosconfig"])
        .arg(talosconfig)
        .args(["-e", node_ip, "-n", node_ip, "kubeconfig"])
        .arg(output_path)
        .output()
        .context("Failed to run talosctl kubeconfig")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get kubeconfig: {stderr}");
    }

    info!("✅ Kubeconfig saved to: {}", output_path.display());
    Ok(())
}

/// Wait for Kubernetes node to be Ready.
///
/// # Errors
///
/// Returns an error if the timeout is reached before the node is Ready.
pub fn wait_for_node_ready(kubeconfig: &Path, timeout: Duration) -> Result<()> {
    let start = Instant::now();

    info!(
        "Waiting for Kubernetes node to be Ready (timeout: {}s)...",
        timeout.as_secs()
    );

    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for node to be Ready");
        }

        let output = Command::new("kubectl")
            .args(["--kubeconfig"])
            .arg(kubeconfig)
            .args([
                "get",
                "nodes",
                "-o",
                "jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.contains("True") {
                    info!("✅ Kubernetes node is Ready!");
                    return Ok(());
                }
            }
        }

        debug!("Node not ready yet...");
        std::thread::sleep(Duration::from_secs(5));
    }
}

/// Full bootstrap workflow.
///
/// This runs the complete bootstrap process:
/// 1. Wait for Talos maintenance mode
/// 2. Generate secrets and config
/// 3. Apply config (triggers install + reboot)
/// 4. Wait for install to complete
/// 5. Bootstrap cluster
/// 6. Wait for Kubernetes
/// 7. Get kubeconfig
///
/// # Errors
///
/// Returns an error if any step in the bootstrap process fails.
pub fn full_bootstrap(config: &BootstrapConfig) -> Result<PathBuf> {
    check_talosctl()?;

    // Step 1: Wait for Talos maintenance mode
    wait_for_talos(&config.node_ip, Duration::from_secs(900))?;

    // Step 2: Generate secrets
    generate_secrets(&config.output_dir)?;

    // Step 3: Generate config
    let configs = generate_config(config)?;

    // Step 4: Apply config
    apply_config(&config.node_ip, &configs.controlplane)?;

    // Step 5: Wait for install
    wait_for_install(
        &config.node_ip,
        &configs.talosconfig,
        Duration::from_secs(600),
    )?;

    // Step 6: Bootstrap cluster
    bootstrap_cluster(&config.node_ip, &configs.talosconfig)?;

    // Step 7: Wait for Kubernetes
    wait_for_kubernetes(
        &config.node_ip,
        &configs.talosconfig,
        Duration::from_secs(300),
    )?;

    // Step 8: Get kubeconfig
    let kubeconfig_path = config.output_dir.join("kubeconfig");
    get_kubeconfig(&config.node_ip, &configs.talosconfig, &kubeconfig_path)?;

    Ok(kubeconfig_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_config() {
        let config = BootstrapConfig::new("test-cluster", "192.168.1.100")
            .with_install_disk("/dev/nvme0n1")
            .with_output_dir("/tmp/test");

        assert_eq!(config.cluster_name, "test-cluster");
        assert_eq!(config.node_ip, "192.168.1.100");
        assert_eq!(config.install_disk, "/dev/nvme0n1");
        assert_eq!(config.output_dir, PathBuf::from("/tmp/test"));
    }
}
