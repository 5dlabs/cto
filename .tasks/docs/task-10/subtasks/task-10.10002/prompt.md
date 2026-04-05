Implement subtask 10002: HA scaling: configure HPA for Equipment Catalog

## Objective
Create a HorizontalPodAutoscaler resource for the Equipment Catalog service that scales from 2 to 4 replicas when CPU utilization exceeds 70%.

## Steps
Step-by-step:
1. Create `hpa-equipment-catalog.yaml` with:
   - `apiVersion: autoscaling/v2`
   - `spec.scaleTargetRef` pointing to Equipment Catalog Deployment
   - `spec.minReplicas: 2`, `spec.maxReplicas: 4`
   - `spec.metrics[0].type: Resource`, `resource.name: cpu`, `resource.target.type: Utilization`, `resource.target.averageUtilization: 70`
   - `spec.behavior.scaleDown.stabilizationWindowSeconds: 300` to prevent flapping
2. Ensure the Equipment Catalog Deployment has `resources.requests.cpu` set (e.g., 250m) so HPA can calculate utilization.
3. Apply the manifest.

## Validation
Apply HPA manifest. Run `kubectl get hpa` and verify targets show current/target CPU values (not <unknown>). Use a load generator (e.g., `hey` or `k6`) to drive CPU above 70% on equipment-catalog, verify `kubectl get hpa` shows scaling to 3+ replicas within 2-3 minutes.