//! OpenBao secrets management bootstrap.
//!
//! Handles initialization, unsealing, and seeding of OpenBao with secrets
//! from 1Password. This enables a fully automated secrets bootstrap where
//! the user only needs to authenticate with 1Password once at the start.

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::ui;

/// OpenBao initialization output.
#[derive(Debug, Deserialize)]
struct OpenBaoInitOutput {
    unseal_keys_b64: Vec<String>,
    root_token: String,
}

/// A single secret mapping entry.
struct SecretMapping {
    /// 1Password path to read from
    op_path: &'static str,
    /// OpenBao path to write to
    bao_path: &'static str,
    /// Key name in OpenBao
    bao_key: &'static str,
}

/// Secrets to seed from 1Password to OpenBao.
const SECRETS_MAPPING: &[SecretMapping] = &[
    // Cloudflare credentials for external-dns
    SecretMapping {
        op_path: "op://CTO Platform/Cloudflare API/credential",
        bao_path: "secret/cloudflare",
        bao_key: "api-key",
    },
    SecretMapping {
        op_path: "op://CTO Platform/Cloudflare API/email",
        bao_path: "secret/cloudflare",
        bao_key: "email",
    },
    // Discord webhook for Alertmanager
    SecretMapping {
        op_path: "op://CTO Platform/Discord Alertmanager Webhook/url",
        bao_path: "secret/alertmanager-discord",
        bao_key: "webhook-url",
    },
    // GitHub credentials for ArgoCD
    SecretMapping {
        op_path: "op://CTO Platform/GitHub PAT - ArgoCD/credential",
        bao_path: "secret/tools-github",
        bao_key: "password",
    },
    SecretMapping {
        op_path: "op://CTO Platform/GitHub PAT - ArgoCD/username",
        bao_path: "secret/tools-github",
        bao_key: "username",
    },
    // GHCR credentials for CTO namespace
    SecretMapping {
        op_path: "op://CTO Platform/GHCR Pull Secret/credential",
        bao_path: "secret/ghcr-secret",
        bao_key: ".dockerconfigjson",
    },
];

/// OpenBao bootstrap manager.
pub struct OpenBaoBootstrap<'a> {
    kubeconfig: &'a Path,
    /// 1Password session token (obtained via interactive signin)
    op_session: Option<String>,
}

impl<'a> OpenBaoBootstrap<'a> {
    /// Create a new OpenBao bootstrap manager.
    pub fn new(kubeconfig: &'a Path) -> Self {
        Self {
            kubeconfig,
            op_session: None,
        }
    }

    /// Run the complete OpenBao bootstrap process.
    ///
    /// This will:
    /// 1. Authenticate with 1Password (interactive if needed)
    /// 2. Wait for OpenBao pod to be ready
    /// 3. Initialize OpenBao (if not already initialized)
    /// 4. Store unseal keys in 1Password
    /// 5. Unseal OpenBao
    /// 6. Create the openbao-token secret for ESO
    /// 7. Seed secrets from 1Password into OpenBao
    pub async fn bootstrap(&mut self) -> Result<()> {
        info!("Starting OpenBao bootstrap");

        // Step 1: Authenticate with 1Password (interactive if needed)
        ui::print_info("Authenticating with 1Password...");
        self.authenticate_1password()?;
        ui::print_success("1Password session active");

        // Step 2: Wait for OpenBao pod
        ui::print_info("Waiting for OpenBao pod to be ready...");
        self.wait_for_openbao_pod().await?;
        ui::print_success("OpenBao pod is running");

        // Step 3: Check if already initialized
        let status = self.get_openbao_status()?;

        let root_token = if !status.initialized {
            // Step 4: Initialize OpenBao
            ui::print_info("Initializing OpenBao...");
            let init_result = self.initialize_openbao()?;

            // Step 5: Store unseal keys in 1Password
            ui::print_info("Storing unseal keys in 1Password...");
            self.store_unseal_keys_in_1password(&init_result)?;
            ui::print_success("Unseal keys stored in 1Password");

            // Step 6: Unseal with the new keys
            ui::print_info("Unsealing OpenBao...");
            self.unseal_openbao(&init_result.unseal_keys_b64)?;
            ui::print_success("OpenBao unsealed");

            init_result.root_token
        } else if status.sealed {
            // Already initialized but sealed - retrieve keys from 1Password
            ui::print_info("OpenBao is sealed, retrieving unseal keys from 1Password...");
            let unseal_keys = self.get_unseal_keys_from_1password()?;

            ui::print_info("Unsealing OpenBao...");
            self.unseal_openbao(&unseal_keys)?;
            ui::print_success("OpenBao unsealed");

            // Get root token from 1Password
            self.get_root_token_from_1password()?
        } else {
            // Already initialized and unsealed
            ui::print_info("OpenBao is already initialized and unsealed");
            self.get_root_token_from_1password()?
        };

        // Step 7: Create openbao-token secret for ESO
        ui::print_info("Creating openbao-token secret for External Secrets Operator...");
        self.create_token_secret(&root_token)?;
        ui::print_success("Token secret created");

        // Step 8: Enable KV secrets engine if needed
        ui::print_info("Enabling KV secrets engine...");
        self.enable_kv_engine(&root_token)?;

        // Step 9: Seed secrets from 1Password
        ui::print_info("Seeding secrets from 1Password to OpenBao...");
        let count = self.seed_secrets(&root_token)?;
        ui::print_success(&format!("Seeded {count} secrets into OpenBao"));

        info!("OpenBao bootstrap complete");
        Ok(())
    }

    /// Authenticate with 1Password, interactively if needed.
    ///
    /// First checks if there's an existing session. If not, prompts
    /// the user to authenticate (Touch ID or password).
    fn authenticate_1password(&mut self) -> Result<()> {
        // First, check if we already have an active session
        let check = Command::new("op")
            .args(["whoami", "--format=json"])
            .output()
            .context("Failed to run 'op whoami'")?;

        if check.status.success() {
            debug!("1Password session already active");
            return Ok(());
        }

        // No active session - need to sign in interactively
        ui::print_info("🔐 1Password authentication required (Touch ID or password)...");
        println!();

        // Run signin interactively - user sees prompts, we capture the token
        let output = Command::new("op")
            .args(["signin", "--raw"])
            .stdin(Stdio::inherit()) // User can interact (Touch ID / password)
            .stderr(Stdio::inherit()) // User sees any error messages
            .stdout(Stdio::piped()) // We capture the session token
            .output()
            .context("Failed to run 'op signin'")?;

        if !output.status.success() {
            bail!("1Password authentication failed. Please try again.");
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            bail!("1Password signin returned empty token");
        }

        self.op_session = Some(token);
        info!("1Password authentication successful");
        Ok(())
    }

    /// Run an `op` command with the session token if available.
    fn op_command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("op");

        // Add session token if we have one
        if let Some(ref token) = self.op_session {
            cmd.arg("--session").arg(token);
        }

        cmd.args(args);
        cmd
    }

    /// Read a secret from 1Password.
    fn op_read(&self, path: &str) -> Result<String> {
        let output = self
            .op_command(&["read", path])
            .output()
            .context("Failed to run 'op read'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to read from 1Password: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Wait for OpenBao pod to be ready.
    async fn wait_for_openbao_pod(&self) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        loop {
            if start.elapsed() > timeout {
                bail!("Timeout waiting for OpenBao pod");
            }

            let output = Command::new("kubectl")
                .args([
                    "--kubeconfig",
                    self.kubeconfig.to_str().unwrap_or_default(),
                    "get",
                    "pod",
                    "openbao-0",
                    "-n",
                    "openbao",
                    "-o",
                    "jsonpath={.status.phase}",
                ])
                .output()
                .context("Failed to check OpenBao pod status")?;

            if output.status.success() {
                let phase = String::from_utf8_lossy(&output.stdout);
                if phase.trim() == "Running" {
                    return Ok(());
                }
                debug!("OpenBao pod phase: {}", phase.trim());
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    /// Get OpenBao status (initialized, sealed).
    fn get_openbao_status(&self) -> Result<OpenBaoStatus> {
        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                self.kubeconfig.to_str().unwrap_or_default(),
                "exec",
                "-n",
                "openbao",
                "openbao-0",
                "--",
                "bao",
                "status",
                "-format=json",
            ])
            .output()
            .context("Failed to get OpenBao status")?;

        // bao status returns exit code 2 if sealed, 0 if unsealed
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.is_empty() {
            // Not initialized yet
            return Ok(OpenBaoStatus {
                initialized: false,
                sealed: true,
            });
        }

        let status: serde_json::Value =
            serde_json::from_str(&stdout).context("Failed to parse OpenBao status")?;

        Ok(OpenBaoStatus {
            initialized: status["initialized"].as_bool().unwrap_or(false),
            sealed: status["sealed"].as_bool().unwrap_or(true),
        })
    }

    /// Initialize OpenBao.
    fn initialize_openbao(&self) -> Result<OpenBaoInitOutput> {
        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                self.kubeconfig.to_str().unwrap_or_default(),
                "exec",
                "-n",
                "openbao",
                "openbao-0",
                "--",
                "bao",
                "operator",
                "init",
                "-format=json",
            ])
            .output()
            .context("Failed to initialize OpenBao")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to initialize OpenBao: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let init_output: OpenBaoInitOutput =
            serde_json::from_str(&stdout).context("Failed to parse OpenBao init output")?;

        info!(
            "OpenBao initialized with {} unseal keys",
            init_output.unseal_keys_b64.len()
        );
        Ok(init_output)
    }

    /// Store unseal keys and root token in 1Password.
    fn store_unseal_keys_in_1password(&self, init: &OpenBaoInitOutput) -> Result<()> {
        // Create a new item in 1Password with the unseal keys
        let title = "OpenBao Unseal Keys - CTO Platform";

        // Build the item creation command
        let mut args = vec![
            "item",
            "create",
            "--category=password",
            "--vault=CTO Platform",
        ];

        // We need to build owned strings for the dynamic parts
        let title_arg = format!("--title={title}");
        let password_arg = format!("password={}", init.root_token);

        args.push(&title_arg);
        args.push(&password_arg);

        // Build unseal key args
        let key_args: Vec<String> = init
            .unseal_keys_b64
            .iter()
            .enumerate()
            .map(|(i, key)| format!("Unseal Key {}[password]={key}", i + 1))
            .collect();

        let key_refs: Vec<&str> = key_args.iter().map(String::as_str).collect();
        args.extend(key_refs);

        let output = self
            .op_command(&args)
            .output()
            .context("Failed to store unseal keys in 1Password")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if item already exists
            if stderr.contains("already exists") {
                warn!("OpenBao unseal keys item already exists in 1Password, updating...");
                return self.update_unseal_keys_in_1password(init);
            }
            bail!("Failed to store unseal keys in 1Password: {stderr}");
        }

        info!("Stored unseal keys and root token in 1Password");
        Ok(())
    }

    /// Update existing unseal keys in 1Password.
    fn update_unseal_keys_in_1password(&self, init: &OpenBaoInitOutput) -> Result<()> {
        let title = "OpenBao Unseal Keys - CTO Platform";

        // First, delete the existing item
        let _ = self
            .op_command(&["item", "delete", title, "--vault=CTO Platform"])
            .output();

        // Then create a new one
        self.store_unseal_keys_in_1password(init)
    }

    /// Get unseal keys from 1Password.
    fn get_unseal_keys_from_1password(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();

        for i in 1..=5 {
            let field = format!("Unseal Key {i}");
            let output = self
                .op_command(&[
                    "item",
                    "get",
                    "OpenBao Unseal Keys - CTO Platform",
                    "--vault=CTO Platform",
                    "--fields",
                    &field,
                ])
                .output()
                .context("Failed to get unseal key from 1Password")?;

            if output.status.success() {
                let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !key.is_empty() {
                    keys.push(key);
                }
            }
        }

        if keys.len() < 3 {
            bail!(
                "Not enough unseal keys found in 1Password (need at least 3, found {})",
                keys.len()
            );
        }

        Ok(keys)
    }

    /// Get root token from 1Password.
    fn get_root_token_from_1password(&self) -> Result<String> {
        let output = self
            .op_command(&[
                "item",
                "get",
                "OpenBao Unseal Keys - CTO Platform",
                "--vault=CTO Platform",
                "--fields",
                "password",
            ])
            .output()
            .context("Failed to get root token from 1Password")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to get root token from 1Password: {stderr}");
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            bail!("Root token is empty in 1Password");
        }

        Ok(token)
    }

    /// Unseal OpenBao with the provided keys.
    fn unseal_openbao(&self, unseal_keys: &[String]) -> Result<()> {
        // Need 3 of 5 keys to unseal (Shamir's threshold)
        for (i, key) in unseal_keys.iter().take(3).enumerate() {
            debug!("Applying unseal key {}/3", i + 1);

            let output = Command::new("kubectl")
                .args([
                    "--kubeconfig",
                    self.kubeconfig.to_str().unwrap_or_default(),
                    "exec",
                    "-n",
                    "openbao",
                    "openbao-0",
                    "--",
                    "bao",
                    "operator",
                    "unseal",
                    key,
                ])
                .output()
                .context("Failed to unseal OpenBao")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // Check if already unsealed
                if !stderr.contains("already unsealed") {
                    bail!("Failed to unseal OpenBao: {stderr}");
                }
            }
        }

        // Verify unsealed
        let status = self.get_openbao_status()?;
        if status.sealed {
            bail!("OpenBao is still sealed after applying unseal keys");
        }

        Ok(())
    }

    /// Create the openbao-token secret for External Secrets Operator.
    fn create_token_secret(&self, root_token: &str) -> Result<()> {
        // Delete existing secret if it exists
        let _ = Command::new("kubectl")
            .args([
                "--kubeconfig",
                self.kubeconfig.to_str().unwrap_or_default(),
                "delete",
                "secret",
                "openbao-token",
                "-n",
                "openbao",
                "--ignore-not-found",
            ])
            .output();

        // Create new secret
        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                self.kubeconfig.to_str().unwrap_or_default(),
                "create",
                "secret",
                "generic",
                "openbao-token",
                &format!("--from-literal=token={root_token}"),
                "-n",
                "openbao",
            ])
            .output()
            .context("Failed to create openbao-token secret")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create openbao-token secret: {stderr}");
        }

        Ok(())
    }

    /// Enable the KV secrets engine if not already enabled.
    fn enable_kv_engine(&self, root_token: &str) -> Result<()> {
        let output = Command::new("kubectl")
            .args([
                "--kubeconfig",
                self.kubeconfig.to_str().unwrap_or_default(),
                "exec",
                "-n",
                "openbao",
                "openbao-0",
                "--",
                "sh",
                "-c",
                &format!(
                    "BAO_TOKEN='{root_token}' bao secrets enable -path=secret -version=2 kv 2>/dev/null || true"
                ),
            ])
            .output()
            .context("Failed to enable KV secrets engine")?;

        // This may fail if already enabled, which is fine
        debug!("KV secrets engine enable result: {:?}", output.status);
        Ok(())
    }

    /// Seed secrets from 1Password into OpenBao.
    fn seed_secrets(&self, root_token: &str) -> Result<usize> {
        let mut secrets_written = 0;
        let mut secrets_by_path: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Read all secrets from 1Password and group by OpenBao path
        for mapping in SECRETS_MAPPING {
            debug!("Reading secret from 1Password: {}", mapping.op_path);

            match self.op_read(mapping.op_path) {
                Ok(value) if !value.is_empty() => {
                    let path_secrets = secrets_by_path
                        .entry(mapping.bao_path.to_string())
                        .or_default();

                    path_secrets.insert(mapping.bao_key.to_string(), value);
                }
                Ok(_) => {
                    warn!("Secret at {} is empty, skipping", mapping.op_path);
                }
                Err(e) => {
                    warn!(
                        "Failed to read secret from 1Password ({}): {}",
                        mapping.op_path, e
                    );
                }
            }
        }

        // Write grouped secrets to OpenBao
        for (bao_path, secrets) in &secrets_by_path {
            if secrets.is_empty() {
                continue;
            }

            // Build key=value pairs for bao kv put
            let kv_pairs: Vec<String> = secrets.iter().map(|(k, v)| format!("{k}={v}")).collect();

            let kv_args = kv_pairs.join(" ");

            debug!("Writing to OpenBao path: {bao_path}");

            let output = Command::new("kubectl")
                .args([
                    "--kubeconfig",
                    self.kubeconfig.to_str().unwrap_or_default(),
                    "exec",
                    "-n",
                    "openbao",
                    "openbao-0",
                    "--",
                    "sh",
                    "-c",
                    &format!("BAO_TOKEN='{root_token}' bao kv put {bao_path} {kv_args}"),
                ])
                .output()
                .context("Failed to write secret to OpenBao")?;

            if output.status.success() {
                secrets_written += 1;
                info!("Wrote secret to OpenBao: {bao_path}");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to write secret to {bao_path}: {stderr}");
            }
        }

        Ok(secrets_written)
    }
}

/// OpenBao status.
#[derive(Debug)]
struct OpenBaoStatus {
    initialized: bool,
    sealed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secrets_mapping_not_empty() {
        assert!(!SECRETS_MAPPING.is_empty());
    }
}
