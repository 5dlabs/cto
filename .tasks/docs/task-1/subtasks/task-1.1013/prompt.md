Implement subtask 1013: Write README Documentation and Verify Helm Idempotency

## Objective
Complete the NOTES.txt with final endpoint values, write comprehensive README.md documenting chart usage, key inventory, capacity gate process, decision records, known limitations, and verify that helm upgrade --install is idempotent.

## Steps
Step-by-step:
1. Finalize `templates/NOTES.txt` with all sections populated:
   - Namespace, Environment, all ConfigMap keys with descriptions, all Secret key names (not values), MinIO bucket info, usage pattern (`envFrom` example for both ConfigMap and Secret), known limitations (no auto-rotation), testing command (`helm test`).
2. Create `charts/hermes-infra/README.md` with:
   - Architecture overview (ASCII diagram showing namespaces, backing services, MinIO, ConfigMap/Secret relationships).
   - Prerequisites: CNPG operator installed, Cilium CNI, MinIO access, Helm 3.x.
   - Pre-installation steps: run MinIO capacity check script, resolve decision points, set values.
   - Installation: `helm upgrade --install hermes-infra charts/hermes-infra -f values-staging.yaml -n hermes-staging --create-namespace`
   - Full key inventory table for ConfigMap (7 keys) and Secret (5 keys) with descriptions.
   - Capacity gate process documentation with decision criteria.
   - Decision records: MinIO strategy, Redis method, CNPG backup, secret management.
   - Rollout risks and mitigations.
   - Testing: `helm test hermes-infra -n hermes-staging`
   - Known limitations: native Secrets, single-replica staging, ESO migration path.
3. Idempotency verification:
   a. Run `helm upgrade --install hermes-infra charts/hermes-infra -f values-staging.yaml -n hermes-staging` twice.
   b. Second run produces no errors.
   c. If `helm-diff` plugin is available, verify no resource drift.
   d. Document idempotency as a verified property in README.

## Validation
`helm install` prints NOTES.txt with all expected sections and correct values for the target environment. README.md exists at `charts/hermes-infra/README.md` and contains: installation instructions, key inventory table with all 12 keys, capacity gate section, decision records section, and testing section. `helm upgrade --install` succeeds on second run with no errors and no resource modifications. README word count > 500 (substantive documentation).