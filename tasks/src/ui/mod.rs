//! Terminal UI helpers for task display.
//!
//! This module uses println! for CLI output, which is appropriate
//! for terminal user interfaces.

#![allow(clippy::disallowed_macros)]

use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table};

use crate::entities::{TagStats, Task, TaskPriority, TaskStatus};

/// Get colored status string
pub fn status_colored(status: TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "pending".yellow().to_string(),
        TaskStatus::InProgress => "in-progress".cyan().to_string(),
        TaskStatus::Done => "done".green().to_string(),
        TaskStatus::Deferred => "deferred".blue().to_string(),
        TaskStatus::Cancelled => "cancelled".red().to_string(),
        TaskStatus::Blocked => "blocked".red().bold().to_string(),
        TaskStatus::Review => "review".magenta().to_string(),
    }
}

/// Get colored priority string
pub fn priority_colored(priority: TaskPriority) -> String {
    match priority {
        TaskPriority::Low => "low".dimmed().to_string(),
        TaskPriority::Medium => "medium".normal().to_string(),
        TaskPriority::High => "high".yellow().to_string(),
        TaskPriority::Critical => "critical".red().bold().to_string(),
    }
}

/// Create a table for displaying tasks
pub fn task_table(tasks: &[Task], show_subtasks: bool) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header
    table.set_header(vec![
        Cell::new("ID").fg(Color::Cyan),
        Cell::new("Title").fg(Color::Cyan),
        Cell::new("Status").fg(Color::Cyan),
        Cell::new("Priority").fg(Color::Cyan),
        Cell::new("Deps").fg(Color::Cyan),
    ]);

    for task in tasks {
        let status_color = match task.status {
            TaskStatus::Pending => Color::Yellow,
            TaskStatus::InProgress => Color::Cyan,
            TaskStatus::Done => Color::Green,
            TaskStatus::Cancelled | TaskStatus::Blocked => Color::Red,
            TaskStatus::Deferred => Color::Blue,
            TaskStatus::Review => Color::Magenta,
        };

        let priority_color = match task.priority {
            TaskPriority::Low => Color::DarkGrey,
            TaskPriority::Medium => Color::White,
            TaskPriority::High => Color::Yellow,
            TaskPriority::Critical => Color::Red,
        };

        let deps = if task.dependencies.is_empty() {
            "-".to_string()
        } else {
            task.dependencies.join(", ")
        };

        table.add_row(vec![
            Cell::new(&task.id),
            Cell::new(&task.title),
            Cell::new(task.status.to_string()).fg(status_color),
            Cell::new(task.priority.to_string()).fg(priority_color),
            Cell::new(deps),
        ]);

        // Add subtasks if requested
        if show_subtasks {
            for subtask in &task.subtasks {
                let sub_status_color = match subtask.status {
                    TaskStatus::Pending => Color::Yellow,
                    TaskStatus::InProgress => Color::Cyan,
                    TaskStatus::Done => Color::Green,
                    TaskStatus::Cancelled | TaskStatus::Blocked => Color::Red,
                    TaskStatus::Deferred => Color::Blue,
                    TaskStatus::Review => Color::Magenta,
                };

                table.add_row(vec![
                    Cell::new(format!("  {}.{}", task.id, subtask.id)).fg(Color::DarkGrey),
                    Cell::new(format!("  └─ {}", subtask.title)).fg(Color::DarkGrey),
                    Cell::new(subtask.status.to_string()).fg(sub_status_color),
                    Cell::new("-"),
                    Cell::new("-"),
                ]);
            }
        }
    }

    table
}

/// Create a table for displaying tags
pub fn tag_table(tags: &[TagStats]) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("Tag").fg(Color::Cyan),
        Cell::new("Tasks").fg(Color::Cyan),
        Cell::new("Done").fg(Color::Cyan),
        Cell::new("Progress").fg(Color::Cyan),
        Cell::new("Current").fg(Color::Cyan),
    ]);

    for tag in tags {
        let progress = if tag.task_count > 0 {
            format!("{:.0}%", tag.completion_percent())
        } else {
            "-".to_string()
        };

        let current_marker = if tag.is_current { "●" } else { "" };

        table.add_row(vec![
            Cell::new(&tag.name),
            Cell::new(tag.task_count),
            Cell::new(tag.completed_tasks).fg(Color::Green),
            Cell::new(progress),
            Cell::new(current_marker).fg(Color::Green),
        ]);
    }

    table
}

/// Display task details in a formatted way
pub fn display_task_details(task: &Task) {
    println!("{}", "═".repeat(60).dimmed());
    println!(
        "{} {} {}",
        "Task".cyan().bold(),
        task.id.cyan().bold(),
        format!("[{}]", task.status).yellow()
    );
    println!("{}", "═".repeat(60).dimmed());
    println!();

    println!("{}: {}", "Title".bold(), task.title);
    println!("{}: {}", "Status".bold(), status_colored(task.status));
    println!(
        "{}: {}",
        "Priority".bold(),
        priority_colored(task.priority)
    );

    if !task.dependencies.is_empty() {
        println!(
            "{}: {}",
            "Dependencies".bold(),
            task.dependencies.join(", ")
        );
    }

    println!();
    println!("{}", "Description".bold().underline());
    println!("{}", task.description);

    if !task.details.is_empty() {
        println!();
        println!("{}", "Details".bold().underline());
        println!("{}", task.details);
    }

    if !task.test_strategy.is_empty() {
        println!();
        println!("{}", "Test Strategy".bold().underline());
        println!("{}", task.test_strategy);
    }

    if !task.subtasks.is_empty() {
        println!();
        println!(
            "{} ({})",
            "Subtasks".bold().underline(),
            task.subtasks.len()
        );
        for subtask in &task.subtasks {
            println!(
                "  {} {} - {} [{}]",
                "•".dimmed(),
                subtask.id,
                subtask.title,
                status_colored(subtask.status)
            );
        }
    }

    if let Some(ref complexity) = task.complexity {
        println!();
        println!("{}", "Complexity".bold().underline());
        println!("  Score: {}/10", complexity.score);
        if let Some(ref reasoning) = complexity.reasoning {
            println!("  Reasoning: {reasoning}");
        }
    }

    println!();
}

/// Print success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print error message
pub fn print_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message);
}

/// Print info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

