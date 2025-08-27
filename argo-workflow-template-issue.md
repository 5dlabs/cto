# Argo Workflow Template Issue

## Problem Description

When attempting to submit CTO play workflows using the `mcp_cto_play` tool, the workflow submission fails with an Argo template configuration error.

## Error Details

```
MCP error -32600: Failed to submit play workflow: Argo command failed: Error: Failed to submit workflow: rpc error: code = InvalidArgument desc = templates.main.tasks.implementation-cycle templates.implementation-cycle.steps[1].wait-for-pr templates.check-or-wait-for-pr: failed to resolve {{workflow.parameters.current-stage}}
```

## Root Cause

The Argo workflow template is missing a required parameter `current-stage` that is referenced in the template but not provided when the workflow is submitted.

### Specific Issue

The template contains a step that tries to resolve `{{workflow.parameters.current-stage}}`, but this parameter is not defined in the workflow parameters or not being passed during submission.

## Affected Components

- **Tool**: `mcp_cto_play` (CTO play workflow submission)
- **Platform**: Argo Workflows
- **Namespace**: `agent-platform`
- **Template**: `play-workflow-template`

## Steps to Reproduce

1. Start the MCP server with proper database configuration
2. Attempt to submit a CTO play workflow:
   ```bash
   mcp_cto_play --task_id 11
   ```
3. Observe the Argo template error

## Current Status

- ✅ **MCP Server**: Running successfully with database connectivity
- ✅ **Database**: Live vector database accessible and working
- ✅ **Database Tests**: Passing locally with live database
- ❌ **CTO Play Workflows**: Failing due to template configuration issue

## Impact

- Cannot submit new CTO play workflows for task implementation
- Previous workflows may still be running (if they were submitted before this issue)
- Development workflow is blocked for new task submissions

## Required Fix

The Argo workflow template needs to be updated to either:

1. **Add the missing parameter**: Include `current-stage` in the workflow parameters
2. **Remove the reference**: Remove the `{{workflow.parameters.current-stage}}` reference if not needed
3. **Provide default value**: Set a default value for the parameter

## Workaround

Until the template is fixed, development can continue using:
- Local development and testing
- Direct code implementation without the CTO play workflow
- Manual task execution

## Technical Context

- **Database**: PostgreSQL with pgvector extension
- **Connection**: `vector-postgres.databases.svc.cluster.local:5432`
- **User**: `vector_user` with proper permissions
- **Tables**: `documents`, `document_sources`, `migration_history`
- **Document Count**: ~4,382 documents in live database

## Related Files

- `docs/requirements.yaml` - Environment configuration
- `db/tests/crate_operations.rs` - Database integration tests
- `.github/workflows/build-server.yml` - CI/CD configuration
- `mcp/src/bin/http_server.rs` - MCP server implementation

## Next Steps

1. **Immediate**: Report template configuration issue to platform administrators
2. **Short-term**: Continue development using local tools and direct implementation
3. **Long-term**: Resume CTO play workflow usage once template is fixed

---

**Date**: August 27, 2025  
**Environment**: Local development with live database connectivity  
**Status**: Blocked on Argo template configuration
