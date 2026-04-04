Implement subtask 10008: Document all production hardening changes in docs/production-hardening.md

## Objective
Create a comprehensive `docs/production-hardening.md` document covering all hardening measures applied, including rationale, configuration details, and verification steps for each.

## Steps
1. Create `docs/production-hardening.md` with sections for each hardening measure:
   - **CloudNative-PG HA**: 3-replica configuration, failover behavior, expected recovery time.
   - **Redis HA/Sentinel**: Sentinel topology, quorum settings, updated connection string.
   - **Ingress & TLS**: Ingress resource, cert-manager issuer, rate limiting and body size annotations, certificate renewal.
   - **Cilium Network Policies**: Deny-all default, allowlisted paths with exact ports, external egress rules.
   - **RBAC**: Per-SA roles, principle of least privilege, what was removed from the original permissive setup.
   - **Secret Rotation**: external-secrets refreshInterval, Reloader integration, rotation flow.
   - **Audit Logging**: Approach chosen, what events are captured, where logs are stored.
2. Each section should include:
   - Rationale (why this hardening is needed).
   - Configuration summary (key YAML snippets or references to manifest files).
   - Verification command(s) to confirm the measure is active.
3. Add a summary table at the top listing all measures and their status.
4. Commit the file alongside the infrastructure changes.

## Validation
Assert `docs/production-hardening.md` exists. Verify it contains at least 6 distinct hardening sections by grep-ing for section headers. Verify each section contains a rationale paragraph and at least one verification command. Word count should be at least 500 words to ensure adequate documentation depth.