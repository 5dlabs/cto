Implement subtask 9006: Update service manifests with readiness/liveness probes and resource limits

## Objective
Add or update readiness probes, liveness probes, startup probes, resource requests, and resource limits for all application Deployments and StatefulSets to ensure production reliability and proper scheduling.

## Steps
1. For each application Deployment/StatefulSet, add or update:
   - `readinessProbe`: HTTP GET to health endpoint (e.g., `/healthz` or `/ready`) with appropriate `initialDelaySeconds`, `periodSeconds`, and `failureThreshold`.
   - `livenessProbe`: HTTP GET or TCP check with more generous thresholds than readiness.
   - `startupProbe` for services with slow initialization (e.g., ML model loading).
2. Set `resources.requests` for CPU and memory based on observed or estimated usage.
3. Set `resources.limits` for memory (and optionally CPU) to prevent noisy-neighbor issues.
4. Ensure all probes use the correct port and path for each service.
5. Apply updated manifests and verify pods restart cleanly with probes passing.
6. Confirm resource requests sum does not exceed cluster capacity with headroom.

## Validation
Verify all pods show `Running` status with `Ready` condition true. Describe each pod and confirm readiness, liveness, and startup probes are configured. Simulate a health endpoint failure (e.g., kill the app process) and verify Kubernetes restarts the pod. Verify `kubectl top pods` shows resource usage within defined limits.