Implement subtask 9006: Failover testing and ConfigMap endpoint updates for HA services

## Objective
Perform comprehensive failover testing for all HA stateful services (PostgreSQL, Redis/Valkey) and update ConfigMap endpoints to reflect HA-aware service addresses. Document all production configurations.

## Steps
1. Validate the `{project}-infra-endpoints` ConfigMap contains HA-aware endpoints for PostgreSQL (read-write and read-only services) and Redis/Valkey (sentinel endpoint).
2. Perform PostgreSQL failover test: delete the primary pod, verify automatic promotion, verify application connectivity resumes, verify no data loss via a test write/read cycle.
3. Perform Redis/Valkey failover test: delete the primary pod, verify sentinel-triggered promotion, verify cache operations resume.
4. Test simultaneous failure of one PG replica and one Redis replica — verify no service degradation.
5. Measure and record Recovery Time Objective (RTO) for each failover scenario.
6. Document all production ingress routes, CDN config, HA topology, and failover procedures in a runbook.
7. Verify all application services reconnect automatically after failover without restart.

## Validation
All failover tests complete with RTO < 60s for PostgreSQL and < 30s for Redis; no data loss confirmed via test record integrity; all application services recover without manual pod restarts; runbook document is complete and reviewed.