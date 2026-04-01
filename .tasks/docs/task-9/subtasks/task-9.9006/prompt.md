Implement subtask 9006: Configure PodDisruptionBudgets for PM server and frontend

## Objective
Create PDB manifests ensuring PM server has minAvailable=2 and frontend has minAvailable=1 during voluntary disruptions.

## Steps
1. Create `templates/pm-server-pdb.yaml`:
   - `apiVersion: policy/v1`, kind `PodDisruptionBudget`.
   - `spec.minAvailable: 2`.
   - `spec.selector.matchLabels` matching the PM server Deployment labels.
2. Create `templates/frontend-pdb.yaml`:
   - `apiVersion: policy/v1`, kind `PodDisruptionBudget`.
   - `spec.minAvailable: 1`.
   - `spec.selector.matchLabels` matching the frontend Deployment labels.
3. Parameterize minAvailable values via Helm values for flexibility.
4. Validate both rendered manifests with `helm template`.

## Validation
Run `helm template` and verify both PDB manifests are rendered with correct minAvailable values and label selectors. After deploy: `kubectl get pdb -n sigma1-prod` shows both PDBs with expected minAvailable.