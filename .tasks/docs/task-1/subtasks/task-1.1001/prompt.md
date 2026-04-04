Implement subtask 1001: Create sigma-1-dev namespace with labels

## Objective
Create the sigma-1-dev Kubernetes namespace with project and environment labels that all subsequent resources will be deployed into.

## Steps
1. Create a namespace manifest YAML file `namespace.yaml` defining `sigma-1-dev` namespace.
2. Add labels: `project: sigma-1`, `env: dev`.
3. Apply the manifest with `kubectl apply -f namespace.yaml`.
4. Verify the namespace is Active via `kubectl get namespace sigma-1-dev`.

## Validation
`kubectl get namespace sigma-1-dev -o jsonpath='{.status.phase}'` returns 'Active'. `kubectl get namespace sigma-1-dev -o jsonpath='{.metadata.labels.project}'` returns 'sigma-1'. `kubectl get namespace sigma-1-dev -o jsonpath='{.metadata.labels.env}'` returns 'dev'.