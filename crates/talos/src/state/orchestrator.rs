use std::time::Duration;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::scaleway::ScalewayClient;
use crate::ssh::SshClient;
use crate::state::InstallationState;
use crate::talosctl::TalosInstaller;

pub struct Orchestrator {
    scaleway: ScalewayClient,
    installer: TalosInstaller,
    config: Config,
}

impl Orchestrator {
    pub fn new(scaleway: ScalewayClient, installer: TalosInstaller, config: Config) -> Self {
        Self {
            scaleway,
            installer,
            config,
        }
    }

    /// Run the full installation workflow
    pub async fn install(&self, server_id: &str) -> Result<InstallationState> {
        tracing::info!("Starting Talos installation for server: {}", server_id);

        // Step 1: Enable rescue mode
        tracing::info!("Enabling rescue mode...");
        let (rescue_ip, rescue_password) = self
            .scaleway
            .enable_rescue_mode(server_id, Duration::from_secs(300))
            .await?;

        // Step 2: SSH to rescue and upload files
        tracing::info!("Connecting to rescue mode at {}...", rescue_ip);
        
        let ip = rescue_ip.clone();
        let password = rescue_password.clone();
        let ssh = tokio::task::spawn_blocking(move || {
            SshClient::connect(&ip, "root", &password)
        }).await.map_err(|e| Error::Ssh(e.to_string()))??;

        tracing::info!("Uploading Talos image...");
        let installer = self.installer.clone();
        let image_path = self.installer.image_path();
        
        tokio::task::spawn_blocking(move || {
            let mut ssh = ssh;
            installer.upload_image_blocking(&mut ssh, &image_path)
        }).await.map_err(|e| Error::Ssh(e.to_string()))??;

        // Step 3: Write image to disk
        tracing::info!("Writing Talos image to disk...");
        let ip = rescue_ip.clone();
        let password = rescue_password.clone();
        let ssh = tokio::task::spawn_blocking(move || {
            SshClient::connect(&ip, "root", &password)
        }).await.map_err(|e| Error::Ssh(e.to_string()))??;
        
        let installer = self.installer.clone();
        let disk = self.config.server.disk.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut ssh = ssh;
            installer.dd_write_blocking(&mut ssh, &disk)
        }).await.map_err(|e| Error::Ssh(e.to_string()))??;

        // Step 4: Disable rescue and power on
        tracing::info!("Disabling rescue mode and powering on...");
        self.scaleway.disable_rescue_mode(server_id).await?;
        self.scaleway.power_on(server_id).await?;

        // Step 5: Wait for Talos to boot
        tracing::info!("Waiting for Talos to boot...");
        self.installer.wait_for_boot(&rescue_ip, Duration::from_secs(300)).await?;

        // Step 6: Bootstrap
        tracing::info!("Bootstrapping Talos cluster...");
        self.installer.bootstrap(&rescue_ip).await?;

        tracing::info!("Installation complete!");
        Ok(InstallationState::Bootstrapped)
    }
}
