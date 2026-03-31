Implement subtask 9009: Validate failover and rollback scenarios

## Objective
Run structured failover and rollback tests to verify HA configurations work correctly under failure conditions. Document runbooks for operational incident response.

## Steps
1. PostgreSQL failover test:
   - Kill the primary PostgreSQL pod (`kubectl delete pod`)
   - Measure time to failover and confirm automatic promotion
   - Verify application services recover and data integrity is maintained
   - Test manual switchover back to original primary
2. Redis failover test:
   - Kill the Redis master pod
   - Confirm Sentinel promotes a replica
   - Verify application sessions/cache are preserved or gracefully rebuilt
3. Cloudflare Tunnel failover test:
   - Kill one cloudflared replica
   - Confirm zero downtime on external endpoints
   - Kill all replicas and verify graceful degradation
4. Rollback test:
   - Deploy a broken application version via Helm
   - Execute `helm rollback` and confirm services restore to previous version
   - Verify database migrations are backward-compatible or have down migrations
5. Document each scenario as a runbook with steps, expected outcomes, and recovery procedures.
6. Record all test results with timestamps and outcomes.

## Validation
All failover tests pass with services remaining available (or recovering within documented SLO). Rollback restores previous working state within 5 minutes. Runbook documentation is complete and reviewed. No data loss occurs during any failover scenario.