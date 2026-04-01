Implement subtask 10005: Implement automated secret rotation via CronJob with rolling restart triggers

## Objective
Create CronJob resources that rotate each pipeline secret on its defined schedule and trigger rolling restarts of affected Deployments.

## Steps
1. Create `infra/secret-rotation/` directory.
2. Create a CronJob manifest for each secret rotation schedule:
   a. `rotate-linear-api-token.yaml`: schedule `0 0 1 */3 *` (every 90 days). The job should:
      - Call the appropriate API or generate a new token (placeholder script if external provider unknown).
      - Update the Kubernetes secret: `kubectl create secret generic linear-api-token --from-literal=token=<new-value> --dry-run=client -o yaml | kubectl apply -f -`.
      - Trigger rolling restart: `kubectl rollout restart deployment/pm-server -n sigma1-prod`.
   b. `rotate-github-pat.yaml`: schedule `0 0 1 */3 *` (every 90 days), same pattern for `github-pat` secret.
   c. `rotate-discord-webhook-url.yaml`: schedule `0 0 1 */6 *` (every 180 days), same pattern for `discord-webhook-url` secret.
   d. `rotate-nous-api-key.yaml`: schedule `0 0 1 */3 *` (every 90 days), same pattern for `nous-api-key` secret.
3. Each CronJob should use a ServiceAccount with permissions to update secrets and restart deployments in `sigma1-prod` only.
4. Create a dedicated `sa-secret-rotator` ServiceAccount, Role (`secret-rotator-role` with update secrets + patch deployments), and RoleBinding.
5. Apply all manifests.
6. Verify CronJobs are registered: `kubectl get cronjobs -n sigma1-prod`.

## Validation
Manually trigger one rotation CronJob: `kubectl create job --from=cronjob/rotate-linear-api-token test-rotation -n sigma1-prod`. Verify the secret value changed by comparing before/after base64 values. Verify affected deployment pods restarted by checking pod creation timestamps are newer than the job completion time.