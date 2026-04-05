## Acceptance Criteria

- [ ] 1. Network policy test: from a test pod in sigma1 namespace, verify connectivity to PostgreSQL (allowed) and verify connectivity to an unrelated namespace (denied). Use `kubectl exec` + `nc` connectivity checks. 2. RBAC test: verify service account for equipment-catalog cannot access secrets in finance namespace. 3. GDPR orchestrator test: run CLI with a test customer ID against all running services, verify each service returns 200/204, verify audit log row created in `audit.gdpr_deletions` table with all service confirmations. 4. CI pipeline test: submit a PR with intentional lint violation, verify pipeline fails. Submit clean PR, verify all stages pass and image is built. 5. ArgoCD sync test: update image tag in manifests, verify ArgoCD detects drift and syncs within 3 minutes. 6. Rollback test: deploy a service with failing health check, verify ArgoCD rolls back to previous healthy version within 5 minutes. 7. Alert test: scale equipment-catalog to 0 replicas, verify Prometheus alert fires within 2 minutes, then scale back. 8. HPA test: generate load on equipment-catalog, verify HPA scales from 2 to 3+ replicas when CPU exceeds 70%. 9. Pod security test: attempt to deploy a pod with `runAsRoot: true` in sigma1 namespace, verify it is rejected by PodSecurity admission.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.