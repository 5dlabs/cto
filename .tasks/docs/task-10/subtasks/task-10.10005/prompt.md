Implement subtask 10005: Integrate security scanning and validate compliance posture

## Objective
Run security scanning tools against the hardened cluster to validate RBAC policies, secret management, and audit logging configurations meet security baselines.

## Steps
1. Run `kubectl-bench-security` or `kube-bench` against CIS Kubernetes Benchmark to validate API server audit logging, RBAC configuration, and secret encryption at rest.
2. Run a RBAC audit tool (e.g., `kubectl-who-can`, `rakkess`, or `rbac-police`) to generate a report of all permissions per ServiceAccount and flag any overly-permissive bindings.
3. Scan for secrets in plain text in ConfigMaps, environment variables, or pod specs using a tool like `kubeaudit` or `trivy` config scanning.
4. Validate that no pods run as root or with privileged security contexts unless explicitly required and documented.
5. Generate a compliance summary document listing: all RBAC roles and their scope, secret rotation schedule and mechanism, audit log coverage and retention, scan results with remediation status.
6. Store compliance documentation under `docs/security/`.

## Validation
All CIS benchmark checks related to RBAC, audit logging, and secret management pass or have documented exceptions. RBAC audit report shows no ServiceAccount with cluster-admin or wildcard permissions unless explicitly justified. Secret scan finds zero plaintext credentials in non-Secret resources. Compliance document is complete and reviewed.