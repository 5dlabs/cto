# TODO Markers Audit

Generated: 2026-01-19  
Total markers: 56

## Summary by Category

### Implementation Placeholders (Priority: Low)
These are stub implementations that return placeholder values or panic. They exist to satisfy the type system but need actual implementation.

| File | Count | Description |
|------|-------|-------------|
| experience/storage/postgres.rs | 14 | PostgreSQL storage methods (SQL queries) |
| experience/tools/handlers.rs | 3 | Tool handler implementations |
| experience/search/embedding.rs | 1 | Embedding search with vector DB |

### Feature Tracking (Priority: Medium)
TODOs tracking features that should be implemented.

| File | Line | Description |
|------|------|-------------|
| healer/ci/server.rs | 384-401 | Track active remediations and query task status |
| healer/play/orchestrator.rs | 652-654 | Session state attempt tracking |
| pm/handlers/agent_session.rs | 95, 231-232, 320 | Per-agent OAuth, workflow start, sidecar signal |
| healer/main.rs | 3724, 3951, 6344 | Log extraction, duration calc, issue comments |
| controller/tasks/workflow.rs | 105 | Workflow resumption |
| controller/tasks/label/*.rs | 64-369 | Cleanup, override storage, conflict detection |

### Configuration Items (Priority: Medium)

| File | Line | Description |
|------|------|-------------|
| controller/tasks/code/controller.rs | 1105 | Make timeout configurable |
| controller/tasks/code/controller.rs | 1165 | Get GitHub token from configuration |
| controller/installer/orchestrator.rs | 856 | Multi-cluster cluster_id |
| tools/server/http_server.rs | 3035 | Server availability pinging |

### Test Markers (No Action)
These are intentional TODOs in test files/starter code for demonstration purposes.

| File | Lines |
|------|-------|
| tests/agent-cli-matrix/e2e/starter-code/rust/* | 5, 25, 46, 80 |
| controller/tests/agent_cli_matrix_tests.rs | 316 |

### Cursor Adapter (Priority: Low)
Specific to Cursor CLI adapter implementation.

| File | Line | Description |
|------|------|-------------|
| controller/cli/adapters/cursor.rs | 5 | Surface clear TODOs |
| controller/cli/bridge.rs | 267 | Populate CLI configuration |
| controller/cli/adapter.rs | 443 | Finalize print/force flags |

## Recommendations

1. **PostgreSQL Storage**: Consider implementing actual SQL queries or removing if not needed
2. **Healer Tracking**: Low priority - monitoring features work without these
3. **Label Controller**: These are deferred architectural improvements
4. **Test Files**: Intentional, no action needed
5. **Cursor Adapter**: Track when Cursor adapter is prioritized

## Next Steps

These TODOs are tracked and do not block CI. They represent future work items that can be addressed as part of feature development rather than code quality cleanup.
