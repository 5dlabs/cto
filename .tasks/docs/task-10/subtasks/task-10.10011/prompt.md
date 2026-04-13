<identity>
You are bolt working on subtask 10011 of task 10.
</identity>

<context>
<scope>
Create a dedicated ServiceAccount for each of the 6 service Deployments. Create a Role per service with least-privilege rules (read own secrets via projected volumes only). Remove default ServiceAccount auto-mount from all non-ArgoCD pods by setting automountServiceAccountToken: false on each Deployment pod spec.
</scope>
</context>

<implementation_plan>
For each service (equipment-catalog, rms, finance, customer-vetting, social-engine, morgan): create helm/sigma1/templates/rbac-<service>.yaml containing: `ServiceAccount` with `automountServiceAccountToken: false`, `Role` with rules limited to `resources: ['secrets'], verbs: ['get']` scoped to only the secrets that service needs (by resourceNames), `RoleBinding` linking the ServiceAccount to the Role. Update each Deployment spec: `spec.template.spec.serviceAccountName: <service>-sa, automountServiceAccountToken: false`. ClusterRoleBinding only for ArgoCD SA (already managed by ArgoCD install). Verify no pod has default SA token mounted.
</implementation_plan>

<validation>
`kubectl get sa -n sigma1` lists one SA per service plus argocd SA. `kubectl exec <equipment-catalog-pod> -- ls /var/run/secrets/kubernetes.io/serviceaccount/` shows no token file (automount disabled). `kubectl exec <equipment-catalog-pod> -- kubectl get pods -n sigma1` returns Forbidden (least privilege). `kubectl get rolebinding -n sigma1` shows 6 service role bindings.
</validation>