Implement subtask 1007: Create sigma-1-infra-endpoints ConfigMap with service endpoints

## Objective
Create the ConfigMap `sigma-1-infra-endpoints` containing all service endpoint URLs that downstream tasks will consume via `envFrom`.

## Steps
1. Create `configmap-infra-endpoints.yaml`:
   ```yaml
   apiVersion: v1
   kind: ConfigMap
   metadata:
     name: sigma-1-infra-endpoints
     namespace: sigma-1
     labels:
       app.kubernetes.io/part-of: sigma-1
       sigma-1-pipeline: infra
   data:
     DISCORD_BRIDGE_URL: "http://discord-bridge-http.bots.svc.cluster.local"
     LINEAR_BRIDGE_URL: "http://linear-bridge.bots.svc.cluster.local"
     PM_SERVER_URL: "http://cto-pm.cto.svc.cluster.local"
     HERMES_URL: ""
     NOUS_API_URL: "https://api.nous.com"
   ```
2. Note: HERMES_URL is left empty until the Hermes endpoint is discovered/confirmed (per decision point). NOUS_API_URL uses the presumed external endpoint.
3. Apply with `kubectl apply -f configmap-infra-endpoints.yaml -n sigma-1`.
4. Downstream tasks will reference this ConfigMap via `envFrom: [{configMapRef: {name: sigma-1-infra-endpoints}}]`.

## Validation
`kubectl get configmap sigma-1-infra-endpoints -n sigma-1 -o json | jq '.data'` returns an object with exactly 5 keys: DISCORD_BRIDGE_URL, LINEAR_BRIDGE_URL, PM_SERVER_URL, HERMES_URL, NOUS_API_URL. DISCORD_BRIDGE_URL, LINEAR_BRIDGE_URL, and PM_SERVER_URL are non-empty strings matching the expected in-cluster DNS patterns.