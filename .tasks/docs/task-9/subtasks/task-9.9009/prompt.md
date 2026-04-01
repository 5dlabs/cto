Implement subtask 9009: Configure pod anti-affinity rules for PM server cross-zone distribution

## Objective
Add pod anti-affinity rules to the PM server Deployment to spread pods across availability zones and nodes.

## Steps
1. In `templates/pm-server-deployment.yaml`, add to `spec.template.spec`:
   ```yaml
   affinity:
     podAntiAffinity:
       preferredDuringSchedulingIgnoredDuringExecution:
         - weight: 100
           podAffinityTerm:
             labelSelector:
               matchLabels:
                 app: pm-server
             topologyKey: topology.kubernetes.io/zone
         - weight: 50
           podAffinityTerm:
             labelSelector:
               matchLabels:
                 app: pm-server
             topologyKey: kubernetes.io/hostname
   ```
2. Use `preferredDuringSchedulingIgnoredDuringExecution` (soft) to avoid scheduling failures in clusters with fewer zones.
3. Parameterize the topology keys and weights via Helm values.
4. Verify with `helm template`.

## Validation
Rendered Deployment YAML includes the affinity block with both zone and hostname anti-affinity rules. After deploy with 3+ replicas: `kubectl get pods -o wide -n sigma1-prod -l app=pm-server` shows pods distributed across at least 2 different nodes/zones.