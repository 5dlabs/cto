# Task 1: Acceptance Criteria

## Overview
This document defines the acceptance criteria for implementing Helm values and Agents ConfigMap for personas and project-wide MCP tools configuration.

## Core Acceptance Criteria

### 1. Helm Chart Structure ✓
- [ ] Chart exists at `charts/platform` with proper directory structure
- [ ] Contains valid Chart.yaml with appropriate metadata
- [ ] Follows Helm 3.x standards and best practices

### 2. Values Configuration ✓
- [ ] `values.yaml` contains `agents` array with 5 agent definitions:
  - [ ] rex (rex-agent)
  - [ ] clippy (clippy-agent)
  - [ ] qa (qa-agent)
  - [ ] triage (triage-agent)
  - [ ] security (security-agent)
- [ ] Each agent has: name, githubApp, systemPromptFile
- [ ] `mcp.requirementsFile` points to `requirements.yaml`
- [ ] Values are overrideable via `-f` or `--set`

### 3. Schema Validation ✓
- [ ] `values.schema.json` exists and validates structure
- [ ] Schema requires all mandatory fields
- [ ] `helm lint` passes without errors or warnings
- [ ] Invalid values cause helm to fail fast

### 4. Helper Templates ✓
- [ ] `_helpers.tpl` contains three helpers:
  - [ ] `platform.renderPrompt` - renders prompt files
  - [ ] `platform.agentVolumes` - defines ConfigMap volumes
  - [ ] `platform.agentVolumeMounts` - defines mount points
- [ ] Helpers use chart-scoped naming (platform.*)
- [ ] Helpers are reusable across templates

### 5. ConfigMap Generation ✓

#### Controller Agents ConfigMap
- [ ] Template generates `controller-agents` ConfigMap
- [ ] Contains all 5 agent prompts as separate keys
- [ ] Keys match systemPromptFile names exactly
- [ ] Preserves formatting and newlines from source files
- [ ] Labels follow Kubernetes conventions

#### MCP Requirements ConfigMap
- [ ] Template generates `mcp-requirements` ConfigMap
- [ ] Contains `requirements.yaml` key with tools configuration
- [ ] Properly indented and formatted YAML content
- [ ] Labels follow Kubernetes conventions

### 6. File Packaging ✓
- [ ] All agent prompt files exist in `files/agents/`:
  - [ ] rex_system-prompt.md
  - [ ] clippy_system-prompt.md
  - [ ] qa_system-prompt.md
  - [ ] triage_system-prompt.md
  - [ ] security_system-prompt.md
- [ ] `files/requirements.yaml` exists with MCP tools config
- [ ] Files are UTF-8 encoded
- [ ] Total size < 900KB

### 7. WorkflowTemplate Integration ✓
- [ ] Smoke test template `agent-mount-smoke` exists
- [ ] Uses helper functions for volumes and mounts
- [ ] Mounts at correct paths:
  - [ ] `/etc/agents` for prompts
  - [ ] `/work/requirements.yaml` for MCP config
- [ ] Outputs "OK" when successful

### 8. Documentation ✓
- [ ] Architecture document exists at `docs/.taskmaster/architecture.md`
- [ ] Documents configuration approach and mount points
- [ ] Includes testing and validation procedures
- [ ] References PRD appropriately

## Test Cases

### Test Case 1: Helm Linting
**Given**: Complete chart structure  
**When**: Run `helm lint charts/platform`  
**Then**: Command succeeds with no errors or warnings

### Test Case 2: Template Rendering
**Given**: Valid values.yaml  
**When**: Run `helm template charts/platform`  
**Then**: 
- Both ConfigMaps are generated
- No template errors occur
- Output contains expected resources

### Test Case 3: ConfigMap Content Validation
**Given**: Rendered templates  
**When**: Inspect ConfigMap data  
**Then**:
- controller-agents has 5 keys (one per agent)
- mcp-requirements has requirements.yaml key
- Content matches source files

### Test Case 4: Deployment Test
**Given**: Kubernetes cluster with dev namespace  
**When**: Run `helm upgrade --install platform charts/platform -n dev`  
**Then**:
- Installation succeeds
- ConfigMaps created in cluster
- No pod errors or crashes

### Test Case 5: Mount Point Verification
**Given**: Deployed chart  
**When**: Run smoke test workflow  
**Then**:
- Workflow completes successfully
- Logs show "OK" output
- Files accessible at mount points

### Test Case 6: Size Constraints
**Given**: All prompt files  
**When**: Calculate total size  
**Then**: Total < 900KB (well under 1MiB limit)

### Test Case 7: Value Overrides
**Given**: Custom values file  
**When**: Deploy with `-f custom-values.yaml`  
**Then**: Custom values take precedence

### Test Case 8: Missing File Handling
**Given**: Missing prompt file  
**When**: Run `helm template`  
**Then**: Command fails with clear error message

### Test Case 9: Invalid Schema
**Given**: Invalid values structure  
**When**: Run `helm lint`  
**Then**: Schema validation fails with specific error

### Test Case 10: UTF-8 Encoding
**Given**: All files in chart  
**When**: Check encoding with `file -I`  
**Then**: All files report UTF-8 charset

## Performance Criteria

- [ ] Helm template renders in < 2 seconds
- [ ] ConfigMap updates propagate in < 30 seconds
- [ ] Smoke test completes in < 10 seconds
- [ ] Total ConfigMap size < 900KB

## Security Criteria

- [ ] ConfigMaps are read-only when mounted
- [ ] No sensitive data in prompts or requirements
- [ ] Proper RBAC for ConfigMap access
- [ ] Files mounted with appropriate permissions

## Rollback Criteria

The implementation must support rollback if:
- [ ] Helm upgrade can revert to previous version
- [ ] ConfigMap changes are tracked in version control
- [ ] No destructive operations on existing resources
- [ ] Backward compatibility maintained

## Definition of Done

- [x] All acceptance criteria met
- [x] All test cases passing
- [x] Documentation complete and accurate
- [x] Code reviewed and approved
- [x] Deployed successfully to dev environment
- [x] Smoke tests passing consistently
- [x] No critical issues or blockers
- [x] Performance and security criteria satisfied

## Validation Commands

```bash
# Quick validation suite
helm lint charts/platform && \
helm template charts/platform > /tmp/rendered.yaml && \
yq '. | select(.kind=="ConfigMap") | .metadata.name' /tmp/rendered.yaml | grep -E "(controller-agents|mcp-requirements)" && \
find charts/platform/files -type f -printf '%s\n' | awk '{s+=$1} END {print s" bytes total (must be < 900000)"}' && \
echo "✅ All quick validations passed"
```

## Notes

- This task is foundational and blocks multiple downstream tasks
- Agent prompts may evolve but structure should remain stable
- Consider future ConfigMap size growth in design
- Ensure compatibility with existing CodeRun/DocsRun controllers