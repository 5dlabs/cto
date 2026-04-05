Implement subtask 10003: Create CronJob for JWT token rotation with graceful rollover

## Objective
Implement a Kubernetes CronJob that runs every 60 days (30 days before token expiry) to regenerate all service JWT tokens, update Secrets, and trigger rolling restarts of services to pick up new tokens without downtime.

## Steps
1. Create a CronJob `jwt-token-rotation` scheduled to run every 60 days (tokens expire at 90 days, rotation at 60 days provides 30-day overlap).
2. The CronJob runs the same token generation logic as the deploy-time Job (can reuse the same container image).
3. After updating all 6 token Secrets, the CronJob triggers rolling restarts of all Deployments by patching an annotation (e.g., `kubectl rollout restart deployment/<name>` for each service).
4. The ServiceAccount for this CronJob needs permissions to: update Secrets, patch Deployments (for rollout restart).
5. Add a ConfigMap annotation or label with last-rotation-timestamp for observability.
6. Ensure graceful rollover: during the rolling restart, both old and new tokens should be valid (the old token has 30 more days of validity, and verifiers only check signature + expiry, so this is inherently safe).
7. Add `concurrencyPolicy: Forbid` and `successfulJobsHistoryLimit: 3`.

## Validation
Trigger the CronJob manually. Verify all 6 token Secrets are updated (compare token values before and after). Verify all service Deployments perform rolling restarts. Decode new tokens and confirm valid claims. Verify old tokens are still valid (not yet expired). Verify services remain available during rotation (zero-downtime).