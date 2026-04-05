Implement subtask 9003: Increase backend service replicas for production concurrency

## Objective
Scale all backend service Deployments to multiple replicas with appropriate resource requests, HPA configuration, and pod disruption budgets.

## Steps
1. For each backend Deployment (API services, workers, etc.), set `replicas: 2` minimum as baseline. 2. Create HorizontalPodAutoscaler (HPA) resources targeting 70% CPU utilization with `minReplicas: 2` and `maxReplicas` based on expected load. 3. Add PodDisruptionBudget (PDB) resources with `minAvailable: 1` for each Deployment to ensure availability during node drains. 4. Configure pod anti-affinity with `preferredDuringSchedulingIgnoredDuringExecution` on `kubernetes.io/hostname` to spread replicas. 5. Set appropriate resource `requests` and `limits` for CPU and memory based on observed usage. 6. Apply all manifests and verify pods are scheduled across multiple nodes.

## Validation
Verify each Deployment has at least 2 ready replicas with `kubectl get deployments`. Trigger a node drain and confirm PDBs prevent full service disruption. Simulate CPU load and verify HPA scales up within 2 minutes.