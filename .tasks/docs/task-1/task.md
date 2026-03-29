## Dev Infrastructure Bootstrap (Bolt - Kubernetes/Helm)

### Objective
Provision the development infrastructure for NotifyCore: a single-replica PostgreSQL instance via CloudNativePG and a single-replica Redis instance, along with namespace, secrets, and a notifycore-infra-endpoints ConfigMap aggregating DATABASE_URL and REDIS_URL for downstream services.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. Create a dedicated `notifycore` namespace.
2. Deploy a CloudNativePG `Cluster` CR named `notifycore-pg` with a single replica, a database `notifycore`, and a user `notifycore_app`. Store generated credentials in a Secret `notifycore-pg-app`.
3. Deploy a single-replica Redis instance (Bitnami Helm chart or Redis Operator CR) named `notifycore-redis` with `requirepass` stored in Secret `notifycore-redis-auth`.
4. Create ConfigMap `notifycore-infra-endpoints` with keys:
   - `DATABASE_URL`: `postgres://notifycore_app:<password>@notifycore-pg-rw.notifycore.svc:5432/notifycore`
   - `REDIS_URL`: `redis://:<password>@notifycore-redis-master.notifycore.svc:6379`
   - `PORT`: `8080`
   - `RUST_LOG`: `info`
5. Validate all pods reach Ready state and connectivity via a Helm test or Job that runs `pg_isready` and `redis-cli PING`.
6. Output a Helm chart under `infra/notifycore/` with `values-dev.yaml` for single-replica sizing.

### Subtasks
- [ ] Create notifycore namespace and Helm chart scaffold: Create the `notifycore` Kubernetes namespace and initialize the Helm chart directory structure under `infra/notifycore/` with Chart.yaml, values-dev.yaml, and templates directory.
- [ ] Deploy CloudNativePG Cluster CR for PostgreSQL: Create the CloudNativePG `Cluster` CR named `notifycore-pg` with a single replica, database `notifycore`, user `notifycore_app`, and credentials stored in Secret `notifycore-pg-app`.
- [ ] Deploy single-replica Redis instance via Bitnami Helm chart: Deploy a single-replica Redis instance named `notifycore-redis` using the Bitnami Helm chart as a subchart dependency, with `requirepass` stored in Secret `notifycore-redis-auth`.
- [ ] Create notifycore-infra-endpoints ConfigMap: Create the `notifycore-infra-endpoints` ConfigMap aggregating DATABASE_URL, REDIS_URL, PORT, and RUST_LOG, dynamically referencing credentials from the PostgreSQL and Redis secrets.
- [ ] Create validation Job/Helm test for infrastructure connectivity: Create a Helm test Job that validates PostgreSQL and Redis connectivity using the ConfigMap-provided DATABASE_URL and REDIS_URL, confirming the infrastructure is fully operational.