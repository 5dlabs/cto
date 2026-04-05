Implement subtask 1001: Create Kubernetes namespaces and base RBAC configuration

## Objective
Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and configure basic RBAC ServiceAccounts so downstream deployments can reference secrets and ConfigMaps across namespaces.

## Steps
1. Create namespace manifests for: databases, sigma1, openclaw, social, web. 2. Apply namespace labels for organization (e.g., app.kubernetes.io/part-of: sigma1). 3. Create ServiceAccounts in each namespace for workloads that need cross-namespace ConfigMap access. 4. If needed, create RoleBindings allowing sigma1 workloads to read the shared ConfigMap from the databases namespace. 5. Apply all manifests via kubectl or Helm.

## Validation
Run 'kubectl get namespaces' and verify all five namespaces exist; verify ServiceAccounts are created in each namespace; verify RoleBindings allow cross-namespace ConfigMap reads.