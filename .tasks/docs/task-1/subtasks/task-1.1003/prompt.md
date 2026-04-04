Implement subtask 1003: Create sigma-1-infra-endpoints ConfigMap

## Objective
Create the ConfigMap `sigma-1-infra-endpoints` in sigma-1-dev namespace with all 4 endpoint keys pointing to existing in-cluster services.

## Steps
1. Create a ConfigMap manifest `configmap-endpoints.yaml` in namespace `sigma-1-dev`.
2. Populate the following keys:
   - `DISCORD_BRIDGE_URL`: `http://discord-bridge-http.bots.svc.cluster.local` (verify actual service name/port from the bots namespace)
   - `LINEAR_BRIDGE_URL`: `http://linear-bridge.bots.svc.cluster.local` (verify actual service name/port from the bots namespace)
   - `NATS_URL`: `nats://openclaw-nats.openclaw.svc.cluster.local:4222`
   - `CLOUDFLARE_OPERATOR_NS`: `cloudflare-operator-system`
3. Apply with `kubectl apply -f configmap-endpoints.yaml`.
4. Verify all 4 keys are present and non-empty.

## Validation
`kubectl get configmap sigma-1-infra-endpoints -n sigma-1-dev -o json | jq '.data'` contains exactly 4 keys (DISCORD_BRIDGE_URL, LINEAR_BRIDGE_URL, NATS_URL, CLOUDFLARE_OPERATOR_NS), each with a non-empty string value.