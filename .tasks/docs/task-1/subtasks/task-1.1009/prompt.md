Implement subtask 1009: Deploy smoke test Pod to validate end-to-end infrastructure connectivity

## Objective
Create and run a smoke test Pod in `sigma-1-dev` that mounts the `sigma-1-infra-endpoints` ConfigMap via envFrom, uses the `sigma-1-pm-sa` ServiceAccount, connects to Postgres (SELECT 1), pings Redis (PING→PONG), and resolves both bridge service DNS names. The Pod must exit 0 on success.

## Steps
1. Create a Pod manifest: name=`sigma-1-smoke-test`, namespace=`sigma-1-dev`, serviceAccountName=`sigma-1-pm-sa`, restartPolicy=Never.
2. Mount ConfigMap `sigma-1-infra-endpoints` via `envFrom: [{configMapRef: {name: sigma-1-infra-endpoints}}]`.
3. Also mount the relevant secrets for Postgres credentials (from CNPG app secret) as environment variables.
4. Use an image with psql and redis-cli (e.g., a custom alpine image with postgresql-client and redis packages, or use a multi-step shell script with appropriate tools).
5. Write an entrypoint script that: (a) runs `psql $CNPG_SIGMA1_PG_URL -c 'SELECT 1'` and asserts exit code 0, (b) runs `redis-cli -u $REDIS_SIGMA1_URL ping` and asserts output contains PONG, (c) runs `nslookup` or `getent hosts` for the hostnames in `DISCORD_BRIDGE_URL` and `LINEAR_BRIDGE_URL` and asserts resolution succeeds, (d) exits 0 if all checks pass, exits 1 with diagnostic output otherwise.
6. Apply the Pod manifest.
7. Wait for Pod completion (timeout 2 minutes).
8. Check Pod exit code and logs.
9. Clean up the Pod after validation.

## Validation
`kubectl get pod sigma-1-smoke-test -n sigma-1-dev -o jsonpath='{.status.phase}'` returns `Succeeded`. `kubectl logs sigma-1-smoke-test -n sigma-1-dev` shows all four checks passing: Postgres SELECT 1 returns 1, Redis PING returns PONG, both bridge DNS names resolve. Pod exit code is 0.