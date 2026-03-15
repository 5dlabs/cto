//! Example: Create a 2-node Scaleway Elastic Metal cluster
//!
//! Usage:
//!     SCALEWAY_SECRET_KEY=xxx SCALEWAY_PROJECT_ID=xxx \
//!     cargo run -p metal --example scaleway_cluster
//!
//! This will create 2 Elastic Metal servers and wait for them to be ready.

use std::env;
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

    println!("\n🧪 Scaleway 2-Node Cluster Test\n");
    println!("Project: {}", project_id);
    println!("Zone: {}", zone);
    println!("SSH Key: {}", ssh_key_id);

    let em = metal::providers::scaleway::Scaleway::new(&secret_key, "", &project_id, &zone)?;

    // Cluster config
    let cluster_nodes = vec![
        ("em-node-1", "8779d2c1-cd10-4a34-a006-cb5b1fb5cbc7"), // EM-A116X-SSD
        ("em-node-2", "8779d2c1-cd10-4a34-a006-cb5b1fb5cbc7"), // EM-A116X-SSD
    ];

    let os_id = "7d1914e1-f4ab-47fc-bd8c-b3a23143e87a"; // Ubuntu

    println!("\n🚀 Creating 2-node cluster...\n");

    let mut created_servers = Vec::new();

    for (i, (hostname, offer_id)) in cluster_nodes.iter().enumerate() {
        println!("[{}/2] Creating {}...", i + 1, hostname);

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

        println!("   ✅ Created: {} ({})", server.hostname, server.id);
        created_servers.push(server);
    }

    println!("\n⏳ Waiting for all nodes to be ready (timeout: 10 min)...\n");

    // Wait for all servers to be ready
    let mut ready_count = 0;
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(600);

    while ready_count < created_servers.len() {
        for server in &created_servers {
            if server.status == metal::ServerStatus::On {
                continue;
            }

            match em.get_server(&server.id).await {
                Ok(updated) => {
                    println!("   {}: {:?}", updated.hostname, updated.status);
                    if updated.status == metal::ServerStatus::On {
                        ready_count += 1;
                    }
                }
                Err(e) => {
                    println!("   Error checking {}: {}", server.hostname, e);
                }
            }
        }

        if ready_count < created_servers.len() {
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    println!("\n🎉 All {} nodes ready!\n", ready_count);

    // List final servers
    let servers = em.list_servers().await?;
    println!("📋 Final cluster state:\n");
    for s in &servers {
        println!(
            "   - {} | {} | {:?}",
            s.hostname,
            s.ipv4.clone().unwrap_or_default(),
            s.status
        );
    }

    // Delete cluster on Ctrl+C or with DELETE env var
    if std::env::var("DELETE_CLUSTER").is_ok() {
        println!("\n🗑️  Deleting cluster...\n");
        for server in created_servers {
            em.delete_server(&server.id).await?;
            println!("   ✅ Deleted: {}", server.hostname);
        }
        println!("\n✅ Cluster deleted!");
    } else {
        println!("\n💡 To delete cluster: DELETE_CLUSTER=1 cargo run -p metal --example scaleway_cluster");
    }

    Ok(())
}
