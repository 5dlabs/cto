Implement subtask 1008: Verify cross-namespace service health checks from sigma-1

## Objective
Run health check probes from within the sigma-1 namespace to confirm that discord-bridge-http, linear-bridge, and cto-pm services in their respective namespaces are reachable and responding.

## Steps
1. Create a one-shot Job or Pod manifest `health-check-job.yaml`:
   - Uses a lightweight image with `curl` or `wget` (e.g., `curlimages/curl`)
   - Runs in sigma-1 namespace
   - Labels: `sigma-1-pipeline: infra`
2. Script logic:
   ```sh
   #!/bin/sh
   set -e
   ENDPOINTS="http://discord-bridge-http.bots.svc.cluster.local http://linear-bridge.bots.svc.cluster.local http://cto-pm.cto.svc.cluster.local"
   FAIL=0
   for url in $ENDPOINTS; do
     HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' --connect-timeout 10 --max-time 15 "$url/health" || echo "000")
     if [ "$HTTP_CODE" = "200" ]; then
       echo "OK: $url/health returned 200"
     else
       echo "FAIL: $url/health returned $HTTP_CODE"
       FAIL=1
     fi
   done
   if [ $FAIL -eq 1 ]; then
     echo "One or more health checks failed."
     exit 1
   fi
   echo "All health checks passed."
   exit 0
   ```
3. Note: The health endpoint path may vary per service (could be `/health`, `/healthz`, `/readyz`). Adjust as needed based on actual service implementations.
4. Apply and wait for completion: `kubectl apply -f health-check-job.yaml -n sigma-1 && kubectl wait --for=condition=complete job/health-check-job -n sigma-1 --timeout=60s`.

## Validation
Health check job completes with exit code 0. Job logs show 'OK' for all three endpoints: discord-bridge-http, linear-bridge, and cto-pm. If any service is unreachable, the job fails with a clear error identifying which service and what HTTP status was returned.