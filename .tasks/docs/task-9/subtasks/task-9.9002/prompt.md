Implement subtask 9002: Create PodDisruptionBudget for cto-pm

## Objective
Define a PodDisruptionBudget resource ensuring at least one cto-pm pod remains available during voluntary disruptions (node drains, upgrades).

## Steps
Step-by-step:
1. Create a PodDisruptionBudget manifest:
   ```yaml
   apiVersion: policy/v1
   kind: PodDisruptionBudget
   metadata:
     name: cto-pm-pdb
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
   spec:
     minAvailable: 1
     selector:
       matchLabels:
         app: cto-pm
   ```
2. Apply with `kubectl apply -f cto-pm-pdb.yaml`.
3. Validate: `kubectl get pdb -n sigma-1` shows `cto-pm-pdb` with `MIN AVAILABLE: 1` and `ALLOWED DISRUPTIONS >= 1` (when 2+ pods are running).

## Validation
`kubectl get pdb cto-pm-pdb -n sigma-1` shows minAvailable=1 and allowedDisruptions >= 1. Attempting `kubectl drain` on one node hosting a cto-pm pod succeeds while the other pod remains running. Attempting to drain both nodes simultaneously is blocked.