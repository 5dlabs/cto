Implement subtask 10005: Enable audit logging for managed services (PostgreSQL, Redis)

## Objective
Configure application-level audit logging for PostgreSQL and Redis to capture data access and administrative operations.

## Steps
1. For PostgreSQL (CloudNative-PG): a) Enable `pgaudit` extension by adding it to the Cluster CR `postgresql.shared_preload_libraries`. b) Configure `pgaudit.log` parameter to capture DDL, ROLE, and WRITE operations at minimum. c) Set `pgaudit.log_catalog = off` to reduce noise. d) Verify audit entries appear in PostgreSQL logs. 2. For Redis: a) Enable `slowlog` with an appropriate threshold (e.g., 10ms) to capture slow operations. b) If the Redis operator supports it, enable `ACL LOG` to capture authentication failures and unauthorized command attempts. c) Configure Redis `loglevel` to `notice` or `verbose` for production monitoring. 3. Ensure all managed service logs are captured by the cluster's log shipping infrastructure (Fluent Bit or similar). 4. Create a dashboard or saved queries for common audit queries (e.g., 'show all DDL operations in last 24h').

## Validation
Execute a DDL statement (e.g., `ALTER TABLE`) in PostgreSQL and verify it appears in pgaudit logs. Execute a `CONFIG SET` command in Redis and verify it's logged. Confirm both service logs are forwarded to the centralized logging backend. Query the logging backend for PostgreSQL audit events and verify results are returned.