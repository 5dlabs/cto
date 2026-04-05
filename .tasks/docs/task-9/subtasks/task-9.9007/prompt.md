Implement subtask 9007: Validate failover and rollback procedures with chaos testing

## Objective
Perform structured chaos testing to validate HA failover for PostgreSQL, Redis, backend services, and Cloudflare Tunnel, and document rollback procedures.

## Steps
1. Test PostgreSQL failover: delete the primary pod, verify automatic promotion, measure downtime, confirm data consistency with a read-after-write test. 2. Test Redis failover: delete the primary pod, verify sentinel promotion, confirm cache operations resume. 3. Test backend service resilience: delete one pod per Deployment, verify remaining replicas serve traffic, confirm HPA behavior under load. 4. Test cloudflared tunnel failover: delete one cloudflared pod, verify external access continues through the remaining pod. 5. Test node failure simulation: cordon and drain a node, verify PDBs protect availability, verify pods reschedule to other nodes. 6. Document each test scenario, expected behavior, observed behavior, and recovery time. 7. Write rollback procedures for each HA component (e.g., rolling back CloudNative-PG cluster to single instance if needed).

## Validation
Each chaos scenario must be executed at least once with documented results. All failovers must complete with less than 60 seconds of downtime. All rollback procedures must be tested and verified to return the system to a known-good state. A summary report must be produced covering all scenarios.