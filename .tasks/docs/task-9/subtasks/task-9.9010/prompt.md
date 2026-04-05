Implement subtask 9010: Tune resource requests and limits for all services

## Objective
Review Prometheus metrics from the dev deployment and set appropriate production resource requests and limits for all 6 backend services and the frontend.

## Steps
1. Query Prometheus for current resource usage of each service:
   - `container_memory_working_set_bytes{namespace='sigma1'}` for memory
   - `rate(container_cpu_usage_seconds_total{namespace='sigma1'}[5m])` for CPU
2. Set resource requests to approximately the p95 observed usage, limits to 1.5-2x requests:
   - Equipment Catalog: requests 256Mi/250m, limits 512Mi/500m (adjust based on metrics)
   - RMS: requests 256Mi/250m, limits 512Mi/500m
   - Finance: requests 256Mi/250m, limits 512Mi/500m
   - Social Engine: requests 256Mi/250m, limits 512Mi/500m
   - Morgan: requests 512Mi/500m, limits 1Gi/1000m (WebSocket connections are memory-intensive)
   - Customer Vetting: requests 128Mi/125m, limits 256Mi/250m
   - Website (frontend): requests 256Mi/250m, limits 512Mi/500m
3. Update each Deployment spec with the tuned values.
4. Apply changes via rolling update and monitor for OOMKills or CPU throttling.
5. Document the chosen values and the metrics they were based on.

## Validation
After applying tuned limits, monitor for 30 minutes: verify no OOMKilled events (`kubectl get events -n sigma1 | grep OOM`), verify no excessive CPU throttling via `container_cpu_cfs_throttled_seconds_total`, verify all pods remain in Running state with no restarts.