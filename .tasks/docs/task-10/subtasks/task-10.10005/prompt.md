Implement subtask 10005: Configure Kubernetes audit policy for sigma-1 namespace

## Objective
Create or update the Kubernetes API server audit policy to capture security-relevant events in the sigma-1 namespace: Secret access, RBAC events, and mutating operations on critical resources.

## Steps
Step-by-step:
1. Create an audit policy file `audit-policy.yaml` with rules scoped to the sigma-1 namespace:
   a. Level `Metadata` for all Secret get/list/watch operations in sigma-1.
   b. Level `RequestResponse` for all create/update/delete/patch operations on any resource in sigma-1.
   c. Level `Metadata` for all authentication and authorization events involving `system:serviceaccount:sigma-1:*`.
2. If the cluster uses a managed Kubernetes service (GKE, EKS, AKS), configure audit logging via the cloud provider's mechanism and document the approach.
3. If self-managed, update the kube-apiserver flags: `--audit-policy-file` and `--audit-log-path`.
4. Document that this step may require cluster-admin privileges and coordination with the platform team.
5. Verify audit log entries appear for test operations (e.g., `kubectl get secret -n sigma-1`).

## Validation
Perform a `kubectl get secret -n sigma-1` and verify a corresponding audit log entry appears in the configured audit log sink. Perform a `kubectl create configmap test-audit -n sigma-1 --from-literal=key=val` then `kubectl delete configmap test-audit -n sigma-1` — verify both create and delete events are logged at RequestResponse level. Audit policy YAML passes `kubeval` or `kubectl apply --dry-run=server` validation.