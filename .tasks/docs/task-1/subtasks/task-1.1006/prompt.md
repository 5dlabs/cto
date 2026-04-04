Implement subtask 1006: Validate cross-namespace connectivity from sigma-1-dev to existing services

## Objective
Deploy a temporary test pod in sigma-1-dev namespace to verify DNS resolution and HTTP connectivity to discord-bridge-http, linear-bridge, and openclaw-nats services.

## Steps
1. Create a Job manifest `connectivity-test.yaml` that runs a busybox/curl container in `sigma-1-dev` namespace.
2. The Job should execute DNS lookups for:
   - `discord-bridge-http.bots.svc.cluster.local`
   - `linear-bridge.bots.svc.cluster.local`
   - `openclaw-nats.openclaw.svc.cluster.local`
3. For HTTP services (discord-bridge-http, linear-bridge), attempt a basic HTTP health check (e.g., `curl -sf http://<service>:<port>/health` or just verify TCP connectivity).
4. For NATS, verify TCP connectivity on port 4222.
5. The Job should exit 0 only if all DNS resolutions succeed and all connectivity checks pass.
6. Log results clearly for debugging if any check fails.
7. Clean up the Job after validation (or set `ttlSecondsAfterFinished`).

## Validation
The connectivity test Job completes with exit code 0. `kubectl logs job/sigma-1-connectivity-test -n sigma-1-dev` shows successful DNS resolution for all 3 services and successful TCP/HTTP connectivity checks. No DNS NXDOMAIN or connection refused errors in the logs.