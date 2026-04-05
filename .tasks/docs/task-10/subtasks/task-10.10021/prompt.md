Implement subtask 10021: Audit logging: configure Loki log collection with retention policies

## Objective
Configure Loki to collect logs from sigma1 and openclaw namespaces, with audit-tagged event filtering and differentiated retention (90 days standard, 1 year audit).

## Steps
Step-by-step:
1. Verify Loki and Promtail/Grafana Agent are deployed (from Task 1 or platform setup).
2. Configure Promtail/Grafana Agent to scrape logs from both `sigma1` and `openclaw` namespaces:
   - Add pipeline stages to parse structured JSON logs
   - Extract the `audit` field from JSON log entries and add it as a Loki label: `audit="true"` or `audit="false"`
3. Configure Loki retention policies:
   - In Loki's `loki.yaml` config (or via Helm values):
     ```yaml
     limits_config:
       retention_period: 2160h  # 90 days default
     compactor:
       retention_enabled: true
       retention_delete_delay: 2h
     ```
   - For audit logs (1 year retention), use Loki's per-stream retention:
     ```yaml
     overrides:
       sigma1:
         retention_stream:
           - selector: '{audit="true"}'
             priority: 1
             period: 8760h  # 1 year
     ```
4. Create a Grafana Explore saved query for audit log investigation: `{namespace=~"sigma1|openclaw", audit="true"}`.
5. Verify services are outputting structured JSON logs with the `audit` field on relevant events (login, data access, deletion).

## Validation
In Grafana Explore, query `{namespace="sigma1"}` and verify logs appear from all sigma1 services. Query `{namespace="sigma1", audit="true"}` and verify audit-tagged events appear (trigger a GDPR delete or login event). Verify Loki compactor config shows retention enabled. Check that logs older than 90 days are deleted for non-audit streams (may require waiting or checking compactor logs for retention plan execution).