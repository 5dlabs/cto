Implement subtask 10004: Configure application-level audit logging for API and database access

## Objective
Enable audit logging at the application level so that all API requests and database queries from services are logged with user identity, action, resource, and timestamp, and shipped to the centralized logging system.

## Steps
1) For each backend service, ensure structured access logs are emitted for every API request including: timestamp, user/session identifier, HTTP method, path, response status, latency, source IP. 2) For database access, enable query logging at the PostgreSQL level: set `log_statement = 'mod'` (or 'all' for initial audit, then tune down) and `log_connections = on`, `log_disconnections = on` in the PostgreSQL CR or ConfigMap. 3) For Redis, enable slow log and command logging if the security policy requires it. 4) Ensure all application logs are in structured JSON format for easy parsing. 5) Verify the log shipper (Fluent Bit/Vector) collects application-level audit logs and forwards them to the centralized logging backend alongside the Kubernetes audit logs. 6) Create log-based alerts for suspicious patterns: repeated 401/403 responses, unusual database query volumes, access from unexpected source IPs.

## Validation
Make authenticated API requests to each service and verify the access log entries appear in the centralized logging backend with correct user identity and request details. Execute database modifications and verify PostgreSQL logs capture the queries with timestamps. Verify log-based alerts fire by triggering the conditions (e.g., send 10 requests with invalid credentials and check if the alert triggers).