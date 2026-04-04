Implement subtask 1001: Create sigma-1 namespace with standard labels

## Objective
Create the Kubernetes namespace `sigma-1` with the required labels for project identification and environment tagging. This is the foundational resource all other subtasks depend on.

## Steps
1. Create a YAML manifest `namespace.yaml` defining:
   ```yaml
   apiVersion: v1
   kind: Namespace
   metadata:
     name: sigma-1
     labels:
       app.kubernetes.io/part-of: sigma-1
       env: dev
       sigma-1-pipeline: infra
   ```
2. Apply with `kubectl apply -f namespace.yaml`.
3. Verify the namespace is Active and labels are present.

## Validation
`kubectl get ns sigma-1 -o jsonpath='{.status.phase}'` returns 'Active'. `kubectl get ns sigma-1 --show-labels` includes `app.kubernetes.io/part-of=sigma-1`, `env=dev`, and `sigma-1-pipeline=infra`.