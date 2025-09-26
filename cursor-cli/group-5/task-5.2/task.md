# Task 5.2 – Staging Rollout

## Dependencies
- Task 5.1 (validation) complete.

## Parallelization Guidance
- Post-validation; coordinates with ops for staging window.

## Task Prompt
Deploy the updated controller with Cursor support to staging (agent-platform namespace) and verify end-to-end behaviour.

Steps:
1. Apply Helm chart with Cursor values enabled (point to staging `values-staging.yaml`).
2. Trigger a sample CodeRun targeting Cursor (e.g., replicate Codex test but using Cursor CLI) once credits available:
   - Confirm job pod pulls Cursor image, mounts templates, and executes `cursor-agent` with expected flags.
3. Validate MCP/toolman connectivity by inspecting logs for `codex_core::mcp_connection_manager` equivalent (Cursor CLI logs) to ensure tools aggregate.
4. Observe PR creation fallback path if agent fails to open PR (should use new branch verification logic).
5. Record findings in `Cursor CLI/group-5/task-5.2/staging-report.md` including pod logs, success/failure states, and any manual interventions.

## Acceptance Criteria
- Cursor job reaches completion (success state) or fails only due to intentional resource limits (e.g., missing credits) with expected error messages.
- Toolman connection attempts logged; failures documented with remediation steps.
- No regressions observed in existing Codex/Claude workloads post-deploy.
- Rollback plan documented in case issues arise (link to Helm command to revert).

## Implementation Notes / References
- Use `kubectl logs code-agent-platform-...` to capture runtime output; focus on approval prompts, PR creation, MCP timeouts.
- If credits unavailable, at minimum confirm container starts and logs expected auth error; note follow-up action once credits restored.
