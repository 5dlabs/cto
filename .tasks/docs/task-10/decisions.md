## Decision Points

- D7 resolution dependency: If D7 resolves to JWT/RBAC instead of Cloudflare Access, subtask 10002 (Cloudflare Access configuration) must be replaced with application-level auth middleware. This fundamentally changes the auth architecture for the production deployment.
- D5 resolution dependency: If D5 defers Tasks 6-9 (frontend), all frontend-related resources (ingress routes to /, frontend ServiceAccount, frontend resource limits, frontend NetworkPolicy) must be removed from scope. This affects subtasks 10001, 10003, 10004, and 10005.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm