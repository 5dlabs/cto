Implement subtask 10005: Configure resource limits and requests on all deployments

## Objective
Set CPU and memory requests and limits on the PM server and frontend deployments per the specified values: PM server 256m/512Mi request, 1000m/1Gi limit; frontend 128m/256Mi request, 500m/512Mi limit.

## Steps
1. Edit the PM server Deployment manifest (or Helm values) to add under `spec.template.spec.containers[0].resources`:
   - requests: cpu=256m, memory=512Mi
   - limits: cpu=1000m, memory=1Gi
2. Edit the frontend Deployment manifest (or Helm values) to add:
   - requests: cpu=128m, memory=256Mi
   - limits: cpu=500m, memory=512Mi
3. If there are any other deployments in sigma-1-dev (bridge services), apply reasonable resource limits (128m/256Mi request, 500m/512Mi limit as a default).
4. Apply updated manifests.
5. Verify with `kubectl describe pod` that all pods show non-zero resource requests and limits.

## Validation
`kubectl describe pod -l app=sigma-1-pm-server -n sigma-1-dev` shows Requests: cpu=256m, memory=512Mi and Limits: cpu=1000m, memory=1Gi. Same verification for frontend pod with its specified values. No pod in the namespace has empty resource requests.