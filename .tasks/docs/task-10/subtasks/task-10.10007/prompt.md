Implement subtask 10007: Configure log shipping to cluster logging infrastructure

## Objective
Ensure audit logs (both Kubernetes-level and application-level) from the sigma-1 namespace are shipped to the cluster's central logging infrastructure (EFK, Loki, or equivalent) with appropriate labels and retention.

## Steps
Step-by-step:
1. Identify the cluster's logging stack (EFK, Loki+Promtail, CloudWatch, Stackdriver, etc.).
2. If using Promtail/Loki: verify the Promtail DaemonSet is scraping pods in the sigma-1 namespace. Add pipeline stages to parse JSON audit logs and add labels (`namespace=sigma-1`, `log_type=audit`).
3. If using Fluentd/EFK: verify Fluentd config includes the sigma-1 namespace. Add a filter to parse structured JSON logs and route audit events to a dedicated index.
4. For Kubernetes audit logs: ensure the audit log backend (webhook or log file) feeds into the same logging stack.
5. Verify log retention meets organizational requirements (minimum 90 days for audit logs recommended).
6. Create a basic dashboard or saved search for sigma-1 audit events (optional but recommended).

## Validation
Query the logging backend (Kibana, Grafana/Loki, CloudWatch Logs Insights) for logs from namespace=sigma-1 with event_type=issue_created within the last hour. Verify results appear. Confirm Kubernetes audit events for sigma-1 Secret access also appear in the logging backend. Check that log retention policy is configured (not relying on defaults).