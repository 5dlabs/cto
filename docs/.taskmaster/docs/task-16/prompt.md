# Task 16: Controller Template Loading - Autonomous Implementation Prompt

## Objective
Implement agent-specific container script selection in the Rust controller based on the `github_app` field. Create a clean mapping system that routes different GitHub App agents to their appropriate container templates.

## Context
You are working on a multi-agent orchestration system where different AI agents (Rex, Blaze, Cleo, Tess) handle different types of workflows. The controller needs to select the appropriate container script template based on which agent is being invoked.

## Agent to Template Mapping
- `5DLabs-Rex` or `5DLabs-Blaze` → `container-rex.sh.hbs` (implementation workflow)
- `5DLabs-Cleo` → `container-cleo.sh.hbs` (code quality workflow)
- `5DLabs-Tess` → `container-tess.sh.hbs` (testing workflow)

## Implementation Requirements

### 1. Create AgentTemplateMapper
**Location**: `controller/src/tasks/code/templates.rs`

Implement a struct that:
- Maps agent names to template filenames
- Extracts agent names from github_app strings (handle "[bot]" suffix)
- Provides fallback to default template for unknown agents
- Returns appropriate error messages for debugging

### 2. Core Functions to Implement
```rust
pub fn get_template_for_agent(&self, github_app: &str) -> Result<String>
pub fn extract_agent_name(&self, github_app: &str) -> Result<String>  
pub fn get_template_for_agent_with_fallback(&self, github_app: &str) -> String
pub fn load_agent_template(github_app: &str) -> Result<String>
```

### 3. Integration Points
- Update existing task processing to use new template selection
- Ensure backward compatibility with current template loading
- Add proper error handling and logging
- Implement caching for performance

### 4. Template File Structure
Each template should be self-contained:
- Proper shebang and error handling (`set -euo pipefail`)
- Environment variable setup
- Agent-specific workflow logic
- Handlebars template variables for dynamic content

## Technical Constraints
- Use standard Rust patterns and error handling
- Maintain compatibility with existing Handlebars integration
- Templates must be stored in `templates/` directory
- Support both direct agent names and "[bot]" suffixed formats
- Implement proper logging for debugging and monitoring

## Success Criteria
1. Agent name extraction works for various input formats
2. Template selection correctly maps agents to appropriate scripts
3. Fallback mechanism handles unknown agents gracefully
4. Error messages are clear and actionable
5. Integration maintains existing functionality
6. Performance is optimized with appropriate caching

## Testing Requirements
- Unit tests for agent mapping logic
- Template loading error scenarios
- Integration tests with actual template files
- Fallback behavior validation
- Performance testing for high-frequency operations

## Files to Modify/Create
- `controller/src/tasks/code/templates.rs` - Main implementation
- `templates/container-cleo.sh.hbs` - Code quality workflow template  
- `templates/container-tess.sh.hbs` - Testing workflow template
- Update existing integration points in task processing

## Dependencies
- Dependencies: Tasks 4, 6 (template system foundation)
- Ensure existing template loading infrastructure is available
- Handlebars crate for template compilation
- Standard file I/O and error handling libraries

## Implementation Notes
- Avoid complex Handlebars conditionals; use separate template files
- Each container script should contain complete workflow logic for its agent
- Consider configuration-driven mapping for future extensibility  
- Implement comprehensive logging for troubleshooting
- Plan for graceful degradation when templates are missing