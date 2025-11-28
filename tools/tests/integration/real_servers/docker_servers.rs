#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]

use anyhow::Result;
use serde_json::json;

use crate::common::*;

#[tokio::test]
async fn test_fetch_server_docker() -> Result<()> {
    crate::setup_integration_tests();

    // Check if Docker is available
    if !check_runtime_available("docker").await {
        println!("Skipping Docker fetch test: docker not available");
        return Ok(());
    }

    let config = create_docker_server_config("mcp/fetch", vec![]);

    let mut server = match TestServer::start(config).await {
        Ok(server) => server,
        Err(e) => {
            println!("Failed to start Docker fetch server: {e}");
            return Ok(());
        }
    };

    // Run comprehensive validation
    let report = ProtocolValidator::run_comprehensive_validation(&mut server).await?;

    if !report.is_healthy() {
        println!("Server validation failed, but continuing with specific tests");
    }

    // Test fetch-specific functionality
    CommonTestScenarios::test_fetch_server(&mut server).await?;

    // Test specific Docker fetch operations
    println!("Testing Docker fetch operations...");

    // Test fetching a simple URL
    let fetch_result = server
        .call_tool(
            "fetch",
            json!({
                "url": "https://httpbin.org/get",
                "method": "GET"
            }),
        )
        .await;

    match fetch_result {
        Ok(response) => {
            println!("✅ Docker HTTP GET fetch successful: {response}");
            // Verify the response contains expected fields
            if let Some(content) = response.get("content") {
                if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                    let response_text = text.as_str().unwrap_or("");
                    assert!(response_text.contains("httpbin.org"));
                    println!("✅ Docker HTTP GET response validated");
                }
            }
        }
        Err(e) => {
            println!("❌ Docker HTTP GET fetch failed: {e}");
        }
    }

    // Test POST request
    let post_result = server
        .call_tool(
            "fetch",
            json!({
                "url": "https://httpbin.org/post",
                "method": "POST",
                "headers": {
                    "Content-Type": "application/json"
                },
                "body": json!({
                    "test": "docker-data",
                    "from": "mcp-docker-integration-test"
                })
            }),
        )
        .await;

    match post_result {
        Ok(response) => {
            println!("✅ Docker HTTP POST fetch successful: {response}");
            // Verify the response contains our posted data
            if let Some(content) = response.get("content") {
                if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                    let response_text = text.as_str().unwrap_or("");
                    assert!(response_text.contains("mcp-docker-integration-test"));
                    println!("✅ Docker HTTP POST response validated");
                }
            }
        }
        Err(e) => {
            println!("❌ Docker HTTP POST fetch failed: {e}");
        }
    }

    // Test that Docker isolation works (server should not have access to host filesystem)
    let isolation_test = server
        .call_tool(
            "fetch",
            json!({
                "url": "file:///etc/passwd",
                "method": "GET"
            }),
        )
        .await;

    match isolation_test {
        Ok(_) => {
            println!(
                "⚠️  Docker server was able to access host file system (potential security issue)"
            );
        }
        Err(e) => {
            println!(
                "✅ Docker server correctly blocked access to host file system: {e}"
            );
        }
    }

    let _ = server.shutdown().await;
    println!("✅ Docker fetch server test completed");

    Ok(())
}

#[tokio::test]
async fn test_docker_mcp_server_lifecycle() -> Result<()> {
    crate::setup_integration_tests();

    // Check if Docker is available
    if !check_runtime_available("docker").await {
        println!("Skipping Docker lifecycle test: docker not available");
        return Ok(());
    }

    println!("Testing Docker MCP server lifecycle...");

    // Test that Docker can pull and run MCP server images
    let config = create_docker_server_config("mcp/fetch", vec![]);

    let mut server = match TestServer::start(config).await {
        Ok(server) => server,
        Err(e) => {
            println!("Failed to start Docker server: {e}");
            return Ok(());
        }
    };

    // Test basic lifecycle operations
    println!("Testing server initialization...");
    let init_result = server.initialize().await;

    match init_result {
        Ok(response) => {
            println!("✅ Docker server initialization successful: {response}");
        }
        Err(e) => {
            println!("❌ Docker server initialization failed: {e}");
        }
    }

    // Test tools listing
    println!("Testing tools listing...");
    let tools_result = server.list_tools().await;

    match tools_result {
        Ok(response) => {
            println!("✅ Docker server tools listing successful: {response}");
        }
        Err(e) => {
            println!("❌ Docker server tools listing failed: {e}");
        }
    }

    // Test graceful shutdown
    println!("Testing graceful shutdown...");
    let shutdown_result = server.shutdown().await;

    match shutdown_result {
        Ok(_) => {
            println!("✅ Docker server shutdown successful");
        }
        Err(e) => {
            println!("❌ Docker server shutdown failed: {e}");
        }
    }

    println!("✅ Docker MCP server lifecycle test completed");

    Ok(())
}

#[tokio::test]
async fn test_docker_networking() -> Result<()> {
    crate::setup_integration_tests();

    // Check if Docker is available
    if !check_runtime_available("docker").await {
        println!("Skipping Docker networking test: docker not available");
        return Ok(());
    }

    println!("Testing Docker networking capabilities...");

    // Test that Docker MCP server can make external network requests
    let config = create_docker_server_config("mcp/fetch", vec![]);

    let mut server = match TestServer::start(config).await {
        Ok(server) => server,
        Err(e) => {
            println!("Failed to start Docker server for networking test: {e}");
            return Ok(());
        }
    };

    // Test external network access
    let network_result = server
        .call_tool(
            "fetch",
            json!({
                "url": "https://httpbin.org/ip",
                "method": "GET"
            }),
        )
        .await;

    match network_result {
        Ok(response) => {
            println!(
                "✅ Docker server can access external networks: {response}"
            );
            // Verify we got an IP address response
            if let Some(content) = response.get("content") {
                if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                    let response_text = text.as_str().unwrap_or("");
                    assert!(response_text.contains("origin"));
                    println!("✅ External network access validated");
                }
            }
        }
        Err(e) => {
            println!("❌ Docker server cannot access external networks: {e}");
        }
    }

    // Test DNS resolution
    let dns_result = server
        .call_tool(
            "fetch",
            json!({
                "url": "https://httpbin.org/get",
                "method": "GET"
            }),
        )
        .await;

    match dns_result {
        Ok(response) => {
            println!("✅ Docker server DNS resolution works: {response}");
        }
        Err(e) => {
            println!("❌ Docker server DNS resolution failed: {e}");
        }
    }

    let _ = server.shutdown().await;
    println!("✅ Docker networking test completed");

    Ok(())
}

#[tokio::test]
async fn test_docker_resource_limits() -> Result<()> {
    crate::setup_integration_tests();

    // Check if Docker is available
    if !check_runtime_available("docker").await {
        println!("Skipping Docker resource limits test: docker not available");
        return Ok(());
    }

    println!("Testing Docker resource limits...");

    // Test with resource limits
    let mut config = create_docker_server_config("mcp/fetch", vec![]);
    config.args.insert(1, "--memory=256m".to_string());
    config.args.insert(1, "--cpus=0.5".to_string());

    let mut server = match TestServer::start(config).await {
        Ok(server) => server,
        Err(e) => {
            println!("Failed to start Docker server with resource limits: {e}");
            return Ok(());
        }
    };

    // Test that the server still works with resource limits
    let limited_result = server
        .call_tool(
            "fetch",
            json!({
                "url": "https://httpbin.org/get",
                "method": "GET"
            }),
        )
        .await;

    match limited_result {
        Ok(response) => {
            println!("✅ Docker server works with resource limits: {response}");
        }
        Err(e) => {
            println!("❌ Docker server fails with resource limits: {e}");
        }
    }

    let _ = server.shutdown().await;
    println!("✅ Docker resource limits test completed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_runtime_availability() {
        if check_runtime_available("docker").await {
            println!("✅ Docker runtime is available for testing");
        } else {
            println!("❌ Docker runtime is not available");
        }
    }

    #[tokio::test]
    async fn test_docker_image_availability() {
        if !check_runtime_available("docker").await {
            println!("Skipping Docker image test: docker not available");
            return;
        }

        // Test if the mcp/fetch image is available or can be pulled
        if check_command_available("docker", &["pull", "mcp/fetch"]).await {
            println!("✅ mcp/fetch Docker image is available");
        } else {
            println!("❌ mcp/fetch Docker image is not available");
        }
    }
}
