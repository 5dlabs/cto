use colored::Colorize;

use crate::config::{ClusterType, InstallConfig};

/// Print a section header
pub fn print_section(title: &str) {
    println!();
    println!("{}", "═".repeat(70).bright_black());
    println!("{}", title.cyan().bold());
    println!("{}", "═".repeat(70).bright_black());
    println!();
}

/// Print a step indicator
pub fn print_step(current: usize, total: usize, message: &str) {
    println!(
        "{} {} {}",
        format!("[{current}/{total}]").bright_black(),
        "▶".cyan(),
        message.bold()
    );
}

/// Print a success message
pub fn print_success(message: &str) {
    println!();
    println!("{} {}", "✓".green().bold(), message.green());
    println!();
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message.yellow());
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print a configuration summary
pub fn print_config_summary(config: &InstallConfig) {
    println!("{}", "Configuration Summary".cyan().bold());
    println!("{}", "─".repeat(70).bright_black());
    println!();

    println!(
        "  {} {}",
        "Profile:".bright_black(),
        config.profile.name().green()
    );
    println!(
        "  {} {}",
        "Cluster:".bright_black(),
        match config.cluster_type {
            ClusterType::Kind => "Local kind cluster".to_string(),
            ClusterType::Remote => "Remote Kubernetes cluster".to_string(),
        }
        .green()
    );
    println!(
        "  {} {}",
        "Namespace:".bright_black(),
        config.namespace.green()
    );

    if let Some(org) = &config.github_org {
        println!("  {} {}", "GitHub Org:".bright_black(), org.green());
    }

    if let Some(repo) = &config.github_repo {
        println!("  {} {}", "GitHub Repo:".bright_black(), repo.green());
    }

    println!(
        "  {} {}",
        "Registry:".bright_black(),
        config.get_registry_prefix().green()
    );

    if let Some(domain) = &config.domain {
        println!("  {} {}", "Domain:".bright_black(), domain.green());
    }

    println!();
    println!("{}", "Components".cyan().bold());
    println!("{}", "─".repeat(70).bright_black());
    println!();

    let components = vec![
        ("ArgoCD", true),
        ("Argo Workflows", true),
        ("Argo Events", true),
        ("CTO Controller", true),
        ("Monitoring Stack", config.install_monitoring),
        ("Database Operators", config.install_databases),
    ];

    for (component, enabled) in components {
        let status = if enabled {
            "✓".green()
        } else {
            "○".bright_black()
        };
        println!("  {status} {component}");
    }

    println!();
    println!("{}", "Resource Requirements".cyan().bold());
    println!("{}", "─".repeat(70).bright_black());
    println!();
    println!(
        "  {} {} cores",
        "CPU:".bright_black(),
        config.profile.cpu_requirement().to_string().yellow()
    );
    println!(
        "  {} {} GB",
        "Memory:".bright_black(),
        config.profile.memory_requirement_gb().to_string().yellow()
    );
    println!();
}

/// Print installation progress
pub fn print_progress(message: &str) {
    println!("  {} {}", "→".cyan(), message);
}

/// Print a component installation header
pub fn print_component(name: &str) {
    println!();
    println!("{} {}", "📦".bold(), name.cyan().bold());
}

/// Print prerequisite check result
pub fn print_check_result(name: &str, passed: bool, message: Option<&str>) {
    let status = if passed { "✓".green() } else { "✗".red() };

    let text = if let Some(msg) = message {
        format!("{name} - {msg}")
    } else {
        name.to_string()
    };

    println!("  {status} {text}");
}
