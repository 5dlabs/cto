# Model Rotation Feature Rollout

**Status**: Design Document  
**Created**: 2025-01-XX  
**Author**: System Design

## Overview

This document outlines the rollout of the model rotation feature across all CLI implementations (Claude, Factory, Codex, Cursor, OpenCode) and all agents (Rex, Cleo, Tess, Morgan, Blaze, Cipher).

## Current State

### What Exists
- Model rotation is **only implemented in Factory CLI** (`container-base.sh.hbs`)
- Uses hardcoded `model_rotation` array passed from CRD/template
- Cycles through models on each retry attempt using modulo arithmetic
- Example: `MODEL_INDEX=$(((ATTEMPT - 1) % MODEL_ROTATION_COUNT))`

### Limitations
1. ❌ Not available in Claude CLI (used by Rex, Cleo, Tess)
2. ❌ Not available in Codex, Cursor, OpenCode CLIs
3. ❌ Configuration is hardcoded in templates, not in `cto-config.json`
4. ❌ No enable/disable toggle
5. ❌ No centralized configuration management

## Proposed Solution

### Design Principles
1. **Single Source of Truth**: All model rotation config lives in `cto-config.json`
2. **Zero Hardcoding**: No model lists or rotation logic hardcoded in templates
3. **Opt-In**: Feature must be explicitly enabled per agent
4. **Universal Support**: Works across ALL CLIs and ALL agents
5. **Backward Compatible**: Existing configs continue to work without changes

### Configuration Structure

#### cto-config.json Schema Addition

```json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "codex",
      "model": "gpt-5-codex",
      "modelRotation": {
        "enabled": true,
        "models": [
          "gpt-5-codex",
          "gpt-5-codex-preview",
          "claude-sonnet-4-20250514"
        ]
      }
    },
    "cleo": {
      "githubApp": "5DLabs-Cleo",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514",
      "modelRotation": {
        "enabled": true,
        "models": [
          "claude-sonnet-4-20250514",
          "claude-opus-4-1-20250805",
          "claude-haiku-4-20250514"
        ]
      }
    }
  }
}
```

#### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `modelRotation.enabled` | boolean | No | Enable model rotation (default: `false`) |
| `modelRotation.models` | string[] | Yes* | Array of model identifiers to rotate through |

*Required only if `enabled: true`

### Implementation Requirements

#### 1. Controller Changes
**File**: `controller/src/coderun/template_renderer.rs` (or equivalent)

**Requirements**:
- Read `modelRotation` config from `cto-config.json` for the specified agent
- Pass to template context as `model_rotation_enabled` and `model_rotation_models`
- Handle missing/invalid config gracefully (disable feature)

#### 2. Template Changes

**Files to Update**:
- ✅ `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs` (already has logic, needs config integration)
- 🔧 `infra/charts/controller/agent-templates/code/claude/container-rex.sh.hbs` (add rotation)
- 🔧 `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs` (add rotation)
- 🔧 `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs` (add rotation)
- 🔧 `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs` (add rotation)

**Common Pattern** (Handlebars):
```handlebars
{{#if model_rotation_enabled}}
# Model rotation configuration
MODEL_ROTATION=(
{{#each model_rotation_models}}
  "{{this}}"
{{/each}}
)
MODEL_ROTATION_COUNT=${#MODEL_ROTATION[@]}

if [ $MODEL_ROTATION_COUNT -gt 0 ]; then
  echo "🎯 Model rotation enabled (${MODEL_ROTATION_COUNT} models): ${MODEL_ROTATION[*]}"
  
  # Calculate which model to use for this attempt
  MODEL_INDEX=$(((ATTEMPT - 1) % MODEL_ROTATION_COUNT))
  CURRENT_MODEL="${MODEL_ROTATION[$MODEL_INDEX]}"
  echo "🎯 Attempt $ATTEMPT will use model: $CURRENT_MODEL"
else
  CURRENT_MODEL="{{model}}"
fi
{{else}}
# Model rotation disabled - using default model
CURRENT_MODEL="{{model}}"
{{/if}}
```

**CLI-Specific Integration**:

| CLI | Model Flag | Implementation |
|-----|------------|----------------|
| Factory | `--model` | Already implemented, needs config integration |
| Claude | `--model` (via settings) | Add model override in retry loop |
| Codex | `--model` | Pass to `codex` CLI |
| Cursor | `--model` | Pass to `cursor` CLI |
| OpenCode | `--model` | Pass to `opencode` CLI |

#### 3. Validation Rules

**Controller Validation**:
1. If `enabled: true`, `models` array must be non-empty
2. Each model in `models` must be a valid model identifier
3. If `enabled: false` or missing, ignore `models` array
4. Log warnings for invalid configurations

**Runtime Validation**:
1. Verify model is available for the CLI
2. Fall back to default model if rotation fails
3. Log model selection for each attempt

### Behavior Specification

#### Normal Operation
```
Attempt 1: Use models[0]
Attempt 2: Use models[1]
Attempt 3: Use models[2]
Attempt 4: Use models[0] (cycle back)
...
```

#### Edge Cases

| Scenario | Behavior |
|----------|----------|
| `enabled: false` | Always use default `model` field |
| `models: []` | Fall back to default `model` field |
| Single model in array | Effectively same as disabled |
| Invalid model identifier | Skip invalid, use next in rotation |
| All models invalid | Fall back to default `model` field |

### Example Configurations

#### Rex with Codex Model Rotation
```json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "codex",
      "model": "gpt-5-codex",
      "modelRotation": {
        "enabled": true,
        "models": [
          "gpt-5-codex",
          "gpt-5-codex-preview",
          "claude-sonnet-4-20250514"
        ]
      }
    }
  }
}
```

**Expected Behavior**:
- Attempt 1: `gpt-5-codex`
- Attempt 2: `gpt-5-codex-preview`
- Attempt 3: `claude-sonnet-4-20250514`
- Attempt 4: `gpt-5-codex` (cycle)

#### Cleo without Rotation (Disabled)
```json
{
  "agents": {
    "cleo": {
      "githubApp": "5DLabs-Cleo",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514",
      "modelRotation": {
        "enabled": false
      }
    }
  }
}
```

**Expected Behavior**:
- All attempts: `claude-sonnet-4-20250514`

#### Tess with Single Model (Effectively Disabled)
```json
{
  "agents": {
    "tess": {
      "githubApp": "5DLabs-Tess",
      "cli": "claude",
      "model": "claude-opus-4-1-20250805",
      "modelRotation": {
        "enabled": true,
        "models": ["claude-opus-4-1-20250805"]
      }
    }
  }
}
```

**Expected Behavior**:
- All attempts: `claude-opus-4-1-20250805` (no actual rotation)

## Implementation Checklist

### Phase 1: Configuration & Controller
- [ ] Update `cto-config.json` schema documentation
- [ ] Add `modelRotation` config to controller config parser
- [ ] Pass `model_rotation_enabled` and `model_rotation_models` to templates
- [ ] Add validation for modelRotation config
- [ ] Test controller with valid/invalid configs

### Phase 2: Template Updates (Priority Order)
- [ ] **Claude CLI** (Rex, Cleo, Tess) - highest priority
- [ ] **Factory CLI** (remove hardcoding, use config)
- [ ] **Codex CLI** (Rex, Blaze, Cipher)
- [ ] **Cursor CLI**
- [ ] **OpenCode CLI**

### Phase 3: Testing & Validation
- [ ] Test each CLI with rotation enabled
- [ ] Test each CLI with rotation disabled
- [ ] Test edge cases (empty array, single model, invalid models)
- [ ] Verify backward compatibility (existing configs)
- [ ] Load test with multiple agents rotating models

### Phase 4: Documentation & Rollout
- [ ] Update cto-config.json documentation
- [ ] Document model rotation feature in README
- [ ] Add examples to documentation
- [ ] Roll out to production agents incrementally

## Benefits

1. **Diversity**: Different models may succeed where others fail
2. **Resilience**: Reduces impact of model-specific bugs or rate limits
3. **Cost Optimization**: Can mix expensive/cheap models strategically
4. **Experimentation**: Easy to test new models in production
5. **Flexibility**: Per-agent configuration allows customization

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Invalid model IDs | Failed attempts | Validation + fallback to default |
| Cross-CLI compatibility | Models may not work with all CLIs | CLI-specific validation |
| Increased complexity | Harder to debug | Extensive logging of model selection |
| Cost unpredictability | Budget overruns | Monitor usage per model |

## Monitoring & Observability

### Metrics to Track
- Model usage count per agent
- Success rate per model
- Cost per model per agent
- Average attempts before success (with/without rotation)

### Logging Requirements
```
🎯 Model rotation enabled (3 models): gpt-5-codex, claude-sonnet-4, opus-4
🎯 Attempt 1 will use model: gpt-5-codex
🎯 Attempt 2 will use model: claude-sonnet-4
```

## Future Enhancements

1. **Smart Rotation**: Use model success history to prioritize better models
2. **Cost-Aware Rotation**: Prefer cheaper models for earlier attempts
3. **Capability-Based Selection**: Choose model based on task requirements
4. **Per-Task Override**: Allow task-specific model rotation config
5. **Dynamic Model Pool**: Fetch available models from provider APIs

## References

- Existing Factory implementation: `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs`
- CTO Config: `cto-config.json`
- Controller template renderer: `controller/src/coderun/template_renderer.rs`

---

**Document Status**: Ready for Review  
**Next Steps**: Review → Approval → Implementation
