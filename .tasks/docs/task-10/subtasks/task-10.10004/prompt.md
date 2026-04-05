Implement subtask 10004: Integrate audit logs and service logs with centralized Loki/Grafana logging

## Objective
Ship Kubernetes API audit logs and managed service logs (PostgreSQL, Redis, operators) into the Loki centralized logging stack and create Grafana dashboards for security monitoring.

## Steps
1. Deploy or configure Promtail/Grafana Agent to scrape Kubernetes audit log files (or receive them via webhook/fluentd).
2. Configure Promtail with pipeline stages to parse audit log JSON, extracting labels: `verb`, `user`, `resource`, `namespace`, `responseStatus`.
3. Add scrape targets for PostgreSQL operator logs, Redis/Valkey operator logs, and cloudflared logs.
4. Create Loki log queries for common security events:
   - Failed authentication attempts
   - Secret access events
   - RBAC changes (Role/ClusterRole/Binding modifications)
   - Pod exec events
5. Create a Grafana dashboard with panels for: audit event volume over time, failed auth attempts, RBAC change log, secret access log.
6. Create Grafana alerting rules for suspicious patterns (e.g., > 10 failed auth attempts in 5 minutes, unexpected secret access).
7. Verify end-to-end log flow from event to Grafana dashboard.

## Validation
Trigger a known audit event and verify it appears in Loki within 60s; verify Grafana dashboard panels populate with real data; trigger a simulated suspicious pattern and confirm Grafana alert fires; verify all managed service logs (PG, Redis, cloudflared) are queryable in Loki.