# Play Workflow Configuration Management Design

## Problem Statement

Currently, the play workflow has inconsistent configuration between Task 1 and subsequent tasks:

- **Task 1**: Uses dynamic settings from MCP call (e.g., `5DLabs-Blaze`, `claude-opus-4-1-20250805`)
- **Task 2+**: Uses hardcoded settings in `merge-to-main-sensor.yaml` (always `5DLabs-Rex`, `claude-sonnet-4-20250514`)

This creates inconsistency and prevents users from changing agent assignments mid-project.

## Solution: Project-Scoped ConfigMap



### Overview

Use a Kubernetes ConfigMap to store project-specific configuration that persists across task completions and can be updated between tasks.



### Architecture









```
┌─────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   MCP Tool      │───▶│   ConfigMap         │◀───│ Task-Complete       │
│   (User Call)   │    │   (Project Config)  │    │ Workflow            │
└─────────────────┘    └─────────────────────┘    └─────────────────────┘
         │                         │                         │
         │                         ▼                         ▼
         ▼              ┌─────────────────────┐    ┌─────────────────────┐
┌─────────────────┐     │ Consistent Settings │    │   Next Task         │
│   Task 1        │     │ Across All Tasks    │    │   CodeRuns          │
│   Workflow      │     └─────────────────────┘    │   (Rex/Cleo/Tess)   │
└─────────────────┘                                └─────────────────────┘








```

### ConfigMap Structure

**Name**: `{service}-play-project-config` (e.g., `cto-play-project-config`)
**Namespace**: `cto`





```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cto-play-project-config
  namespace: cto
  labels:
    type: play-workflow-config
    service: cto
    project: cto-play-test
data:
  # Agent assignments
  implementationAgent: "5DLabs-Rex"
  qualityAgent: "5DLabs-Cleo"
  testingAgent: "5DLabs-Tess"

  # Runtime settings
  model: "claude-sonnet-4-20250514"
  repository: "5dlabs/cto-play-test"
  service: "cto"
  docsRepository: "5dlabs/cto-play-test"
  docsProjectDirectory: "docs"

  # Optional overrides
  workingDirectory: "."
  continueSession: "true"
  overwriteMemory: "false"
  contextVersion: "1"








```

## Implementation Plan



### Phase 1: MCP Tool Updates

**File**: `mcp/src/tools.rs` (or equivalent)





```rust
// In mcp_cto_play function:


1. Create/update ConfigMap with user-provided settings


2. Use ConfigMap values as defaults if parameters not provided


3. Start workflow with task_id as usual








```

**Behavior**:
- **First run**: Creates ConfigMap with settings, starts Task 1
- **Subsequent runs**: Updates ConfigMap with new settings, starts specified task
- **No changes**: Uses existing ConfigMap values



### Phase 2: Task-Complete Workflow Updates

**File**: `infra/gitops/resources/github-webhooks/merge-to-main-sensor.yaml`

Replace hardcoded values:




```bash


# OLD (hardcoded):
githubApp: "5DLabs-Rex"
model: "claude-sonnet-4-20250514"

# NEW (dynamic from ConfigMap):
IMPL_AGENT=$(kubectl get configmap ${SERVICE}-play-project-config -o jsonpath='{.data.implementationAgent}')
MODEL=$(kubectl get configmap ${SERVICE}-play-project-config -o jsonpath='{.data.model}')

# Use in YAML generation:
githubApp: "$IMPL_AGENT"
model: "$MODEL"








```

### Phase 3: Error Handling & Fallbacks

**ConfigMap Missing**: Fall back to `cto-config.json` defaults
**Invalid Values**: Validate agent names exist in secrets
**Network Issues**: Retry with exponential backoff

## User Experience

### Scenario 1: Fresh Project Start




```bash


# User starts with Blaze
mcp_cto_play task_id=1 implementation_agent=5DLabs-Blaze model=claude-opus-4-1-20250805



# Result:
# - ConfigMap created with Blaze + Opus settings


# - Task 1 uses Blaze + Opus


# - Future tasks (2,3,4...) will use Blaze + Opus








```

### Scenario 2: Mid-Project Agent Change




```bash
# After Task 2 completes, user wants to switch to Rex
mcp_cto_play task_id=3 implementation_agent=5DLabs-Rex



# Result:
# - ConfigMap updated to use Rex (keeping other settings)


# - Task 3 starts with Rex


# - Future tasks (4,5,6...) will use Rex








```

### Scenario 3: Resume Without Changes




```bash
# User just wants to resume from Task 4
mcp_cto_play task_id=4



# Result:
# - ConfigMap unchanged
# - Task 4 uses existing ConfigMap settings








```

## Benefits

1. **Consistency**: All tasks in a project use same agent/model settings
2. **Flexibility**: Can change settings between tasks
3. **Persistence**: Settings survive task completions
4. **Single Source**: ConfigMap is the authoritative config for the project
5. **Debuggability**: Easy to inspect current project settings via kubectl

## Migration Strategy



### Backwards Compatibility


- Keep hardcoded defaults as fallback if ConfigMap doesn't exist


- Gradually migrate existing projects on next MCP call

### Rollout Plan


1. **Deploy MCP changes** (create ConfigMaps)


2. **Deploy sensor changes** (read from ConfigMaps)


3. **Test with new projects** first


4. **Migrate existing projects** as they're used

## Testing Strategy

### Unit Tests


- ConfigMap creation/update logic


- Fallback behavior when ConfigMap missing


- Value validation and sanitization

### Integration Tests


- Full task sequence with consistent settings


- Mid-project agent changes


- Error conditions and fallbacks

### Manual Testing


- Start fresh project → verify consistency


- Change agents mid-project → verify new settings propagate
- Edge cases: missing ConfigMap, invalid agents, etc.

## Implementation Complexity

**Estimated Effort**: 4-6 hours
- **MCP Tool Changes**: 2 hours
- **Sensor Template Changes**: 2 hours
- **Testing & Validation**: 2 hours

**Risk Level**: Low-Medium


- Non-breaking change (fallbacks maintained)


- Isolated to play workflow functionality


- Well-defined scope and interfaces

## Future Enhancements

### Project Isolation
Support multiple parallel projects:




```yaml
# Multiple ConfigMaps:
cto-play-project-config        # Default project
cto-play-feature-xyz-config    # Feature branch project
cto-play-experiment-config     # Experimental project








```

### Web UI Integration
Provide web interface to view/modify project configurations without kubectl.

### Configuration Templates
Pre-defined templates for common workflows:


- "Fast Development" (Rex/Sonnet)


- "High Quality" (Blaze/Opus)


- "Cost Optimized" (Rex/Haiku)
