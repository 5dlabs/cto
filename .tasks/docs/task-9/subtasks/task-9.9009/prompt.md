Implement subtask 9009: Configure Horizontal Pod Autoscalers for all services

## Objective
Create HPA resources for all 6 backend services with appropriate min/max replicas and CPU target thresholds.

## Steps
1. Create HPA for Equipment Catalog:
   ```yaml
   apiVersion: autoscaling/v2
   kind: HorizontalPodAutoscaler
   metadata:
     name: equipment-catalog-hpa
     namespace: sigma1
   spec:
     scaleTargetRef:
       apiVersion: apps/v1
       kind: Deployment
       name: equipment-catalog
     minReplicas: 2
     maxReplicas: 5
     metrics:
     - type: Resource
       resource:
         name: cpu
         target:
           type: Utilization
           averageUtilization: 70
   ```
2. Create HPA for RMS: min 2, max 5, target CPU 70%
3. Create HPA for Finance: min 2, max 3, target CPU 70%
4. Create HPA for Social Engine: min 1, max 3, target CPU 70%
5. Create HPA for Morgan: min 1, max 2, target CPU 70% (note: Morgan is stateful with WebSocket connections, scale carefully)
6. Create HPA for Customer Vetting: min 1, max 2, target CPU 70%
7. Ensure all target Deployments have resource requests set (HPA requires this to calculate utilization).
8. Apply all HPAs and verify they show `<unknown>` or actual metrics (not errors).

## Validation
Verify all 6 HPAs are created: `kubectl get hpa -n sigma1` shows all with TARGETS column populated (not `<unknown>/70%` after metrics are available). Generate synthetic load on Equipment Catalog using `hey` or `k6` (50 req/sec for 2 minutes), verify pod count scales from 2 to 3+ and scales back down after load stops.