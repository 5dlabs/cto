Implement subtask 10004: Configure audit log aggregation and forwarding to monitoring stack

## Objective
Forward Kubernetes audit logs to the existing log aggregation/monitoring stack for centralized querying, alerting, and long-term retention.

## Steps
1. Deploy a log forwarder (Fluent Bit DaemonSet or sidecar) configured to read from the audit log path or receive audit webhook events.
2. Configure the forwarder to parse audit log JSON, enrich with cluster metadata, and ship to the existing log aggregation system (e.g., Elasticsearch/Loki/CloudWatch).
3. Alternatively, if the API server supports audit webhook backend, configure `--audit-webhook-config-file` to send events directly to the log aggregation endpoint.
4. Create a dedicated index/stream for audit logs separate from application logs.
5. Set up retention policies (e.g., 90 days for audit logs).
6. Create basic alerting rules for high-severity events: unauthorized access attempts (403s on sensitive resources), secret access from unexpected ServiceAccounts, RBAC modifications outside of CI/CD.

## Validation
Verify audit log entries appear in the centralized logging system within 60 seconds of the event. Query for a known test event and confirm all fields are intact. Trigger an alert condition (e.g., access a secret from an unauthorized SA) and verify the alert fires.