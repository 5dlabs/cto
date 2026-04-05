Implement subtask 10001: HA scaling: update replica counts and pod anti-affinity for all application services

## Objective
Update Kubernetes deployment manifests for Equipment Catalog, RMS, Finance, Customer Vetting, and Social Engine to 2 replicas each with pod anti-affinity rules. Document Morgan single-replica limitation with session affinity notes.

## Steps
Step-by-step:
1. For each service (equipment-catalog, rms, finance, customer-vetting, social-engine), update the Deployment manifest:
   - Set `spec.replicas: 2`
   - Add `spec.template.spec.affinity.podAntiAffinity` with `preferredDuringSchedulingIgnoredDuringExecution` targeting `topology.kubernetes.io/zone` and `requiredDuringSchedulingIgnoredDuringExecution` targeting `kubernetes.io/hostname` using label selector matching the service.
2. For Equipment Catalog, verify the Task 2 manifest already has this; only patch if missing.
3. For Morgan, add a comment block in the manifest documenting that HA requires session affinity design (workspace PVC is ReadWriteOnce), keep at 1 replica.
4. All manifests should be in the Helm chart values or kustomize overlays under a `production` environment.

## Validation
Apply manifests to cluster. Run `kubectl get pods -l app=<service> -o wide` for each service and verify 2 pods are scheduled on different nodes. Verify Morgan remains at 1 replica.