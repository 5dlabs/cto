Implement subtask 9010: Create PodDisruptionBudgets for all application services

## Objective
Create PodDisruptionBudget resources for both the backend and frontend application services with minAvailable: 1 to ensure availability during voluntary disruptions.

## Steps
1. Create PDB for backend service: `minAvailable: 1`, selector matching backend pod labels.
2. Create PDB for frontend service: `minAvailable: 1`, selector matching frontend pod labels.
3. Verify PDBs do not conflict with HPA minReplicas (minAvailable should be less than minReplicas).
4. Note: PDBs for data services (CNPG, Redis, NATS) are created in their respective subtasks (9002, 9003, 9004).

## Validation
Verify `kubectl get pdb -n hermes-production` shows PDBs for frontend and backend with `ALLOWED DISRUPTIONS >= 1`. Attempt `kubectl drain` on a node hosting a backend pod and verify the drain respects the PDB (pod is not evicted if it would violate minAvailable).