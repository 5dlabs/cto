# Review-Scope Workflow Design

## Overview

The review-scope workflow is an optional step that runs after intake completes, using an AI agent (Morgan/Cleo) to analyze the complexity report and automatically adjust task scope using the `scope_up_task` and `scope_down_task` MCP tools.

## Workflow Position

```
┌─────────────────────────────────────────────────────────────────┐
│                        Intake Workflow                          │
├─────────────────────────────────────────────────────────────────┤
│  PRD → Parse → Complexity → Expand → Route → Docs → Config     │
│                    │                                            │
│                    ▼                                            │
│           complexity-report.json                                │
└───────────────────────┬─────────────────────────────────────────┘
                        │
                        ▼ (optional)
┌─────────────────────────────────────────────────────────────────┐
│                    Review-Scope Step                            │
├─────────────────────────────────────────────────────────────────┤
│  1. Read complexity-report.json                                 │
│  2. Identify tasks with score >= threshold (scope_down)         │
│  3. Identify trivially small tasks (scope_up candidates)        │
│  4. Call scope_down_task for complex tasks                      │
│  5. Call scope_up_task for trivial tasks                        │
│  6. Re-expand adjusted tasks                                    │
│  7. Update tasks.json                                           │
│  8. Regenerate documentation                                    │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Options

### Option A: MCP Tool `review_scope`

A single MCP tool that orchestrates the entire review process:

```json
{
  "name": "review_scope",
  "description": "Review complexity report and automatically adjust task scope",
  "inputSchema": {
    "type": "object",
    "properties": {
      "complexityThreshold": {
        "type": "integer",
        "description": "Tasks >= this score are candidates for scope_down (default: 7)"
      },
      "simplicityThreshold": {
        "type": "integer", 
        "description": "Tasks <= this score are candidates for scope_up (default: 2)"
      },
      "scopeDownStrength": {
        "type": "string",
        "description": "Strength for scope_down: light, regular, heavy (default: regular)"
      },
      "scopeUpStrength": {
        "type": "string",
        "description": "Strength for scope_up: light, regular, heavy (default: light)"
      },
      "autoExpand": {
        "type": "boolean",
        "description": "Automatically re-expand adjusted tasks (default: true)"
      },
      "dryRun": {
        "type": "boolean",
        "description": "Preview changes without applying (default: false)"
      },
      "model": {
        "type": "string",
        "description": "AI model to use"
      },
      "tag": {
        "type": "string",
        "description": "Tag context"
      }
    }
  }
}
```

**Pros:**
- Simple to invoke
- Self-contained logic
- Can be called from any context

**Cons:**
- Less flexible
- Agent-like behavior hidden in tool

### Option B: Argo Workflow Step (Recommended)

Add a new template step to the intake workflow that creates a CodeRun for Morgan/Cleo to analyze and adjust tasks.

#### Workflow Template Addition

```yaml
# Add to intake workflow templates
- name: review-scope-step
  inputs:
    parameters:
      - name: repository
      - name: docs-project-directory
      - name: complexity-threshold
        default: "7"
      - name: simplicity-threshold
        default: "2"
      - name: scope-down-strength
        default: "regular"
      - name: scope-up-strength
        default: "light"
      - name: review-agent
        default: "morgan"  # or cleo
      - name: review-cli
        default: "claude"
      - name: review-model
        default: "claude-sonnet-4-20250514"
  container:
    image: ghcr.io/5dlabs/cto-tools:latest
    command: ["/bin/sh", "-c"]
    args:
      - |
        cat > /tmp/review-prompt.md << 'EOF'
        # Task Scope Review
        
        Review the complexity report and adjust task scope:
        
        ## Rules
        1. Tasks with complexity >= {{inputs.parameters.complexity-threshold}} → call scope_down_task
        2. Tasks with complexity <= {{inputs.parameters.simplicity-threshold}} → consider scope_up_task
        3. Use strength "{{inputs.parameters.scope-down-strength}}" for scope_down
        4. Use strength "{{inputs.parameters.scope-up-strength}}" for scope_up
        
        ## Steps
        1. Read complexity-report.json
        2. Identify tasks needing adjustment
        3. Call appropriate scope tools
        4. Re-expand adjusted tasks with expand_all
        5. Save updated tasks
        EOF
        
        # Create CodeRun for Morgan/Cleo
        kubectl create -f - << 'YAML'
        apiVersion: agents.platform/v1
        kind: CodeRun
        metadata:
          generateName: review-scope-
          namespace: cto
          labels:
            workflow-type: review-scope
        spec:
          agent: {{inputs.parameters.review-agent}}
          repositoryUrl: {{inputs.parameters.repository}}
          cliConfig:
            cliType: {{inputs.parameters.review-cli}}
            model: {{inputs.parameters.review-model}}
          promptFile: /tmp/review-prompt.md
          workingDirectory: {{inputs.parameters.docs-project-directory}}/.tasks
          remoteTools:
            - name: tasks-mcp
              url: http://tasks-mcp.cto.svc.cluster.local:8080
        YAML
```

**Pros:**
- Leverages existing agent infrastructure
- Full agent capabilities (reasoning, iteration)
- Observable in Argo UI
- Can use different agents for different needs

**Cons:**
- More complex setup
- Requires agent coordination

## Recommended Approach

**Option B (Argo Workflow Step)** is recommended because:

1. **Agent Reasoning**: Morgan/Cleo can use judgment to determine which tasks really need adjustment
2. **Flexibility**: Can iterate multiple times if initial scope adjustments aren't sufficient
3. **Observability**: Progress visible in Argo Workflows UI
4. **Consistency**: Uses same patterns as other workflow steps

## Integration with Intake

The review-scope step should be an **optional** parameter for the intake workflow:

```yaml
# intake parameters
- name: review-scope
  description: "Enable automatic scope review after task generation"
  value: "false"  # Default disabled

- name: review-scope-agent
  description: "Agent for scope review (morgan or cleo)"
  value: "morgan"

- name: complexity-threshold
  description: "Threshold for scope_down (tasks >= this)"
  value: "7"
```

When `review-scope: true`, the workflow adds the review-scope step after documentation generation.

## MCP Tools Required

The review-scope step relies on these MCP tools (already implemented):

| Tool | Purpose |
|------|---------|
| `complexity_report` | Read saved complexity analysis |
| `scope_down_task` | Reduce task complexity |
| `scope_up_task` | Consolidate trivial tasks |
| `expand_all` | Re-expand adjusted tasks |
| `get_tasks` | List current tasks |

## Agent Prompt Template

Create a prompt template for the review agent:

```markdown
# Task Scope Review

You are reviewing the complexity analysis for a software project and adjusting task scope.

## Current Complexity Report

{{complexity_report}}

## Rules

1. **High Complexity Tasks** (score >= {{complexity_threshold}}):
   - Call `scope_down_task` with strength "{{scope_down_strength}}"
   - Split into smaller, more manageable tasks
   - Focus on MVP functionality first

2. **Low Complexity Tasks** (score <= {{simplicity_threshold}}):
   - Consider if they should be merged with related tasks
   - Call `scope_up_task` with strength "{{scope_up_strength}}"
   - Only merge if tasks are truly trivial and related

3. **Medium Complexity Tasks** (between thresholds):
   - Leave unchanged unless there's a clear reason to adjust

## Process

1. Read the complexity report
2. List tasks that need adjustment
3. For each task needing scope_down, call the tool
4. For related trivial tasks, consider scope_up
5. After all adjustments, call `expand_all` to regenerate subtasks
6. Verify the final task structure makes sense

## Output

Provide a summary of:
- Tasks that were scoped down and why
- Tasks that were scoped up and why
- Final task count vs original
- Any concerns or recommendations
```

## Future Enhancements

1. **Learning from outcomes**: Track which scope adjustments led to successful implementations
2. **Project-specific rules**: Configure thresholds per project type
3. **Interactive review**: Allow human approval before applying changes
4. **Rollback support**: Save original tasks for comparison/rollback
