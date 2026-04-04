Implement subtask 10002: Update cto-pm Deployment to use sigma-1-pipeline-sa ServiceAccount

## Objective
Patch the cto-pm Deployment spec to reference the new ServiceAccount `sigma-1-pipeline-sa`, replacing the default service account. Also set `automountServiceAccountToken: true` only if needed, or `false` if the pod does not require Kubernetes API access.

## Steps
Step-by-step:
1. Edit the cto-pm Deployment manifest (or Helm values).
2. Set `spec.template.spec.serviceAccountName: sigma-1-pipeline-sa`.
3. Evaluate whether the pod needs the Kubernetes API token mounted. If not (all config comes from ConfigMaps/Secrets mounted as volumes or env vars), set `automountServiceAccountToken: false` for additional security.
4. Apply the updated Deployment and verify the pod restarts with the new SA.
5. Confirm the pod can still read its required ConfigMaps and Secrets.

## Validation
After rollout, `kubectl get pod -n sigma-1 -o jsonpath='{.items[0].spec.serviceAccountName}'` returns `sigma-1-pipeline-sa`. The pod reaches Running/Ready state. A pipeline run completes successfully, confirming the SA has sufficient permissions for normal operations.