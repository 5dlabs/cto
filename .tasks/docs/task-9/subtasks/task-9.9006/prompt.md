Implement subtask 9006: Test failover scenarios and document rollback procedures

## Objective
Execute structured failover tests for all HA services and document step-by-step rollback procedures for deployments, database migrations, and infrastructure changes.

## Steps
1) Pod failover test: For each service, delete one pod and measure time to recovery. Verify the remaining replica(s) handle traffic without errors during recovery. Record results. 2) Node failover test: Cordon and drain a node, verify all pods reschedule to other nodes, verify services remain available. Uncordon the node after test. 3) Tunnel failover test: Kill one cloudflared replica and verify the tunnel remains functional. 4) Rollback procedure documentation: For each service, document how to rollback a bad deployment using `kubectl rollout undo`. Document database migration rollback steps. Document how to revert Cloudflare DNS/tunnel changes. 5) Create a runbook with step-by-step instructions for common failure scenarios: service crash loop, database connection exhaustion, certificate expiry, tunnel failure. 6) Store all documentation in the repository under `docs/operations/`.

## Validation
Each failover test passes: pod deletion results in zero dropped requests (verified by continuous load test during deletion), node drain completes without service interruption, tunnel failover is seamless. Rollback procedure is validated by actually performing a rollback of a test deployment and confirming the previous version is restored. Runbook is reviewed by a second person for completeness and clarity.