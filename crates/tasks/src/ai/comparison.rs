//! Comparison module for testing old API-based vs new CLI-based providers.
//!
//! This module is temporary for verification and should be removed after testing.

use super::anthropic::AnthropicProvider;
use super::cli_provider::CLITextGenerator;
use super::provider::{AIMessage, AIProvider, GenerateOptions};
use crate::errors::TasksResult;
use cli::CLIType;
use std::time::Instant;
use tracing::info;

/// Results from a provider comparison test.
#[derive(Debug)]
pub struct ComparisonResult {
    /// Name of the provider
    pub provider_name: String,
    /// Response text
    pub response_text: String,
    /// Token usage
    pub input_tokens: u32,
    pub output_tokens: u32,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the test succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Run a comparison test between old API-based and new CLI-based providers.
pub async fn compare_providers(
    prompt: &str,
    model: &str,
) -> TasksResult<(ComparisonResult, ComparisonResult)> {
    let messages = vec![
        AIMessage::system("You are a helpful assistant that generates structured JSON output."),
        AIMessage::user(prompt),
    ];

    let options = GenerateOptions {
        temperature: Some(0.7),
        max_tokens: Some(2048),
        json_mode: true,
        ..Default::default()
    };

    // Test old API-based provider
    info!("Testing old API-based provider...");
    let api_result = test_api_provider(&messages, model, &options).await;

    // Test new CLI-based provider
    info!("Testing new CLI-based provider...");
    let cli_result = test_cli_provider(&messages, model, &options).await;

    Ok((api_result, cli_result))
}

async fn test_api_provider(
    messages: &[AIMessage],
    model: &str,
    options: &GenerateOptions,
) -> ComparisonResult {
    let start = Instant::now();

    match AnthropicProvider::from_env() {
        Ok(provider) => {
            if !provider.is_configured() {
                return ComparisonResult {
                    provider_name: "anthropic-api".to_string(),
                    response_text: String::new(),
                    input_tokens: 0,
                    output_tokens: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: false,
                    error: Some("ANTHROPIC_API_KEY not configured".to_string()),
                };
            }

            match provider.generate_text(model, messages, options).await {
                Ok(response) => ComparisonResult {
                    provider_name: "anthropic-api".to_string(),
                    response_text: response.text,
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: true,
                    error: None,
                },
                Err(e) => ComparisonResult {
                    provider_name: "anthropic-api".to_string(),
                    response_text: String::new(),
                    input_tokens: 0,
                    output_tokens: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        }
        Err(e) => ComparisonResult {
            provider_name: "anthropic-api".to_string(),
            response_text: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            execution_time_ms: start.elapsed().as_millis() as u64,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

async fn test_cli_provider(
    messages: &[AIMessage],
    model: &str,
    options: &GenerateOptions,
) -> ComparisonResult {
    let start = Instant::now();

    match CLITextGenerator::new(CLIType::Claude) {
        Ok(provider) => {
            if !provider.is_configured() {
                return ComparisonResult {
                    provider_name: "cli-claude".to_string(),
                    response_text: String::new(),
                    input_tokens: 0,
                    output_tokens: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: false,
                    error: Some("Claude CLI not available".to_string()),
                };
            }

            match provider.generate_text(model, messages, options).await {
                Ok(response) => ComparisonResult {
                    provider_name: "cli-claude".to_string(),
                    response_text: response.text,
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: true,
                    error: None,
                },
                Err(e) => ComparisonResult {
                    provider_name: "cli-claude".to_string(),
                    response_text: String::new(),
                    input_tokens: 0,
                    output_tokens: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        }
        Err(e) => ComparisonResult {
            provider_name: "cli-claude".to_string(),
            response_text: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            execution_time_ms: start.elapsed().as_millis() as u64,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

/// Print comparison results in a formatted way.
pub fn print_comparison(api_result: &ComparisonResult, cli_result: &ComparisonResult) {
    println!("\n{}", "=".repeat(60));
    println!("PROVIDER COMPARISON RESULTS");
    println!("{}\n", "=".repeat(60));

    println!("--- {} ---", api_result.provider_name);
    println!("  Success: {}", api_result.success);
    println!("  Time: {}ms", api_result.execution_time_ms);
    if api_result.success {
        println!("  Input tokens: {}", api_result.input_tokens);
        println!("  Output tokens: {}", api_result.output_tokens);
        println!(
            "  Response length: {} chars",
            api_result.response_text.len()
        );
        println!(
            "  Response preview: {}...",
            &api_result.response_text[..200.min(api_result.response_text.len())]
        );
    } else {
        println!("  Error: {:?}", api_result.error);
    }

    println!("\n--- {} ---", cli_result.provider_name);
    println!("  Success: {}", cli_result.success);
    println!("  Time: {}ms", cli_result.execution_time_ms);
    if cli_result.success {
        println!("  Input tokens: {}", cli_result.input_tokens);
        println!("  Output tokens: {}", cli_result.output_tokens);
        println!(
            "  Response length: {} chars",
            cli_result.response_text.len()
        );
        println!(
            "  Response preview: {}...",
            &cli_result.response_text[..200.min(cli_result.response_text.len())]
        );
    } else {
        println!("  Error: {:?}", cli_result.error);
    }

    println!("\n{}", "=".repeat(60));
}
