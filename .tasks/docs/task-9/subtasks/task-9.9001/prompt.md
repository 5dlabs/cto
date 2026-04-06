Implement subtask 9001: Scale all backend and frontend deployments to minimum 2 replicas with PodDisruptionBudgets

## Objective
Update every service Deployment to run at least 2 replicas with appropriate resource requests/limits and create PodDisruptionBudgets to guarantee availability during rolling updates and node maintenance.

## Steps
For each service deployment (backend services and frontend): 1) Set `spec.replicas: 2` (or more based on expected load). 2) Define `resources.requests` and `resources.limits` for CPU and memory based on observed dev usage with a safety margin. 3) Set `spec.strategy.rollingUpdate.maxUnavailable: 0` and `maxSurge: 1` to ensure zero-downtime deploys. 4) Create a PodDisruptionBudget for each Deployment with `minAvailable: 1` to protect against voluntary disruptions. 5) Add anti-affinity rules (`podAntiAffinity` with `preferredDuringSchedulingIgnoredDuringExecution`) to spread replicas across nodes. 6) Ensure readiness and liveness probes are configured on every container so the scheduler only routes traffic to healthy pods.

## Validation
Verify each Deployment has >=2 ready replicas via `kubectl get deployments`. Confirm PDBs exist for every Deployment via `kubectl get pdb`. Perform a `kubectl drain` on a node and verify no service goes fully unavailable. Trigger a rolling update and confirm zero-downtime by continuously curling service endpoints during rollout.