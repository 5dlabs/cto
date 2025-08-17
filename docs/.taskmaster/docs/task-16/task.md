# Task 16: Implement Controller Template Loading

## Overview
Update the Rust controller to implement agent-specific container script selection based on the `github_app` field. This creates clean separation between different agent workflows while avoiding complex template conditionals.

## Technical Implementation

### Architecture
The template loading system maps GitHub App identifiers to specific container script templates:
- `5DLabs-Rex` / `5DLabs-Blaze` → `container-rex.sh.hbs` (implementation workflow)
- `5DLabs-Cleo` → `container-cleo.sh.hbs` (code quality workflow)  
- `5DLabs-Tess` → `container-tess.sh.hbs` (testing workflow)

### Implementation Steps

#### 1. Update Template Selection Logic

**File**: `controller/src/tasks/code/templates.rs`

```rust
use std::collections::HashMap;
use anyhow::{Result, anyhow};

pub struct AgentTemplateMapper {
    agent_templates: HashMap<String, String>,
}

impl AgentTemplateMapper {
    pub fn new() -> Self {
        let mut agent_templates = HashMap::new();
        
        // Implementation workflow agents
        agent_templates.insert("5DLabs-Rex".to_string(), "container-rex.sh.hbs".to_string());
        agent_templates.insert("5DLabs-Blaze".to_string(), "container-rex.sh.hbs".to_string());
        
        // Code quality workflow agent
        agent_templates.insert("5DLabs-Cleo".to_string(), "container-cleo.sh.hbs".to_string());
        
        // Testing workflow agent
        agent_templates.insert("5DLabs-Tess".to_string(), "container-tess.sh.hbs".to_string());
        
        Self { agent_templates }
    }
    
    pub fn get_template_for_agent(&self, github_app: &str) -> Result<String> {
        // Extract agent name from github_app field
        let agent_name = self.extract_agent_name(github_app)?;
        
        self.agent_templates
            .get(&agent_name)
            .ok_or_else(|| anyhow!("No template found for agent: {}", agent_name))
            .map(|template| template.clone())
    }
    
    fn extract_agent_name(&self, github_app: &str) -> Result<String> {
        // Handle various github_app formats
        if github_app.contains("[bot]") {
            // Format: "5DLabs-Rex[bot]" -> "5DLabs-Rex"
            github_app.split("[bot]")
                .next()
                .ok_or_else(|| anyhow!("Invalid github_app format: {}", github_app))
                .map(|s| s.to_string())
        } else {
            // Direct format: "5DLabs-Rex" -> "5DLabs-Rex"
            Ok(github_app.to_string())
        }
    }
}

// Integration with existing template loading
pub fn load_agent_template(github_app: &str) -> Result<String> {
    let mapper = AgentTemplateMapper::new();
    let template_name = mapper.get_template_for_agent(github_app)?;
    
    // Load template content from filesystem
    load_template_file(&template_name)
}

fn load_template_file(template_name: &str) -> Result<String> {
    use std::fs;
    use std::path::Path;
    
    let template_path = Path::new("templates").join(template_name);
    fs::read_to_string(&template_path)
        .map_err(|e| anyhow!("Failed to load template {}: {}", template_name, e))
}
```

#### 2. Update Task Processing Integration

```rust
// In task processing logic
pub fn process_code_task(task: &CodeTask) -> Result<String> {
    let template_content = load_agent_template(&task.github_app)?;
    
    // Compile template with task context
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("container", &template_content)?;
    
    let context = create_template_context(task)?;
    let rendered = handlebars.render("container", &context)?;
    
    Ok(rendered)
}
```

#### 3. Error Handling and Fallbacks

```rust
impl AgentTemplateMapper {
    pub fn get_template_for_agent_with_fallback(&self, github_app: &str) -> String {
        match self.get_template_for_agent(github_app) {
            Ok(template) => template,
            Err(_) => {
                // Log warning and use default template
                log::warn!("Unknown agent {}, falling back to default template", github_app);
                "container-rex.sh.hbs".to_string() // Default implementation workflow
            }
        }
    }
}
```

### Template Structure

Each container template should be self-contained with complete workflow logic:

**container-rex.sh.hbs** - Implementation workflow
```bash
#!/bin/bash
# Rex/Blaze Implementation Workflow
set -euo pipefail

# Setup environment
export GITHUB_TOKEN="${{github_token}}"
export REPO_URL="${{repo_url}}"

# Clone and setup
git clone "$REPO_URL" /workspace
cd /workspace

# Implementation-specific logic
echo "Starting implementation workflow for {{github_app}}"
# ... implementation steps
```

**container-cleo.sh.hbs** - Code quality workflow  
```bash
#!/bin/bash
# Cleo Code Quality Workflow
set -euo pipefail

# Setup environment
export GITHUB_TOKEN="${{github_token}}"
export REPO_URL="${{repo_url}}"

# Quality analysis specific logic
echo "Starting code quality workflow for {{github_app}}"
# ... quality checks, linting, static analysis
```

**container-tess.sh.hbs** - Testing workflow
```bash
#!/bin/bash
# Tess Testing Workflow  
set -euo pipefail

# Setup environment
export GITHUB_TOKEN="${{github_token}}"
export REPO_URL="${{repo_url}}"

# Testing specific logic
echo "Starting testing workflow for {{github_app}}"
# ... test execution, coverage analysis
```

## Integration Points

### 1. Controller Entry Point
Update the main task processing function to use the new template selection:

```rust
pub async fn execute_code_task(task: CodeTask) -> Result<TaskResult> {
    // Select appropriate template based on agent
    let container_script = load_agent_template(&task.github_app)?;
    
    // Create and execute container with agent-specific script
    let result = execute_container_workflow(&container_script, &task).await?;
    
    Ok(result)
}
```

### 2. Configuration Management
Allow template mappings to be configured externally:

```rust
// config.rs
#[derive(Deserialize)]
pub struct TemplateConfig {
    pub agent_templates: HashMap<String, String>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        let mut agent_templates = HashMap::new();
        agent_templates.insert("5DLabs-Rex".to_string(), "container-rex.sh.hbs".to_string());
        // ... other mappings
        
        Self { agent_templates }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_template_mapping() {
        let mapper = AgentTemplateMapper::new();
        
        assert_eq!(
            mapper.get_template_for_agent("5DLabs-Rex").unwrap(),
            "container-rex.sh.hbs"
        );
        assert_eq!(
            mapper.get_template_for_agent("5DLabs-Cleo").unwrap(), 
            "container-cleo.sh.hbs"
        );
    }
    
    #[test]
    fn test_agent_name_extraction() {
        let mapper = AgentTemplateMapper::new();
        
        assert_eq!(
            mapper.extract_agent_name("5DLabs-Rex[bot]").unwrap(),
            "5DLabs-Rex"
        );
        assert_eq!(
            mapper.extract_agent_name("5DLabs-Tess").unwrap(),
            "5DLabs-Tess"
        );
    }
    
    #[test]
    fn test_unknown_agent_fallback() {
        let mapper = AgentTemplateMapper::new();
        let template = mapper.get_template_for_agent_with_fallback("Unknown-Agent");
        
        assert_eq!(template, "container-rex.sh.hbs");
    }
    
    #[test]
    fn test_template_loading() {
        // Test that template files exist and are valid
        let template_content = load_template_file("container-rex.sh.hbs").unwrap();
        assert!(!template_content.is_empty());
        assert!(template_content.contains("#!/bin/bash"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_template_selection() {
    let task = CodeTask {
        github_app: "5DLabs-Cleo".to_string(),
        // ... other fields
    };
    
    let result = execute_code_task(task).await;
    assert!(result.is_ok());
    
    // Verify Cleo-specific workflow was executed
    // Check for Cleo-specific outputs or side effects
}
```

## Performance Considerations

1. **Template Caching**: Cache loaded templates to avoid file I/O on every request
2. **Agent Detection**: Optimize agent name extraction for high-frequency operations  
3. **Memory Usage**: Use string interning for template names to reduce allocations

## Security Considerations

1. **Template Validation**: Validate template content before execution
2. **Path Security**: Prevent path traversal attacks in template loading
3. **Agent Authentication**: Verify github_app authenticity before template selection

## Rollback Strategy

1. Keep existing template loading as fallback
2. Feature flag to enable/disable new agent-specific templates
3. Monitoring and alerting for template selection failures
4. Quick rollback capability through configuration change