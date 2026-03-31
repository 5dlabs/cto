## Decision Points

- MinIO strategy: Must be resolved BEFORE implementation begins. Option A: Reuse existing GitLab MinIO instance (lower complexity, shared blast radius). Option B: Provision a dedicated MinIO instance in hermes-minio namespace (isolation, independent scaling, ~50Gi PV required). The capacity gate in subtask 1007 produces data to inform this decision, but the architectural choice and corresponding values (minio.dedicated, minio.endpoint) must be locked before subtask 1008 proceeds. Recommend running the capacity check script manually first, then recording the decision in a lightweight ADR.
- Redis deployment method: Option A: bitnami/redis Helm subchart (well-documented, standard). Option B: Redis Operator CR if one is already installed cluster-wide (operator-managed lifecycle, but API version coupling). Check `kubectl get crd | grep redis` before deciding. This affects Chart.yaml dependencies and template structure.
- CNPG backup target: Option A: Configure CNPG automated backups to MinIO artifact buckets now (couples Postgres backup lifecycle to artifact storage). Option B: Defer backups until a dedicated backup bucket strategy is defined (simpler v1, backup gap). Recommend Option B for v1 unless PRD mandates backup SLA.
- Secret management approach: Native Kubernetes Secrets are specified with no auto-rotation. Option A: Accept native Secrets for v1, document limitation, defer ESO. Option B: Integrate External Secrets Operator now (adds operator dependency, more complex). Recommend Option A for v1 with explicit documentation of the limitation.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm