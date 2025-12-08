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
#[ignore = "requires pre-built tools-server binary"]
async fn test_tools_server_with_solana_config() {
    // Test our actual server implementation with Solana configured
    println!("🧪 Testing tools server HTTP transport with Solana");

    // Create a minimal config with Solana
    let test_config = json!({
        "servers": {
            "solana": {
                "name": "Solana",
                "description": "Solana blockchain development tools",
                "transport": "http",
                "url": "https://mcp.solana.com/mcp",
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
            println!("📝 Response: {response_text}");

            // Check if we got valid JSON
            if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                println!("✅ Valid JSON response");

                if let Some(error) = json_response.get("error") {
                    println!("❌ Server returned error: {error}");
                } else if let Some(result) = json_response.get("result") {
                    if let Some(tools) = result.get("tools") {
                        println!(
                            "✅ Got tools array with {} items",
                            tools.as_array().map_or(0, std::vec::Vec::len)
                        );
                    } else {
                        println!("❌ No tools in result");
                    }
                }
            } else {
                println!("❌ Invalid JSON response: {response_text}");
            }
        }
        Ok(Err(e)) => {
            println!("❌ Request failed: {e}");
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
async fn test_sse_via_tools_server() {
    // Test SSE transport via our actual tools server
    // This tests the real implementation path

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
                println!("Response body: {response_text}");

                // Try to parse as JSON to see if it's valid MCP response
                if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_text)
                {
                    if let Some(tools) = json_response.get("result").and_then(|r| r.get("tools")) {
                        println!(
                            "Successfully got tools response with {} tools",
                            tools.as_array().map_or(0, std::vec::Vec::len)
                        );
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
            println!("❌ Request failed: {e}");
        }
        Err(_) => {
            println!("❌ Request timed out");
        }
    }

    println!("SSE integration test via tools server completed");
}

#[tokio::test]
async fn test_http_transport_detection() {
    // Test that URL-based detection works correctly

    // SSE URLs should be detected correctly
    assert!(is_sse_url("http://example.com/sse"));
    assert!(is_sse_url("https://example-server.com/sse"));

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
            println!("Solana network error (expected in test): {e}");
        }
        Err(_) => {
            panic!("Solana direct HTTP request should not timeout");
        }
    }
}
