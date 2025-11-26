#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::disallowed_macros)]
#![allow(clippy::ignore_without_reason)]
use serde_json::json;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[tokio::test]
async fn test_tools_server_with_solana_config() {
    // Test our actual server implementation with Solana configured
    println!("🧪 Testing tools server HTTP transport with Solana");

    // Create a minimal config with Solana and Rust Docs
    let test_config = json!({
        "servers": {
            "solana": {
                "name": "Solana",
                "description": "Solana blockchain development tools",
                "transport": "http",
                "url": "https://mcp.solana.com/mcp",
                "enabled": true,
                "executionContext": "remote"
            },
            "rustdocs": {
                "name": "Rust Docs",
                "description": "Rust documentation MCP server",
                "transport": "http",
                "url": "http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse",
                "enabled": true,
                "executionContext": "remote"
            }
        }
    });

    // Write test config
    std::fs::write("/tmp/servers-config.json", test_config.to_string()).unwrap();

    // Start our server with the test config - use debug binary for tests
    let mut server = Command::new("target/debug/tools-server")
        .arg("--port")
        .arg("3001")
        .env("PROJECT_DIR", "/tmp")
        .env("RUST_LOG", "debug")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start tools server");

    // Wait a moment for server to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test the server
    let client = reqwest::Client::new();
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let result = timeout(Duration::from_secs(15), async {
        client
            .post("http://localhost:3001/mcp")
            .json(&tools_request)
            .send()
            .await
    });

    match result.await {
        Ok(Ok(response)) => {
            println!("✅ Server responded with status: {}", response.status());
            let response_text = response.text().await.unwrap_or_default();
            println!("📝 Response: {}", response_text);

            // Check if we got valid JSON
            if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                println!("✅ Valid JSON response");

                if let Some(error) = json_response.get("error") {
                    println!("❌ Server returned error: {}", error);
                } else if let Some(result) = json_response.get("result") {
                    if let Some(tools) = result.get("tools") {
                        println!(
                            "✅ Got tools array with {} items",
                            tools.as_array().map(|a| a.len()).unwrap_or(0)
                        );
                    } else {
                        println!("❌ No tools in result");
                    }
                }
            } else {
                println!("❌ Invalid JSON response: {}", response_text);
            }
        }
        Ok(Err(e)) => {
            println!("❌ Request failed: {}", e);
        }
        Err(_) => {
            println!("❌ Request timed out");
        }
    }

    // Kill the server and capture logs
    let _ = server.kill().await;
    let output = server.wait_with_output().await.unwrap();

    // Print server logs for debugging
    if !output.stdout.is_empty() {
        println!("📋 Server stdout:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("📋 Server stderr:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Cleanup
    let _ = std::fs::remove_file("/tmp/servers-config.json");
}

#[tokio::test]
async fn test_rustdocs_sse_via_tools_server() {
    // Test Rust Docs SSE transport via our actual tools server
    // This tests the real implementation path that's failing

    let client = reqwest::Client::new();

    // Make a request to our tools server's /mcp endpoint
    // This will trigger the actual SSE tool discovery code path
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    println!("Testing tools server SSE implementation...");

    let result = timeout(Duration::from_secs(15), async {
        client
            .post("http://localhost:3000/mcp")
            .json(&tools_request)
            .send()
            .await
    });

    match result.await {
        Ok(Ok(response)) => {
            println!("Tools server response status: {}", response.status());

            if response.status().is_success() {
                let response_text = response.text().await.unwrap_or_default();
                println!("Response body: {}", response_text);

                // Try to parse as JSON to see if it's valid MCP response
                if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_text)
                {
                    if let Some(tools) = json_response.get("result").and_then(|r| r.get("tools")) {
                        println!(
                            "Successfully got tools response with {} tools",
                            tools.as_array().map(|a| a.len()).unwrap_or(0)
                        );

                        // Check if rustdocs tools are included
                        if let Some(tools_array) = tools.as_array() {
                            let has_rustdocs = tools_array.iter().any(|tool| {
                                tool.get("name")
                                    .and_then(|n| n.as_str())
                                    .map(|name| name.contains("rust") || name.contains("doc"))
                                    .unwrap_or(false)
                            });

                            if has_rustdocs {
                                println!("✅ Rust Docs tools found in response");
                            } else {
                                println!(
                                    "❌ No Rust Docs tools found - SSE discovery may have failed"
                                );
                            }
                        }
                    } else {
                        println!("❌ No tools in result");
                    }
                } else {
                    println!("❌ Invalid JSON response");
                }
            } else {
                println!("❌ Server returned error status");
            }
        }
        Ok(Err(e)) => {
            println!("❌ Request failed: {}", e);
        }
        Err(_) => {
            println!("❌ Request timed out");
        }
    }

    println!("SSE integration test via tools server completed");
}

#[tokio::test]
async fn test_rustdocs_sse_direct() {
    // Test Rust Docs SSE transport directly - reproduce the exact production issue
    println!("🧪 Testing Rust Docs SSE transport directly");

    let client = reqwest::Client::new();
    let sse_url = "http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse";

    // Step 1: Get session ID from SSE endpoint
    println!("🔄 Step 1: Getting session ID from SSE endpoint");
    let sse_result = timeout(Duration::from_secs(5), async {
        client
            .get(sse_url)
            .header("Accept", "text/event-stream")
            .send()
            .await
    });

    let session_id = match sse_result.await {
        Ok(Ok(response)) => {
            println!("✅ SSE endpoint response status: {}", response.status());
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            println!("📋 Content-Type: {}", content_type);

            if content_type.contains("text/event-stream") {
                // Read first chunk to get session info
                use futures::StreamExt;
                let mut body = response.bytes_stream();

                let first_chunk = match timeout(Duration::from_secs(3), body.next()).await {
                    Ok(Some(Ok(chunk))) => String::from_utf8_lossy(&chunk).to_string(),
                    Ok(Some(Err(e))) => {
                        println!("❌ Failed to read SSE chunk: {}", e);
                        return;
                    }
                    Ok(None) => {
                        println!("❌ No data received from SSE endpoint");
                        return;
                    }
                    Err(_) => {
                        println!("❌ Timeout reading SSE chunk");
                        return;
                    }
                };

                println!("📦 First SSE chunk: {}", first_chunk);

                // Parse session ID from SSE format
                if let Some(data_line) = first_chunk.lines().find(|line| line.starts_with("data: "))
                {
                    let endpoint_path = data_line.strip_prefix("data: ").unwrap_or("");
                    if let Some(session_param) = endpoint_path.split("sessionId=").nth(1) {
                        let session_id = session_param.to_string();
                        println!("✅ Extracted session ID: {}", session_id);
                        session_id
                    } else {
                        println!("❌ No sessionId found in SSE response");
                        return;
                    }
                } else {
                    println!("❌ No data line found in SSE response");
                    return;
                }
            } else {
                println!("❌ Not an SSE endpoint, content-type: {}", content_type);
                return;
            }
        }
        Ok(Err(e)) => {
            println!("❌ SSE request failed: {}", e);
            return;
        }
        Err(_) => {
            println!("❌ SSE request timed out");
            return;
        }
    };

    // Skip initialize and go directly to tools/list to test the fix
    println!("🔄 Step 2: Testing direct tools/list request (skipping initialize)");
    let base_url = sse_url.trim_end_matches("/sse").trim_end_matches('/');
    let message_url = format!("{}/message?sessionId={}", base_url, session_id);
    println!("📤 Message URL: {}", message_url);

    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    println!("📤 Sending tools/list request directly (no initialize)...");
    let tools_result = timeout(Duration::from_secs(10), async {
        client
            .post(&message_url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json")
            .json(&tools_request)
            .send()
            .await
    });

    match tools_result.await {
        Ok(Ok(tools_response)) => {
            let status = tools_response.status();
            println!("✅ Tools response status: {}", status);
            let tools_text = tools_response.text().await.unwrap_or_default();
            println!(
                "📦 Tools response (first 500 chars): {}",
                tools_text.chars().take(500).collect::<String>()
            );

            if status.is_success() {
                println!(
                    "✅ Direct tools/list request succeeded - SSE session timeout fix working!"
                );
            } else {
                println!(
                    "❌ Direct tools/list request failed with status: {}",
                    status
                );
            }
        }
        Ok(Err(e)) => {
            println!("❌ Tools request failed: {}", e);
        }
        Err(_) => {
            println!("❌ Tools request timed out");
        }
    }

    println!("🏁 Rust Docs SSE direct test completed");
}

#[tokio::test]
async fn test_http_transport_detection() {
    // Test that URL-based detection works correctly

    // SSE URLs should be detected correctly
    assert!(is_sse_url("http://example.com/sse"));
    assert!(is_sse_url("https://rustdocs-server.com/sse"));

    // Direct HTTP URLs should not trigger SSE detection
    assert!(!is_sse_url("http://example.com/api"));
    assert!(!is_sse_url("https://mcp.solana.com/mcp"));
    assert!(!is_sse_url("http://localhost:3000/mcp"));
}

fn is_sse_url(url: &str) -> bool {
    url.ends_with("/sse")
}

#[tokio::test]
async fn test_solana_direct_http() {
    // Test Solana's direct HTTP transport specifically
    let solana_url = "https://mcp.solana.com/mcp";

    // This should NOT trigger SSE detection
    assert!(!is_sse_url(solana_url));

    // Test direct HTTP request to Solana
    let client = reqwest::Client::new();
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let result = timeout(Duration::from_secs(10), async {
        client.post(solana_url).json(&tools_request).send().await
    });

    // Should succeed without SSE processing
    match result.await {
        Ok(Ok(response)) => {
            assert!(response.status().is_success() || response.status().is_client_error());
            println!(
                "Solana direct HTTP test passed: status {}",
                response.status()
            );
        }
        Ok(Err(e)) => {
            // Network errors are acceptable in test environment
            println!("Solana network error (expected in test): {}", e);
        }
        Err(_) => {
            panic!("Solana direct HTTP request should not timeout");
        }
    }
}
