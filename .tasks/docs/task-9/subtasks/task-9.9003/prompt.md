Implement subtask 9003: Scale backend service replicas for all application services

## Objective
Increase Deployment replica counts for Equipment Catalog, RMS, Finance, Vetting, Social, and Morgan services to at least 2 replicas each, with appropriate PodDisruptionBudgets and anti-affinity rules.

## Steps
1. For each service (Equipment Catalog, RMS, Finance, Vetting, Social, Morgan), update the Deployment or Helm values to set `replicas: 2` (minimum) or higher based on expected load.
2. Add `topologySpreadConstraints` or pod anti-affinity to distribute replicas across nodes.
3. Add PodDisruptionBudgets with `minAvailable: 1` for each Deployment.
4. Ensure readiness and liveness probes are properly configured for rolling updates.
5. Apply all changes and verify each service has the correct number of running, ready pods.
6. Confirm rolling update strategy is set to `RollingUpdate` with `maxUnavailable: 0` and `maxSurge: 1`.

## Validation
Verify each of the 6 services has at least 2 running, ready pods. Perform a rolling restart of one service and confirm zero downtime by continuously hitting its health endpoint. Verify PDBs are created for each service.