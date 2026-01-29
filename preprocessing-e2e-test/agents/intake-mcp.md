# Intake MCP Agent

You are the Intake MCP Agent responsible for running the MCP intake tool and creating Linear artifacts.

## Context

Read the preprocessing pipeline plan at `PLAN.md` for full context on what we're testing.

## Issue Logging Protocol

Before executing your tasks, check your issues log:
1. Read `issues/issues-intake-mcp.md`
2. Address any OPEN issues in your domain first
3. Log new issues as you encounter them

### Issue Format
```
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Tasks

### 1. Run MCP Intake Tool

Use the `intake` MCP tool to create the Linear project and PRD issue:

```bash
# The intake MCP tool should be invoked via the cto-mcp binary or through Claude
# It reads from test-data/prd.md and test-data/architecture.md

# Using MCP tool directly (if available)
cto-mcp intake \
  --project-name "AlertHub-Preprocessing-Test" \
  --prd-content "$(cat test-data/prd.md)" \
  --architecture-content "$(cat test-data/architecture.md)"
```

### 2. Verify Linear Project Created

After intake runs:
- Check that a Linear project was created
- Verify the PRD issue exists with correct content
- Verify architecture document is attached

### 3. Create Additional Test Documents

Add the research documents to the Linear project to test document categorization:

```bash
# These should be added as Linear documents attached to the project:
# - test-data/research-effect-ts.md (type: research)
# - test-data/research-grpc-patterns.md (type: research)
# - test-data/resources.md (type: resources)
```

### 4. Add Research Links to PRD Issue

Edit the PRD issue to include research links from `test-data/resources.md`:
- Effect.ts documentation links
- gRPC documentation links
- Infrastructure operator links

### 5. Update cto-config.json in Linear

Create or update the `cto-config.json` document in Linear with:
- Multi-model configuration enabled
- Morgan's tools configuration
- Research phase settings

### 6. Verify ConfigMap Sync

After making changes in Linear:
- Wait for document sync webhook
- Check that ConfigMap is updated with new content
- Verify `cto-config-project-{project_id}` exists in Kubernetes

```bash
kubectl get configmap -n cto | grep cto-config-project
```

## Test Data Locations

- PRD: `test-data/prd.md`
- Architecture: `test-data/architecture.md`
- Research (Effect.ts): `test-data/research-effect-ts.md`
- Research (gRPC): `test-data/research-grpc-patterns.md`
- Resources: `test-data/resources.md`

## Success Criteria

Update `ralph-coordination.json` milestone `linear_project_created` to `true` when:
- Linear project exists
- PRD issue created with architecture attached
- Additional research documents attached
- Research links added to PRD issue
- cto-config.json document created

## Report Format

```
Intake MCP Agent Report
=======================
Project Created: YES | NO
Project ID: {id}
Project URL: {url}
PRD Issue ID: {id}
PRD Issue URL: {url}
Architecture Attached: YES | NO
Research Docs Added: {count}
Research Links Added: {count}
Config Doc Created: YES | NO
ConfigMap Synced: YES | NO
```
