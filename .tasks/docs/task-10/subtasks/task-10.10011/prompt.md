Implement subtask 10011: Create production readiness checklist document

## Objective
Create docs/hermes/production-readiness-checklist.md covering all security, reliability, and operational readiness items with evidence links and verification commands.

## Steps
1. Create `docs/hermes/production-readiness-checklist.md` with the following sections, each with a checkbox, description, and evidence/verification command:
   - [ ] Kubernetes RBAC policies applied (link to `kubectl auth can-i` verification)
   - [ ] Application RBAC claims enforced (link to test results)
   - [ ] All secrets rotated at least once (last-rotated annotations)
   - [ ] Audit logging verified in Loki (link to Loki query)
   - [ ] Pod security contexts applied (link to kubectl output)
   - [ ] Container images scanned (link to CI scan results)
   - [ ] Secret encryption at rest verified (link to documentation)
   - [ ] Network policies verified (from Task 9, link to verification)
   - [ ] TLS verified (from Task 9, link to curl output)
   - [ ] HA data services verified (from Task 9, link to failover test)
   - [ ] HPA autoscaling verified (from Task 9, link to load test)
   - [ ] E2E tests passing (from Task 7, link to CI run)
   - [ ] Rollback procedures documented (from Task 8, link to docs)
   - [ ] Backup and restore procedures tested
2. Include a 'How to use this checklist' section explaining the sign-off process.
3. Include a 'Known risks and mitigations' section for any accepted vulnerabilities or operational caveats.

## Validation
Verify the document exists at `docs/hermes/production-readiness-checklist.md`. Verify all checklist items have associated verification commands or evidence links. Verify all items from this task (10) and Task 9 are represented.