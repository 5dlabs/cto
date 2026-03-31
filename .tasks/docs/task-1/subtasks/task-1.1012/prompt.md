Implement subtask 1012: Create Helm Test Pods for Full Infrastructure Validation

## Objective
Create Helm test pod templates under templates/tests/ that verify Postgres connectivity, Redis PING, NATS JetStream, MinIO bucket read/write, ConfigMap key completeness, and Secret key completeness.

## Steps
Step-by-step:
1. Create `templates/tests/test-postgres.yaml`:
   - Annotation: `helm.sh/hook: test`
   - Image: `postgres:16-alpine`
   - Mount CNPG operator Secret `hermes-pg-app` for the connection URI.
   - Command: `psql "$POSTGRES_URI" -c 'SELECT 1'`
   - `restartPolicy: Never`
2. Create `templates/tests/test-redis.yaml`:
   - Image: `redis:7-alpine`
   - Mount `hermes-infra-secrets` for REDIS_URL.
   - Command: `redis-cli -u $REDIS_URL PING` expecting `PONG`.
3. Create `templates/tests/test-nats.yaml`:
   - Image: `natsio/nats-box:latest`
   - Mount `hermes-infra-secrets` for NATS_URL.
   - Command: `nats server check connection --server=$NATS_URL`
4. Create `templates/tests/test-minio.yaml`:
   - Image: `minio/mc:latest`
   - Mount `hermes-infra-secrets` for MINIO credentials and `hermes-infra-endpoints` via envFrom.
   - Commands: set alias, `mc cp /dev/stdin hermes/$MINIO_BUCKET/helm-test-object <<< 'test'`, `mc cat hermes/$MINIO_BUCKET/helm-test-object`, `mc rm hermes/$MINIO_BUCKET/helm-test-object`.
5. Create `templates/tests/test-configmap-keys.yaml`:
   - Image: `busybox:latest`
   - envFrom: configMapRef hermes-infra-endpoints
   - Script: check all 7 env vars are set and non-empty, exit 1 on any missing.
6. Create `templates/tests/test-secret-keys.yaml`:
   - Mount hermes-infra-secrets as env vars.
   - Script: verify all expected keys are non-empty.
7. All test pods: `restartPolicy: Never`, appropriate resource limits (100m CPU, 128Mi).

## Validation
`helm test hermes-infra -n hermes-staging` runs all 6 test pods and all complete with exit code 0. Pod logs contain confirmation: Postgres test shows '1', Redis test shows 'PONG', NATS test shows connection success, MinIO test shows successful put/get/delete, ConfigMap test shows 'All 7 keys present', Secret test shows 'All keys present'. Same for production.