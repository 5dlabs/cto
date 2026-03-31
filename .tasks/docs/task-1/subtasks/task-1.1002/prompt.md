Implement subtask 1002: Create RBAC, ResourceQuota, and LimitRange Templates

## Objective
Create namespace-scoped ServiceAccount, Role, RoleBinding, ResourceQuota, and LimitRange templates. These are all small governance YAML resources with identical lifecycle and dependencies, grouped into a single subtask.

## Steps
Step-by-step:
1. Create `templates/serviceaccount.yaml`: ServiceAccount `hermes-pipeline-sa` in `{{ .Values.namespace }}` with standard labels.
2. Create `templates/role.yaml`: namespace-scoped Role with least-privilege verbs. Grant: get/list/watch on pods, services, configmaps, secrets, endpoints; create/delete on jobs. Do NOT use ClusterRole.
3. Create `templates/rolebinding.yaml`: RoleBinding binding the Role to `hermes-pipeline-sa`. Namespace: `{{ .Values.namespace }}`.
4. Create `templates/resourcequota.yaml`: ResourceQuota with `spec.hard` containing `requests.cpu: {{ .Values.resourceQuota.cpu }}`, `requests.memory: {{ .Values.resourceQuota.memory }}`, `pods: {{ .Values.resourceQuota.pods }}`. Apply standard labels.
5. Create `templates/limitrange.yaml`: LimitRange with `spec.limits` type Container: `default.cpu: {{ .Values.limitRange.defaultCpu }}` (500m), `default.memory: {{ .Values.limitRange.defaultMemory }}` (512Mi), `defaultRequest.cpu: 250m`, `defaultRequest.memory: 256Mi`, `max.cpu: {{ .Values.limitRange.maxCpu }}` (2), `max.memory: {{ .Values.limitRange.maxMemory }}` (2Gi).
6. All resources namespaced to `{{ .Values.namespace }}` with standard labels.
7. Verify: `helm template --debug` renders all 5 resources for both environments with correct parameterized values.

## Validation
`kubectl auth can-i --as=system:serviceaccount:hermes-staging:hermes-pipeline-sa --namespace=hermes-staging list pods` returns `yes`. Cross-namespace access denied: same SA cannot access hermes-production. `kubectl get clusterrolebinding | grep hermes` returns nothing. `kubectl describe resourcequota -n hermes-staging` shows cpu=8, memory=16Gi, pods=20. `kubectl get limitrange -n hermes-staging -o yaml` confirms default cpu=500m, memory=512Mi, max cpu=2, memory=2Gi. Production namespace shows production-tier quota values.