Implement subtask 1007: Create sigma-1-infra-endpoints ConfigMap with all connection strings

## Objective
Create the ConfigMap `sigma-1-infra-endpoints` in `sigma-1-dev` aggregating the Postgres connection string from the CNPG-generated secret, the Redis service URL, and the in-cluster URLs for the discord-bridge-http and linear-bridge services.

## Steps
1. Retrieve the Postgres connection URI from the CNPG-generated app secret (e.g., `sigma-1-pg-app`). The ConfigMap should reference this as `CNPG_SIGMA1_PG_URL`.
2. Construct the Redis URL: `redis://sigma-1-redis.sigma-1-dev.svc.cluster.local:6379`.
3. Set `DISCORD_BRIDGE_URL` to the in-cluster service URL: `http://discord-bridge-http.bots.svc.cluster.local` (adjust port as needed).
4. Set `LINEAR_BRIDGE_URL` to the in-cluster service URL: `http://linear-bridge.bots.svc.cluster.local` (adjust port as needed).
5. Create a ConfigMap manifest with these 4 keys: `CNPG_SIGMA1_PG_URL`, `REDIS_SIGMA1_URL`, `DISCORD_BRIDGE_URL`, `LINEAR_BRIDGE_URL`.
6. Note: For the Postgres URL, either use a Kustomize secret generator or a scripted approach to inject the value from the CNPG secret into the ConfigMap at apply time. Alternatively, use an init container pattern at runtime. Document the chosen approach.
7. Apply the ConfigMap manifest.

## Validation
`kubectl get configmap sigma-1-infra-endpoints -n sigma-1-dev -o json | jq '.data | keys'` returns exactly `["CNPG_SIGMA1_PG_URL", "DISCORD_BRIDGE_URL", "LINEAR_BRIDGE_URL", "REDIS_SIGMA1_URL"]`. Each value is non-empty. The Postgres URL is a valid connection string format.