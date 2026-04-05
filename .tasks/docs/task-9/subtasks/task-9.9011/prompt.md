Implement subtask 9011: Update sigma1-infra-endpoints ConfigMap with production hostnames

## Objective
Update the sigma1-infra-endpoints ConfigMap to reflect production hostnames, CDN URLs, and any changed connection strings for production topology.

## Steps
1. Edit the `sigma1-infra-endpoints` ConfigMap in the sigma1 namespace:
   - Update `PUBLIC_URL` to `https://sigma-1.com`
   - Update `API_BASE_URL` to `https://api.sigma-1.com`
   - Update `ASSETS_CDN_URL` to `https://assets.sigma-1.com`
   - Update `WS_URL` to `wss://api.sigma-1.com/ws`
   - Verify PostgreSQL connection string reflects PgBouncer endpoint if using connection pooling
   - Verify Valkey connection string reflects sentinel endpoint if using sentinel mode
2. Ensure all services reference this ConfigMap via `envFrom` and will pick up changes on next restart.
3. Perform a rolling restart of all services to pick up new ConfigMap values:
   - `kubectl rollout restart deployment -n sigma1`
4. Verify services are using the new hostnames in their logs and health checks.

## Validation
After updating ConfigMap and restarting services, exec into a backend pod and verify environment variables: `env | grep PUBLIC_URL` shows `https://sigma-1.com`. Verify services respond correctly using the production URLs.