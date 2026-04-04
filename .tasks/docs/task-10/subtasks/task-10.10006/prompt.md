Implement subtask 10006: Configure liveness and readiness probes on all deployments

## Objective
Add HTTP liveness probes (GET /health, 10s interval) and readiness probes (GET /ready, 5s interval) to all service deployments in sigma-1-dev.

## Steps
1. Edit the PM server Deployment manifest to add probes under `spec.template.spec.containers[0]`:
   - livenessProbe: httpGet path=/health port=8080, periodSeconds=10, initialDelaySeconds=5, failureThreshold=3
   - readinessProbe: httpGet path=/ready port=8080, periodSeconds=5, initialDelaySeconds=3, failureThreshold=3
2. Edit the frontend Deployment manifest similarly:
   - livenessProbe: httpGet path=/health port=3000, periodSeconds=10, initialDelaySeconds=5
   - readinessProbe: httpGet path=/ready port=3000, periodSeconds=5, initialDelaySeconds=3
3. Ensure the PM server and frontend applications actually implement /health and /ready endpoints. If not, document that those endpoints need to be added (or use TCP socket probes as a fallback).
4. Apply manifests and verify pods transition to Ready state.
5. Verify probe configuration via `kubectl get deployment -o json`.

## Validation
`kubectl get deployment sigma-1-pm-server -n sigma-1-dev -o jsonpath='{.spec.template.spec.containers[0].livenessProbe}'` returns a non-empty JSON object with httpGet path=/health. `kubectl get deployment sigma-1-pm-server -n sigma-1-dev -o jsonpath='{.spec.template.spec.containers[0].readinessProbe}'` returns httpGet path=/ready. Pods are in Ready state (READY 1/1).