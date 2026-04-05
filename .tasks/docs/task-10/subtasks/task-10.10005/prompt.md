Implement subtask 10005: Enable application-level audit logging for all services

## Objective
Configure all backend services (Equipment Catalog, RMS, Finance, Vetting, Social, Morgan) to emit structured audit logs for API access events, and ship these logs to Loki for centralized analysis.

## Steps
1. Ensure each service emits structured JSON logs for API requests including: timestamp, user/client identity, action/endpoint, resource affected, result (success/failure), source IP.
2. If services use a shared logging library, add audit log fields at the middleware level.
3. Configure Promtail/log agent to scrape application logs and add labels (service, namespace, log_type=audit).
4. Verify audit log entries from each service appear in Loki with correct labels.
5. Create a Loki LogQL query template for common audit queries (e.g., all actions by a specific user, all failed requests in the last hour).

## Validation
Make an API call to each service and verify a corresponding audit log entry appears in Loki within 60 seconds, containing user identity, endpoint, and result. Verify failed/unauthorized requests are also logged with appropriate error codes.