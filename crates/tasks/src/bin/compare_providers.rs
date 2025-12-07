//! Provider comparison test binary.
//!
//! Compares the old API-based provider with the new CLI-based provider.

use tasks::ai::{compare_providers, print_comparison};
use tracing_subscriber::EnvFilter;

const TEST_PRD: &str = r#"
# Simple Calculator App

## Overview
Build a simple calculator web application that can perform basic arithmetic operations.

## Features
1. Addition of two numbers
2. Subtraction of two numbers  
3. Multiplication of two numbers
4. Division of two numbers (with error handling for division by zero)

## Technical Requirements
- Use HTML, CSS, and JavaScript
- Responsive design for mobile and desktop
- Clear button to reset the calculator
- Display area for showing input and results

## Non-functional Requirements
- Load time under 2 seconds
- Work on modern browsers (Chrome, Firefox, Safari, Edge)
"#;

const TEST_PROMPT: &str = r#"
Parse the following PRD and generate a list of development tasks. Return the result as JSON with this structure:
{
  "tasks": [
    {
      "id": "1",
      "title": "Task title",
      "description": "Task description",
      "priority": "high|medium|low",
      "dependencies": []
    }
  ]
}

PRD:
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("tasks=info".parse()?))
        .init();

    println!("=== Provider Comparison Test ===\n");
    println!("This test compares the old API-based provider with the new CLI-based provider.\n");

    let full_prompt = format!("{}\n{}", TEST_PROMPT, TEST_PRD);
    let model = "claude-sonnet-4-20250514";

    println!("Model: {model}");
    println!("Prompt length: {} chars\n", full_prompt.len());

    match compare_providers(&full_prompt, model).await {
        Ok((api_result, cli_result)) => {
            print_comparison(&api_result, &cli_result);

            // Summary
            println!("\n=== Summary ===");
            if api_result.success && cli_result.success {
                println!("✓ Both providers succeeded!");
                let api_len = api_result.response_text.len();
                let cli_len = cli_result.response_text.len();
                let diff_pct = ((api_len as f64 - cli_len as f64).abs() / api_len as f64) * 100.0;
                println!(
                    "  Response length difference: {:.1}% ({} vs {} chars)",
                    diff_pct, api_len, cli_len
                );
            } else if api_result.success {
                println!("⚠ Only API provider succeeded");
                println!("  CLI error: {:?}", cli_result.error);
            } else if cli_result.success {
                println!("⚠ Only CLI provider succeeded");
                println!("  API error: {:?}", api_result.error);
            } else {
                println!("✗ Both providers failed!");
                println!("  API error: {:?}", api_result.error);
                println!("  CLI error: {:?}", cli_result.error);
            }
        }
        Err(e) => {
            eprintln!("Comparison failed: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
