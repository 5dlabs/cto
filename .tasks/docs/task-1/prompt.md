Implement task 1: Dev Infrastructure Bootstrap (Bolt - Kubernetes/Helm)

## Goal
Provision the development infrastructure for NotifyCore: a single-replica PostgreSQL instance via CloudNativePG and a single-replica Redis instance, along with namespace, secrets, and a notifycore-infra-endpoints ConfigMap aggregating DATABASE_URL and REDIS_URL for downstream services.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
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

## Acceptance Criteria
1. `kubectl get pods -n notifycore` shows notifycore-pg and notifycore-redis pods in Running/Ready state within 120s. 2. A test Job in the namespace successfully connects to PostgreSQL (`SELECT 1` returns 1) using DATABASE_URL from the ConfigMap. 3. The same Job connects to Redis (`PING` returns `PONG`) using REDIS_URL from the ConfigMap. 4. ConfigMap `notifycore-infra-endpoints` exists and contains all four keys (DATABASE_URL, REDIS_URL, PORT, RUST_LOG) with non-empty values.

## Subtasks
- Create notifycore namespace and Helm chart scaffold: Create the `notifycore` Kubernetes namespace and initialize the Helm chart directory structure under `infra/notifycore/` with Chart.yaml, values-dev.yaml, and templates directory.
- Deploy CloudNativePG Cluster CR for PostgreSQL: Create the CloudNativePG `Cluster` CR named `notifycore-pg` with a single replica, database `notifycore`, user `notifycore_app`, and credentials stored in Secret `notifycore-pg-app`.
- Deploy single-replica Redis instance via Bitnami Helm chart: Deploy a single-replica Redis instance named `notifycore-redis` using the Bitnami Helm chart as a subchart dependency, with `requirepass` stored in Secret `notifycore-redis-auth`.
- Create notifycore-infra-endpoints ConfigMap: Create the `notifycore-infra-endpoints` ConfigMap aggregating DATABASE_URL, REDIS_URL, PORT, and RUST_LOG, dynamically referencing credentials from the PostgreSQL and Redis secrets.
- Create validation Job/Helm test for infrastructure connectivity: Create a Helm test Job that validates PostgreSQL and Redis connectivity using the ConfigMap-provided DATABASE_URL and REDIS_URL, confirming the infrastructure is fully operational.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.