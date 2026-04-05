Implement subtask 1004: Create Valkey instance via Opstree Redis operator CR

## Objective
Deploy a Valkey 7.2 instance using the existing Opstree Redis operator by creating a Redis custom resource in the sigma1 namespace.

## Steps
1. Create `sigma1-valkey.yaml` with an Opstree `Redis` CR:
   - `metadata.name: sigma1-valkey`, `metadata.namespace: sigma1`
   - `spec.kubernetesConfig.image: valkey/valkey:7.2-alpine`
   - `spec.kubernetesConfig.imagePullPolicy: IfNotPresent`
   - `spec.redisExporter.enabled: true` (for Prometheus scraping)
   - Single replica configuration (standalone mode, not cluster or sentinel)
   - Resource requests: 128Mi memory, 100m CPU (dev sizing)
2. Apply the manifest: `kubectl apply -f sigma1-valkey.yaml`.
3. Wait for the Valkey pod to reach Running state.
4. Verify connectivity: `kubectl exec` into a temporary pod and run `redis-cli -h sigma1-valkey -p 6379 PING`.
5. Note the service DNS name: `sigma1-valkey.sigma1.svc.cluster.local:6379` for the ConfigMap.

## Validation
`kubectl get redis sigma1-valkey -n sigma1` shows the CR in a ready state. `kubectl get pods -n sigma1 -l app=sigma1-valkey` shows 1/1 Running. `kubectl run --rm -it --image=valkey/valkey:7.2-alpine test-valkey -- redis-cli -h sigma1-valkey.sigma1.svc.cluster.local PING` returns `PONG`.