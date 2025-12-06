//! Documentation generation for tasks.
//!
//! This module generates per-task documentation files:
//! - prompt.xml - Structured XML prompt for agents
//! - prompt.md - Markdown prompt for CLIs
//! - acceptance.md - Checklist for task completion

use std::fmt::Write as _;
use std::path::Path;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};

use super::routing::get_role_for_hint;

/// XML escape special characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate task.xml content for a task.
#[must_use]
pub fn generate_task_xml(task: &Task) -> String {
    let agent_hint = task.agent_hint.as_deref().unwrap_or("rex");
    let role = get_role_for_hint(agent_hint);
    let priority = task.priority.to_string();
    let dependencies = task.dependencies.join(", ");

    let title_esc = xml_escape(&task.title);
    let desc_esc = xml_escape(&task.description);
    let details_esc = xml_escape(&task.details);
    let test_esc = xml_escape(&task.test_strategy);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<task id="{id}" priority="{priority}" agent="{agent}">
    <meta>
        <title>{title}</title>
        <priority>{priority}</priority>
        <dependencies>{deps}</dependencies>
        <agent_hint>{agent}</agent_hint>
    </meta>

    <role>You are a {role}. Implement Task {id} with production-quality code.</role>

    <context>
        <overview>{desc}</overview>
        <background>Review PRD and architecture in .tasks/docs/ for full context.</background>
    </context>

    <requirements>
        <description>{details}</description>
        <constraints>
            <constraint>Match existing codebase patterns</constraint>
            <constraint>Create PR with atomic commits</constraint>
            <constraint>Include unit tests</constraint>
            <constraint>PR title: feat(task-{id}): {title}</constraint>
        </constraints>
    </requirements>

    <acceptance_criteria>
        <criterion>All requirements implemented</criterion>
        <criterion>Tests passing with adequate coverage</criterion>
        <criterion>Code follows project conventions</criterion>
        <criterion>{test_strategy}</criterion>
    </acceptance_criteria>

    <validation>
        <command>cargo test (Rust)</command>
        <command>cargo clippy -- -D warnings (Rust)</command>
        <command>go test ./... (Go)</command>
        <command>npm test (Node.js)</command>
    </validation>

    <deliverables>
        <deliverable>Working implementation</deliverable>
        <deliverable>Unit tests</deliverable>
        <deliverable>Pull request</deliverable>
    </deliverables>
</task>
"#,
        id = task.id,
        priority = priority,
        agent = agent_hint,
        title = title_esc,
        deps = dependencies,
        role = role,
        desc = desc_esc,
        details = details_esc,
        test_strategy = test_esc,
    )
}

/// Generate prompt.md content for a task.
#[must_use]
pub fn generate_task_prompt(task: &Task) -> String {
    let agent_hint = task.agent_hint.as_deref().unwrap_or("rex");
    let role = get_role_for_hint(agent_hint);

    let mut content = String::new();

    writeln!(content, "# Task {}: {}\n", task.id, task.title).ok();
    writeln!(content, "## Role\n").ok();
    writeln!(
        content,
        "You are a {} implementing Task {}.\n",
        role, task.id
    )
    .ok();
    writeln!(content, "## Goal\n").ok();
    writeln!(content, "{}\n", task.description).ok();
    writeln!(content, "## Requirements\n").ok();
    writeln!(
        content,
        "{}\n",
        if task.details.is_empty() {
            "(See task description above)"
        } else {
            &task.details
        }
    )
    .ok();
    writeln!(content, "## Acceptance Criteria\n").ok();
    writeln!(
        content,
        "{}\n",
        if task.test_strategy.is_empty() {
            "- All requirements implemented\n- Tests passing\n- Code follows conventions"
        } else {
            &task.test_strategy
        }
    )
    .ok();
    writeln!(content, "## Constraints\n").ok();
    writeln!(content, "- Match codebase patterns").ok();
    writeln!(content, "- Create PR with atomic commits").ok();
    writeln!(content, "- Include unit tests").ok();
    writeln!(
        content,
        "- PR title: `feat(task-{}): {}`",
        task.id, task.title
    )
    .ok();

    content
}

/// Generate acceptance-criteria.md content for a task.
#[must_use]
pub fn generate_acceptance_criteria(task: &Task) -> String {
    let mut content = String::new();

    writeln!(content, "# Acceptance Criteria: Task {}\n", task.id).ok();
    writeln!(content, "- [ ] {}", task.description).ok();

    if !task.test_strategy.is_empty() {
        writeln!(content, "- [ ] {}", task.test_strategy).ok();
    }

    writeln!(content, "- [ ] All requirements implemented").ok();
    writeln!(content, "- [ ] Tests passing").ok();
    writeln!(content, "- [ ] Code follows conventions").ok();
    writeln!(content, "- [ ] PR created and ready for review").ok();

    // Add subtask acceptance criteria if any
    if !task.subtasks.is_empty() {
        writeln!(content, "\n## Subtasks\n").ok();
        for subtask in &task.subtasks {
            writeln!(
                content,
                "- [ ] {}.{}: {}",
                task.id, subtask.id, subtask.title
            )
            .ok();
        }
    }

    content
}

/// Result of generating documentation for all tasks.
#[derive(Debug, Clone, Default)]
pub struct DocsGenerationResult {
    /// Number of task directories created.
    pub task_dirs_created: usize,
    /// Number of XML files generated.
    pub xml_files: usize,
    /// Number of prompt.md files generated.
    pub prompt_files: usize,
    /// Number of acceptance-criteria.md files generated.
    pub acceptance_files: usize,
}

/// Generate all documentation files for a list of tasks.
///
/// Creates the following structure:
/// ```text
/// output_dir/
/// ├── task-1/
/// │   ├── prompt.xml
/// │   ├── prompt.md
/// │   └── acceptance.md
/// ├── task-2/
/// │   └── ...
/// ```
pub async fn generate_all_docs(
    tasks: &[Task],
    output_dir: &Path,
) -> TasksResult<DocsGenerationResult> {
    let mut result = DocsGenerationResult::default();

    for task in tasks {
        let task_dir = output_dir.join(format!("task-{}", task.id));

        // Create task directory
        tokio::fs::create_dir_all(&task_dir)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: task_dir.display().to_string(),
                reason: e.to_string(),
            })?;
        result.task_dirs_created += 1;

        // Generate prompt.xml
        let xml_path = task_dir.join("prompt.xml");
        let xml_content = generate_task_xml(task);
        tokio::fs::write(&xml_path, &xml_content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: xml_path.display().to_string(),
                reason: e.to_string(),
            })?;
        result.xml_files += 1;

        // Generate prompt.md
        let prompt_path = task_dir.join("prompt.md");
        let prompt_content = generate_task_prompt(task);
        tokio::fs::write(&prompt_path, &prompt_content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: prompt_path.display().to_string(),
                reason: e.to_string(),
            })?;
        result.prompt_files += 1;

        // Generate acceptance.md
        let acceptance_path = task_dir.join("acceptance.md");
        let acceptance_content = generate_acceptance_criteria(task);
        tokio::fs::write(&acceptance_path, &acceptance_content)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: acceptance_path.display().to_string(),
                reason: e.to_string(),
            })?;
        result.acceptance_files += 1;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::TaskPriority;

    fn sample_task() -> Task {
        let mut task = Task::new("1", "Implement User API", "Create CRUD endpoints for users");
        task.details = "Use Axum framework with PostgreSQL".to_string();
        task.test_strategy = "Unit tests for handlers, integration tests for DB".to_string();
        task.priority = TaskPriority::High;
        task.agent_hint = Some("rex".to_string());
        task
    }

    #[test]
    fn test_generate_task_xml() {
        let task = sample_task();
        let xml = generate_task_xml(&task);

        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"<task id="1""#));
        assert!(xml.contains("<title>Implement User API</title>"));
        assert!(xml.contains("<agent_hint>rex</agent_hint>"));
        assert!(xml.contains("Rust Engineer"));
    }

    #[test]
    fn test_generate_task_xml_escaping() {
        let mut task = Task::new(
            "1",
            "Test <special> & \"chars\"",
            "Description with 'quotes'",
        );
        task.agent_hint = Some("rex".to_string());
        let xml = generate_task_xml(&task);

        assert!(xml.contains("&lt;special&gt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;chars&quot;"));
    }

    #[test]
    fn test_generate_task_prompt() {
        let task = sample_task();
        let md = generate_task_prompt(&task);

        assert!(md.contains("# Task 1: Implement User API"));
        assert!(md.contains("## Role"));
        assert!(md.contains("Rust Engineer"));
        assert!(md.contains("## Goal"));
        assert!(md.contains("CRUD endpoints"));
        assert!(md.contains("## Requirements"));
        assert!(md.contains("Axum framework"));
    }

    #[test]
    fn test_generate_acceptance_criteria() {
        let task = sample_task();
        let md = generate_acceptance_criteria(&task);

        assert!(md.contains("# Acceptance Criteria: Task 1"));
        assert!(md.contains("- [ ] Create CRUD endpoints"));
        assert!(md.contains("- [ ] All requirements implemented"));
        assert!(md.contains("- [ ] Tests passing"));
    }

    #[test]
    fn test_acceptance_criteria_with_subtasks() {
        let mut task = sample_task();
        task.subtasks.push(crate::entities::Subtask::new(
            1,
            "1",
            "Create endpoint",
            "POST /users",
        ));
        task.subtasks.push(crate::entities::Subtask::new(
            2,
            "1",
            "Add validation",
            "Input validation",
        ));

        let md = generate_acceptance_criteria(&task);

        assert!(md.contains("## Subtasks"));
        assert!(md.contains("- [ ] 1.1: Create endpoint"));
        assert!(md.contains("- [ ] 1.2: Add validation"));
    }

    #[test]
    fn test_default_agent_hint() {
        let task = Task::new("1", "Generic task", "No agent hint set");
        let xml = generate_task_xml(&task);

        // Should default to rex
        assert!(xml.contains("<agent_hint>rex</agent_hint>"));
    }
}
