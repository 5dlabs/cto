Implement subtask 9009: Configure health probes (liveness, readiness, startup) for cto-pm

## Objective
Add liveness, readiness, and startup probes to the cto-pm Deployment to enable Kubernetes to detect unhealthy pods and manage rolling updates correctly.

## Steps
Step-by-step:
1. Update the cto-pm container spec with probes:
   ```yaml
   spec:
     containers:
     - name: cto-pm
       livenessProbe:
         httpGet:
           path: /health
           port: 3000
         initialDelaySeconds: 10
         periodSeconds: 30
         timeoutSeconds: 5
         failureThreshold: 3
       readinessProbe:
         httpGet:
           path: /ready
           port: 3000
         initialDelaySeconds: 5
         periodSeconds: 10
         timeoutSeconds: 5
         failureThreshold: 3
       startupProbe:
         httpGet:
           path: /health
           port: 3000
         failureThreshold: 30
         periodSeconds: 10
   ```
2. Ensure the cto-pm Elysia application exposes `/health` and `/ready` endpoints. `/health` should return 200 if the process is alive. `/ready` should return 200 only when the app is fully initialized and can serve traffic (e.g., all clients connected).
3. If `/ready` does not exist yet, coordinate with the application team (Task 3/4) — this subtask only covers the Kubernetes probe configuration.
4. Apply and verify: `kubectl describe pod <cto-pm-pod> -n sigma-1` shows all three probes configured.

## Validation
`kubectl describe pod <cto-pm-pod> -n sigma-1` shows Liveness, Readiness, and Startup probes configured with the specified paths and parameters. A pod that fails the liveness check is restarted (simulate by temporarily breaking /health). A pod that fails readiness is removed from the Service endpoints (verify with `kubectl get endpoints -n sigma-1`).