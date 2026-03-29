Implement subtask 3005: Configure HPA, RBAC ServiceAccount, and ResourceQuota

## Objective
Create the HorizontalPodAutoscaler (min 2, max 10, 70% CPU), a minimal ServiceAccount with no cluster-wide roles, and a namespace-level ResourceQuota.

## Steps
1. Create `infra/notifycore/templates/hpa.yaml`:
   - HorizontalPodAutoscaler targeting notifycore Deployment.
   - spec.minReplicas: 2, spec.maxReplicas: 10.
   - spec.metrics: [{type: Resource, resource: {name: cpu, target: {type: Utilization, averageUtilization: 70}}}].
   - Conditionally render when `hpa.enabled: true`.
2. Create `infra/notifycore/templates/serviceaccount.yaml`:
   - ServiceAccount `notifycore-sa` in notifycore namespace.
   - `automountServiceAccountToken: false` (no need for K8s API access).
3. Update the Deployment template to use `serviceAccountName: notifycore-sa`.
4. Create `infra/notifycore/templates/resource-quota.yaml`:
   - ResourceQuota in notifycore namespace.
   - spec.hard: requests.cpu: "2", requests.memory: "1Gi", limits.cpu: "4", limits.memory: "2Gi".
   - Conditionally render when `resourceQuota.enabled: true`.
5. Create `infra/notifycore/templates/app-pdb.yaml`:
   - PodDisruptionBudget for the notifycore app Deployment.
   - maxUnavailable: 1.
6. In `values-prod.yaml`: hpa.enabled: true, resourceQuota.enabled: true.
7. In `values-dev.yaml`: hpa.enabled: false, resourceQuota.enabled: false.

## Validation
`helm template` with values-prod.yaml renders: HPA with min=2/max=10/cpu=70%, ServiceAccount named notifycore-sa, ResourceQuota with specified limits, and app PDB with maxUnavailable=1. Deployment references serviceAccountName: notifycore-sa. `values-dev.yaml` does not render HPA or ResourceQuota but does render the ServiceAccount.