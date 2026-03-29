Implement subtask 1005: Create validation Job/Helm test for infrastructure connectivity

## Objective
Create a Helm test Job that validates PostgreSQL and Redis connectivity using the ConfigMap-provided DATABASE_URL and REDIS_URL, confirming the infrastructure is fully operational.

## Steps
1. Create `infra/notifycore/templates/tests/test-connectivity.yaml` as a Helm test Pod (annotation `helm.sh/hook: test`).
2. The test pod should use a lightweight image (e.g., `bitnami/postgresql` or a custom alpine with psql and redis-cli).
3. Mount/envFrom `notifycore-infra-endpoints` ConfigMap and relevant secrets.
4. Run two commands:
   a. `pg_isready -h notifycore-pg-rw.notifycore.svc -p 5432 -U notifycore_app` and then `psql $DATABASE_URL -c 'SELECT 1'` — expect exit code 0.
   b. `redis-cli -u $REDIS_URL PING` — expect output `PONG`.
5. Pod restartPolicy: Never. Set a timeout via activeDeadlineSeconds: 60.
6. Test can be run with `helm test notifycore -n notifycore`.

## Validation
`helm test notifycore -n notifycore` completes successfully with exit code 0. Pod logs show `SELECT 1` returning `1` and Redis PING returning `PONG`. The test pod reaches Succeeded status.