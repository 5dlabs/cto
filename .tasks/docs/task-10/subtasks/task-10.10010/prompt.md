Implement subtask 10010: Document HA scaling Helm values and conditional scope adjustment for D5

## Objective
Document the Helm values needed to scale PM server to 2+ replicas with PodDisruptionBudget for production. Also document the scope adjustments needed if D5 resolves to defer Tasks 6-9 (remove frontend ingress routes, ServiceAccount, resource limits, and NetworkPolicies).

## Steps
1. Create `docs/production/ha-scaling.md`.
2. Document Helm values for HA PM server:
   - `replicaCount: 2`
   - PodDisruptionBudget manifest: `minAvailable: 1`, selector matching PM server pods.
   - Note: for dev/validation, single replica is acceptable.
3. Include the PDB manifest in `manifests/production/pdb-pm-server.yaml` (but do not apply it in dev — annotate with `# Apply for production only`).
4. Create `docs/production/scope-adjustment-d5.md` documenting what to remove if D5 defers Tasks 6-9:
   - Remove frontend route from Cloudflare Tunnel CR (subtask 10001).
   - Remove sigma-1-frontend ServiceAccount and RBAC (subtask 10004).
   - Remove frontend resource limits from deployment (subtask 10005).
   - Remove frontend NetworkPolicy (subtask 10007).
   - Update parent task dependencies from [2,3,4,5,6,7,8,9] to [2,3,4,5].
5. List all affected manifest files and specific YAML blocks to remove.

## Validation
docs/production/ha-scaling.md exists and contains Helm values for replicaCount=2 and a PDB manifest with minAvailable=1. docs/production/scope-adjustment-d5.md exists and lists all frontend-related resources to remove with specific file paths and YAML references. PDB manifest file exists at manifests/production/pdb-pm-server.yaml.