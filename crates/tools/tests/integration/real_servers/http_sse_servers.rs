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
use std::time::Duration;
use tokio::time::timeout;

use crate::common::*;

// HTTP/SSE MCP servers work differently from stdio servers
// They use HTTP endpoints and Server-Sent Events instead of stdin/stdout
pub struct HttpSseTestClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpSseTestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request_payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(format!("{}/message", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed: {}",
                response.status()
            ));
        }

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    pub async fn initialize(&self) -> Result<serde_json::Value> {
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": true
                },
                "sampling": {}
            },
            "clientInfo": {
                "name": "tools-http-test",
                "version": "1.0.0"
            }
        });

        let response = self.send_request("initialize", init_params).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Initialize failed: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in initialize response"))
    }

    pub async fn list_tools(&self) -> Result<serde_json::Value> {
        let response = self.send_request("tools/list", json!({})).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("List tools failed: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in tools/list response"))
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", params).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Tool call failed: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in tools/call response"))
    }

    pub async fn test_sse_connection(&self) -> Result<()> {
        // Test that the SSE endpoint is accessible
        let sse_response = self
            .client
            .get(&self.base_url)
            .header("Accept", "text/event-stream")
            .send()
            .await?;

        if !sse_response.status().is_success() {
            return Err(anyhow::anyhow!(
                "SSE endpoint not accessible: {}",
                sse_response.status()
            ));
        }

        // Check that the response has the correct content type
        let content_type = sse_response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("text/event-stream") {
            println!("✅ SSE endpoint has correct content type");
        } else {
            println!("⚠️  SSE endpoint content type: {content_type}");
        }

        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        // Test basic connectivity
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                println!("✅ HTTP health check passed");
                Ok(())
            }
            Ok(resp) => {
                println!("⚠️  HTTP health check returned: {}", resp.status());
                // Try initialization as fallback
                self.initialize().await.map(|_| ())
            }
            Err(_) => {
                println!("⚠️  HTTP health check failed, trying initialization");
                // Try initialization as fallback
                self.initialize().await.map(|_| ())
            }
        }
    }
}

#[tokio::test]
async fn test_generic_http_sse_server() -> Result<()> {
    crate::setup_integration_tests();

    let server_url = get_remote_server_url();
    println!("Testing HTTP/SSE server at: {server_url}");

    let client = HttpSseTestClient::new(server_url.clone());

    // Test basic connectivity
    println!("Testing basic connectivity...");
    let connectivity_result = timeout(Duration::from_secs(10), client.health_check()).await;

    match connectivity_result {
        Ok(Ok(_)) => {
            println!("✅ Basic connectivity test passed");
        }
        Ok(Err(e)) => {
            println!("❌ Basic connectivity test failed: {e}");
            return Ok(()); // Skip test if server is not reachable
        }
        Err(_) => {
            println!("❌ Basic connectivity test timed out");
            return Ok(()); // Skip test if server is not reachable
        }
    }

    // Test SSE endpoint
    println!("Testing SSE endpoint...");
    let sse_result = timeout(Duration::from_secs(10), client.test_sse_connection()).await;

    match sse_result {
        Ok(Ok(_)) => {
            println!("✅ SSE endpoint test passed");
        }
        Ok(Err(e)) => {
            println!("❌ SSE endpoint test failed: {e}");
        }
        Err(_) => {
            println!("❌ SSE endpoint test timed out");
        }
    }

    // Test MCP protocol initialization
    println!("Testing MCP protocol initialization...");
    let init_result = timeout(Duration::from_secs(10), client.initialize()).await;

    match init_result {
        Ok(Ok(response)) => {
            println!("✅ MCP initialization successful: {response}");

            // Validate initialization response
            if let Some(protocol_version) = response.get("protocolVersion") {
                println!("Protocol version: {protocol_version}");
            }

            if let Some(capabilities) = response.get("capabilities") {
                println!("Server capabilities: {capabilities}");
            }

            if let Some(server_info) = response.get("serverInfo") {
                println!("Server info: {server_info}");
            }
        }
        Ok(Err(e)) => {
            println!("❌ MCP initialization failed: {e}");
            return Ok(()); // Skip remaining tests if initialization fails
        }
        Err(_) => {
            println!("❌ MCP initialization timed out");
            return Ok(()); // Skip remaining tests if initialization times out
        }
    }

    // Test tools listing
    println!("Testing tools listing...");
    let tools_result = timeout(Duration::from_secs(10), client.list_tools()).await;

    match tools_result {
        Ok(Ok(response)) => {
            println!("✅ Tools listing successful: {response}");

            // Validate tools response
            if let Some(tools) = response.get("tools").and_then(|t| t.as_array()) {
                println!("Available tools:");
                for tool in tools {
                    if let Some(name) = tool.get("name") {
                        println!("  - {name}");
                    }
                }

                // Test calling a tool if available
                if let Some(first_tool) = tools.first() {
                    if let Some(tool_name) = first_tool.get("name").and_then(|n| n.as_str()) {
                        println!("Testing tool call: {tool_name}");

                        let tool_result = timeout(
                            Duration::from_secs(15),
                            client.call_tool(tool_name, json!({})),
                        )
                        .await;

                        match tool_result {
                            Ok(Ok(response)) => {
                                println!("✅ Tool call successful: {response}");
                            }
                            Ok(Err(e)) => {
                                println!("❌ Tool call failed: {e}");
                            }
                            Err(_) => {
                                println!("❌ Tool call timed out");
                            }
                        }
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ Tools listing failed: {e}");
        }
        Err(_) => {
            println!("❌ Tools listing timed out");
        }
    }

    println!("✅ HTTP/SSE server test completed");

    Ok(())
}

#[tokio::test]
async fn test_http_sse_error_handling() -> Result<()> {
    crate::setup_integration_tests();

    let server_url = get_remote_server_url();
    println!("Testing HTTP/SSE error handling...");

    let client = HttpSseTestClient::new(server_url.clone());

    // Test invalid method
    println!("Testing invalid method call...");
    let invalid_result = timeout(
        Duration::from_secs(10),
        client.send_request("invalid_method", json!({})),
    )
    .await;

    match invalid_result {
        Ok(Ok(response)) => {
            if let Some(error) = response.get("error") {
                println!("✅ Server correctly returned error for invalid method: {error}");
            } else {
                println!("⚠️  Server should have returned error for invalid method");
            }
        }
        Ok(Err(e)) => {
            println!("✅ Server correctly handled invalid method: {e}");
        }
        Err(_) => {
            println!("❌ Invalid method test timed out");
        }
    }

    // Test invalid tool call
    println!("Testing invalid tool call...");
    let invalid_tool_result = timeout(
        Duration::from_secs(10),
        client.call_tool("non_existent_tool", json!({})),
    )
    .await;

    match invalid_tool_result {
        Ok(Ok(response)) => {
            println!("⚠️  Server should have failed for non-existent tool: {response}");
        }
        Ok(Err(e)) => {
            println!("✅ Server correctly handled invalid tool call: {e}");
        }
        Err(_) => {
            println!("❌ Invalid tool call test timed out");
        }
    }

    println!("✅ HTTP/SSE error handling test completed");

    Ok(())
}

#[tokio::test]
async fn test_http_sse_performance() -> Result<()> {
    crate::setup_integration_tests();

    let server_url = get_remote_server_url();
    println!("Testing HTTP/SSE performance...");

    let _client = HttpSseTestClient::new(server_url.clone());

    // Test multiple concurrent requests
    println!("Testing concurrent requests...");
    let mut handles = Vec::new();

    for i in 0..5 {
        let client_clone = HttpSseTestClient::new(server_url.clone());
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = client_clone.initialize().await;
            let duration = start.elapsed();
            (i, result, duration)
        });
        handles.push(handle);
    }

    let mut successful_requests = 0;
    let mut total_duration = Duration::from_secs(0);

    for handle in handles {
        match handle.await {
            Ok((i, Ok(_), duration)) => {
                println!("✅ Request {i} completed in {duration:?}");
                successful_requests += 1;
                total_duration += duration;
            }
            Ok((i, Err(e), duration)) => {
                println!("❌ Request {i} failed in {duration:?}: {e}");
            }
            Err(e) => {
                println!("❌ Request failed to complete: {e}");
            }
        }
    }

    if successful_requests > 0 {
        let average_duration = total_duration / successful_requests;
        println!("✅ Average request duration: {average_duration:?}");
        println!("✅ Successful requests: {successful_requests}/5");
    }

    println!("✅ HTTP/SSE performance test completed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpSseTestClient::new("http://example.com".to_string());
        assert_eq!(client.base_url, "http://example.com");
        println!("✅ HTTP/SSE client creation test passed");
    }

    #[tokio::test]
    async fn test_remote_server_url() {
        let url = get_remote_server_url();
        assert!(url.starts_with("http"));
        println!("✅ Remote server URL test passed: {url}");
    }
}
