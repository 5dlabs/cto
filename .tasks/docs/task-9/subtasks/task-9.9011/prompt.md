Implement subtask 9011: Label all sigma-1 production resources with pipeline label

## Objective
Apply the label `sigma-1-pipeline: production` to all resources in the sigma-1 namespace for consistent identification and management.

## Steps
Step-by-step:
1. Audit all resources created/modified in this task: Deployment, Service, PodDisruptionBudget, Ingress, NetworkPolicies, HPA, LimitRange.
2. Ensure each manifest includes the label:
   ```yaml
   metadata:
     labels:
       sigma-1-pipeline: production
   ```
3. For any existing resources that were not updated in other subtasks, apply the label:
   ```bash
   kubectl label deployment cto-pm -n sigma-1 sigma-1-pipeline=production --overwrite
   kubectl label service cto-pm -n sigma-1 sigma-1-pipeline=production --overwrite
   # ... repeat for all resources
   ```
4. Verify: `kubectl get all -n sigma-1 -l sigma-1-pipeline=production` lists all expected resources.
5. Consider adding pod template labels so that pods inherit the label as well.

## Validation
`kubectl get all,networkpolicy,pdb,hpa,ingress -n sigma-1 -l sigma-1-pipeline=production` returns all production-hardened resources. No resource in sigma-1 that should be labeled is missing from the output.