## Acceptance Criteria

- [ ] 1. HA test: kill one CNPG replica pod, verify remaining replicas continue serving reads AND writes within 30 seconds (automatic failover). 2. Backup test: trigger manual backup, verify barman object appears in R2 bucket, restore to a test cluster, verify data integrity. 3. Tunnel test: curl https://sigma-1.com returns 200 with website content; curl https://api.sigma-1.com/catalog/health/ready returns 200. 4. CDN test: request image from assets.sigma-1.com, verify CF-Cache-Status header is HIT on second request. 5. Network policy test: exec into finance pod, attempt curl to social-engine pod — verify connection refused/timeout. Exec into Morgan pod, attempt curl to finance pod — verify success. 6. HPA test: generate load on Equipment Catalog (e.g., 50 concurrent requests/sec), verify pod count scales from 2 to 3+ within 2 minutes. 7. PDB test: attempt to drain node hosting 1 of 2 equipment-catalog pods, verify at least 1 pod remains running. 8. Grafana dashboard test: all 4 dashboards load without errors and display data from sigma1 services.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.