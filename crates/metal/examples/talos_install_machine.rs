//! Example: Install Talos Kubernetes on Scaleway Elastic Metal using Talos-in-Machine
//!
//! Usage:
//!     SCALEWAY_SECRET_KEY=xxx SCALEWAY_PROJECT_ID=xxx \
//!     cargo run -p metal --example talos_install_machine
//!
//! This will:
//!     1. Provision 2 Elastic Metal servers
//!     2. Wait for them to be ready
//!     3. Install Talos using talosctl (Talos-in-Machine approach)

use std::env;
use std::process::Command;
use std::time::Duration;

use metal::Provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let secret_key = env::var("SCALEWAY_SECRET_KEY").expect("Set SCALEWAY_SECRET_KEY env var");
    let project_id = env::var("SCALEWAY_PROJECT_ID").expect("Set SCALEWAY_PROJECT_ID env var");
    let zone = env::var("SCALEWAY_ZONE").unwrap_or_else(|_| "fr-par-1".to_string());
    let ssh_key_id = env::var("SCALEWAY_SSH_KEY_ID")
        .unwrap_or_else(|_| "97bb8cbc-5757-4cb2-ad2f-f272e189f223".to_string());

    println!("\n☸️  Talos Kubernetes on Scaleway Elastic Metal\n");
    println!("Project: {}", project_id);
    println!("Zone: {}", zone);

    let em = metal::providers::scaleway::Scaleway::new(&secret_key, "", &project_id, &zone)?;

    let offer_id = "8779d2c1-cd10-4a34-a006-cb5b1fb5cbc7"; // EM-A116X-SSD
    let os_id = "7d1914e1-f4ab-47fc-bd8c-b3a23143e87a"; // Ubuntu

    println!("\n🚀 Provisioning 2-node cluster...");

    let mut nodes = Vec::new();
    for hostname in ["talos-cp", "talos-worker"] {
        println!("   Creating {}...", hostname);

        let server = em
            .create_server(metal::CreateServerRequest {
                hostname: hostname.to_string(),
                plan: offer_id.to_string(),
                os: os_id.to_string(),
                ssh_keys: vec![ssh_key_id.clone()],
                ip_addresses: vec![],
                region: zone.clone(),
            })
            .await?;

        println!("   ✅ {}: {}", hostname, server.id);
        nodes.push(server);
    }

    println!("\n⏳ Waiting for servers to be ready...");

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

    println!("\n📋 Nodes:");
    for s in &servers {
        println!(
            "   - {} @ {}",
            s.hostname,
            s.ipv4.clone().unwrap_or_default()
        );
    }

    println!("\n💡 Talos-in-Machine Installation Steps:\n");
    println!("For each server, SSH in and run:\n");

    let cp_ip = servers[0].ipv4.clone().unwrap_or_default();
    println!("=== Control Plane ({}) ===", cp_ip);
    println!(
        r#"# SSH into the server
ssh ubuntu@{cp_ip}

# Download Talos installer
curl -sL https://tal.dev/install | bash -s -

# Download machine config
curl -sLO https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.9.0/metal-amd64

# Apply config (control plane)
talosctl apply-config --insecure --nodes {cp_ip} --file controlplane.yaml

# Bootstrap
talosctl bootstrap --nodes {cp_ip}

# Get kubeconfig
talosctl kubeconfig -n {cp_ip}

# Repeat for worker nodes with worker.yaml
"#
    );

    println!("\n📖 See: agents/metal/infra/talos/README.md for full instructions");
    println!("\n🧹 To clean up when done:");
    for s in &servers {
        println!(
            "   scw baremetal server delete zone=fr-par-1 server-id={}",
            s.id
        );
    }

    Ok(())
}
