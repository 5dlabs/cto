Implement subtask 9002: Update Helm production values for frontend replicas and resource limits

## Objective
Update `values-sigma1-prod.yaml` to set the frontend deployment to 2 replicas with appropriate resource requests.

## Steps
1. In `values-sigma1-prod.yaml`, set `frontend.replicaCount: 2`.
2. Set `frontend.resources.requests.memory: 128Mi`, `frontend.resources.requests.cpu: 100m`.
3. Optionally set `frontend.resources.limits.memory: 256Mi`, `frontend.resources.limits.cpu: 200m` for safety.
4. Ensure the Helm template in `templates/frontend-deployment.yaml` references `.Values.frontend.resources`.
5. Verify with `helm template . -f values-sigma1-prod.yaml` that the rendered frontend Deployment has replicas=2 and correct resources.

## Validation
Run `helm template . -f values-sigma1-prod.yaml` and confirm the frontend Deployment renders with replicas=2 and the expected resource requests.