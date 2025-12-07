//! Detailed content comparison between providers.

// Allow pedantic lints in test binary for cleaner test code
#![allow(
    clippy::needless_raw_string_hashes,
    clippy::uninlined_format_args,
    clippy::too_many_lines
)]

use cli::CLIType;
use serde::Deserialize;
use tasks::ai::anthropic::AnthropicProvider;
use tasks::ai::cli_provider::CLITextGenerator;
use tasks::ai::{AIMessage, AIProvider, GenerateOptions};
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

#[derive(Debug, Deserialize)]
struct TasksResponse {
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
struct Task {
    id: String,
    title: String,
    description: String,
    priority: String,
    #[serde(default)]
    dependencies: Vec<String>,
}

fn extract_json(text: &str) -> Option<&str> {
    // Try to find JSON in markdown code blocks
    if let Some(start) = text.find("```json") {
        let json_start = start + 7;
        if let Some(end) = text[json_start..].find("```") {
            return Some(text[json_start..json_start + end].trim());
        }
    }
    if let Some(start) = text.find("```") {
        let json_start = start + 3;
        if let Some(end) = text[json_start..].find("```") {
            return Some(text[json_start..json_start + end].trim());
        }
    }
    // Try to parse the whole text as JSON
    if text.trim().starts_with('{') || text.trim().starts_with('[') {
        return Some(text.trim());
    }
    None
}

fn parse_tasks(response: &str) -> Option<TasksResponse> {
    let json_str = extract_json(response)?;
    serde_json::from_str(json_str).ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("tasks=info".parse()?))
        .init();

    println!("=== Detailed Task Content Comparison ===\n");

    let full_prompt = format!("{}\n{}", TEST_PROMPT, TEST_PRD);
    let model = "claude-sonnet-4-20250514";

    let messages = vec![
        AIMessage::system("You are a helpful assistant that generates structured JSON output."),
        AIMessage::user(&full_prompt),
    ];

    let options = GenerateOptions {
        temperature: Some(0.7),
        max_tokens: Some(2048),
        json_mode: true,
        ..Default::default()
    };

    // Get API response
    println!("Calling API provider...");
    let api_provider = AnthropicProvider::from_env()?;
    let api_response = api_provider
        .generate_text(model, &messages, &options)
        .await?;

    // Get CLI response
    println!("Calling CLI provider...");
    let cli_provider = CLITextGenerator::new(CLIType::Claude)?;
    let cli_response = cli_provider
        .generate_text(model, &messages, &options)
        .await?;

    println!("\n{}", "=".repeat(70));
    println!("RAW RESPONSES");
    println!("{}\n", "=".repeat(70));

    println!("--- API Response (first 500 chars) ---");
    println!(
        "{}\n",
        &api_response.text[..500.min(api_response.text.len())]
    );

    println!("--- CLI Response (first 500 chars) ---");
    println!(
        "{}\n",
        &cli_response.text[..500.min(cli_response.text.len())]
    );

    // Parse tasks
    let api_tasks = parse_tasks(&api_response.text);
    let cli_tasks = parse_tasks(&cli_response.text);

    println!("\n{}", "=".repeat(70));
    println!("PARSED TASKS COMPARISON");
    println!("{}\n", "=".repeat(70));

    match (&api_tasks, &cli_tasks) {
        (Some(api), Some(cli)) => {
            println!("API generated {} tasks", api.tasks.len());
            println!("CLI generated {} tasks\n", cli.tasks.len());

            println!("--- API Tasks ---");
            for task in &api.tasks {
                println!("  [{}] {} ({})", task.id, task.title, task.priority);
                println!("      {}", truncate(&task.description, 80));
                if !task.dependencies.is_empty() {
                    println!("      deps: {:?}", task.dependencies);
                }
            }

            println!("\n--- CLI Tasks ---");
            for task in &cli.tasks {
                println!("  [{}] {} ({})", task.id, task.title, task.priority);
                println!("      {}", truncate(&task.description, 80));
                if !task.dependencies.is_empty() {
                    println!("      deps: {:?}", task.dependencies);
                }
            }

            // Compare task coverage
            println!("\n{}", "=".repeat(70));
            println!("ANALYSIS");
            println!("{}\n", "=".repeat(70));

            let api_titles: Vec<&str> = api.tasks.iter().map(|t| t.title.as_str()).collect();
            let cli_titles: Vec<&str> = cli.tasks.iter().map(|t| t.title.as_str()).collect();

            println!(
                "Task count: API={}, CLI={}",
                api.tasks.len(),
                cli.tasks.len()
            );

            // Check for similar themes
            let themes = [
                "HTML",
                "CSS",
                "JavaScript",
                "button",
                "display",
                "responsive",
                "error",
                "clear",
            ];
            println!("\nTheme coverage:");
            for theme in themes {
                let api_has = api_titles
                    .iter()
                    .any(|t| t.to_lowercase().contains(&theme.to_lowercase()))
                    || api
                        .tasks
                        .iter()
                        .any(|t| t.description.to_lowercase().contains(&theme.to_lowercase()));
                let cli_has = cli_titles
                    .iter()
                    .any(|t| t.to_lowercase().contains(&theme.to_lowercase()))
                    || cli
                        .tasks
                        .iter()
                        .any(|t| t.description.to_lowercase().contains(&theme.to_lowercase()));
                let status = match (api_has, cli_has) {
                    (true, true) => "✓ Both",
                    (true, false) => "⚠ API only",
                    (false, true) => "⚠ CLI only",
                    (false, false) => "✗ Neither",
                };
                println!("  {}: {}", theme, status);
            }

            // Priority distribution
            println!("\nPriority distribution:");
            for priority in ["high", "medium", "low"] {
                let api_count = api
                    .tasks
                    .iter()
                    .filter(|t| t.priority.to_lowercase() == priority)
                    .count();
                let cli_count = cli
                    .tasks
                    .iter()
                    .filter(|t| t.priority.to_lowercase() == priority)
                    .count();
                println!("  {}: API={}, CLI={}", priority, api_count, cli_count);
            }
        }
        (None, Some(_)) => println!("Failed to parse API response as tasks"),
        (Some(_), None) => println!("Failed to parse CLI response as tasks"),
        (None, None) => println!("Failed to parse both responses as tasks"),
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
