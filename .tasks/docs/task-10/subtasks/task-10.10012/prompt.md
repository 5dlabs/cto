Implement subtask 10012: Create secret rotation CronJob and ops runbook

## Objective
Create a Kubernetes CronJob named secret-rotation-reminder that runs monthly (schedule: '0 9 1 * *'). The job logs a rotation reminder message to stdout (picked up by Loki) listing all secrets requiring rotation: stripe, opencorporates, linkedin, google, elevenlabs, twilio, openai, cloudflare. Write the rotation runbook at ops/runbooks/secret-rotation.md.

## Steps
Create helm/sigma1/templates/cronjob-secret-rotation.yaml: `apiVersion: batch/v1, kind: CronJob, metadata.name: secret-rotation-reminder, spec.schedule: '0 9 1 * *', spec.jobTemplate.spec.template.spec.containers[0].image: busybox, spec.jobTemplate.spec.template.spec.containers[0].command: ['sh', '-c', 'echo "[SECRET ROTATION REMINDER] Monthly rotation required for: stripe-api-key, opencorporates-api-key, linkedin-client-secret, google-service-account, elevenlabs-api-key, twilio-auth-token, openai-api-key, cloudflare-tunnel-token. See ops/runbooks/secret-rotation.md"']`. Set `restartPolicy: Never`. Create ops/runbooks/secret-rotation.md with step-by-step instructions: 1) Obtain new secret value from provider, 2) `kubectl create secret generic <name> --from-literal=key=<value> -n sigma1 --dry-run=client -o yaml | kubectl apply -f -`, 3) Trigger rolling restart of affected deployment, 4) Verify service health.

## Validation
Manually trigger the CronJob with `kubectl create job --from=cronjob/secret-rotation-reminder test-rotation -n sigma1`. Check `kubectl logs -n sigma1 job/test-rotation` — log line contains all 8 secret names. Loki query `{namespace='sigma1', job='test-rotation'}` returns the log entry. Confirm ops/runbooks/secret-rotation.md exists and contains all 8 secret names with rotation steps.