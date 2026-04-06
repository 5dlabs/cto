Implement subtask 7011: Implement Signal account rotation and health monitoring

## Objective
Configure secondary Signal number as failover, implement monitoring for account ban signals (HTTP 403/rate limiting), and set up Grafana alerts for Signal-CLI health degradation.

## Steps
1. Secondary Signal number configuration:
   a. Register a secondary Signal number with Signal-CLI.
   b. Store both primary and secondary credentials in Kubernetes secrets.
   c. Configure Morgan to use primary by default, with ability to switch to secondary.
2. Ban detection:
   a. Monitor Signal-CLI REST API responses for HTTP 403, 429, or connection refusal patterns.
   b. Track message send success/failure rate over a sliding window (e.g., 5 minutes).
   c. If failure rate exceeds threshold (e.g., 50% over 5 minutes), flag primary account as potentially banned.
3. Failover logic:
   a. On ban detection, automatically switch outbound messages to secondary number.
   b. Notify Mike via secondary number that primary account may be banned.
   c. Log failover event for audit trail.
   d. Do NOT automatically fail back — require manual confirmation from Mike.
4. Grafana monitoring:
   a. Export metrics: signal_messages_sent_total, signal_messages_failed_total, signal_cli_health_status.
   b. Create Grafana alert rule: fire when signal_messages_failed_total rate > 0.5 for 5 minutes.
   c. Alert notification to Mike via secondary Signal number or email.
5. Signal-CLI sidecar health:
   a. Liveness probe already configured in 7010.
   b. Add startup probe with longer timeout for Signal-CLI registration/sync on cold start.

## Validation
Simulate primary account failure by blocking Signal-CLI API responses (return 403). Verify ban detection triggers within the configured window. Verify failover switches to secondary number and Mike receives notification. Verify Grafana alert fires. Verify manual fail-back works when primary is restored.