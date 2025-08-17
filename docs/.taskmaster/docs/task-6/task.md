# Task 6: Develop Agent-Specific Handlebars Templates

## Overview

Implement specialized container scripts for Rex/Blaze, Cleo, and Tess agents with template selection logic based on github_app parameter. This simplifies the architecture by creating agent-specific container scripts instead of complex template conditionals.

## Technical Context

The current template system uses a single `container.sh.hbs` for all agents. Multi-agent orchestration requires specialized workflows:
- **Rex/Blaze**: Documentation-first implementation workflow
- **Cleo**: Code quality and formatting workflow with PR labeling
- **Tess**: Testing workflow with deployment validation and comprehensive test coverage

## Implementation Guide

### Phase 1: Create Rex Container Template

1. **Create container-rex.sh.hbs**
   ```bash
   # Location: infra/charts/controller/claude-templates/container-rex.sh.hbs
   ```

   Key features:
   - Documentation-first approach via MCP documentation server
   - Task file copying from `.taskmaster/docs/task-{id}/`
   - Implementation workflow focus
   - Branch naming: `task-{id}-{description}`
   - PR labeling with `task-{id}` for correlation

2. **Rex-Specific Workflow Implementation**
   ```bash
   #!/bin/bash
   echo "🔥 Rex/Blaze: Implementation Agent"
   echo "Mission: Documentation-first feature implementation"
   
   # Documentation research phase
   echo "📚 Consulting documentation server for implementation patterns..."
   
   # Copy task files
   cp /workspace/docs/.taskmaster/docs/task-{{task_id}}/* /workspace/task-context/
   
   # Set implementation context
   export AGENT_ROLE="implementation"
   export WORKFLOW_STAGE="implementation"
   ```

### Phase 2: Create Cleo Container Template

1. **Create container-cleo.sh.hbs**
   ```bash
   # Location: infra/charts/controller/claude-templates/container-cleo.sh.hbs
   ```

   Key features:
   - Code quality checks (Clippy pedantic, rustfmt)
   - CI test validation before completion
   - Ready-for-QA label addition as handoff signal
   - Zero tolerance for quality issues

2. **Cleo-Specific Workflow Implementation**
   ```bash
   #!/bin/bash
   echo "🎯 Cleo: Code Quality & Formatting Agent"
   echo "Mission: Relentless pursuit of 100% code quality"
   
   # Quality validation setup
   export AGENT_ROLE="quality"
   export WORKFLOW_STAGE="quality-work"
   export QUALITY_TOLERANCE="zero"
   
   # GitHub API setup for label management
   export GITHUB_TOKEN=$(cat /etc/github-app/token)
   export GITHUB_APP_ID=$(cat /etc/github-app/app-id)
   ```

### Phase 3: Create Tess Container Template

1. **Create container-tess.sh.hbs**
   ```bash
   # Location: infra/charts/controller/claude-templates/container-tess.sh.hbs
   ```

   Key features:
   - Comprehensive code review against acceptance criteria
   - Live Kubernetes deployment testing
   - Test coverage enhancement to near 100%
   - Admin access to all infrastructure components

2. **Tess-Specific Workflow Implementation**
   ```bash
   #!/bin/bash
   echo "🧪 Tess: Quality Assurance & Testing Agent"
   echo "Mission: 120% satisfaction through comprehensive testing"
   
   # Testing and deployment setup
   export AGENT_ROLE="testing"
   export WORKFLOW_STAGE="testing-work"
   export KUBECONFIG=/etc/kube/admin-config
   export SATISFACTION_LEVEL="120"
   
   # Admin access setup
   source /etc/admin-credentials/postgres-admin
   source /etc/admin-credentials/redis-admin
   ```

### Phase 4: Implement Template Selection Logic

1. **Controller Template Resolution**
   ```rust
   // In controller/src/tasks/code/templates.rs
   fn get_container_template(github_app: &str) -> String {
       match github_app {
           "5DLabs-Rex" | "5DLabs-Blaze" => "container-rex.sh.hbs",
           "5DLabs-Cleo" => "container-cleo.sh.hbs",
           "5DLabs-Tess" => "container-tess.sh.hbs",
           _ => "container.sh.hbs", // Fallback to default
       }
   }
   ```

2. **Template Loading Enhancement**
   ```rust
   // Enhanced template loading with agent-specific selection
   pub fn render_container_script(context: &Context) -> Result<String> {
       let github_app = context.get("github_app")
           .and_then(|v| v.as_str())
           .unwrap_or("default");
       
       let template_name = get_container_template(github_app);
       load_and_render_template(template_name, context)
   }
   ```

### Phase 5: Update Template Loading Mechanism

1. **Modify Controller Resource Creation**
   ```rust
   // In create_configmap function
   let container_script = render_container_script(&context)?;
   
   configmap_data.insert(
       "container.sh".to_string(),
       container_script,
   );
   ```

2. **Backward Compatibility**
   - Keep existing `container.sh.hbs` as fallback
   - Agent-specific templates override when `github_app` matches
   - No breaking changes to existing Rex/Blaze workflows

## Code Examples

### Rex Container Script Template
```handlebars
#!/bin/bash
set -euo pipefail

echo "🔥 Rex/Blaze: Implementation Agent Starting"
echo "Repository: {{service}}"
echo "GitHub App: {{github_app}}"
echo "Task ID: {{task_id}}"

# Documentation research phase
echo "📚 Consulting MCP documentation server..."

# Task context setup
mkdir -p /workspace/task-context
{{#if task_id}}
cp -r /workspace/docs/.taskmaster/docs/task-{{task_id}}/* /workspace/task-context/ 2>/dev/null || true
{{/if}}

# Set Rex-specific environment
export AGENT_ROLE="implementation"
export WORKFLOW_STAGE="implementation"
export DOCUMENTATION_FIRST="true"

# Start Claude with implementation focus
exec /app/claude-desktop \
  --config /etc/claude/client-config.json \
  --memory /workspace/CLAUDE.md \
  --continue-session={{continue_session}}
```

### Cleo Container Script Template
```handlebars
#!/bin/bash
set -euo pipefail

echo "🎯 Cleo: Code Quality Agent Starting"
echo "Repository: {{service}}"
echo "Mission: Zero tolerance for quality issues"

# Quality tools setup
export AGENT_ROLE="quality"
export WORKFLOW_STAGE="quality-work"
export CLIPPY_OPTS="--all-targets --all-features -- -D warnings"

# GitHub API authentication for label management
export GITHUB_TOKEN=$(cat /etc/github-app/token 2>/dev/null || echo "")
export GITHUB_APP_ID=$(cat /etc/github-app/app-id 2>/dev/null || echo "")

# Wait for ready-for-qa prerequisite check
if [ -f /workspace/PR_INFO ]; then
  echo "📋 Checking if PR is ready for quality work..."
  # Additional validation logic here
fi

# Start Claude with quality focus
exec /app/claude-desktop \
  --config /etc/claude/client-config.json \
  --memory /workspace/CLAUDE.md \
  --continue-session={{continue_session}}
```

### Tess Container Script Template
```handlebars
#!/bin/bash
set -euo pipefail

echo "🧪 Tess: Quality Assurance Agent Starting"
echo "Repository: {{service}}"
echo "Mission: 120% satisfaction through comprehensive testing"

# Testing environment setup
export AGENT_ROLE="testing"
export WORKFLOW_STAGE="testing-work"
export SATISFACTION_LEVEL="120"

# Kubernetes admin access
export KUBECONFIG=/etc/kube/admin-config

# Database admin credentials
{{#if postgres_admin_secret}}
export PGPASSWORD=$(cat /etc/admin-credentials/postgres-password)
export PGUSER=$(cat /etc/admin-credentials/postgres-user)
{{/if}}

# Redis admin credentials
{{#if redis_admin_secret}}
export REDIS_PASSWORD=$(cat /etc/admin-credentials/redis-password)
{{/if}}

# Only start if PR has ready-for-qa label
if [ -f /workspace/PR_INFO ]; then
  LABELS=$(cat /workspace/PR_INFO | jq -r '.labels[].name' | tr '\n' ' ')
  if [[ ! "$LABELS" =~ "ready-for-qa" ]]; then
    echo "⏳ Waiting for ready-for-qa label before starting..."
    exit 0
  fi
fi

# Start Claude with testing focus
exec /app/claude-desktop \
  --config /etc/claude/client-config.json \
  --memory /workspace/CLAUDE.md \
  --continue-session={{continue_session}}
```

## Architecture Patterns

### Agent-Specific Template Selection
```
infra/charts/controller/claude-templates/
├── container.sh.hbs           # Default/fallback template
├── container-rex.sh.hbs       # Rex/Blaze implementation agent
├── container-cleo.sh.hbs      # Cleo quality agent
├── container-tess.sh.hbs      # Tess testing agent
└── agents/                    # Agent-specific system prompts
    ├── rex-system-prompt.md.hbs
    ├── cleo-system-prompt.md.hbs
    └── tess-system-prompt.md.hbs
```

### Template Context Enhancement
The existing Handlebars context is enhanced with agent-specific variables:
- `github_app`: Determines which template to load
- `agent_role`: Sets agent behavioral focus
- `workflow_stage`: Defines current workflow phase
- `task_id`: Enables task-specific resource access

## Testing Strategy

### Template Selection Testing
1. **Verify Correct Template Selection**
   ```bash
   # Test Rex template selection
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-rex-template
   spec:
     github_app: "5DLabs-Rex"
     service: "cto"
   EOF
   
   # Verify container-rex.sh.hbs was used
   kubectl get configmap test-rex-template-config -o yaml | grep "Rex/Blaze"
   ```

2. **Test Agent-Specific Behaviors**
   - Rex: Documentation consultation and task file access
   - Cleo: Code quality tools and GitHub API access
   - Tess: Admin credentials and testing infrastructure

### Workflow Integration Testing
1. **End-to-End Agent Handoff**
   - Rex creates PR → triggers Cleo
   - Cleo adds ready-for-qa label → triggers Tess
   - Tess completes testing → PR approval

2. **Template Rendering Validation**
   - All Handlebars variables resolve correctly
   - Agent-specific environment variables are set
   - Container scripts execute without errors

## Key Design Decisions

1. **Simplified Template Architecture**: Agent-specific container scripts avoid complex conditionals
2. **Backward Compatibility**: Existing Rex/Blaze workflows continue to work
3. **Clear Agent Separation**: Each agent has distinct responsibilities and tooling
4. **GitHub App Based Selection**: Template selection driven by `github_app` field
5. **Maintainable Structure**: Easy to add new agents or modify existing workflows

## References

- [Multi-Agent Architecture](/.taskmaster/docs/architecture.md)
- [CodeRun CRD Specification](controller/src/crds/coderun.rs)  
- [Template System Implementation](controller/src/tasks/code/templates.rs)
- [Container Script Examples](infra/charts/controller/claude-templates/)