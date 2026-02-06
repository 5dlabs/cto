//! Example: Test Scaleway Elastic Metal provider
//!
//! Usage:
//!     SCALEWAY_SECRET_KEY=xxx SCALEWAY_PROJECT_ID=xxx \
//!     cargo run -p metal --example scaleway_test
//!
//! This will:
//!     1. List existing servers
//!     2. Create a test server (if ARGS="create")
//!     3. Delete it immediately (if ARGS="create")

use std::env;

use metal::Provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let secret_key = env::var("SCALEWAY_SECRET_KEY")
        .expect("Set SCALEWAY_SECRET_KEY env var");
    let project_id = env::var("SCALEWAY_PROJECT_ID")
        .expect("Set SCALEWAY_PROJECT_ID env var");
    let zone = env::var("SCALEWAY_ZONE").unwrap_or_else(|_| "fr-par-1".to_string());

    println!("\n🧪 Scaleway Elastic Metal Provider Test\n");
    println!("Project: {}", project_id);
    println!("Zone: {}", zone);

    // Test Elastic Metal
    println!("\n🔩 Testing Elastic Metal...");
    let em = metal::providers::scaleway::Scaleway::new(
        &secret_key,
        "", // org_id not needed for bare metal API v1
        &project_id,
        &zone,
    )?;

    let em_servers = em.list_servers().await?;
    println!("   Found {} Elastic Metal server(s)", em_servers.len());
    for s in &em_servers {
        println!("   - {} | {} | {:?}", s.hostname, s.ipv4.clone().unwrap_or_default(), s.status);
    }

    // Check for create flag
    if env::args().nth(1).as_deref() == Some("create") {
        println!("\n🚀 Creating test server...");

        // Create a test server (hourly billing)
        println!("   Creating Elastic Metal server 'test-em'...");
        let server = em.create_server(metal::CreateServerRequest {
            hostname: "test-em".to_string(),
            plan: "8779d2c1-cd10-4a34-a006-cb5b1fb5cbc7".to_string(), // EM-A116X-SSD (offer ID)
            os: "7d1914e1-f4ab-47fc-bd8c-b3a23143e87a".to_string(), // Ubuntu (OS ID)
            ssh_keys: vec!["97bb8cbc-5757-4cb2-ad2f-f272e189f223".to_string()], // SSH key ID
            ip_addresses: vec![],
            region: zone.clone(),
        }).await?;
        println!("   ✅ Created: {} ({})", server.hostname, server.id);

        // Wait for it to be ready
        println!("   Waiting for server to be ready...");
        let ready = em.wait_ready(&server.id, 300).await?;
        println!("   ✅ Server ready: {} ({})", ready.hostname, ready.ipv4.clone().unwrap_or_default());

        // Delete immediately
        println!("   Deleting server...");
        em.delete_server(&server.id).await?;
        println!("   ✅ Deleted: {}", server.id);

        println!("\n✅ Test complete!");
    }

    println!("\n🎉 Elastic Metal provider working!");
    Ok(())
}
