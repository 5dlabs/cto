//! Metal CLI - Bare metal provisioning tool for CTO Platform.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use cto_metal::providers::latitude::Latitude;
use cto_metal::providers::{CreateServerRequest, Provider, ReinstallIpxeRequest};
use cto_metal::talos::{self, BootstrapConfig, TalosConfig};
use tokio::task::JoinSet;

/// Metal CLI - Bare metal provisioning for CTO Platform.
#[derive(Parser)]
#[command(name = "metal")]
#[command(about = "Provision and manage bare metal servers")]
struct Cli {
    /// Latitude.sh API key (or set `LATITUDE_API_KEY` env var).
    #[arg(long, env = "LATITUDE_API_KEY")]
    api_key: String,

    /// Latitude.sh Project ID (or set `LATITUDE_PROJECT_ID` env var).
    #[arg(long, env = "LATITUDE_PROJECT_ID")]
    project_id: String,

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

    // Create provider
    let provider = Latitude::new(&cli.api_key, &cli.project_id)
        .context("Failed to create Latitude provider")?;

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
            let talos =
                TalosConfig::new("cto-cluster").with_version(cto_metal::talos::TalosVersion::new(
                    &talos_version,
                    cto_metal::talos::DEFAULT_SCHEMATIC_ID,
                ));

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

            let talos =
                TalosConfig::new("cto-cluster").with_version(cto_metal::talos::TalosVersion::new(
                    &talos_version,
                    cto_metal::talos::DEFAULT_SCHEMATIC_ID,
                ));

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
            let talos =
                TalosConfig::new(&cluster_name).with_version(cto_metal::talos::TalosVersion::new(
                    &talos_version,
                    cto_metal::talos::DEFAULT_SCHEMATIC_ID,
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
        } => {
            info!("Provisioning 2-node cluster: {name}");

            let cp_hostname = format!("{name}-cp1");
            let worker_hostname = format!("{name}-worker1");

            // Create second provider instance for parallel ops
            let provider2 = Latitude::new(&cli.api_key, &cli.project_id)
                .context("Failed to create second Latitude provider")?;

            // Step 1: Create BOTH servers in parallel
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

            let (cp_server, worker_server) = tokio::try_join!(
                provider.create_server(cp_req),
                provider2.create_server(worker_req),
            )?;

            let cp_id = cp_server.id.clone();
            let worker_id = worker_server.id.clone();
            println!(
                "   Control Plane: {} ({})",
                cp_id,
                cp_server.ipv4.clone().unwrap_or_default()
            );
            println!(
                "   Worker:        {} ({})",
                worker_id,
                worker_server.ipv4.clone().unwrap_or_default()
            );

            // Step 2: Wait for BOTH servers to be ready in parallel
            println!("\n⏳ Step 2/9: Waiting for both servers to be ready...");
            let (cp_ready, worker_ready) = tokio::try_join!(
                provider.wait_ready(&cp_id, timeout),
                provider2.wait_ready(&worker_id, timeout),
            )?;

            let cp_addr = cp_ready.ipv4.clone().unwrap_or_default();
            let worker_addr = worker_ready.ipv4.clone().unwrap_or_default();
            println!("   ✅ Control plane ready: {cp_addr}");
            println!("   ✅ Worker ready: {worker_addr}");

            // Step 3: Trigger Talos iPXE on BOTH in parallel
            println!("\n🔄 Step 3/9: Triggering Talos iPXE boot on both...");
            let talos_cfg =
                TalosConfig::new(&name).with_version(cto_metal::talos::TalosVersion::new(
                    &talos_version,
                    cto_metal::talos::DEFAULT_SCHEMATIC_ID,
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

            tokio::try_join!(
                provider.reinstall_ipxe(&cp_id, cp_ipxe),
                provider2.reinstall_ipxe(&worker_id, worker_ipxe),
            )?;
            println!("   ✅ iPXE triggered on both servers!");

            // Step 4: Wait for BOTH to enter Talos maintenance mode in parallel
            println!("\n📡 Step 4/9: Waiting for Talos maintenance mode on both...");
            println!("   (This typically takes 10-15 minutes for iPXE boot)");

            // Use a JoinSet to poll both in parallel
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
                    Ok(Ok((node, addr))) => println!("   ✅ {node} Talos ready: {addr}"),
                    Ok(Err(e)) => anyhow::bail!("Failed waiting for Talos: {e}"),
                    Err(e) => anyhow::bail!("Task panicked: {e}"),
                }
            }

            // Step 5: Generate secrets and config
            println!("\n🔐 Step 5/9: Generating secrets and configs...");
            talos::generate_secrets(&output_dir)?;

            let config = BootstrapConfig::new(&name, &cp_addr)
                .with_install_disk(&install_disk)
                .with_output_dir(&output_dir)
                .with_talos_version(&talos_version);
            let configs = talos::generate_config(&config)?;

            // Step 6: Apply config to control plane
            println!("\n🚀 Step 6/9: Applying config to control plane...");
            talos::apply_config(&cp_addr, &configs.controlplane)?;

            // Step 7: Wait for control plane install and bootstrap
            println!("\n⏳ Step 7/9: Waiting for control plane installation...");
            talos::wait_for_install(&cp_addr, &configs.talosconfig, Duration::from_secs(timeout))?;

            println!("\n🎯 Bootstrapping control plane...");
            talos::bootstrap_cluster(&cp_addr, &configs.talosconfig)?;

            println!("\n☸️  Waiting for Kubernetes API...");
            talos::wait_for_kubernetes(&cp_addr, &configs.talosconfig, Duration::from_secs(300))?;

            // Get kubeconfig
            let kubeconfig_path = output_dir.join("kubeconfig");
            talos::get_kubeconfig(&cp_addr, &configs.talosconfig, &kubeconfig_path)?;
            println!("   ✅ Control plane bootstrapped!");

            // Step 8: Apply config to worker and join
            println!("\n🔄 Step 8/9: Applying config to worker node...");
            talos::apply_config(&worker_addr, &configs.worker)?;

            // Step 9: Wait for worker to join
            println!("\n⏳ Step 9/9: Waiting for worker to join cluster...");
            talos::wait_for_install(
                &worker_addr,
                &configs.talosconfig,
                Duration::from_secs(timeout),
            )?;
            talos::wait_for_node_ready(&kubeconfig_path, Duration::from_secs(300))?;

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
            println!("\n📋 Next steps:");
            println!("   export KUBECONFIG={}", kubeconfig_path.display());
            println!("   kubectl get nodes");
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
        } => {
            info!("Joining worker node: {hostname}");

            // Verify configs exist
            if !worker_config.exists() {
                anyhow::bail!("Worker config not found: {}", worker_config.display());
            }
            if !talosconfig.exists() {
                anyhow::bail!("Talosconfig not found: {}", talosconfig.display());
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
            let talos =
                TalosConfig::new("worker").with_version(cto_metal::talos::TalosVersion::new(
                    &talos_version,
                    cto_metal::talos::DEFAULT_SCHEMATIC_ID,
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
    }

    Ok(())
}
