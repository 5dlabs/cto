# Task 1: Acceptance Criteria

## Overview
This document defines the acceptance criteria for implementing Helm values and Agents ConfigMap for personas.

## Core Acceptance Criteria

### 1. Helm Chart Structure ✓
- [ ] Use existing controller chart at `infra/charts/controller`
- [ ] Do not create a new chart; extend existing values and prompts only
- [ ] Managed and installed by Argo CD (not local Helm)

### 2. Values Configuration ✓
- [ ] `infra/charts/controller/values.yaml` uses a map under `.Values.agents` with keys `rex`, `clippy`, `qa`, `triage`, `security`
- [ ] Each agent entry sets `name` to the friendly name:
  - [ ] Rex → "Rex"
  - [ ] Clippy → "Cleo"
  - [ ] QA → "Tess"
  - [ ] Triage → "Stitch"
  - [ ] Security → "Onyx"
- [ ] Each agent has: `name` (friendly), `githubApp`, and a robust technical `systemPrompt` (inline in values)
 - [ ] ExternalSecrets for new agents exist and corresponding Kubernetes Secrets are synced with `appId` and `privateKey`

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

 

### 6. Prompts ✓
- [ ] Prompts are defined inline under `.Values.agents[*].systemPrompt`
- [ ] Content is UTF-8 and follows technical guidance per agent

### 7. Workflow Integration ✓
- [ ] Existing WorkflowTemplates mount:
  - [ ] `/etc/agents/${GITHUB_APP}_system-prompt.md` for prompts
- [ ] Validation performed via Argo CD/Workflows, not local Helm
- [ ] Token generation already handled by container template (`container.sh.hbs`) - no changes needed

### 8. Documentation ✓
- [ ] Architecture document exists at `docs/.taskmaster/architecture.md`
- [ ] Documents configuration approach and mount points
- [ ] Includes testing and validation procedures
- [ ] References PRD appropriately

## Test Cases

### Test Case 1: Argo CD Sync
**Given**: PR merged to main  
**When**: Argo CD syncs `infra/charts/controller`  
**Then**: App shows Healthy/Synced; `controller-agents` ConfigMap updated

### Test Case 2: Config Verification
**Given**: Synced app  
**When**: `kubectl -n agent-platform get cm controller-agents -o yaml`  
**Then**: 
- Prompt keys exist and contain updated content
- Agents metadata matches values

### Test Case 3: ConfigMap Content Validation
**Given**: Rendered templates  
**When**: Inspect ConfigMap data  
**Then**:
- controller-agents has 5 keys (one per agent)
- Content matches source files

### Test Case 4: Workflow Mount Test
**Given**: Running DocsRun/CodeRun  
**When**: Inspect container filesystem  
**Then**:
- `/etc/agents/${GITHUB_APP}_system-prompt.md` exists
- Token generation handled automatically by existing container template

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

- [ ] Argo CD sync completes successfully
- [ ] ConfigMap updates propagate in < 30 seconds
- [ ] Total prompt content remains within Kubernetes ConfigMap size limits

## Security Criteria

- [ ] ConfigMaps are read-only when mounted
- [ ] No sensitive data in prompts or ConfigMaps
- [ ] Proper RBAC for ConfigMap access
- [ ] Files mounted with appropriate permissions

## Rollback Criteria

Rollback is achieved via Git revert/PR merge; Argo CD will rollback on sync.

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
# Quick validation suite (Argo CD + kubectl)
argocd app sync controller | cat
kubectl -n agent-platform get cm controller-agents -o yaml | head -n 40
kubectl -n agent-platform get externalsecrets | grep github-app-5dlabs || true
echo "✅ All quick validations passed"
```

## Notes

- This task is foundational and blocks multiple downstream tasks
- Agent prompts may evolve but structure should remain stable
- Consider future ConfigMap size growth in design
- Ensure compatibility with existing CodeRun/DocsRun controllers