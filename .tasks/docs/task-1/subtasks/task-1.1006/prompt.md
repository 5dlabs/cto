Implement subtask 1006: Implement secret-validation-job with conditional logic for optional NOUS_API_KEY

## Objective
Create a Kubernetes Job that waits for ExternalSecrets to sync, validates that all required secrets are non-empty, warns (but does not fail) if the optional NOUS_API_KEY is absent, and fails clearly if any required secret is missing.

## Steps
1. Create `secret-validation-job.yaml` defining a Kubernetes Job:
   - `metadata.name`: `secret-validation-job`
   - `metadata.namespace`: `sigma-1`
   - `metadata.labels`: include `sigma-1-pipeline: infra`
   - `spec.backoffLimit`: 3
   - `spec.ttlSecondsAfterFinished`: 3600
   - `spec.template.spec.restartPolicy`: `Never`
   - `spec.template.spec.serviceAccountName`: use a SA with read access to secrets in sigma-1 namespace
2. Container image: use `bitnami/kubectl` or a lightweight image with `kubectl` and `sh`.
3. Container script logic:
   ```sh
   #!/bin/sh
   set -e
   TIMEOUT=60
   INTERVAL=5
   ELAPSED=0
   REQUIRED_SECRETS="sigma-1-linear-token sigma-1-discord-webhook sigma-1-github-token"
   OPTIONAL_SECRETS="sigma-1-nous-api-key"
   
   # Wait for required secrets to appear
   for secret in $REQUIRED_SECRETS; do
     while [ $ELAPSED -lt $TIMEOUT ]; do
       VALUE=$(kubectl get secret $secret -n sigma-1 -o jsonpath='{.data}' 2>/dev/null || echo "")
       if [ -n "$VALUE" ] && [ "$VALUE" != "{}" ]; then
         echo "OK: $secret is populated"
         break
       fi
       echo "Waiting for $secret..."
       sleep $INTERVAL
       ELAPSED=$((ELAPSED + INTERVAL))
     done
     if [ $ELAPSED -ge $TIMEOUT ]; then
       echo "FAIL: $secret did not resolve within ${TIMEOUT}s"
       exit 1
     fi
     ELAPSED=0
   done
   
   # Check optional secrets with warning only
   for secret in $OPTIONAL_SECRETS; do
     VALUE=$(kubectl get secret $secret -n sigma-1 -o jsonpath='{.data}' 2>/dev/null || echo "")
     if [ -z "$VALUE" ] || [ "$VALUE" = "{}" ]; then
       echo "WARN: $secret is empty or absent — pipeline will skip NOUS-dependent features (D8 graceful skip)"
     else
       echo "OK: $secret is populated"
     fi
   done
   
   echo "All required secrets validated successfully."
   exit 0
   ```
4. Create a ServiceAccount and RoleBinding granting get/list on secrets in sigma-1 if not already available.
5. Apply with `kubectl apply -f secret-validation-job.yaml -n sigma-1`.
6. Wait for Job completion: `kubectl wait --for=condition=complete job/secret-validation-job -n sigma-1 --timeout=120s`.

## Validation
Job completes with exit code 0: `kubectl get job secret-validation-job -n sigma-1 -o jsonpath='{.status.succeeded}'` returns 1. Job logs (`kubectl logs job/secret-validation-job -n sigma-1`) contain 'OK' lines for linear-token, discord-webhook, and github-token. If NOUS_API_KEY is absent, logs contain the 'WARN' line but job still exits 0.