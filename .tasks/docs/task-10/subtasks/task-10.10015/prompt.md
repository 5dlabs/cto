Implement subtask 10015: GDPR deletion orchestrator: create Dockerfile and Kubernetes Job template

## Objective
Package the GDPR orchestrator CLI as a Docker image and create a Kubernetes Job manifest template that Morgan can trigger via MCP tool.

## Steps
Step-by-step:
1. Create `crates/gdpr-orchestrator/Dockerfile`:
   - Multi-stage build: Rust builder stage compiles the binary, final stage uses `gcr.io/distroless/cc-debian12` or `alpine` with the static binary.
   - Copy only the compiled binary.
   - Set ENTRYPOINT to the binary.
2. Create `k8s/gdpr-orchestrator-job.yaml`:
   ```yaml
   apiVersion: batch/v1
   kind: Job
   metadata:
     generateName: gdpr-delete-
     namespace: sigma1
   spec:
     ttlSecondsAfterFinished: 3600
     backoffLimit: 1
     template:
       spec:
         serviceAccountName: sa-gdpr-orchestrator
         restartPolicy: Never
         containers:
           - name: gdpr-orchestrator
             image: <registry>/gdpr-orchestrator:latest
             args: ["--customer-id", "$(CUSTOMER_ID)"]
             envFrom:
               - configMapRef:
                   name: sigma1-infra-endpoints
             env:
               - name: DATABASE_URL
                 valueFrom:
                   secretKeyRef:
                     name: sigma1-db-app
                     key: uri
         securityContext:
           runAsNonRoot: true
           readOnlyRootFilesystem: true
   ```
3. Document how Morgan's MCP tool `sigma1_gdpr_delete` should create this Job with the customer ID substituted.
4. Create the `audit.gdpr_deletions` table migration SQL in the shared migrations directory.

## Validation
Build the Docker image locally. Run `docker run --rm <image> --help` and verify output. Apply the Job manifest to the cluster with a test customer ID. Verify the Job completes (or fails gracefully if services aren't running). Check `kubectl get jobs` shows the job with the expected status. Verify the `audit.gdpr_deletions` table exists after running migrations.