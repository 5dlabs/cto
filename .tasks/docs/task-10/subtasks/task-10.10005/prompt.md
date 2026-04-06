Implement subtask 10005: Configure log shipping from audit logs to Grafana/Loki stack

## Objective
Set up log collection agents to ship Kubernetes audit logs and critical service logs to the existing Loki instance, and create Grafana dashboards for audit visibility.

## Steps
1. Deploy Promtail (or Grafana Alloy/Agent) as a DaemonSet to collect logs from all nodes if not already present. 2. Configure Promtail to scrape the Kubernetes audit log file path and label logs with `job=kube-audit`. 3. Add Promtail pipeline stages to parse the JSON audit log format and extract fields: user, verb, resource, namespace, responseStatus. 4. Configure additional Promtail scrape configs for critical service stdout/stderr logs with appropriate labels. 5. Verify logs appear in Loki by querying `{job="kube-audit"}` in Grafana Explore. 6. Create a Grafana dashboard with panels: a) Audit events over time (grouped by verb). b) Failed authentication/authorization attempts. c) Secret access events. d) Pod exec/attach events. e) RBAC change events. 7. Set up Grafana alerts for critical audit events (e.g., unauthorized access attempts > threshold).

## Validation
Query Loki via Grafana for `{job="kube-audit"}` and confirm audit events from the last hour are present. Verify the dashboard renders all panels with real data. Trigger a test alert condition (e.g., multiple failed auth attempts) and confirm the alert fires. Verify log latency: perform an action and confirm it appears in Loki within 60 seconds.