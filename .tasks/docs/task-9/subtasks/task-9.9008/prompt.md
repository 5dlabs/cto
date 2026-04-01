Implement subtask 9008: Add readiness and liveness probes to frontend deployment

## Objective
Configure HTTP health probes for the frontend: readiness and liveness via GET / on port 3000.

## Steps
1. In `templates/frontend-deployment.yaml`, add to the container spec:
   ```yaml
   readinessProbe:
     httpGet:
       path: /
       port: 3000
     initialDelaySeconds: 5
     periodSeconds: 10
     failureThreshold: 3
   livenessProbe:
     httpGet:
       path: /
       port: 3000
     initialDelaySeconds: 5
     periodSeconds: 10
     failureThreshold: 3
   ```
2. Parameterize probe paths, ports, and timing via Helm values.
3. Verify with `helm template` that probes appear in the rendered Deployment.

## Validation
Rendered Deployment YAML includes both probes with httpGet path `/`, port 3000, initialDelaySeconds=5, periodSeconds=10. After deploy: `kubectl describe pod <frontend-pod> -n sigma1-prod` shows probes configured.