# Agent-Specific Handlebars Templates Implementation

You are implementing specialized container script templates for multi-agent orchestration. Create agent-specific Handlebars templates that enable Rex, Cleo, and Tess to execute their distinct workflows.

## Objective

Create agent-specific container scripts and template selection logic to replace the current single-template approach with specialized workflows for each agent type.

## Context

The Task Master system requires multi-agent orchestration where:
- **Rex/Blaze**: Documentation-first implementation workflow
- **Cleo**: Code quality and formatting workflow with CI validation
- **Tess**: Comprehensive testing workflow with live deployment validation

## Implementation Requirements

### 1. Create Agent-Specific Container Templates

**Rex/Blaze Template (container-rex.sh.hbs)**
- Documentation-first approach via MCP server queries
- Task file copying from `.taskmaster/docs/task-{id}/`
- Implementation-focused environment setup
- PR creation with proper task labeling

**Cleo Template (container-cleo.sh.hbs)**
- Code quality tools setup (Clippy, rustfmt)
- GitHub API authentication for label management
- CI test validation workflow
- Ready-for-QA label addition logic

**Tess Template (container-tess.sh.hbs)**
- Kubernetes admin access configuration
- Database admin credentials setup
- Testing infrastructure access
- 120% satisfaction requirement enforcement

### 2. Implement Template Selection Logic

Modify the controller's template loading mechanism:
```rust
fn get_container_template(github_app: &str) -> String {
    match github_app {
        "5DLabs-Rex" | "5DLabs-Blaze" => "container-rex.sh.hbs",
        "5DLabs-Cleo" => "container-cleo.sh.hbs", 
        "5DLabs-Tess" => "container-tess.sh.hbs",
        _ => "container.sh.hbs", // Fallback
    }
}
```

### 3. Template Structure Organization

```
infra/charts/controller/claude-templates/
├── container.sh.hbs           # Default template
├── container-rex.sh.hbs       # Rex implementation agent
├── container-cleo.sh.hbs      # Cleo quality agent  
├── container-tess.sh.hbs      # Tess testing agent
└── agents/                    # Agent-specific prompts
    ├── rex-system-prompt.md.hbs
    ├── cleo-system-prompt.md.hbs
    └── tess-system-prompt.md.hbs
```

## Technical Specifications

### Agent Environment Variables
- `AGENT_ROLE`: Defines agent's primary function (implementation/quality/testing)
- `WORKFLOW_STAGE`: Current stage in multi-agent workflow
- `GITHUB_APP`: Used for template selection and authentication
- `TASK_ID`: Enables task-specific resource access

### Authentication Requirements
- **Rex**: Standard GitHub App authentication
- **Cleo**: GitHub API access for PR labeling
- **Tess**: Admin credentials for infrastructure testing

### Workflow Integration Points
- Template selection based on `github_app` CRD field
- Agent-specific environment setup
- Handoff signals between agents (labels, events)
- Session continuity within agent workspaces

## Success Criteria

1. **Template Selection Works**: Correct container script selected based on `github_app`
2. **Agent Isolation**: Each agent runs with appropriate tools and permissions
3. **Workflow Handoffs**: Clear signals between agent stages (PR labels, events)
4. **Backward Compatibility**: Existing Rex/Blaze workflows continue functioning
5. **Maintainable Architecture**: Easy to add new agents or modify workflows

## Testing Requirements

- Verify template selection logic with different `github_app` values
- Test agent-specific environment setup and tool availability
- Validate workflow progression through all agent stages
- Confirm backward compatibility with existing implementations
- Test failure scenarios and error handling

## File Locations

All templates should be created in:
- `infra/charts/controller/claude-templates/container-{agent}.sh.hbs`
- Controller modifications in `controller/src/tasks/code/templates.rs`
- Template loading updates in `controller/src/tasks/code/resources.rs`

Focus on creating a clean, maintainable template architecture that supports multi-agent workflows while preserving existing functionality.