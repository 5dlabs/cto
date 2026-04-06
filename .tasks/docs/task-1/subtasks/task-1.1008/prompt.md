Implement subtask 1008: Validate end-to-end connectivity from test pod to all services

## Objective
Deploy a test pod in the sigma1 namespace and validate connectivity to every provisioned service: PostgreSQL, Redis, S3/R2, Signal-CLI, and verify external API secret availability.

## Steps
1. Deploy a lightweight test pod (e.g., busybox or alpine with curl, psql, redis-cli, aws-cli) in the sigma1 namespace.
2. Mount the sigma1-infra-endpoints ConfigMap via envFrom.
3. Mount all external service secrets.
4. Test PostgreSQL connectivity: `psql -h $POSTGRES_HOST -p $POSTGRES_PORT -U <user> -c '\dn'` and verify 5 schemas.
5. Test Redis connectivity: `redis-cli -h $REDIS_HOST -p $REDIS_PORT -a <password> PING`.
6. Test S3/R2 connectivity: `aws s3 ls s3://$S3_PRODUCT_IMAGES_BUCKET --endpoint-url $S3_ENDPOINT`.
7. Test Signal-CLI connectivity: `curl $SIGNAL_CLI_URL/v1/about`.
8. Verify all external API secrets are mounted and contain non-empty values.
9. Clean up the test pod after validation.
10. Document results for downstream teams.

## Validation
All 4 service connectivity tests pass (PostgreSQL returns schemas, Redis returns PONG, S3 lists bucket, Signal-CLI returns about info); all external API secret keys are mounted and non-empty; test pod exits with code 0.