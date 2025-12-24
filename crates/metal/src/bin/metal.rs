//! Metal CLI - Bare metal provisioning tool for CTO Platform.

#![allow(clippy::similar_names)]

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use metal::providers::latitude::Latitude;
use metal::providers::{CreateServerRequest, Provider, ReinstallIpxeRequest};
use metal::stack;
use metal::state::{with_retry_async, ClusterState, ProvisionStep, RetryConfig};
use metal::talos::{self, BootstrapConfig, TalosConfig};
use tokio::task::JoinSet;

/// Metal CLI - Bare metal provisioning for CTO Platform.
#[derive(Parser)]
#[command(name = "metal")]
#[command(about = "Provision and manage bare metal servers")]
struct Cli {
    /// Latitude.sh API key (or set `LATITUDE_API_KEY` env var).
    ///
    /// If omitted, `metal` can optionally fetch this from 1Password via `op`.
    #[arg(long, env = "LATITUDE_API_KEY", default_value = "")]
    api_key: String,

    /// Latitude.sh Project ID (or set `LATITUDE_PROJECT_ID` env var).
    ///
    /// If omitted, `metal` can optionally fetch this from 1Password via `op`.
    #[arg(long, env = "LATITUDE_PROJECT_ID", default_value = "")]
    project_id: String,

    /// Fetch missing Latitude credentials from 1Password via the `op` CLI.
    ///
    /// If `--api-key` or `--project-id` are not provided, we will also try 1Password automatically.
    #[arg(long, default_value = "false")]
    use_1password: bool,

    /// 1Password vault name (optional). If empty, `op` will use your default.
    #[arg(long, env = "OP_VAULT", default_value = "")]
    op_vault: String,

    /// 1Password item title containing Latitude credentials.
    #[arg(long, env = "OP_LATITUDE_ITEM", default_value = "Latitude.sh API")]
    op_latitude_item: String,

    /// 1Password field label/key for Latitude API key.
    ///
    /// Common choices: `credential`, `api key`, etc.
    #[arg(long, env = "OP_LATITUDE_API_KEY_FIELD", default_value = "credential")]
    op_latitude_api_key_field: String,

    /// 1Password field label/key for Latitude project ID.
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
    /// List all servers in the project.
    List,

    /// List available plans (hardware configurations).
    ///
    /// Shows plan specs, pricing, and stock availability by region.
    Plans {
        /// Filter by region slug (e.g., ASH, DAL, MIA2).
        #[arg(long)]
        region: Option<String>,

        /// Show only plans in stock.
        #[arg(long, default_value = "false")]
        in_stock: bool,

        /// Show only Gen 4 plans (10G+ networking).
        #[arg(long, default_value = "false")]
        gen4: bool,
    },

    /// List available regions.
    Regions,

    /// Get details of a specific server.
    Get {
        /// Server ID.
        #[arg(long)]
        id: String,
    },

    /// Provision a new server.
    Create {
        /// Server hostname.
        #[arg(long)]
        hostname: String,

        /// Server plan (e.g., c2-small-x86).
        #[arg(long, default_value = "c2-small-x86")]
        plan: String,

        /// Region/site (e.g., MIA2, DAL, LAX).
        #[arg(long, default_value = "MIA2")]
        region: String,

        /// Operating system slug.
        #[arg(long, default_value = "ubuntu_24_04_x64_lts")]
        os: String,

        /// SSH key IDs (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,
    },

    /// Delete a server.
    Delete {
        /// Server ID.
        #[arg(long)]
        id: String,
    },

    /// Provision a Talos node (create + wait + iPXE boot).
    Talos {
        /// Server hostname.
        #[arg(long)]
        hostname: String,

        /// Server plan (e.g., c2-small-x86).
        #[arg(long, default_value = "c2-small-x86")]
        plan: String,

        /// Region/site (e.g., MIA2, DAL, LAX).
        #[arg(long, default_value = "MIA2")]
        region: String,

        /// SSH key IDs for initial Ubuntu boot (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Timeout in seconds to wait for server to be ready.
        #[arg(long, default_value = "600")]
        timeout: u64,

        /// Skip the iPXE reinstall (just provision Ubuntu).
        #[arg(long, default_value = "false")]
        skip_talos: bool,
    },

    /// Reinstall a server with Talos via iPXE.
    Reinstall {
        /// Server ID.
        #[arg(long)]
        id: String,

        /// Server hostname.
        #[arg(long)]
        hostname: String,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,
    },

    /// Bootstrap a Talos cluster on an existing server.
    ///
    /// This assumes the server is already booted into Talos maintenance mode
    /// (via iPXE). It will generate configs, apply them, and bootstrap the cluster.
    Bootstrap {
        /// Node IP address.
        #[arg(long)]
        ip: String,

        /// Cluster name.
        #[arg(long, default_value = "cto-cluster")]
        cluster_name: String,

        /// Install disk (e.g., /dev/sda, /dev/nvme0n1).
        #[arg(long, default_value = "/dev/sda")]
        install_disk: String,

        /// Output directory for generated configs.
        #[arg(long, default_value = "/tmp/talos-bootstrap")]
        output_dir: PathBuf,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Timeout in seconds to wait for each step.
        #[arg(long, default_value = "600")]
        timeout: u64,
    },

    /// Full provisioning: create server + Talos boot + bootstrap cluster.
    ///
    /// This is the all-in-one command that provisions a bare metal server,
    /// boots Talos via iPXE, and bootstraps a Kubernetes cluster.
    Provision {
        /// Server hostname.
        #[arg(long)]
        hostname: String,

        /// Cluster name.
        #[arg(long, default_value = "cto-cluster")]
        cluster_name: String,

        /// Server plan (e.g., c2-small-x86).
        #[arg(long, default_value = "c2-small-x86")]
        plan: String,

        /// Region/site (e.g., MIA2, DAL, LAX).
        #[arg(long, default_value = "MIA2")]
        region: String,

        /// SSH key IDs for initial Ubuntu boot (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Install disk (e.g., /dev/sda, /dev/nvme0n1).
        #[arg(long, default_value = "/dev/sda")]
        install_disk: String,

        /// Output directory for generated configs.
        #[arg(long, default_value = "/tmp/talos-bootstrap")]
        output_dir: PathBuf,

        /// Timeout in seconds to wait for each step.
        #[arg(long, default_value = "900")]
        timeout: u64,
    },

    /// Provision a full 2-node cluster (CP + worker) in parallel.
    ///
    /// Creates both servers simultaneously, waits for both to be ready,
    /// boots Talos on both, then bootstraps CP and joins worker.
    Cluster {
        /// Cluster name.
        #[arg(long)]
        name: String,

        /// Region/site (e.g., MIA2, DAL, LAX).
        #[arg(long, default_value = "MIA2")]
        region: String,

        /// Control plane server plan.
        #[arg(long, default_value = "c2-small-x86")]
        cp_plan: String,

        /// Worker server plan.
        #[arg(long, default_value = "c2-small-x86")]
        worker_plan: String,

        /// SSH key IDs for initial Ubuntu boot (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Install disk (e.g., /dev/sda, /dev/nvme0n1).
        #[arg(long, default_value = "/dev/sda")]
        install_disk: String,

        /// Output directory for generated configs.
        #[arg(long, default_value = "/tmp/cto-cluster")]
        output_dir: PathBuf,

        /// Timeout in seconds to wait for each step.
        #[arg(long, default_value = "1200")]
        timeout: u64,

        /// Resume from saved state if interrupted.
        #[arg(long, default_value = "false")]
        resume: bool,

        /// Deploy the platform stack after cluster bootstrap.
        /// Installs: Cert-Manager, `ArgoCD`, `OpenBao`, Ingress-NGINX, Argo Workflows.
        #[arg(long, default_value = "false")]
        deploy_stack: bool,

        /// Initialize and unseal `OpenBao` after deployment (requires --deploy-stack).
        #[arg(long, default_value = "false")]
        init_openbao: bool,
    },

    /// Proof-of-concept: provision a 3-node cluster and validate Mayastor replicated storage.
    ///
    /// Provisions 1 control plane + 2 workers, installs `OpenEBS` Mayastor, creates one `DiskPool` per node,
    /// creates a 3-replica `StorageClass`, and runs an fio Job to capture baseline performance.
    ///
    /// Note: for meaningful results, pick hardware/plans with **10GbE+** networking.
    /// Gen 4 plans (`m4-metal-small`, `f4-metal-small`, etc.) have 2x10Gbps NICs.
    MayastorPoc {
        /// Cluster name (used as hostname prefix).
        #[arg(long)]
        name: String,

        /// Region/site (e.g., ASH, DAL, LAX2).
        #[arg(long, default_value = "ASH")]
        region: String,

        /// Control plane server plan (Gen 4 recommended for 10G networking).
        #[arg(long, default_value = "m4-metal-small")]
        cp_plan: String,

        /// Worker server plan (Gen 4 recommended for 10G networking).
        #[arg(long, default_value = "m4-metal-small")]
        worker_plan: String,

        /// SSH key IDs for initial Ubuntu boot (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Install disk (e.g., /dev/sda, /dev/nvme0n1).
        #[arg(long, default_value = "/dev/sda")]
        install_disk: String,

        /// Output directory for generated configs and benchmark logs.
        #[arg(long, default_value = "/tmp/cto-mayastor-poc")]
        output_dir: PathBuf,

        /// Timeout in seconds to wait for each provisioning step.
        #[arg(long, default_value = "1800")]
        timeout: u64,

        /// Mayastor Helm chart version (e.g., 2.4.0).
        #[arg(long, default_value = "2.4.0")]
        mayastor_chart_version: String,

        /// Namespace to install Mayastor into (defaults to `openebs`).
        #[arg(long, default_value = "openebs")]
        mayastor_namespace: String,

        /// Disk URI to allocate to each `DiskPool` (example: `<aio:///dev/disk/by-id/DEVICE_ID>`).
        ///
        /// For Gen 4 hardware with 2x 960GB `NVMe`, use the second drive for Mayastor:
        /// - `aio:///dev/nvme1n1` (second `NVMe` drive)
        /// - Or use `by-id` path for stability: `aio:///dev/disk/by-id/nvme-*`
        #[arg(long, default_value = "aio:///dev/nvme1n1")]
        disk_uri: String,

        /// `StorageClass` name to create for 3-replica Mayastor volumes.
        #[arg(long, default_value = "mayastor-3")]
        storage_class: String,

        /// PVC size for the fio benchmark (e.g., 20Gi).
        #[arg(long, default_value = "20Gi")]
        bench_pvc_size: String,

        /// fio runtime in seconds.
        #[arg(long, default_value = "120")]
        bench_runtime_seconds: u32,
    },

    /// Join a worker node to an existing Talos cluster.
    ///
    /// This provisions a new server, boots Talos via iPXE, and joins it
    /// to an existing cluster as a worker node.
    Join {
        /// Server hostname.
        #[arg(long)]
        hostname: String,

        /// Server plan (e.g., c2-small-x86, c2-medium-x86).
        #[arg(long, default_value = "c2-medium-x86")]
        plan: String,

        /// Region/site (e.g., MIA2, DAL, LAX).
        #[arg(long, default_value = "MIA2")]
        region: String,

        /// SSH key IDs for initial Ubuntu boot (comma-separated).
        #[arg(long, value_delimiter = ',')]
        ssh_keys: Vec<String>,

        /// Talos version (e.g., v1.9.0).
        #[arg(long, default_value = "v1.9.0")]
        talos_version: String,

        /// Install disk (e.g., /dev/sda, /dev/nvme0n1).
        #[arg(long, default_value = "/dev/sda")]
        install_disk: String,

        /// Path to existing worker.yaml config from bootstrap.
        #[arg(long)]
        worker_config: PathBuf,

        /// Path to existing talosconfig from bootstrap.
        #[arg(long)]
        talosconfig: PathBuf,

        /// Path to kubeconfig for verifying node joined.
        #[arg(long)]
        kubeconfig: PathBuf,

        /// Timeout in seconds to wait for each step.
        #[arg(long, default_value = "900")]
        timeout: u64,

        /// Control plane server ID for same-site validation.
        ///
        /// If provided, verifies that the worker region matches the control plane's
        /// site before creating the server. This prevents cross-site clusters which
        /// break VLAN networking.
        #[arg(long)]
        control_plane_id: Option<String>,
    },

    /// Deploy the CTO platform stack to an existing cluster.
    ///
    /// Deploys Argo CD, Cert-Manager, `OpenBao`, and Ingress to the cluster.
    Stack {
        /// Path to kubeconfig for the cluster.
        #[arg(long)]
        kubeconfig: PathBuf,

        /// Deploy only Argo CD.
        #[arg(long, default_value = "false")]
        argocd_only: bool,

        /// Deploy only Cert-Manager.
        #[arg(long, default_value = "false")]
        cert_manager_only: bool,

        /// Deploy only `OpenBao`.
        #[arg(long, default_value = "false")]
        openbao_only: bool,

        /// Deploy only Ingress.
        #[arg(long, default_value = "false")]
        ingress_only: bool,

        /// Initialize and unseal `OpenBao` after deployment.
        #[arg(long, default_value = "false")]
        init_openbao: bool,
    },

    /// Initialize and unseal `OpenBao`.
    ///
    /// Run this after deploying `OpenBao` for the first time.
    OpenbaoInit {
        /// Path to kubeconfig for the cluster.
        #[arg(long)]
        kubeconfig: PathBuf,

        /// Output file for `OpenBao` credentials (JSON).
        #[arg(long, default_value = "openbao-init.json")]
        output: PathBuf,
    },

    /// Unseal an existing `OpenBao` instance.
    OpenbaoUnseal {
        /// Path to kubeconfig for the cluster.
        #[arg(long)]
        kubeconfig: PathBuf,

        /// Unseal key (base64 encoded).
        #[arg(long)]
        unseal_key: String,
    },
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Resolve Latitude credentials (optionally via 1Password)
    let (api_key, project_id) = resolve_latitude_creds(&cli)?;

    // Create provider
    let provider =
        Latitude::new(&api_key, &project_id).context("Failed to create Latitude provider")?;

    match cli.command {
        Commands::List => {
            let servers = provider.list_servers().await?;
            println!(
                "\n{:<20} {:<20} {:<10} {:<16}",
                "ID", "HOSTNAME", "STATUS", "IPv4"
            );
            println!("{}", "-".repeat(70));
            for server in servers {
                println!(
                    "{:<20} {:<20} {:<10} {:<16}",
                    server.id,
                    server.hostname,
                    server.status,
                    server.ipv4.unwrap_or_default()
                );
            }
        }

        Commands::Plans {
            region,
            in_stock,
            gen4,
        } => {
            let plans = provider.list_plans().await?;

            println!("\n📦 Available Plans");
            println!("{}", "=".repeat(100));

            for plan in plans {
                let slug = plan.attributes.slug.as_deref().unwrap_or("unknown");

                // Filter for Gen 4 plans (they start with m4, f4, rs4, etc.)
                if gen4
                    && !slug.contains("4-metal")
                    && !slug.starts_with("m4")
                    && !slug.starts_with("f4")
                    && !slug.starts_with("rs4")
                {
                    continue;
                }

                let name = plan.attributes.name.as_deref().unwrap_or("Unknown");
                let specs = plan.attributes.specs.as_ref();

                // Format CPU
                let cpu_desc = specs.and_then(|s| s.cpu.as_ref()).map_or_else(
                    || "N/A".to_string(),
                    |c| {
                        let cores = c.cores.unwrap_or(0);
                        let clock = c.clock.unwrap_or(0.0);
                        let cpu_type = c.cpu_type.as_deref().unwrap_or("Unknown");
                        format!("{cores} cores @ {clock:.1}GHz ({cpu_type})")
                    },
                );

                // Format RAM
                let ram = specs
                    .and_then(|s| s.memory.as_ref())
                    .and_then(|m| m.total)
                    .map_or_else(|| "N/A".to_string(), |gb| format!("{gb} GB"));

                // Format Storage
                let storage = specs.and_then(|s| s.drives.as_ref()).map_or_else(
                    || "N/A".to_string(),
                    |drives| {
                        drives
                            .iter()
                            .map(|d| {
                                let count = d.count.unwrap_or(1);
                                let size = d.size.as_deref().unwrap_or("?");
                                let dtype = d.drive_type.as_deref().unwrap_or("?");
                                format!("{count}x {size} {dtype}")
                            })
                            .collect::<Vec<_>>()
                            .join(" + ")
                    },
                );

                // Format NICs
                let nics = specs.and_then(|s| s.nics.as_ref()).map_or_else(
                    || "N/A".to_string(),
                    |nics| {
                        nics.iter()
                            .map(|n| {
                                let count = n.count.unwrap_or(1);
                                let ntype = n.nic_type.as_deref().unwrap_or("?");
                                format!("{count}x {ntype}")
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                );

                println!("\n{name} ({slug})");
                println!("  CPU:     {cpu_desc}");
                println!("  RAM:     {ram}");
                println!("  Storage: {storage}");
                println!("  Network: {nics}");

                // Show regions with stock
                if let Some(regions) = &plan.attributes.regions {
                    let mut in_stock_regions = Vec::new();
                    let mut out_of_stock_regions = Vec::new();

                    for r in regions {
                        let region_name = r.name.as_deref().unwrap_or("?");
                        let stock_level = r.stock_level.as_deref().unwrap_or("unknown");
                        let in_stock_sites = r
                            .locations
                            .as_ref()
                            .and_then(|l| l.in_stock.as_ref())
                            .map(|v| v.join(", "))
                            .unwrap_or_default();

                        // Check if any sites are in stock
                        let is_in_stock = r
                            .locations
                            .as_ref()
                            .and_then(|l| l.in_stock.as_ref())
                            .is_some_and(|sites| !sites.is_empty());

                        // Apply region filter (check if any site slug matches)
                        if let Some(ref filter_region) = region {
                            let matches = r
                                .locations
                                .as_ref()
                                .and_then(|l| l.available.as_ref())
                                .is_some_and(|sites| {
                                    sites.iter().any(|s| s.eq_ignore_ascii_case(filter_region))
                                });
                            if !matches {
                                continue;
                            }
                        }

                        // Apply in_stock filter
                        if in_stock && !is_in_stock {
                            continue;
                        }

                        let price_hr = r
                            .pricing
                            .as_ref()
                            .and_then(|p| p.usd.as_ref())
                            .and_then(|u| u.hour)
                            .map_or_else(|| "N/A".to_string(), |h| format!("${h:.2}/hr"));

                        let entry = if is_in_stock {
                            format!("{region_name} [{in_stock_sites}] {price_hr} ({stock_level})")
                        } else {
                            format!("{region_name} {price_hr}")
                        };

                        if is_in_stock {
                            in_stock_regions.push(entry);
                        } else {
                            out_of_stock_regions.push(entry);
                        }
                    }

                    if !in_stock_regions.is_empty() {
                        println!("  ✅ In Stock:");
                        for r in in_stock_regions {
                            println!("     - {r}");
                        }
                    }
                    if !out_of_stock_regions.is_empty() && !in_stock {
                        println!("  ❌ Out of Stock:");
                        for r in out_of_stock_regions {
                            println!("     - {r}");
                        }
                    }
                }
            }
        }

        Commands::Regions => {
            let regions = provider.list_regions().await?;

            println!("\n🌍 Available Regions");
            println!("{}", "=".repeat(60));
            println!("\n{:<10} {:<25} {:<20}", "SLUG", "NAME", "COUNTRY");
            println!("{}", "-".repeat(60));

            for region in regions {
                let slug = region.attributes.slug.as_deref().unwrap_or("?");
                let name = region.attributes.name.as_deref().unwrap_or("?");
                let country = region
                    .attributes
                    .country
                    .as_ref()
                    .and_then(|c| c.name.as_deref())
                    .unwrap_or("?");

                println!("{slug:<10} {name:<25} {country:<20}");
            }
        }

        Commands::Get { id } => {
            let server = provider.get_server(&id).await?;
            println!("\nServer Details:");
            println!("  ID:       {}", server.id);
            println!("  Hostname: {}", server.hostname);
            println!("  Status:   {}", server.status);
            println!("  IPv4:     {}", server.ipv4.unwrap_or_default());
            println!("  IPv6:     {}", server.ipv6.unwrap_or_default());
            println!("  Plan:     {}", server.plan);
        }

        Commands::Create {
            hostname,
            plan,
            region,
            os,
            ssh_keys,
        } => {
            info!("Creating server: {hostname}");
            let server = provider
                .create_server(CreateServerRequest {
                    hostname: hostname.clone(),
                    plan,
                    region,
                    os,
                    ssh_keys,
                })
                .await?;

            println!("\n✅ Server created!");
            println!("  ID:       {}", server.id);
            println!("  Hostname: {}", server.hostname);
            println!("  Status:   {}", server.status);
            println!("  IPv4:     {}", server.ipv4.unwrap_or_default());
        }

        Commands::Delete { id } => {
            info!("Deleting server: {id}");
            provider.delete_server(&id).await?;
            println!("\n✅ Server deleted: {id}");
        }

        Commands::Talos {
            hostname,
            plan,
            region,
            ssh_keys,
            talos_version,
            timeout,
            skip_talos,
        } => {
            // Step 1: Create server with Ubuntu
            info!("Step 1/3: Creating server with Ubuntu...");
            let server = provider
                .create_server(CreateServerRequest {
                    hostname: hostname.clone(),
                    plan,
                    region,
                    os: "ubuntu_24_04_x64_lts".to_string(),
                    ssh_keys,
                })
                .await?;

            println!("\n✅ Server created: {}", server.id);
            println!("  IPv4: {}", server.ipv4.clone().unwrap_or_default());

            // Step 2: Wait for server to be ready
            info!("Step 2/3: Waiting for server to be ready (timeout: {timeout}s)...");
            let server = provider.wait_ready(&server.id, timeout).await?;
            println!("✅ Server is ready!");

            if skip_talos {
                println!("\n⏭️  Skipping Talos install (--skip-talos)");
                println!("  Server ID: {}", server.id);
                return Ok(());
            }

            // Step 3: Reinstall with Talos iPXE
            info!("Step 3/3: Triggering Talos iPXE boot...");
            let talos = TalosConfig::new("cto-cluster").with_version(
                metal::talos::TalosVersion::new(&talos_version, metal::talos::DEFAULT_SCHEMATIC_ID),
            );

            provider
                .reinstall_ipxe(
                    &server.id,
                    ReinstallIpxeRequest {
                        hostname,
                        ipxe_url: talos.ipxe_url(),
                    },
                )
                .await?;

            println!("✅ Talos iPXE reinstall triggered!");
            println!("\n📋 Next steps:");
            println!("  1. Wait ~10-15 min for Talos to boot");
            println!(
                "  2. Connect with: talosctl --nodes {} version --insecure",
                server.ipv4.unwrap_or_default()
            );
            println!("  3. Apply machine config to install to disk");
            println!("\n  Server ID: {}", server.id);
        }

        Commands::Reinstall {
            id,
            hostname,
            talos_version,
        } => {
            info!("Triggering Talos iPXE reinstall on server: {id}");

            let talos = TalosConfig::new("cto-cluster").with_version(
                metal::talos::TalosVersion::new(&talos_version, metal::talos::DEFAULT_SCHEMATIC_ID),
            );

            provider
                .reinstall_ipxe(
                    &id,
                    ReinstallIpxeRequest {
                        hostname,
                        ipxe_url: talos.ipxe_url(),
                    },
                )
                .await?;

            println!("\n✅ Talos iPXE reinstall triggered!");
            println!("  Server ID: {id}");
            println!("  iPXE URL:  {}", talos.ipxe_url());
        }

        Commands::Bootstrap {
            ip,
            cluster_name,
            install_disk,
            output_dir,
            talos_version,
            timeout,
        } => {
            info!("Bootstrapping Talos cluster on {ip}...");

            let config = BootstrapConfig::new(&cluster_name, &ip)
                .with_install_disk(&install_disk)
                .with_output_dir(&output_dir)
                .with_talos_version(&talos_version);

            // Step 1: Wait for Talos maintenance mode
            println!("\n📡 Step 1/7: Waiting for Talos maintenance mode...");
            talos::wait_for_talos(&ip, Duration::from_secs(timeout))?;

            // Step 2: Generate secrets
            println!("\n🔐 Step 2/7: Generating secrets...");
            talos::generate_secrets(&output_dir)?;

            // Step 3: Generate config
            println!("\n📝 Step 3/7: Generating machine config...");
            let configs = talos::generate_config(&config)?;

            // Step 4: Apply config
            println!("\n🚀 Step 4/7: Applying config (triggers install + reboot)...");
            talos::apply_config(&ip, &configs.controlplane)?;

            // Step 5: Wait for install
            println!("\n⏳ Step 5/7: Waiting for installation to complete...");
            talos::wait_for_install(&ip, &configs.talosconfig, Duration::from_secs(timeout))?;

            // Step 6: Bootstrap cluster
            println!("\n🎯 Step 6/7: Bootstrapping cluster...");
            talos::bootstrap_cluster(&ip, &configs.talosconfig)?;

            // Step 7: Wait for Kubernetes
            println!("\n☸️  Step 7/7: Waiting for Kubernetes API...");
            talos::wait_for_kubernetes(&ip, &configs.talosconfig, Duration::from_secs(300))?;

            // Get kubeconfig
            let kubeconfig_path = output_dir.join("kubeconfig");
            talos::get_kubeconfig(&ip, &configs.talosconfig, &kubeconfig_path)?;

            println!("\n🎉 Cluster bootstrapped successfully!");
            println!("\n📁 Generated files:");
            println!("   - {}", configs.controlplane.display());
            println!("   - {}", configs.worker.display());
            println!("   - {}", configs.talosconfig.display());
            println!("   - {}", kubeconfig_path.display());
            println!("\n📋 Next steps:");
            println!("   export KUBECONFIG={}", kubeconfig_path.display());
            println!("   kubectl get nodes");
        }

        Commands::Provision {
            hostname,
            cluster_name,
            plan,
            region,
            ssh_keys,
            talos_version,
            install_disk,
            output_dir,
            timeout,
        } => {
            info!("Full provisioning: {hostname} -> {cluster_name}");

            // Step 1: Create server with Ubuntu
            println!("\n🖥️  Step 1/10: Creating server with Ubuntu...");
            let server = provider
                .create_server(CreateServerRequest {
                    hostname: hostname.clone(),
                    plan,
                    region,
                    os: "ubuntu_24_04_x64_lts".to_string(),
                    ssh_keys,
                })
                .await?;

            println!("   Server ID: {}", server.id);
            let server_ip = server.ipv4.clone().unwrap_or_default();
            println!("   IPv4: {server_ip}");

            // Step 2: Wait for server to be ready
            println!("\n⏳ Step 2/10: Waiting for server to be ready...");
            let server = provider.wait_ready(&server.id, timeout).await?;
            let server_ip = server.ipv4.clone().unwrap_or_default();
            println!("   ✅ Server is ready!");

            // Step 3: Reinstall with Talos iPXE
            println!("\n🔄 Step 3/10: Triggering Talos iPXE boot...");
            let talos = TalosConfig::new(&cluster_name).with_version(
                metal::talos::TalosVersion::new(&talos_version, metal::talos::DEFAULT_SCHEMATIC_ID),
            );

            provider
                .reinstall_ipxe(
                    &server.id,
                    ReinstallIpxeRequest {
                        hostname: hostname.clone(),
                        ipxe_url: talos.ipxe_url(),
                    },
                )
                .await?;
            println!("   ✅ iPXE reinstall triggered!");

            // Step 4: Wait for Talos maintenance mode
            println!("\n📡 Step 4/10: Waiting for Talos maintenance mode...");
            talos::wait_for_talos(&server_ip, Duration::from_secs(timeout))?;

            // Step 5: Generate secrets
            println!("\n🔐 Step 5/10: Generating secrets...");
            talos::generate_secrets(&output_dir)?;

            // Step 6: Generate config
            println!("\n📝 Step 6/10: Generating machine config...");
            let config = BootstrapConfig::new(&cluster_name, &server_ip)
                .with_install_disk(&install_disk)
                .with_output_dir(&output_dir)
                .with_talos_version(&talos_version);
            let configs = talos::generate_config(&config)?;

            // Step 7: Apply config
            println!("\n🚀 Step 7/10: Applying config (triggers install + reboot)...");
            talos::apply_config(&server_ip, &configs.controlplane)?;

            // Step 8: Wait for install
            println!("\n⏳ Step 8/10: Waiting for installation to complete...");
            talos::wait_for_install(
                &server_ip,
                &configs.talosconfig,
                Duration::from_secs(timeout),
            )?;

            // Step 9: Bootstrap cluster
            println!("\n🎯 Step 9/10: Bootstrapping cluster...");
            talos::bootstrap_cluster(&server_ip, &configs.talosconfig)?;

            // Step 10: Wait for Kubernetes
            println!("\n☸️  Step 10/10: Waiting for Kubernetes API...");
            talos::wait_for_kubernetes(&server_ip, &configs.talosconfig, Duration::from_secs(300))?;

            // Get kubeconfig
            let kubeconfig_path = output_dir.join("kubeconfig");
            talos::get_kubeconfig(&server_ip, &configs.talosconfig, &kubeconfig_path)?;

            println!("\n🎉 Full provisioning complete!");
            println!("\n📊 Summary:");
            println!("   Server ID:    {}", server.id);
            println!("   Server IP:    {server_ip}");
            println!("   Cluster:      {cluster_name}");
            println!("\n📁 Generated files:");
            println!("   - {}", configs.controlplane.display());
            println!("   - {}", configs.worker.display());
            println!("   - {}", configs.talosconfig.display());
            println!("   - {}", kubeconfig_path.display());
            println!("\n📋 Next steps:");
            println!("   export KUBECONFIG={}", kubeconfig_path.display());
            println!("   kubectl get nodes");
        }

        Commands::Cluster {
            name,
            region,
            cp_plan,
            worker_plan,
            ssh_keys,
            talos_version,
            install_disk,
            output_dir,
            timeout,
            resume,
            deploy_stack,
            init_openbao,
        } => {
            info!("Provisioning 2-node cluster: {name}");

            let retry_config = RetryConfig::default();
            let cp_hostname = format!("{name}-cp1");
            let worker_hostname = format!("{name}-worker1");

            // Check for existing state to resume
            let mut state = if resume {
                if let Some(existing) = ClusterState::load(&output_dir)? {
                    if existing.can_resume() {
                        println!("📂 Found saved state at step {:?}", existing.step);
                        println!("   Resuming from last checkpoint...");
                        existing
                    } else {
                        println!("📂 Found completed/failed state, starting fresh");
                        ClusterState::new(&name, &output_dir)
                    }
                } else {
                    ClusterState::new(&name, &output_dir)
                }
            } else {
                ClusterState::new(&name, &output_dir)
            };

            // Create second provider instance for parallel ops
            let provider2 = Latitude::new(&api_key, &project_id)
                .context("Failed to create second Latitude provider")?;

            // Variables to track server state (may be restored from saved state)
            let (cp_id, worker_id, cp_addr, worker_addr) =
                if state.step >= ProvisionStep::WaitingServersReady {
                    // Restore from saved state
                    let cp = state
                        .control_plane
                        .as_ref()
                        .context("No control plane in saved state")?;
                    let wk = state.worker.as_ref().context("No worker in saved state")?;
                    println!("📂 Restored server info from state:");
                    println!("   Control Plane: {} ({})", cp.id, cp.ip);
                    println!("   Worker:        {} ({})", wk.id, wk.ip);
                    (cp.id.clone(), wk.id.clone(), cp.ip.clone(), wk.ip.clone())
                } else {
                    // Step 1: Create BOTH servers in parallel
                    state.set_step(ProvisionStep::CreatingServers)?;
                    println!("\n🖥️  Step 1/9: Creating both servers in parallel...");

                    let cp_req = CreateServerRequest {
                        hostname: cp_hostname.clone(),
                        plan: cp_plan,
                        region: region.clone(),
                        os: "ubuntu_24_04_x64_lts".to_string(),
                        ssh_keys: ssh_keys.clone(),
                    };
                    let worker_req = CreateServerRequest {
                        hostname: worker_hostname.clone(),
                        plan: worker_plan,
                        region,
                        os: "ubuntu_24_04_x64_lts".to_string(),
                        ssh_keys,
                    };

                    // Use retry for API calls
                    let (cp_server, worker_server) =
                        with_retry_async(&retry_config, "Create servers", || {
                            let p1 = &provider;
                            let p2 = &provider2;
                            let cp = cp_req.clone();
                            let wk = worker_req.clone();
                            async move {
                                tokio::try_join!(p1.create_server(cp), p2.create_server(wk))
                                    .map_err(|e| anyhow::anyhow!("{e}"))
                            }
                        })
                        .await?;

                    let cp_id = cp_server.id.clone();
                    let worker_id = worker_server.id.clone();
                    let cp_ip = cp_server.ipv4.clone().unwrap_or_default();
                    let wk_ip = worker_server.ipv4.clone().unwrap_or_default();

                    println!("   Control Plane: {cp_id} ({cp_ip})");
                    println!("   Worker:        {worker_id} ({wk_ip})");

                    // Save server info to state
                    state.set_control_plane(cp_id.clone(), cp_ip.clone(), cp_hostname.clone())?;
                    state.set_worker(worker_id.clone(), wk_ip.clone(), worker_hostname.clone())?;

                    // Step 2: Wait for BOTH servers to be ready in parallel
                    state.set_step(ProvisionStep::WaitingServersReady)?;
                    println!("\n⏳ Step 2/9: Waiting for both servers to be ready...");

                    let (cp_ready, worker_ready) =
                        with_retry_async(&retry_config, "Wait for servers", || {
                            let p1 = &provider;
                            let p2 = &provider2;
                            let cid = cp_id.clone();
                            let wid = worker_id.clone();
                            async move {
                                tokio::try_join!(
                                    p1.wait_ready(&cid, timeout),
                                    p2.wait_ready(&wid, timeout)
                                )
                                .map_err(|e| anyhow::anyhow!("{e}"))
                            }
                        })
                        .await?;

                    let cp_addr = cp_ready.ipv4.clone().unwrap_or_default();
                    let worker_addr = worker_ready.ipv4.clone().unwrap_or_default();
                    println!("   ✅ Control plane ready: {cp_addr}");
                    println!("   ✅ Worker ready: {worker_addr}");

                    // Update state with actual IPs
                    state.set_control_plane(cp_id.clone(), cp_addr.clone(), cp_hostname.clone())?;
                    state.set_worker(
                        worker_id.clone(),
                        worker_addr.clone(),
                        worker_hostname.clone(),
                    )?;

                    (cp_id, worker_id, cp_addr, worker_addr)
                };

            // Step 3: Trigger Talos iPXE on BOTH (skip if already past this step)
            if state.step < ProvisionStep::WaitingTalos {
                state.set_step(ProvisionStep::WaitingTalos)?;
                println!("\n🔄 Step 3/9: Triggering Talos iPXE boot on both...");

                let talos_cfg =
                    TalosConfig::new(&name).with_version(metal::talos::TalosVersion::new(
                        &talos_version,
                        metal::talos::DEFAULT_SCHEMATIC_ID,
                    ));
                let ipxe_url = talos_cfg.ipxe_url();

                let cp_ipxe = ReinstallIpxeRequest {
                    hostname: cp_hostname.clone(),
                    ipxe_url: ipxe_url.clone(),
                };
                let worker_ipxe = ReinstallIpxeRequest {
                    hostname: worker_hostname.clone(),
                    ipxe_url,
                };

                with_retry_async(&retry_config, "Trigger iPXE", || {
                    let p1 = &provider;
                    let p2 = &provider2;
                    let cid = cp_id.clone();
                    let wid = worker_id.clone();
                    let cipxe = cp_ipxe.clone();
                    let wipxe = worker_ipxe.clone();
                    async move {
                        tokio::try_join!(
                            p1.reinstall_ipxe(&cid, cipxe),
                            p2.reinstall_ipxe(&wid, wipxe)
                        )
                        .map_err(|e| anyhow::anyhow!("{e}"))
                    }
                })
                .await?;
                println!("   ✅ iPXE triggered on both servers!");
            } else {
                println!("\n⏭️  Step 3/9: Skipping iPXE (already triggered)");
            }

            // Step 4: Wait for BOTH to enter Talos maintenance mode
            if state.step <= ProvisionStep::WaitingTalos {
                println!("\n📡 Step 4/9: Waiting for Talos maintenance mode on both...");
                println!("   (This typically takes 10-15 minutes for iPXE boot)");

                let cp_poll = cp_addr.clone();
                let worker_poll = worker_addr.clone();
                let timeout_duration = Duration::from_secs(timeout);

                let mut set = JoinSet::new();
                set.spawn(async move {
                    talos::wait_for_talos(&cp_poll, timeout_duration)?;
                    Ok::<_, anyhow::Error>(("cp", cp_poll))
                });
                set.spawn(async move {
                    talos::wait_for_talos(&worker_poll, timeout_duration)?;
                    Ok::<_, anyhow::Error>(("worker", worker_poll))
                });

                while let Some(result) = set.join_next().await {
                    match result {
                        Ok(Ok((node, addr))) => {
                            println!("   ✅ {node} Talos ready: {addr}");
                            if node == "cp" {
                                state.set_cp_talos_ready()?;
                            } else {
                                state.set_worker_talos_ready()?;
                            }
                        }
                        Ok(Err(e)) => anyhow::bail!("Failed waiting for Talos: {e}"),
                        Err(e) => anyhow::bail!("Task panicked: {e}"),
                    }
                }
            } else {
                println!("\n⏭️  Step 4/9: Skipping Talos wait (already ready)");
            }

            // Step 5: Generate secrets and config
            if state.step < ProvisionStep::GeneratingConfigs {
                state.set_step(ProvisionStep::GeneratingConfigs)?;
                println!("\n🔐 Step 5/9: Generating secrets and configs...");
                talos::generate_secrets(&output_dir)?;
            } else {
                println!("\n⏭️  Step 5/9: Skipping config generation (already done)");
            }

            let config = BootstrapConfig::new(&name, &cp_addr)
                .with_install_disk(&install_disk)
                .with_output_dir(&output_dir)
                .with_talos_version(&talos_version);
            let configs = talos::generate_config(&config)?;
            let kubeconfig_path = output_dir.join("kubeconfig");

            // Step 6: Apply config to control plane
            if state.step < ProvisionStep::ApplyingCpConfig {
                state.set_step(ProvisionStep::ApplyingCpConfig)?;
                println!("\n🚀 Step 6/9: Applying config to control plane...");
                talos::apply_config(&cp_addr, &configs.controlplane)?;
            } else {
                println!("\n⏭️  Step 6/9: Skipping CP config (already applied)");
            }

            // Step 7: Wait for control plane install and bootstrap
            if state.step < ProvisionStep::Bootstrapping {
                state.set_step(ProvisionStep::WaitingCpInstall)?;
                println!("\n⏳ Step 7/9: Waiting for control plane installation...");
                talos::wait_for_install(
                    &cp_addr,
                    &configs.talosconfig,
                    Duration::from_secs(timeout),
                )?;

                state.set_step(ProvisionStep::Bootstrapping)?;
                println!("\n🎯 Bootstrapping control plane...");
                talos::bootstrap_cluster(&cp_addr, &configs.talosconfig)?;

                state.set_step(ProvisionStep::WaitingKubernetes)?;
                println!("\n☸️  Waiting for Kubernetes API...");
                talos::wait_for_kubernetes(
                    &cp_addr,
                    &configs.talosconfig,
                    Duration::from_secs(300),
                )?;

                // Get kubeconfig
                talos::get_kubeconfig(&cp_addr, &configs.talosconfig, &kubeconfig_path)?;
                println!("   ✅ Control plane bootstrapped!");
            } else {
                println!("\n⏭️  Step 7/9: Skipping bootstrap (already done)");
            }

            // Step 8: Apply config to worker and join
            if state.step < ProvisionStep::ApplyingWorkerConfig {
                state.set_step(ProvisionStep::ApplyingWorkerConfig)?;
                println!("\n🔄 Step 8/9: Applying config to worker node...");
                talos::apply_config(&worker_addr, &configs.worker)?;
            } else {
                println!("\n⏭️  Step 8/9: Skipping worker config (already applied)");
            }

            // Step 9: Wait for worker to join
            let total_steps = if deploy_stack { 10 } else { 9 };
            if state.step < ProvisionStep::Complete {
                state.set_step(ProvisionStep::WaitingWorkerJoin)?;
                println!("\n⏳ Step 9/{total_steps}: Waiting for worker to join cluster...");
                talos::wait_for_install(
                    &worker_addr,
                    &configs.talosconfig,
                    Duration::from_secs(timeout),
                )?;
                talos::wait_for_node_ready(&kubeconfig_path, Duration::from_secs(300))?;
                state.set_step(ProvisionStep::Complete)?;
            } else {
                println!("\n⏭️  Step 9/{total_steps}: Skipping worker join (already complete)");
            }

            println!("\n🎉 2-node cluster provisioned successfully!");
            println!("\n📊 Summary:");
            println!("   Cluster:        {name}");
            println!("   Control Plane:  {cp_addr} ({cp_id})");
            println!("   Worker:         {worker_addr} ({worker_id})");
            println!("\n📁 Generated files:");
            println!("   - {}", configs.controlplane.display());
            println!("   - {}", configs.worker.display());
            println!("   - {}", configs.talosconfig.display());
            println!("   - {}", kubeconfig_path.display());

            // Step 10: Deploy platform stack (optional)
            if deploy_stack {
                println!("\n📦 Step 10/{total_steps}: Deploying platform stack...");
                println!(
                    "   This installs: Cert-Manager, ArgoCD, OpenBao, Ingress-NGINX, Argo Workflows"
                );

                // Install local-path-provisioner for bare metal PVCs
                // Note: Pass `true` for multi-region clusters where provisioner needs CP scheduling
                println!("\n   Installing local-path-provisioner (for bare metal storage)...");
                stack::deploy_local_path_provisioner(&kubeconfig_path, false)?;

                // Deploy core components
                println!("\n   Deploying Cert-Manager...");
                stack::deploy_cert_manager(&kubeconfig_path)?;

                println!("\n   Deploying ArgoCD...");
                stack::deploy_argocd(&kubeconfig_path)?;

                println!("\n   Deploying OpenBao...");
                stack::deploy_openbao(&kubeconfig_path)?;

                println!("\n   Deploying Ingress-NGINX...");
                if let Err(e) = stack::deploy_ingress_nginx(&kubeconfig_path) {
                    // Ingress may timeout on bare metal (no LoadBalancer), but pods still deploy
                    println!("   ⚠️  Ingress deployment warning: {e}");
                    println!("   (This is expected on bare metal - pods are still deploying)");
                }

                println!("\n   Deploying Argo Workflows...");
                stack::deploy_argo_workflows(&kubeconfig_path)?;

                println!("\n   ✅ Platform stack deployed!");

                // Initialize and unseal OpenBao if requested
                if init_openbao {
                    println!("\n🔐 Initializing and unsealing OpenBao...");
                    let openbao_init = stack::init_openbao(&kubeconfig_path)?;

                    // Save keys to file
                    let openbao_keys_path = output_dir.join("openbao-keys.json");
                    let keys_json = serde_json::json!({
                        "unseal_keys": openbao_init.unseal_keys,
                        "root_token": openbao_init.root_token,
                    });
                    std::fs::write(
                        &openbao_keys_path,
                        serde_json::to_string_pretty(&keys_json)?,
                    )?;

                    // Unseal with first key
                    if let Some(key) = openbao_init.unseal_keys.first() {
                        stack::unseal_openbao(&kubeconfig_path, key)?;
                    }

                    println!("   ✅ OpenBao initialized and unsealed!");
                    println!("   Root token: {}", openbao_init.root_token);
                    println!("   Keys saved to: {}", openbao_keys_path.display());
                }

                // Get ArgoCD password
                if let Ok(password) = stack::get_argocd_password(&kubeconfig_path) {
                    println!("\n🔑 ArgoCD Credentials:");
                    println!("   Username: admin");
                    println!("   Password: {password}");
                }
            }

            println!("\n📁 State file (for resume):");
            println!("   - {}", ClusterState::state_file(&output_dir).display());
            println!("\n📋 Next steps:");
            println!("   export KUBECONFIG={}", kubeconfig_path.display());
            println!("   kubectl get nodes");
            if !deploy_stack {
                println!("\n💡 To deploy stack:");
                println!("   metal stack --kubeconfig {}", kubeconfig_path.display());
            }
            println!("\n💡 To resume if interrupted:");
            println!(
                "   metal cluster --name {name} --resume --output-dir {}",
                output_dir.display()
            );
        }

        Commands::Join {
            hostname,
            plan,
            region,
            ssh_keys,
            talos_version,
            install_disk: _, // Worker config already has install disk
            worker_config,
            talosconfig,
            kubeconfig,
            timeout,
            control_plane_id,
        } => {
            info!("Joining worker node: {hostname}");

            // Verify configs exist
            if !worker_config.exists() {
                anyhow::bail!("Worker config not found: {}", worker_config.display());
            }
            if !talosconfig.exists() {
                anyhow::bail!("Talosconfig not found: {}", talosconfig.display());
            }

            // Validate same-site if control plane ID is provided
            if let Some(ref cp_id) = control_plane_id {
                println!("\n🔍 Validating same-site configuration...");
                let cp_server = provider
                    .get_server(cp_id)
                    .await
                    .context("Failed to get control plane server info")?;

                // Extract site slug from control plane's region
                let cp_site = cp_server.region.to_uppercase();
                let worker_site = region.to_uppercase();

                if cp_site != worker_site {
                    anyhow::bail!(
                        "❌ Site mismatch! Worker region '{}' does not match control plane site '{}'.\n\n\
                         VLAN networking requires all cluster nodes to be in the SAME site.\n\
                         Cross-site clusters will have broken networking:\n\
                         - Worker won't have access to the private VLAN\n\
                         - DNS resolution will fail\n\
                         - Service routing will be unreliable\n\n\
                         Please use --region {} to create the worker in the same site.",
                        region, cp_site, cp_site
                    );
                }

                println!("   ✅ Site validation passed: both nodes will be in {}", cp_site);
            } else {
                println!("\n⚠️  Warning: --control-plane-id not provided, skipping same-site validation.");
                println!("   For VLAN networking, ensure worker is in the same site as control plane.");
            }

            // Step 1: Create server with Ubuntu
            println!("\n🖥️  Step 1/6: Creating worker server with Ubuntu...");
            let server = provider
                .create_server(CreateServerRequest {
                    hostname: hostname.clone(),
                    plan,
                    region,
                    os: "ubuntu_24_04_x64_lts".to_string(),
                    ssh_keys,
                })
                .await?;

            println!("   Server ID: {}", server.id);
            let server_ip = server.ipv4.clone().unwrap_or_default();
            println!("   IPv4: {server_ip}");

            // Step 2: Wait for server to be ready
            println!("\n⏳ Step 2/6: Waiting for server to be ready...");
            let server = provider.wait_ready(&server.id, timeout).await?;
            let server_ip = server.ipv4.clone().unwrap_or_default();
            println!("   ✅ Server is ready!");

            // Step 3: Reinstall with Talos iPXE
            println!("\n🔄 Step 3/6: Triggering Talos iPXE boot...");
            let talos = TalosConfig::new("worker").with_version(metal::talos::TalosVersion::new(
                &talos_version,
                metal::talos::DEFAULT_SCHEMATIC_ID,
            ));

            provider
                .reinstall_ipxe(
                    &server.id,
                    ReinstallIpxeRequest {
                        hostname: hostname.clone(),
                        ipxe_url: talos.ipxe_url(),
                    },
                )
                .await?;
            println!("   ✅ iPXE reinstall triggered!");

            // Step 4: Wait for Talos maintenance mode
            println!("\n📡 Step 4/6: Waiting for Talos maintenance mode...");
            talos::wait_for_talos(&server_ip, Duration::from_secs(timeout))?;

            // Step 5: Apply worker config
            println!("\n🚀 Step 5/6: Applying worker config (triggers install + reboot)...");
            talos::apply_config(&server_ip, &worker_config)?;

            // Step 6: Wait for install and node to join
            println!("\n⏳ Step 6/6: Waiting for worker to join cluster...");
            talos::wait_for_install(&server_ip, &talosconfig, Duration::from_secs(timeout))?;

            // Wait for node to appear in kubectl
            println!("\n☸️  Waiting for node to register with Kubernetes...");
            talos::wait_for_node_ready(&kubeconfig, Duration::from_secs(300))?;

            println!("\n🎉 Worker node joined successfully!");
            println!("\n📊 Summary:");
            println!("   Server ID:    {}", server.id);
            println!("   Server IP:    {server_ip}");
            println!("   Hostname:     {hostname}");
            println!("\n📋 Verify with:");
            println!("   kubectl get nodes");
        }

        Commands::Stack {
            kubeconfig,
            argocd_only,
            cert_manager_only,
            openbao_only,
            ingress_only,
            init_openbao,
        } => {
            info!("Deploying CTO platform stack...");

            // Check if kubeconfig exists
            if !kubeconfig.exists() {
                anyhow::bail!("Kubeconfig not found: {}", kubeconfig.display());
            }

            // Deploy specific component or full stack
            if argocd_only {
                println!("\n🚀 Deploying ArgoCD...");
                stack::deploy_argocd(&kubeconfig)?;
            } else if cert_manager_only {
                println!("\n🔐 Deploying Cert-Manager...");
                stack::deploy_cert_manager(&kubeconfig)?;
            } else if openbao_only {
                println!("\n🔒 Deploying OpenBao...");
                stack::deploy_openbao(&kubeconfig)?;
            } else if ingress_only {
                println!("\n🌐 Deploying Ingress-NGINX...");
                stack::deploy_ingress_nginx(&kubeconfig)?;
            } else {
                // Deploy full stack
                println!("\n📦 Deploying full CTO platform stack...");
                println!(
                    "   Components: Cert-Manager, ArgoCD, OpenBao, Ingress-NGINX, Argo Workflows"
                );

                println!("\n🔐 Step 1/5: Deploying Cert-Manager...");
                stack::deploy_cert_manager(&kubeconfig)?;

                println!("\n🚀 Step 2/5: Deploying ArgoCD...");
                stack::deploy_argocd(&kubeconfig)?;

                println!("\n🔒 Step 3/5: Deploying OpenBao...");
                stack::deploy_openbao(&kubeconfig)?;

                println!("\n🌐 Step 4/5: Deploying Ingress-NGINX...");
                stack::deploy_ingress_nginx(&kubeconfig)?;

                println!("\n⚙️  Step 5/5: Deploying Argo Workflows...");
                stack::deploy_argo_workflows(&kubeconfig)?;
            }

            // Get ArgoCD password if ArgoCD was deployed
            if !cert_manager_only && !openbao_only && !ingress_only {
                println!("\n📊 Stack deployment complete!");

                if let Ok(password) = stack::get_argocd_password(&kubeconfig) {
                    println!("\n🔑 ArgoCD Credentials:");
                    println!("   Username: admin");
                    println!("   Password: {password}");
                }

                // Initialize OpenBao if requested
                if init_openbao {
                    println!("\n🔐 Initializing OpenBao...");
                    match stack::init_openbao(&kubeconfig) {
                        Ok(openbao_creds) => {
                            // Unseal OpenBao with the first key
                            if let Some(key) = openbao_creds.unseal_keys.first() {
                                if let Err(e) = stack::unseal_openbao(&kubeconfig, key) {
                                    println!("   ⚠️  OpenBao unseal failed: {e}");
                                    println!("   Run manually: metal openbao-unseal --kubeconfig {} --unseal-key {key}", kubeconfig.display());
                                } else {
                                    println!("   ✅ OpenBao initialized and unsealed!");
                                }
                            } else {
                                println!("   ⚠️  No unseal keys returned from OpenBao init");
                            }
                            println!("\n🔑 OpenBao Credentials (SAVE THESE!):");
                            println!(
                                "   Unseal Key: {}",
                                openbao_creds.unseal_keys.first().unwrap_or(&String::new())
                            );
                            println!("   Root Token: {}", openbao_creds.root_token);
                        }
                        Err(e) => {
                            println!("   ⚠️  OpenBao init: {e}");
                        }
                    }
                } else {
                    println!("\n⚠️  OpenBao needs initialization. Run:");
                    println!(
                        "   metal openbao-init --kubeconfig {}",
                        kubeconfig.display()
                    );
                }

                println!("\n📋 Access services:");
                println!("   ArgoCD:  kubectl port-forward svc/argocd-server -n argocd 8080:443");
                println!("   OpenBao: kubectl port-forward svc/openbao -n openbao 8200:8200");
            }

            println!("\n🎉 Stack deployment successful!");
        }

        Commands::OpenbaoInit { kubeconfig, output } => {
            info!("Initializing OpenBao...");

            if !kubeconfig.exists() {
                anyhow::bail!("Kubeconfig not found: {}", kubeconfig.display());
            }

            match stack::init_openbao(&kubeconfig) {
                Ok(openbao_creds) => {
                    // Unseal OpenBao with the first key
                    if let Some(key) = openbao_creds.unseal_keys.first() {
                        stack::unseal_openbao(&kubeconfig, key)?;
                        println!("\n🔐 OpenBao initialized and unsealed!");
                    } else {
                        println!("\n🔐 OpenBao initialized (no unseal keys returned)!");
                    }
                    println!("\n🔑 OpenBao Credentials:");
                    println!(
                        "   Unseal Key: {}",
                        openbao_creds.unseal_keys.first().unwrap_or(&String::new())
                    );
                    println!("   Root Token: {}", openbao_creds.root_token);

                    // Save to file
                    let json = serde_json::json!({
                        "unseal_keys_b64": openbao_creds.unseal_keys,
                        "root_token": openbao_creds.root_token,
                    });
                    std::fs::write(&output, serde_json::to_string_pretty(&json)?)?;
                    println!("\n💾 Credentials saved to: {}", output.display());
                }
                Err(e) => {
                    println!("❌ OpenBao initialization failed: {e}");
                    return Err(e);
                }
            }
        }

        Commands::OpenbaoUnseal {
            kubeconfig,
            unseal_key,
        } => {
            info!("Unsealing OpenBao...");

            if !kubeconfig.exists() {
                anyhow::bail!("Kubeconfig not found: {}", kubeconfig.display());
            }

            stack::unseal_openbao(&kubeconfig, &unseal_key)?;
            println!("\n🔓 OpenBao unsealed successfully!");
        }

        Commands::MayastorPoc {
            name,
            region,
            cp_plan,
            worker_plan,
            ssh_keys,
            talos_version,
            install_disk,
            output_dir,
            timeout,
            mayastor_chart_version,
            mayastor_namespace,
            disk_uri,
            storage_class,
            bench_pvc_size,
            bench_runtime_seconds,
        } => {
            info!("Mayastor POC: provisioning 3-node cluster: {name}");

            let retry_config = RetryConfig::default();
            let cp_hostname = format!("{name}-cp1");
            let worker1_hostname = format!("{name}-worker1");
            let worker2_hostname = format!("{name}-worker2");

            // Step 1: Create all 3 servers in parallel
            println!("\n🖥️  Step 1/12: Creating 3 servers in parallel...");
            let cp_req = CreateServerRequest {
                hostname: cp_hostname.clone(),
                plan: cp_plan,
                region: region.clone(),
                os: "ubuntu_24_04_x64_lts".to_string(),
                ssh_keys: ssh_keys.clone(),
            };
            let w1_req = CreateServerRequest {
                hostname: worker1_hostname.clone(),
                plan: worker_plan.clone(),
                region: region.clone(),
                os: "ubuntu_24_04_x64_lts".to_string(),
                ssh_keys: ssh_keys.clone(),
            };
            let w2_req = CreateServerRequest {
                hostname: worker2_hostname.clone(),
                plan: worker_plan,
                region,
                os: "ubuntu_24_04_x64_lts".to_string(),
                ssh_keys,
            };

            let provider2 = Latitude::new(&api_key, &project_id)
                .context("Failed to create second Latitude provider")?;
            let provider3 = Latitude::new(&api_key, &project_id)
                .context("Failed to create third Latitude provider")?;

            let (cp_server, w1_server, w2_server) =
                with_retry_async(&retry_config, "Create 3 servers", || {
                    let p1 = &provider;
                    let p2 = &provider2;
                    let p3 = &provider3;
                    let cp = cp_req.clone();
                    let w1 = w1_req.clone();
                    let w2 = w2_req.clone();
                    async move {
                        let (a, b, c) = tokio::try_join!(
                            p1.create_server(cp),
                            p2.create_server(w1),
                            p3.create_server(w2)
                        )
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                        Ok::<_, anyhow::Error>((a, b, c))
                    }
                })
                .await?;

            let cp_id = cp_server.id.clone();
            let w1_id = w1_server.id.clone();
            let w2_id = w2_server.id.clone();
            println!("   Control Plane: {cp_id}");
            println!("   Worker1:       {w1_id}");
            println!("   Worker2:       {w2_id}");

            // Step 2: Wait for all servers to be ready
            println!("\n⏳ Step 2/12: Waiting for all servers to be ready...");
            let (cp_ready, w1_ready, w2_ready) =
                with_retry_async(&retry_config, "Wait for 3 servers", || {
                    let p1 = &provider;
                    let p2 = &provider2;
                    let p3 = &provider3;
                    let cp = cp_id.clone();
                    let w1 = w1_id.clone();
                    let w2 = w2_id.clone();
                    async move {
                        let (a, b, c) = tokio::try_join!(
                            p1.wait_ready(&cp, timeout),
                            p2.wait_ready(&w1, timeout),
                            p3.wait_ready(&w2, timeout)
                        )
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                        Ok::<_, anyhow::Error>((a, b, c))
                    }
                })
                .await?;

            let cp_addr = cp_ready.ipv4.clone().unwrap_or_default();
            let w1_addr = w1_ready.ipv4.clone().unwrap_or_default();
            let w2_addr = w2_ready.ipv4.clone().unwrap_or_default();
            println!("   ✅ Control plane ready: {cp_addr}");
            println!("   ✅ Worker1 ready:       {w1_addr}");
            println!("   ✅ Worker2 ready:       {w2_addr}");

            // Step 3: Trigger Talos iPXE on all 3
            println!("\n🔄 Step 3/12: Triggering Talos iPXE boot on all 3 nodes...");
            let talos_cfg = TalosConfig::new(&name).with_version(metal::talos::TalosVersion::new(
                &talos_version,
                metal::talos::DEFAULT_SCHEMATIC_ID,
            ));
            let ipxe_url = talos_cfg.ipxe_url();
            with_retry_async(&retry_config, "Trigger iPXE", || {
                let p1 = &provider;
                let p2 = &provider2;
                let p3 = &provider3;
                let cp = cp_id.clone();
                let w1 = w1_id.clone();
                let w2 = w2_id.clone();
                let cp_req = ReinstallIpxeRequest {
                    hostname: cp_hostname.clone(),
                    ipxe_url: ipxe_url.clone(),
                };
                let w1_req = ReinstallIpxeRequest {
                    hostname: worker1_hostname.clone(),
                    ipxe_url: ipxe_url.clone(),
                };
                let w2_req = ReinstallIpxeRequest {
                    hostname: worker2_hostname.clone(),
                    ipxe_url: ipxe_url.clone(),
                };
                async move {
                    tokio::try_join!(
                        p1.reinstall_ipxe(&cp, cp_req),
                        p2.reinstall_ipxe(&w1, w1_req),
                        p3.reinstall_ipxe(&w2, w2_req)
                    )
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                    Ok::<_, anyhow::Error>(())
                }
            })
            .await?;
            println!("   ✅ iPXE triggered on all 3 servers!");

            // Step 4: Wait for Talos maintenance mode on all 3
            println!("\n📡 Step 4/12: Waiting for Talos maintenance mode on all 3 nodes...");
            println!("   (This typically takes 10-15 minutes for iPXE boot)");
            let timeout_duration = Duration::from_secs(timeout);
            let mut set = JoinSet::new();
            for (label, ip) in [
                ("cp", cp_addr.clone()),
                ("worker1", w1_addr.clone()),
                ("worker2", w2_addr.clone()),
            ] {
                set.spawn(async move {
                    talos::wait_for_talos(&ip, timeout_duration)?;
                    Ok::<_, anyhow::Error>((label, ip))
                });
            }
            while let Some(result) = set.join_next().await {
                match result {
                    Ok(Ok((node, addr))) => println!("   ✅ {node} Talos ready: {addr}"),
                    Ok(Err(e)) => anyhow::bail!("Failed waiting for Talos: {e}"),
                    Err(e) => anyhow::bail!("Task panicked: {e}"),
                }
            }

            // Step 5: Generate secrets + configs
            println!("\n🔐 Step 5/12: Generating Talos secrets and machine configs...");
            talos::generate_secrets(&output_dir)?;
            let config = BootstrapConfig::new(&name, &cp_addr)
                .with_install_disk(&install_disk)
                .with_output_dir(&output_dir)
                .with_talos_version(&talos_version);
            let configs = talos::generate_config(&config)?;

            // Step 6: Apply config to control plane
            println!("\n🚀 Step 6/12: Applying control plane config (install + reboot)...");
            talos::apply_config(&cp_addr, &configs.controlplane)?;

            // Step 7: Wait for CP install + bootstrap + kubeconfig
            println!("\n⏳ Step 7/12: Waiting for control plane installation...");
            talos::wait_for_install(&cp_addr, &configs.talosconfig, Duration::from_secs(timeout))?;

            println!("\n🎯 Step 8/12: Bootstrapping control plane...");
            talos::bootstrap_cluster(&cp_addr, &configs.talosconfig)?;

            println!("\n☸️  Step 9/12: Waiting for Kubernetes API...");
            talos::wait_for_kubernetes(&cp_addr, &configs.talosconfig, Duration::from_secs(300))?;

            let kubeconfig_path = output_dir.join("kubeconfig");
            talos::get_kubeconfig(&cp_addr, &configs.talosconfig, &kubeconfig_path)?;

            // Step 10: Apply worker config to both workers
            println!("\n🔄 Step 10/12: Applying worker configs (install + reboot)...");
            talos::apply_config(&w1_addr, &configs.worker)?;
            talos::apply_config(&w2_addr, &configs.worker)?;

            // Step 11: Wait for workers to install + join
            println!("\n⏳ Step 11/12: Waiting for workers to install and join...");
            let mut set = JoinSet::new();
            let tcfg = configs.talosconfig.clone();
            let w1 = w1_addr.clone();
            set.spawn(async move {
                talos::wait_for_install(&w1, &tcfg, Duration::from_secs(timeout))?;
                Ok::<_, anyhow::Error>(w1)
            });
            let tcfg2 = configs.talosconfig.clone();
            let w2 = w2_addr.clone();
            set.spawn(async move {
                talos::wait_for_install(&w2, &tcfg2, Duration::from_secs(timeout))?;
                Ok::<_, anyhow::Error>(w2)
            });
            while let Some(result) = set.join_next().await {
                match result {
                    Ok(Ok(addr)) => println!("   ✅ Worker installed: {addr}"),
                    Ok(Err(e)) => anyhow::bail!("Failed waiting for worker install: {e}"),
                    Err(e) => anyhow::bail!("Task panicked: {e}"),
                }
            }
            talos::wait_for_node_ready(&kubeconfig_path, Duration::from_secs(600))?;

            // Step 12: Install Mayastor + pools + storageclass + fio
            println!("\n📦 Step 12/12: Installing Mayastor and running fio benchmark...");
            stack::deploy_mayastor(
                &kubeconfig_path,
                &mayastor_namespace,
                &mayastor_chart_version,
            )?;

            // Discover node names (for DiskPool.spec.node)
            let nodes = Command::new("kubectl")
                .arg("--kubeconfig")
                .arg(&kubeconfig_path)
                .args(["get", "nodes", "-o", "jsonpath={.items[*].metadata.name}"])
                .output()
                .context("Failed to list Kubernetes nodes")?;
            if !nodes.status.success() {
                anyhow::bail!("kubectl get nodes failed");
            }
            let nodes = String::from_utf8_lossy(&nodes.stdout);
            let node_names: Vec<&str> = nodes.split_whitespace().collect();
            println!("   Nodes: {}", node_names.join(", "));

            for node in &node_names {
                let pool_name = format!("pool-{node}");
                stack::create_mayastor_diskpool(
                    &kubeconfig_path,
                    &mayastor_namespace,
                    &pool_name,
                    node,
                    &disk_uri,
                )?;
            }
            println!("   ✅ DiskPools created (one per node)");

            stack::create_mayastor_storage_class(&kubeconfig_path, &storage_class, 3, false)?;
            println!("   ✅ StorageClass created: {storage_class}");

            let fio_logs = stack::run_fio_benchmark_job(
                &kubeconfig_path,
                "bench",
                "mayastor-fio",
                &storage_class,
                &bench_pvc_size,
                bench_runtime_seconds,
            )?;

            std::fs::create_dir_all(&output_dir)?;
            let bench_log_path = output_dir.join("mayastor-fio.log");
            std::fs::write(&bench_log_path, &fio_logs)?;
            println!("\n📈 fio logs saved to: {}", bench_log_path.display());

            println!("\n🎉 Mayastor POC complete!");
            println!("\n📊 Summary:");
            println!("   Cluster:        {name}");
            println!("   Control Plane:  {cp_addr} ({cp_id})");
            println!("   Worker1:        {w1_addr} ({w1_id})");
            println!("   Worker2:        {w2_addr} ({w2_id})");
            println!("   Mayastor NS:    {mayastor_namespace}");
            println!("   StorageClass:   {storage_class} (repl=3)");
            println!("\n📋 Next steps:");
            println!("   export KUBECONFIG={}", kubeconfig_path.display());
            println!("   kubectl get nodes");
            println!("   kubectl get dsp -n {mayastor_namespace}");
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

    // If missing, try 1Password (either explicitly or as a fallback).
    if !cli.use_1password && api_key.is_empty() && project_id.is_empty() {
        // still attempt, since user preference is to pull from 1Password when not provided.
    }

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
            "Latitude credentials are missing. Provide --api-key/--project-id (or env vars), or configure 1Password flags: --op-latitude-item/--op-latitude-*-field"
        );
    }

    Ok((api_key, project_id))
}

fn ensure_op_ready() -> Result<()> {
    let version = Command::new("op").arg("--version").output();
    if version.is_err() {
        anyhow::bail!("1Password CLI `op` not found. Install with: brew install 1password-cli, then run: eval $(op signin)");
    }

    // Verify auth (do not attempt signin automatically because it requires shell eval).
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
    let out = cmd.output().context("Failed to execute `op item get`")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("`op item get` failed: {stderr}");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
