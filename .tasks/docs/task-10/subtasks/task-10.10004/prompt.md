Implement subtask 10004: Create rotation validation CronJob with Discord alerting

## Objective
Create a Kubernetes CronJob that runs daily to validate ExternalSecret sync status, verify rotated secrets are non-empty, and alert via the Discord bridge if any rotation has failed.

## Steps
Step-by-step:
1. Create `cronjobs/rotation-validation.yaml` defining a CronJob `sigma-1-rotation-validator` with schedule `0 8 * * *` (daily at 8am, adjustable).
2. The Job container runs a shell script or lightweight image (e.g., bitnami/kubectl) that:
   a. Lists all ExternalSecret resources in sigma-1 namespace.
   b. For each, checks `.status.conditions[?(@.type=="Ready")].status == True`.
   c. For each corresponding Secret, checks that all data keys are non-empty.
   d. If any check fails, sends an alert payload to the Discord bridge webhook URL (read from the sigma-1-infra-endpoints ConfigMap or a Secret).
3. The Job should use the `sigma-1-pipeline-sa` ServiceAccount (or a dedicated SA with read-only RBAC for ExternalSecrets and Secrets).
4. Set `successfulJobsHistoryLimit: 3` and `failedJobsHistoryLimit: 3`.
5. Set `restartPolicy: OnFailure` with `backoffLimit: 2`.

## Validation
Run `kubectl get cronjob sigma-1-rotation-validator -n sigma-1` — exists with correct schedule. Manually trigger with `kubectl create job --from=cronjob/sigma-1-rotation-validator test-rotation -n sigma-1`. Verify the Job completes successfully. Check Job logs for validation output covering all ExternalSecrets. To test alerting, temporarily break an ExternalSecret (e.g., invalid key path) and re-trigger — verify Discord alert is received.