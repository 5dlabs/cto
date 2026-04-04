Implement subtask 1001: Create sigma-1-dev namespace with resource quotas and limit ranges

## Objective
Create the Kubernetes namespace `sigma-1-dev` and apply ResourceQuota and LimitRange objects scoped for a dev validation run, ensuring pods cannot exceed reasonable CPU/memory bounds.

## Steps
1. Create a YAML manifest for the `sigma-1-dev` namespace with labels `project: sigma-1`, `env: dev`.
2. Define a ResourceQuota CR in the namespace: requests.cpu=4, requests.memory=8Gi, limits.cpu=8, limits.memory=16Gi, pods=20, persistentvolumeclaims=5.
3. Define a LimitRange CR: default container limits cpu=500m, memory=512Mi; default requests cpu=100m, memory=128Mi; max cpu=2, memory=2Gi.
4. Apply all manifests with `kubectl apply -f`.
5. Verify the namespace exists and both ResourceQuota and LimitRange are active.

## Validation
`kubectl get ns sigma-1-dev` returns Active. `kubectl get resourcequota -n sigma-1-dev` shows the quota with expected hard limits. `kubectl get limitrange -n sigma-1-dev` shows the limit range with expected defaults and max values.