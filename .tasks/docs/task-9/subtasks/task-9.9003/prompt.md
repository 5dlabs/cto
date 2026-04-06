Implement subtask 9003: Increase replicas for backend services and Morgan agent with HPA

## Objective
Scale all backend service Deployments to multiple replicas and configure Horizontal Pod Autoscalers (HPA) for CPU/memory-based autoscaling.

## Steps
1. For each backend Deployment (API services, Morgan agent, web frontend), set `spec.replicas` to at least 2 for baseline availability. 2. Create HPA resources targeting 70% CPU utilization with min=2, max=5 replicas (adjust per service based on expected load). 3. Ensure metrics-server is installed and functioning in the cluster. 4. Set PodDisruptionBudgets (minAvailable: 1) for each Deployment to protect availability during rolling updates. 5. Configure anti-affinity (preferredDuringSchedulingIgnoredDuringExecution) to spread pods across nodes. 6. Apply all manifests and verify HPA status shows current metrics.

## Validation
Verify each service has at least 2 running pods. Check `kubectl get hpa` shows valid current/target metrics. Simulate CPU load on one service and confirm HPA scales up within 2 minutes. Verify PDB prevents simultaneous eviction of all pods during a drain.