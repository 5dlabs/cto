# Workflow Parameter Persistence Design

## Problem Statement

When users initiate multi-task workflows through the MCP server, they specify parameters such as:
- Model selection (e.g., `claude-3-5-sonnet-20241022`)
- Agent assignments (Rex vs Blaze for implementation)
- Repository configuration
- Service identifiers

Currently, when task-1 completes and automatically triggers task-2, these user-specified parameters are lost. Task-2 starts with hardcoded defaults instead of the user's original choices, breaking workflow continuity.

## Solution: Annotation-Based Parameter Persistence

We will use Kubernetes annotations to store and forward workflow parameters across the task chain.

### Why Annotations?

After evaluating multiple approaches:

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Labels** | Easy to query, self-contained | 63 char limit, restricted charset | ❌ Too restrictive |
| **ConfigMap** | Unlimited size, structured storage | Separate lifecycle, orphaning risk | ❌ Over-engineered |
| **Annotations** | No size limits, self-contained, standard K8s pattern | Not queryable like labels | ✅ **Best fit** |

### Implementation Design

#### 1. Parameter Storage (Workflow Creation)

When creating a workflow, store initial parameters as JSON in an annotation:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: play-workflow-template-abc123
  annotations:
    platform.agents/initial-params: |
      {
        "model": "claude-3-5-sonnet-20241022",
        "implementation-agent": "5DLabs-Rex",
        "quality-agent": "5DLabs-Cleo",
        "testing-agent": "5DLabs-Tess",
        "repository": "5dlabs/cto-play-test",
        "service": "cto"
      }
spec:
  # ... workflow spec
```

#### 2. Parameter Retrieval (Task Completion)

When task N completes, the `merge-to-main-sensor` retrieves parameters from the workflow:

```bash
# Get the original workflow name from the PR/task context
ORIGINAL_WORKFLOW=$(kubectl get workflows -n agent-platform \
  -l task-id=$TASK_ID,workflow-type=play-orchestration \
  -o jsonpath='{.items[0].metadata.name}')

# Retrieve stored parameters
INITIAL_PARAMS=$(kubectl get workflow $ORIGINAL_WORKFLOW -n agent-platform \
  -o jsonpath='{.metadata.annotations.platform\.agents/initial-params}')

# Parse individual parameters
MODEL=$(echo "$INITIAL_PARAMS" | jq -r '.model')
IMPL_AGENT=$(echo "$INITIAL_PARAMS" | jq -r '."implementation-agent"')
# ... etc
```

#### 3. Parameter Forwarding (Next Task Creation)

Create task N+1 with the same parameters:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: play-workflow-template-
  annotations:
    platform.agents/initial-params: | 
      # Same JSON from original workflow
spec:
  workflowTemplateRef:
    name: play-workflow-template
  arguments:
    parameters:
      - name: model
        value: "$MODEL"  # From parsed JSON
      - name: implementation-agent  
        value: "$IMPL_AGENT"  # From parsed JSON
      # ... etc
```

### Key Benefits

1. **Parameter Consistency**: All tasks in a chain use the same user-specified parameters
2. **Self-Contained**: Parameters travel with workflows, no external dependencies
3. **Clean Lifecycle**: Automatic cleanup when workflows are deleted
4. **Extensible**: Easy to add new parameters without schema changes
5. **Debuggable**: Parameters visible in workflow YAML for troubleshooting

### Files to Modify

1. **`infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`**
   - Add annotation to store initial parameters when workflow is created

2. **`infra/gitops/resources/github-webhooks/merge-to-main-sensor.yaml`**
   - Replace inline workflow YAML with WorkflowTemplate reference
   - Add logic to retrieve and parse parameters from previous task's workflow
   - Forward parameters to next task's workflow

### Migration Path

1. New workflows will automatically get parameter persistence
2. Existing workflows without annotations will fall back to defaults
3. No breaking changes to current functionality

### Testing Strategy

1. Start task-1 with specific parameters
2. Complete task-1 (merge PR)
3. Verify task-2 starts with same parameters
4. Check Rex labels PRs correctly (not "run-unknown")
5. Verify full chain: task-1 → task-2 → task-3 with consistent parameters

## Implementation Status

- [x] Feature branch created: `feat/workflow-parameter-persistence`
- [ ] Workflow template updated to store parameters
- [ ] Task completion handler updated to read/forward parameters
- [ ] Testing completed
- [ ] Documentation updated