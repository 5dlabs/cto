Implement subtask 1001: Create sigma1-dev Kubernetes namespace

## Objective
Create the dedicated `sigma1-dev` namespace that will host all Sigma-1 pipeline resources including secrets, ConfigMaps, and workloads.

## Steps
1. Author a namespace manifest `namespace-sigma1-dev.yaml` with `apiVersion: v1`, `kind: Namespace`, `metadata.name: sigma1-dev`.
2. Add standard labels: `app.kubernetes.io/part-of: sigma1`, `environment: dev`.
3. Apply the manifest with `kubectl apply -f namespace-sigma1-dev.yaml`.
4. Verify the namespace is in Active state.

## Validation
`kubectl get namespace sigma1-dev` returns status Active. `kubectl get namespace sigma1-dev -o jsonpath='{.metadata.labels}'` includes expected labels.