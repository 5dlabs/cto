## Acceptance Criteria

- [ ] 1. RBAC enforcement: The backend ServiceAccount can read `hermes-infra-endpoints` ConfigMap but CANNOT list Secrets in other namespaces — verified by `kubectl auth can-i` with the SA token.
- [ ] 2. Application RBAC: A user with only `hermes:read` claim receives 403 when calling `POST /api/hermes/deliberations` and 200 when calling `GET /api/hermes/deliberations`.
- [ ] 3. Secret rotation: After rotating PostgreSQL credentials via the rotation mechanism, the Hermes backend service continues to serve requests within 60 seconds (rolling restart completes, no 500 errors during rotation window).
- [ ] 4. Audit logging: Create a deliberation via the API; within 30 seconds, query Loki with `{app="hermes", audit="true"} | json | audit_action="create_deliberation"` and verify the log entry contains the correct `audit_actor` (user ID) and `audit_resource` (deliberation ID).
- [ ] 5. Pod security: `kubectl get pod -n hermes-production -o jsonpath='{.items[0].spec.containers[0].securityContext}'` shows `runAsNonRoot: true` and `allowPrivilegeEscalation: false` for all Hermes pods.
- [ ] 6. Production readiness: All items in `docs/hermes/production-readiness-checklist.md` are marked complete with evidence links (Loki queries, kubectl outputs, or CI run URLs).
- [ ] 7. Claim taxonomy: The `hermes:admin` claim gates access to `POST /api/hermes/admin/migrate-artifacts` — verified by 403 without claim and 202 with claim.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.