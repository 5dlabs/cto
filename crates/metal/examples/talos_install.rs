//! Example: Install Talos Kubernetes on Scaleway Elastic Metal
//!
//! Usage:
//!     SCALEWAY_SECRET_KEY=xxx SCALEWAY_PROJECT_ID=xxx \
//!     cargo run -p metal --example talos_install
//!
//! This will:
//!     1. Provision 2 Elastic Metal servers
//!     2. Wait for them to be ready
//!     3. Show Talos bootstrap module usage

use std::env;
use std::time::Duration;

use metal::Provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let secret_key = env::var("SCALEWAY_SECRET_KEY")
        .expect("Set SCALEWAY_SECRET_KEY env var");
    let project_id = env::var("SCALEWAY_PROJECT_ID")
        .expect("Set SCALEWAY_PROJECT_ID env var");
    let zone = env::var("SCALEWAY_ZONE").unwrap_or_else(|_| "fr-par-1".to_string());
    let ssh_key_id = env::var("SCALEWAY_SSH_KEY_ID")
        .unwrap_or_else(|_| "97bb8cbc-5757-4cb2-ad2f-f272e189f223".to_string());

    println!("\n☸️  Talos Kubernetes on Scaleway Elastic Metal\n");
    println!("Project: {}", project_id);
    println!("Zone: {}", zone);

    let em = metal::providers::scaleway::Scaleway::new(
        &secret_key,
        "",
        &project_id,
        &zone,
    )?;

    // Cluster config
    let offer_id = "8779d2c1-cd10-4a34-a006-cb5b1fb5cbc7"; // EM-A116X-SSD
    let os_id = "7d1914e1-f4ab-47fc-bd8c-b3a23143e87a"; // Ubuntu

    println!("\n🚀 Provisioning 2-node cluster...");

    // Create 2 servers
    let mut nodes = Vec::new();
    for hostname in ["talos-cp", "talos-worker"] {
        println!("   Creating {}...", hostname);

        let server = em.create_server(metal::CreateServerRequest {
            hostname: hostname.to_string(),
            plan: offer_id.to_string(),
            os: os_id.to_string(),
            ssh_keys: vec![ssh_key_id.clone()],
            ip_addresses: vec![],
            region: zone.clone(),
        }).await?;

        println!("   ✅ {}: {}", hostname, server.id);
        nodes.push(server);
    }

    println!("\n⏳ Waiting for servers to be ready...");

    // Wait for all nodes
    let mut servers = Vec::new();
    for mut server in nodes {
        loop {
            match em.get_server(&server.id).await {
                Ok(updated) => {
                    println!("   {}: {:?}", updated.hostname, updated.status);
                    if updated.status == metal::ServerStatus::On {
                        servers.push(updated);
                        break;
                    }
                }
                Err(e) => {
                    println!("   Error: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    println!("\n🎉 All {} servers ready!", servers.len());

    println!("\n📦 Talos Bootstrap Module Usage\n");
    println!("Nodes:");
    for s in &servers {
        println!("   - {} @ {}", s.hostname, s.ipv4.clone().unwrap_or_default());
    }

    println!(r#"
💡 Talos Bootstrap Example:

    use metal::talos::{{BootstrapConfig, generate_config, apply_config, bootstrap_cluster}};

    let config = BootstrapConfig::new("my-cluster", &servers[0].ipv4)
        .with_talos_version("v1.9.0")
        .with_install_disk("/dev/sda");

    // Generate Talos machine configs
    let configs = generate_config(&config)?;

    // Apply config to each node (via talosctl)
    apply_config(&config, &configs).await?;

    // Bootstrap the cluster
    bootstrap_cluster(&config).await?;

    // Get kubeconfig
    get_kubeconfig(&config)?;
"#);

    println!("\n📖 Full Talos docs: agents/metal/infra/talos/README.md");

    Ok(())
}
