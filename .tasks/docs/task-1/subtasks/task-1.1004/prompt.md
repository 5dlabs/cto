Implement subtask 1004: Create notifycore-infra-endpoints ConfigMap

## Objective
Create the `notifycore-infra-endpoints` ConfigMap aggregating DATABASE_URL, REDIS_URL, PORT, and RUST_LOG, dynamically referencing credentials from the PostgreSQL and Redis secrets.

## Steps
1. Create `infra/notifycore/templates/configmap-endpoints.yaml` defining ConfigMap `notifycore-infra-endpoints` in the `notifycore` namespace.
2. Set keys:
   - `DATABASE_URL`: `postgres://notifycore_app:<password>@notifycore-pg-rw.notifycore.svc:5432/notifycore` — Note: since ConfigMaps cannot reference secrets natively, either: (a) use a Helm template that reads the password from a known source at deploy time, or (b) document that this ConfigMap must be regenerated after secret creation. Consider using an init container or a Kubernetes Job to populate this.
   - `REDIS_URL`: `redis://:<redis-password>@notifycore-redis-master.notifycore.svc:6379`
   - `PORT`: `8080`
   - `RUST_LOG`: `info`
3. An alternative approach: create the ConfigMap with static host/port info and mount secrets separately. The downstream Deployment can combine them. Document the chosen approach clearly.
4. Ensure the ConfigMap is created in the correct namespace and all four keys have non-empty values.

## Validation
`kubectl get configmap notifycore-infra-endpoints -n notifycore -o json` contains all four keys (DATABASE_URL, REDIS_URL, PORT, RUST_LOG) with non-empty values. DATABASE_URL contains the correct hostname `notifycore-pg-rw.notifycore.svc`. REDIS_URL contains `notifycore-redis-master.notifycore.svc`.