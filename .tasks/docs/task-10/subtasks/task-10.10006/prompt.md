Implement subtask 10006: Create Grafana dashboards and Prometheus alerting rules for security monitoring

## Objective
Build Grafana dashboards for RBAC violations, secret rotation status, and audit log analysis. Create Prometheus alerting rules for security-relevant events such as failed auth attempts, secret expiry warnings, and unauthorized access patterns.

## Steps
1. Create a Grafana dashboard 'Security Overview' with panels:
   - Kubernetes audit event volume by verb and user (from Loki).
   - Failed authentication attempts over time.
   - Secret rotation status (last rotation timestamp per secret).
   - RBAC denial events.
2. Create a Grafana dashboard 'API Audit Trail' with panels:
   - Request volume per service and endpoint.
   - Error rate by service.
   - Top users/clients by request count.
3. Create Prometheus alerting rules (PrometheusRule CR):
   - Alert on >10 failed auth attempts in 5 minutes.
   - Alert when a secret has not been rotated in >35 days.
   - Alert on RBAC denied requests from service accounts.
4. Configure alert routing to the existing notification channel (Slack/email).
5. Store dashboards as JSON in the repo and provision via Grafana ConfigMap or dashboard provisioner.

## Validation
Verify dashboards load in Grafana with data populated. Trigger a test alert by simulating >10 failed auth attempts and verify the alert fires and notification is received. Verify secret rotation age metric is accurate for at least one secret.