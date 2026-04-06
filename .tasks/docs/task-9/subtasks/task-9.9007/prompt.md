Implement subtask 9007: Full failover and recovery simulation testing

## Objective
Execute comprehensive failover tests for all critical services (PostgreSQL, Redis, backend, cloudflared) to validate HA configurations and document recovery procedures.

## Steps
1. PostgreSQL failover test: delete the primary pod, verify automatic promotion of a replica, confirm application reconnects and data integrity is maintained. 2. Redis failover test: delete the master pod, verify sentinel promotes a replica, confirm application reconnects. 3. Backend service failover: delete one pod of each backend service, verify requests continue to be served by remaining pods with no errors. 4. Cloudflared failover: delete one cloudflared pod, verify external traffic continues through the remaining pod. 5. Node drain simulation: cordon and drain a node, verify PDBs prevent full outage, all pods reschedule to other nodes. 6. Document each test scenario, expected behavior, actual behavior, and recovery time. 7. Create a runbook for each failure scenario.

## Validation
Each failover scenario must complete with zero downtime visible to end users (measured by continuous HTTP health checks during the test). Recovery time for each component must be under 30 seconds for Redis/backend and under 60 seconds for PostgreSQL. All tests must be documented with pass/fail status.