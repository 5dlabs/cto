//! GPU CLI - GPU VM provisioning tool for AI/ML workloads.

use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use gpu::providers::latitude::Latitude;
use gpu::providers::traits::{CreateGpuVmRequest, GpuProvider};

/// GPU CLI - Provision GPU VMs for AI/ML workloads.
#[derive(Parser)]
#[command(name = "gpu")]
#[command(about = "Provision and manage GPU VMs for AI/ML inference")]
struct Cli {
    /// Latitude.sh API key (or set `LATITUDE_API_KEY` env var).
    #[arg(long, env = "LATITUDE_API_KEY", default_value = "")]
    api_key: String,

    /// Latitude.sh Project ID (or set `LATITUDE_PROJECT_ID` env var).
    #[arg(long, env = "LATITUDE_PROJECT_ID", default_value = "")]
    project_id: String,

    /// Fetch missing credentials from 1Password via the `op` CLI.
    #[arg(long, default_value = "false")]
    use_1password: bool,

    /// 1Password vault name (optional).
    #[arg(long, env = "OP_VAULT", default_value = "")]
    op_vault: String,

    /// 1Password item title containing Latitude credentials.
    #[arg(long, env = "OP_LATITUDE_ITEM", default_value = "Latitude.sh API")]
    op_latitude_item: String,

    /// 1Password field label for API key.
    #[arg(long, env = "OP_LATITUDE_API_KEY_FIELD", default_value = "credential")]
    op_latitude_api_key_field: String,

    /// 1Password field label for project ID.
    #[arg(
        long,
        env = "OP_LATITUDE_PROJECT_ID_FIELD",
        default_value = "Project ID"
    )]
    op_latitude_project_id_field: String,

    /// Enable verbose logging.
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available GPU plans.
    Plans,

    /// List all GPU VMs.
    List,

    /// Create a new GPU VM.
    Create {
        /// VM name.
        #[arg(long)]
        name: String,

        /// Plan ID (from `plans` command).
        #[arg(long)]
        plan: String,

        /// SSH key IDs (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,
    },

    /// Get details of a GPU VM.
    Get {
        /// VM ID.
        #[arg(long)]
        id: String,
    },

    /// Delete a GPU VM.
    Delete {
        /// VM ID.
        #[arg(long)]
        id: String,

        /// Skip confirmation prompt.
        #[arg(long, short = 'y', default_value = "false")]
        yes: bool,
    },

    /// Run a power action on a GPU VM.
    Action {
        /// VM ID.
        #[arg(long)]
        id: String,

        /// Action: power_on, power_off, or reboot.
        #[arg(long)]
        action: String,
    },

    /// Wait for a GPU VM to be ready.
    Wait {
        /// VM ID.
        #[arg(long)]
        id: String,

        /// Timeout in seconds.
        #[arg(long, default_value = "600")]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Resolve credentials
    let (api_key, project_id) = resolve_latitude_creds(&cli)?;

    // Create provider
    let provider =
        Latitude::new(&api_key, &project_id).context("Failed to create Latitude provider")?;

    match cli.command {
        Commands::Plans => {
            let plans = provider.list_gpu_plans().await?;

            println!("\n🎮 Available GPU Plans");
            println!("{}", "=".repeat(90));

            for plan in plans {
                let stock_emoji = match plan.stock_level.as_str() {
                    "high" => "🟢",
                    "medium" => "🟡",
                    "low" => "🟠",
                    "unavailable" => "🔴",
                    _ => "⚪",
                };

                println!("\n{} {} (ID: {})", stock_emoji, plan.name, plan.id);
                println!(
                    "   GPU: {} x{} | vCPUs: {} | RAM: {} GB | Storage: {} GB",
                    plan.specs.gpu_model,
                    plan.specs.gpu_count,
                    plan.specs.vcpus,
                    plan.specs.ram_gb,
                    plan.specs.storage_gb
                );
                println!(
                    "   💰 ${:.2}/hr (${:.0}/mo)",
                    plan.price_per_hour, plan.price_per_month
                );
                println!(
                    "   📍 Regions: {}",
                    if plan.available_regions.is_empty() {
                        "None".to_string()
                    } else {
                        plan.available_regions.join(", ")
                    }
                );
            }
            println!();
        }

        Commands::List => {
            let vms = provider.list_gpu_vms().await?;

            println!(
                "\n{:<20} {:<25} {:<18} {:<20}",
                "ID", "NAME", "STATUS", "HOST"
            );
            println!("{}", "-".repeat(85));

            for vm in vms {
                let status_emoji = match vm.status {
                    gpu::GpuVmStatus::Running => "🟢",
                    gpu::GpuVmStatus::Starting
                    | gpu::GpuVmStatus::Scheduling
                    | gpu::GpuVmStatus::Scheduled
                    | gpu::GpuVmStatus::ConfiguringNetwork => "🟡",
                    gpu::GpuVmStatus::Stopped => "🔴",
                    _ => "⚪",
                };

                println!(
                    "{:<20} {:<25} {} {:<15} {:<20}",
                    vm.id,
                    vm.name,
                    status_emoji,
                    vm.status,
                    vm.host.unwrap_or_default()
                );
            }
            println!();
        }

        Commands::Create {
            name,
            plan,
            ssh_keys,
        } => {
            info!(name = %name, plan = %plan, "Creating GPU VM");

            let vm = provider
                .create_gpu_vm(CreateGpuVmRequest {
                    name: name.clone(),
                    plan_id: plan,
                    ssh_keys,
                })
                .await?;

            println!("\n✅ GPU VM created successfully!");
            println!("   ID:     {}", vm.id);
            println!("   Name:   {}", vm.name);
            println!("   Status: {}", vm.status);
            println!("\n💡 Run `gpu wait --id {}` to wait for it to be ready", vm.id);
        }

        Commands::Get { id } => {
            let vm = provider.get_gpu_vm(&id).await?;

            println!("\n🖥️  GPU VM: {}", vm.name);
            println!("   ID:       {}", vm.id);
            println!("   Status:   {}", vm.status);
            println!("   Plan:     {}", vm.plan_id);

            if let Some(specs) = &vm.specs {
                println!(
                    "   GPU:      {} x{}",
                    specs.gpu_model, specs.gpu_count
                );
                println!("   vCPUs:    {}", specs.vcpus);
            }

            if let Some(host) = &vm.host {
                println!("\n   📡 SSH: {}@{}", vm.username.as_deref().unwrap_or("root"), host);
            }

            if let Some(created) = &vm.created_at {
                println!("   Created:  {}", created.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }

        Commands::Delete { id, yes } => {
            if !yes {
                println!("⚠️  Are you sure you want to delete GPU VM {}?", id);
                println!("   This action cannot be undone.");
                println!("   Use --yes to skip this prompt.");
                return Ok(());
            }

            info!(vm_id = %id, "Deleting GPU VM");
            provider.delete_gpu_vm(&id).await?;
            println!("\n✅ GPU VM {} deleted successfully!", id);
        }

        Commands::Action { id, action } => {
            info!(vm_id = %id, action = %action, "Running GPU VM action");
            provider.gpu_vm_action(&id, &action).await?;
            println!("\n✅ Action '{}' executed on GPU VM {}", action, id);
        }

        Commands::Wait { id, timeout } => {
            println!("⏳ Waiting for GPU VM {} to be ready...", id);
            let vm = provider.wait_ready(&id, timeout).await?;
            println!("\n✅ GPU VM is ready!");
            let host = vm.host.as_deref().unwrap_or("N/A");
            println!("   Host: {host}");
            if let Some(user) = &vm.username {
                println!("   SSH:  ssh {user}@{host}");
            }
        }
    }

    Ok(())
}

fn resolve_latitude_creds(cli: &Cli) -> Result<(String, String)> {
    let mut api_key = cli.api_key.trim().to_string();
    let mut project_id = cli.project_id.trim().to_string();

    if !api_key.is_empty() && !project_id.is_empty() {
        return Ok((api_key, project_id));
    }

    // Try 1Password
    ensure_op_ready()?;

    let vault = cli.op_vault.trim();
    let item = cli.op_latitude_item.trim();

    if api_key.is_empty() {
        api_key = op_item_get_field(
            item,
            if vault.is_empty() { None } else { Some(vault) },
            cli.op_latitude_api_key_field.trim(),
        )
        .context("Failed to read Latitude API key from 1Password")?;
    }

    if project_id.is_empty() {
        project_id = op_item_get_field(
            item,
            if vault.is_empty() { None } else { Some(vault) },
            cli.op_latitude_project_id_field.trim(),
        )
        .context("Failed to read Latitude project ID from 1Password")?;
    }

    if api_key.is_empty() || project_id.is_empty() {
        anyhow::bail!(
            "Latitude credentials are missing. Provide --api-key/--project-id or configure 1Password."
        );
    }

    Ok((api_key, project_id))
}

fn ensure_op_ready() -> Result<()> {
    let version = Command::new("op").arg("--version").output();
    if version.is_err() {
        anyhow::bail!("1Password CLI `op` not found. Install it and run: `eval $(op signin)`");
    }

    let auth = Command::new("op").args(["account", "get"]).output();
    match auth {
        Ok(out) if out.status.success() => Ok(()),
        _ => anyhow::bail!("Not authenticated to 1Password. Run: `eval $(op signin)`"),
    }
}

fn op_item_get_field(item: &str, vault: Option<&str>, field: &str) -> Result<String> {
    let mut cmd = Command::new("op");
    cmd.args(["item", "get", item, "--fields", field, "--reveal"]);
    if let Some(v) = vault {
        cmd.args(["--vault", v]);
    }

    let output = cmd.output().context("Failed to run `op` command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("1Password error: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}





