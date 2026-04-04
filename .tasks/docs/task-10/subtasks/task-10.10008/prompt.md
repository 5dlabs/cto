Implement subtask 10008: Create security scanning CronJob with Trivy

## Objective
Add a Kubernetes CronJob that runs Trivy (or equivalent vulnerability scanner) weekly against the cto-pm container image and alerts via the Discord bridge on HIGH/CRITICAL findings.

## Steps
Step-by-step:
1. Create `cronjobs/security-scan.yaml` defining a CronJob `sigma-1-security-scan` with schedule `0 2 * * 0` (weekly, Sunday 2am).
2. The Job container uses the `aquasec/trivy:latest` image (or a pinned version).
3. The Job command: `trivy image --severity HIGH,CRITICAL --exit-code 1 --format json <cto-pm-image>:<tag>`
4. If exit code is 1 (vulnerabilities found), a sidecar step or post-scan script sends an alert to the Discord bridge webhook with a summary (CVE count by severity).
5. Store the scan report as a Job log (captured by cluster logging).
6. Use the `sigma-1-pipeline-sa` ServiceAccount (ensure it has no extra privileges).
7. Set `successfulJobsHistoryLimit: 3` and `failedJobsHistoryLimit: 5`.
8. The image reference should be parameterized (via ConfigMap or Helm value) so it stays in sync with deployments.

## Validation
Run `kubectl get cronjob sigma-1-security-scan -n sigma-1` — exists with weekly schedule. Manually trigger with `kubectl create job --from=cronjob/sigma-1-security-scan test-scan -n sigma-1`. Verify the Job runs to completion. Check Job logs for a Trivy scan report (JSON output). If vulnerabilities are found, verify the Discord alert fires. If no vulnerabilities, verify the Job exits with code 0 and logs a clean report.