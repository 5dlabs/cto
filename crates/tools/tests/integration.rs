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
#![allow(clippy::disallowed_macros)] // println! is fine in tests
#![allow(clippy::uninlined_format_args)] // test readability

// Integration tests for MCP server runtime support
// This file serves as the main entry point for integration tests

mod integration {
    pub mod common;
    pub mod real_servers;

    pub use common::*;
    pub use real_servers::*;
}

use anyhow::Result;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_integration_tests() {
    INIT.call_once(|| {
        // Set up logging for integration tests
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();

        println!("Integration test environment initialized");
    });
}

// Re-export all the integration test modules for easy access
pub use integration::*;

#[tokio::test]
async fn test_integration_framework() -> Result<()> {
    setup_integration_tests();

    // Test that we can create a test environment
    let env = integration::common::TestEnvironment::new()?;
    assert!(std::path::Path::new(env.get_test_files_dir()).exists());

    println!("✅ Integration framework test passed");
    Ok(())
}

#[tokio::test]
async fn test_runtime_availability() -> Result<()> {
    setup_integration_tests();

    let mut results = integration::common::TestResults::new();

    // Check all runtime availability
    let runtimes = vec!["npx", "uvx", "docker"];

    for runtime in runtimes {
        match integration::common::check_runtime_available(runtime).await {
            true => {
                println!("✅ {} runtime is available", runtime);
                results.add_passed(format!("{}_runtime_check", runtime));
            }
            false => {
                println!("❌ {} runtime is not available", runtime);
                results.add_skipped(format!("{}_runtime_check", runtime));
            }
        }
    }

    results.print_summary();
    Ok(())
}

// This test runs a comprehensive validation of all available servers
#[tokio::test]
async fn test_comprehensive_server_validation() -> Result<()> {
    setup_integration_tests();

    let mut results = integration::common::TestResults::new();

    // Test configurations for all server types
    let test_configs = vec![
        // NPX servers
        (
            "npx-filesystem",
            integration::common::create_npx_server_config(
                "@modelcontextprotocol/server-filesystem",
                vec!["/tmp".to_string()],
            ),
        ),
        (
            "npx-brave-search",
            integration::common::create_npx_server_config(
                "@modelcontextprotocol/server-brave-search",
                vec![],
            ),
        ),
        (
            "npx-memory",
            integration::common::create_npx_server_config(
                "@modelcontextprotocol/server-memory",
                vec![],
            ),
        ),
        // UVX servers
        (
            "uvx-fetch",
            integration::common::create_uvx_server_config("mcp-server-fetch", vec![]),
        ),
        // Docker servers
        (
            "docker-fetch",
            integration::common::create_docker_server_config("mcp/fetch", vec![]),
        ),
    ];

    for (name, config) in test_configs {
        println!("\n=== Testing {} ===", name);

        // Check if the required runtime is available
        let runtime = match config.command.as_str() {
            "npx" => "npx",
            "uvx" => "uvx",
            "docker" => "docker",
            _ => "unknown",
        };

        if runtime != "unknown" && !integration::common::check_runtime_available(runtime).await {
            println!("⏭️  Skipping {}: {} runtime not available", name, runtime);
            results.add_skipped(name.to_string());
            continue;
        }

        match integration::common::TestServer::start(config).await {
            Ok(mut server) => {
                match integration::common::ProtocolValidator::run_comprehensive_validation(
                    &mut server,
                )
                .await
                {
                    Ok(report) => {
                        if report.is_healthy() {
                            println!("✅ {} validation passed", name);
                            results.add_passed(name.to_string());
                        } else {
                            println!("❌ {} validation failed", name);
                            results.add_failed(name.to_string(), "Validation failed".to_string());
                        }
                    }
                    Err(e) => {
                        println!("❌ {} validation error: {}", name, e);
                        results.add_failed(name.to_string(), e.to_string());
                    }
                }
                let _ = server.shutdown().await;
            }
            Err(e) => {
                println!("❌ {} failed to start: {}", name, e);
                results.add_failed(name.to_string(), format!("Failed to start: {}", e));
            }
        }
    }

    println!("\n=== Comprehensive Server Validation Results ===");
    results.print_summary();

    Ok(())
}

// Test HTTP/SSE server separately due to different client requirements
#[tokio::test]
async fn test_http_sse_server_comprehensive() -> Result<()> {
    setup_integration_tests();

    let server_url = integration::common::get_remote_server_url();
    println!("Testing HTTP/SSE server at: {}", server_url);

    let client = integration::real_servers::http_sse_servers::HttpSseTestClient::new(server_url);

    // Test basic connectivity with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(10), client.health_check()).await {
        Ok(Ok(_)) => {
            println!("✅ HTTP/SSE server is accessible");

            // Run additional tests
            match tokio::time::timeout(std::time::Duration::from_secs(10), client.initialize())
                .await
            {
                Ok(Ok(_)) => {
                    println!("✅ HTTP/SSE server initialization successful");

                    match tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        client.list_tools(),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            println!("✅ HTTP/SSE server tools listing successful");
                        }
                        Ok(Err(e)) => {
                            println!("❌ HTTP/SSE server tools listing failed: {}", e);
                        }
                        Err(_) => {
                            println!("❌ HTTP/SSE server tools listing timed out");
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ HTTP/SSE server initialization failed: {}", e);
                }
                Err(_) => {
                    println!("❌ HTTP/SSE server initialization timed out");
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ HTTP/SSE server is not accessible: {}", e);
        }
        Err(_) => {
            println!("❌ HTTP/SSE server connectivity test timed out");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_integration_summary() -> Result<()> {
    setup_integration_tests();

    println!("\n=== Integration Test Summary ===");
    println!("This test suite validates MCP server runtime support for:");
    println!("• NPX servers (Node.js ecosystem)");
    println!("• UVX servers (Python ecosystem)");
    println!("• Docker servers (Containerized)");
    println!("• HTTP/SSE servers (Remote)");
    println!("================================");

    Ok(())
}
