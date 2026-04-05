Implement subtask 1001: Create sigma1 Kubernetes namespace

## Objective
Create the sigma1 namespace with appropriate labels for network policy selection, monitoring, and resource quota boundaries.

## Steps
1. Create a namespace manifest `sigma1-namespace.yaml` with:
   - `metadata.name: sigma1`
   - Labels: `app.kubernetes.io/part-of: sigma1`, `monitoring: enabled`, `networking: sigma1`
2. Apply the manifest via `kubectl apply -f sigma1-namespace.yaml`.
3. Verify the namespace exists and is in Active phase.
4. This is the prerequisite for every other subtask in this parent task.

## Validation
`kubectl get namespace sigma1 -o jsonpath='{.status.phase}'` returns `Active`. Labels `app.kubernetes.io/part-of=sigma1` and `monitoring=enabled` are present.