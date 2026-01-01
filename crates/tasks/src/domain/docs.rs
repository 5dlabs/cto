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

/// Extract code blocks from a string (markdown format).
///
/// Returns a tuple of (code_blocks, remaining_text).
fn extract_code_blocks(text: &str) -> (Vec<(String, String)>, String) {
    let mut code_blocks = Vec::new();
    let mut remaining = String::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut current_code = String::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_code_block {
                // End of code block
                code_blocks.push((current_lang.clone(), current_code.trim().to_string()));
                current_lang.clear();
                current_code.clear();
                in_code_block = false;
            } else {
                // Start of code block - extract language
                let lang = line.trim_start().trim_start_matches('`').trim();
                current_lang = lang.to_string();
                in_code_block = true;
            }
        } else if in_code_block {
            current_code.push_str(line);
            current_code.push('\n');
        } else {
            remaining.push_str(line);
            remaining.push('\n');
        }
    }

    (code_blocks, remaining.trim().to_string())
}

/// Get the primary language for an agent hint.
fn get_language_for_agent(agent_hint: &str) -> &'static str {
    match agent_hint {
        "rex" => "rust",
        "grizz" => "go",
        "nova" | "spark" => "typescript",
        "blaze" | "tap" => "tsx",
        "bolt" => "yaml",
        _ => "text",
    }
}

/// Get validation commands based on agent type.
fn get_validation_commands(agent_hint: &str) -> &'static str {
    match agent_hint {
        "bolt" => {
            r"<command>kubectl get pods -n {namespace} - verify all pods running</command>
        <command>kubectl get secrets - verify connection secrets created</command>
        <command>kubectl logs {pod} - check for errors</command>"
        }
        "rex" => {
            r"<command>cargo test --all</command>
        <command>cargo clippy -- -D warnings</command>
        <command>cargo fmt --check</command>"
        }
        "grizz" => {
            r"<command>go test ./...</command>
        <command>golangci-lint run</command>
        <command>go build ./...</command>"
        }
        "nova" => {
            r"<command>bun test</command>
        <command>bun run lint</command>
        <command>bun run typecheck</command>"
        }
        "blaze" => {
            r"<command>npm run build</command>
        <command>npm run lint</command>
        <command>npm run test</command>
        <command>npm run typecheck</command>"
        }
        "tap" => {
            r"<command>npx expo-doctor</command>
        <command>npm run lint</command>
        <command>npm run test</command>"
        }
        "spark" => {
            r"<command>npm run build</command>
        <command>npm run lint</command>
        <command>npm run test</command>"
        }
        "tess" => {
            r"<command>Run test suite for target codebase</command>
        <command>Verify coverage meets threshold</command>"
        }
        "cipher" => {
            r"<command>Run security audit tool</command>
        <command>Check for known vulnerabilities</command>"
        }
        _ => {
            r"<command>Run project test suite</command>
        <command>Run linter</command>"
        }
    }
}

/// Generate task.xml content for a task.
#[must_use]
pub fn generate_task_xml(task: &Task) -> String {
    let agent_hint = task.agent_hint.as_deref().unwrap_or("rex");
    let role = get_role_for_hint(agent_hint);
    let priority = task.priority.to_string();
    let dependencies = task.dependencies.join(", ");
    let validation_commands = get_validation_commands(agent_hint);

    let title_esc = xml_escape(&task.title);
    let desc_esc = xml_escape(&task.description);
    let test_esc = xml_escape(&task.test_strategy);

    // Extract code blocks from details for the code_signatures section
    let (code_blocks, remaining_details) = extract_code_blocks(&task.details);
    let details_esc = xml_escape(&remaining_details);

    // Build code signatures section if code blocks were found
    let code_signatures_section = if code_blocks.is_empty() {
        String::new()
    } else {
        let expected_lang = get_language_for_agent(agent_hint);
        let mut signatures = String::new();
        for (lang, code) in &code_blocks {
            let display_lang = if lang.is_empty() { expected_lang } else { lang };
            write!(
                signatures,
                "\n        <signature language=\"{}\">\n<![CDATA[\n{}\n]]>\n        </signature>",
                display_lang, code
            )
            .ok();
        }
        format!(
            "\n    <code_signatures expected_language=\"{}\">{}\n    </code_signatures>\n",
            expected_lang, signatures
        )
    };

    // Build requirements section - use details if available, otherwise provide guidance
    let requirements_content = if details_esc.is_empty() {
        format!(
            "Implement {} as described in the overview. Refer to PRD at .tasks/docs/prd.txt for detailed specifications.",
            title_esc
        )
    } else {
        details_esc.clone()
    };

    // Build acceptance criteria - use test_strategy if available
    let acceptance_content = if test_esc.is_empty() {
        String::new()
    } else {
        format!("<criterion>{}</criterion>", test_esc)
    };

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
        <background>
            This task is part of a larger project. Key resources:
            - PRD: .tasks/docs/prd.txt (full requirements and architecture)
            - Related tasks: {deps}
            - Infrastructure config: Check ConfigMaps for connection strings
        </background>
    </context>
{code_signatures}
    <requirements>
        <description>{requirements}</description>
        <constraints>
            <constraint>Match existing codebase patterns and conventions</constraint>
            <constraint>Create PR with atomic, well-described commits</constraint>
            <constraint>Include unit tests for new functionality</constraint>
            <constraint>PR title format: feat(task-{id}): {title}</constraint>
        </constraints>
    </requirements>

    <acceptance_criteria>
        <criterion>All requirements from task description implemented</criterion>
        <criterion>Tests passing with adequate coverage</criterion>
        <criterion>Code follows project conventions and style guide</criterion>
        {acceptance}
    </acceptance_criteria>

    <validation>
        {validation}
    </validation>

    <deliverables>
        <deliverable>Working implementation of described functionality</deliverable>
        <deliverable>Unit tests covering new code paths</deliverable>
        <deliverable>Pull request with clear description</deliverable>
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
        code_signatures = code_signatures_section,
        requirements = requirements_content,
        acceptance = acceptance_content,
        validation = validation_commands,
    )
}

/// Generate prompt.md content for a task.
#[must_use]
pub fn generate_task_prompt(task: &Task) -> String {
    let agent_hint = task.agent_hint.as_deref().unwrap_or("rex");
    let role = get_role_for_hint(agent_hint);
    let expected_lang = get_language_for_agent(agent_hint);

    let mut content = String::new();

    writeln!(content, "# Task {}: {}\n", task.id, task.title).ok();
    writeln!(
        content,
        "**Agent**: {} | **Language**: {}\n",
        agent_hint, expected_lang
    )
    .ok();

    writeln!(content, "## Role\n").ok();
    writeln!(
        content,
        "You are a {} implementing Task {}.\n",
        role, task.id
    )
    .ok();
    writeln!(content, "## Goal\n").ok();
    writeln!(content, "{}\n", task.description).ok();

    // Extract and display code signatures prominently
    let (code_blocks, remaining_details) = extract_code_blocks(&task.details);
    if !code_blocks.is_empty() {
        writeln!(content, "## Code Signatures\n").ok();
        writeln!(content, "Implement the following signatures:\n").ok();
        for (lang, code) in &code_blocks {
            let display_lang = if lang.is_empty() {
                expected_lang
            } else {
                lang.as_str()
            };
            writeln!(content, "```{}", display_lang).ok();
            writeln!(content, "{}", code).ok();
            writeln!(content, "```\n").ok();
        }
    }

    writeln!(content, "## Requirements\n").ok();
    if remaining_details.is_empty() && task.details.is_empty() {
        writeln!(
            content,
            "Implement the functionality described above. Refer to `.tasks/docs/prd.txt` for detailed specifications and architecture.\n"
        )
        .ok();
    } else if !remaining_details.is_empty() {
        writeln!(content, "{}\n", remaining_details).ok();
    } else {
        writeln!(content, "{}\n", task.details).ok();
    }

    writeln!(content, "## Acceptance Criteria\n").ok();
    if task.test_strategy.is_empty() {
        writeln!(content, "- All requirements from goal section implemented").ok();
        writeln!(content, "- Tests passing with adequate coverage").ok();
        writeln!(content, "- Code follows project conventions\n").ok();
    } else {
        writeln!(content, "{}\n", task.test_strategy).ok();
    }

    writeln!(content, "## Constraints\n").ok();
    writeln!(content, "- Match existing codebase patterns and style").ok();
    writeln!(content, "- Create PR with atomic, well-described commits").ok();
    writeln!(content, "- Include unit tests for new functionality").ok();
    writeln!(
        content,
        "- PR title: `feat(task-{}): {}`\n",
        task.id, task.title
    )
    .ok();

    // Add agent-specific guidance
    writeln!(content, "## Resources\n").ok();
    writeln!(content, "- PRD: `.tasks/docs/prd.txt`").ok();
    if !task.dependencies.is_empty() {
        writeln!(content, "- Dependencies: {}", task.dependencies.join(", ")).ok();
    }

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
        // Use task ID directly if it already starts with "task-", otherwise prefix it
        let dir_name = if task.id.starts_with("task-") {
            task.id.clone()
        } else {
            format!("task-{}", task.id)
        };
        let task_dir = output_dir.join(&dir_name);

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
    fn test_extract_code_blocks() {
        let text = r#"Some text before

```rust
fn hello() {
    println!("Hello");
}
```

Some text after

```typescript
const x = 1;
```
"#;
        let (blocks, remaining) = extract_code_blocks(text);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, "rust");
        assert!(blocks[0].1.contains("fn hello()"));
        assert_eq!(blocks[1].0, "typescript");
        assert!(blocks[1].1.contains("const x = 1"));
        assert!(remaining.contains("Some text before"));
        assert!(remaining.contains("Some text after"));
    }

    #[test]
    fn test_extract_code_blocks_no_language() {
        let text = r#"```
plain code
```"#;
        let (blocks, _) = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, "");
        assert!(blocks[0].1.contains("plain code"));
    }

    #[test]
    fn test_get_language_for_agent() {
        assert_eq!(get_language_for_agent("rex"), "rust");
        assert_eq!(get_language_for_agent("grizz"), "go");
        assert_eq!(get_language_for_agent("nova"), "typescript");
        assert_eq!(get_language_for_agent("blaze"), "tsx");
        assert_eq!(get_language_for_agent("bolt"), "yaml");
        assert_eq!(get_language_for_agent("unknown"), "text");
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
    fn test_generate_task_xml_with_code_signatures() {
        let mut task = sample_task();
        task.details = r#"Implement the API

```rust
pub struct User {
    pub id: Uuid,
    pub name: String,
}

pub async fn create_user(req: CreateUserRequest) -> Result<User, Error> {
    todo!()
}
```

Additional requirements here."#
            .to_string();

        let xml = generate_task_xml(&task);

        // Should contain code_signatures section with CDATA
        assert!(xml.contains("<code_signatures expected_language=\"rust\">"));
        assert!(xml.contains("<![CDATA["));
        assert!(xml.contains("pub struct User"));
        assert!(xml.contains("]]>"));
        // Should also contain the non-code requirements
        assert!(xml.contains("Additional requirements here"));
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
        assert!(md.contains("**Agent**: rex"));
        assert!(md.contains("**Language**: rust"));
        assert!(md.contains("## Role"));
        assert!(md.contains("Rust Engineer"));
        assert!(md.contains("## Goal"));
        assert!(md.contains("CRUD endpoints"));
        assert!(md.contains("## Requirements"));
        assert!(md.contains("Axum framework"));
    }

    #[test]
    fn test_generate_task_prompt_with_code_signatures() {
        let mut task = sample_task();
        task.details = r#"```rust
pub fn handler() -> impl IntoResponse {
    todo!()
}
```

Other requirements."#
            .to_string();

        let md = generate_task_prompt(&task);

        assert!(md.contains("## Code Signatures"));
        assert!(md.contains("```rust"));
        assert!(md.contains("pub fn handler()"));
        assert!(md.contains("Other requirements"));
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
