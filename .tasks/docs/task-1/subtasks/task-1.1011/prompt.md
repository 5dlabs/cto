Implement subtask 1011: Create PodDisruptionBudgets for Production Backing Services

## Objective
Create PodDisruptionBudget resources for Postgres, Redis, and NATS in the production namespace only (gated by environment=production) with minAvailable: 1.

## Steps
Step-by-step:
1. Create `templates/pdb-postgres.yaml` gated with `{{ if eq .Values.environment "production" }}`:
   - `spec.minAvailable: 1`
   - `spec.selector.matchLabels`: match CNPG pod labels (`cnpg.io/cluster: hermes-pg`)
2. Create `templates/pdb-redis.yaml` (production-only):
   - `spec.minAvailable: 1`
   - `spec.selector.matchLabels`: match Redis pod labels (`app.kubernetes.io/name: hermes-redis, app.kubernetes.io/component: master`)
3. Create `templates/pdb-nats.yaml` (production-only):
   - `spec.minAvailable: 1`
   - `spec.selector.matchLabels`: match NATS pod labels (`app.kubernetes.io/name: hermes-nats`)
4. All PDBs namespaced to `{{ .Values.namespace }}` with standard labels.
5. Verify: `helm template` with staging values renders zero PDBs; with production values renders exactly 3.

## Validation
`kubectl get pdb -n hermes-production` lists 3 PDBs for postgres, redis, and nats, each with minAvailable=1. `kubectl get pdb -n hermes-staging` returns no resources. `helm template` with staging values contains no PDB manifests; with production values contains exactly 3.