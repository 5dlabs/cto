Implement subtask 9007: Add readiness and liveness probes to PM server deployment

## Objective
Configure HTTP health probes for the PM server: readiness and liveness via GET /health on port 3000.

## Steps
1. In `templates/pm-server-deployment.yaml`, add to the container spec:
   ```yaml
   readinessProbe:
     httpGet:
       path: /health
       port: 3000
     initialDelaySeconds: 10
     periodSeconds: 15
     failureThreshold: 3
   livenessProbe:
     httpGet:
       path: /health
       port: 3000
     initialDelaySeconds: 10
     periodSeconds: 15
     failureThreshold: 3
   ```
2. Parameterize probe paths, ports, and timing via Helm values for overridability.
3. Verify with `helm template` that probes appear in the rendered Deployment.

## Validation
Rendered Deployment YAML includes both readinessProbe and livenessProbe with correct httpGet path `/health`, port 3000, initialDelaySeconds=10, periodSeconds=15. After deploy: `kubectl describe pod <pm-server-pod> -n sigma1-prod` shows probes configured.