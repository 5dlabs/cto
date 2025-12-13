//! UI helpers for the installer CLI.
//!
//! Provides consistent formatting for console output during installation.

use colored::Colorize;

/// Print the CTO Platform banner.
pub fn print_banner() {
    println!();
    println!(
        "{}",
        r"
   _____ _____ ___    ____  _       _    __                     
  / ____|_   _/ _ \  |  _ \| |     | |  / _|                    
 | |      | || | | | | |_) | | __ _| |_| |_ ___  _ __ _ __ ___  
 | |      | || | | | |  __/| |/ _` | __|  _/ _ \| '__| '_ ` _ \ 
 | |____ _| || |_| | | |   | | (_| | |_| || (_) | |  | | | | | |
  \_____|_____\___/  |_|   |_|\__,_|\__|_| \___/|_|  |_| |_| |_|
"
        .cyan()
    );
    println!("  {}", "Bare Metal Kubernetes Platform".bright_black());
    println!();
}

/// Print a section header.
pub fn print_section(title: &str) {
    println!();
    println!("{}", "═".repeat(70).bright_black());
    println!("{}", title.cyan().bold());
    println!("{}", "═".repeat(70).bright_black());
    println!();
}

/// Print a step indicator with message.
pub fn print_step(message: &str) {
    println!("{} {}", "▶".cyan(), message.bold());
}

/// Print a progress step with step number.
pub fn print_progress_step(current: u8, total: u8, message: &str) {
    println!(
        "{} {} {}",
        format!("[{current}/{total}]").bright_black(),
        "▶".cyan(),
        message.bold()
    );
}

/// Print a success message.
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

/// Print a warning message.
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message.yellow());
}

/// Print an error message.
pub fn print_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message.red());
}

/// Print an info message.
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print installation progress.
#[allow(dead_code)]
pub fn print_progress(message: &str) {
    println!("  {} {}", "→".cyan(), message);
}

/// Print a component installation header.
#[allow(dead_code)]
pub fn print_component(name: &str) {
    println!();
    println!("{} {}", "📦".bold(), name.cyan().bold());
}

/// Print prerequisite check result.
pub fn print_check_result(name: &str, passed: bool, message: Option<&str>) {
    let status = if passed { "✓".green() } else { "✗".red() };

    let text = if let Some(msg) = message {
        format!("{name} - {msg}")
    } else {
        name.to_string()
    };

    println!("  {status} {text}");
}

/// Print GitOps sync progress.
pub fn print_gitops_progress(synced: usize, healthy: usize, total: usize) {
    let health_pct = if total > 0 {
        (healthy * 100) / total
    } else {
        0
    };

    // Build progress bar
    let bar_width = 30;
    let filled = (health_pct * bar_width) / 100;
    let empty = bar_width - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled).green(),
        "░".repeat(empty).bright_black()
    );

    // Clear line and print progress
    print!(
        "\r  {} GitOps: [{}] {}/{} synced, {}/{} healthy",
        "⟳".cyan(),
        bar,
        synced,
        total,
        healthy,
        total
    );

    // Flush to ensure immediate display
    use std::io::Write;
    let _ = std::io::stdout().flush();

    // If complete, print newline
    if synced == total && healthy == total && total > 0 {
        println!();
    }
}

/// Print a key-value pair.
#[allow(dead_code)]
pub fn print_kv(key: &str, value: &str) {
    println!("  {} {}", format!("{key}:").bright_black(), value.green());
}

/// Print a list item.
#[allow(dead_code)]
pub fn print_list_item(item: &str) {
    println!("  {} {item}", "•".bright_black());
}

/// Print a numbered step.
#[allow(dead_code)]
pub fn print_numbered_step(num: usize, message: &str) {
    println!("  {}. {}", num.to_string().cyan(), message);
}
