Implement subtask 9001: Update Helm production values for PM server replicas and resource limits

## Objective
Create or update `values-sigma1-prod.yaml` to set PM server deployment to 3 replicas with production-grade resource requests and limits.

## Steps
1. In `values-sigma1-prod.yaml`, set `pmServer.replicaCount: 3`.
2. Set `pmServer.resources.requests.memory: 256Mi`, `pmServer.resources.requests.cpu: 250m`.
3. Set `pmServer.resources.limits.memory: 512Mi`, `pmServer.resources.limits.cpu: 500m`.
4. Ensure the Helm template in `templates/pm-server-deployment.yaml` references `.Values.pmServer.resources` in the container spec.
5. Verify with `helm template . -f values-sigma1-prod.yaml | grep -A 10 resources` that the rendered manifest has correct values.

## Validation
Run `helm template . -f values-sigma1-prod.yaml` and confirm the PM server Deployment has replicas=3 and the correct resource requests/limits in the rendered output.