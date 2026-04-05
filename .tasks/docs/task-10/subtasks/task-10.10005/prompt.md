Implement subtask 10005: Enable audit logging for managed services (PostgreSQL, Redis)

## Objective
Configure audit logging within PostgreSQL and Redis to capture data access, administrative operations, and authentication events for compliance and forensics.

## Steps
1. **PostgreSQL audit logging**:
   a. Enable `pgaudit` extension in the PostgreSQL operator CR.
   b. Configure `pgaudit.log` to capture: `READ`, `WRITE`, `DDL`, `ROLE` events.
   c. Set `log_connections = on` and `log_disconnections = on`.
   d. Configure `log_statement = 'ddl'` for DDL statement logging.
   e. Ensure PostgreSQL logs are collected by the cluster's log aggregation pipeline.
2. **Redis audit logging**:
   a. Enable `acllog-max-len` for ACL violation logging.
   b. Configure Redis slowlog with appropriate threshold.
   c. If the Redis operator supports it, enable command logging for administrative commands (CONFIG, FLUSHALL, etc.).
   d. Ensure Redis logs are collected by the log aggregation pipeline.
3. Verify logs include timestamps, client IPs, usernames, and operations.
4. Document what events are logged and where they are stored.

## Validation
Execute test queries against PostgreSQL (SELECT, INSERT, CREATE TABLE, ALTER ROLE) and verify each appears in pgaudit logs with correct metadata. Execute Redis commands (SET, GET, CONFIG, ACL) and verify relevant events appear in Redis logs. Confirm logs are forwarded to the central log aggregation system.