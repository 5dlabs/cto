# Task 5.3 – Production Enablement

## Dependencies
- Successful staging rollout (Task 5.2) with sign-off.

## Parallelization Guidance
- Final step; do not execute until all prior groups completed.

## Task Prompt
Promote Cursor CLI support to production and update operational records.

Actions:
1. Merge Helm values enabling Cursor into production overlay (`infra/gitops/environments/prod/**`).
2. Coordinate with ops to schedule deployment; monitor rollout via ArgoCD/GitOps pipelines.
3. Perform smoke test (once credits available) using a lightweight Cursor task; verify logs and PR creation.
4. Update runbooks/incident checklists to include Cursor-specific diagnostics (link to docs from Task 4).
5. Record go/no-go decision, deployment time, and validation results in `Cursor CLI/group-5/task-5.3/prod-rollout.md`.

## Acceptance Criteria
- Production controller pods running latest image with Cursor support.
- Post-deploy metrics/alerts show no regressions (document monitoring sources).
- Ops team confirms readiness; communication sent to stakeholders announcing availability.
- Any follow-up tasks (e.g., budget for Cursor credits) captured in tracking system.

## Implementation Notes / References
- Use same rollback commands as Codex rollout (documented previously) but ensure new Helm values can be toggled quickly.
- Keep eye on auto-PR fallback logs; confirm labels/branch handling works with real data.
