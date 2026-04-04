Implement subtask 9001: Configure HA replica scaling and anti-affinity for cto-pm Deployment

## Objective
Update the cto-pm Deployment to run at least 2 replicas with pod anti-affinity rules to spread pods across nodes, ensuring high availability.

## Steps
Step-by-step:
1. In the cto-pm Deployment manifest (or Helm values), set `spec.replicas: 2`.
2. Add `spec.template.spec.affinity.podAntiAffinity` with a `preferredDuringSchedulingIgnoredDuringExecution` rule keyed on `app=cto-pm` and `topologyKey: kubernetes.io/hostname` to spread pods across nodes.
3. Verify the Elysia application has no in-memory session state (no sticky sessions needed). Check for any in-process caches, singleton stores, or WebSocket state that would break with multiple replicas. If found, document and flag.
4. Apply the updated Deployment: `kubectl apply -f` or `helm upgrade`.
5. Confirm 2 pods are scheduled on different nodes: `kubectl get pods -n sigma-1 -o wide -l app=cto-pm`.

## Validation
`kubectl get pods -n sigma-1 -l app=cto-pm -o wide` shows >= 2 Running pods on different nodes. Killing one pod does not cause downtime — the remaining pod continues serving requests while the replacement starts.