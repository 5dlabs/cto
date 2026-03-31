Implement subtask 1001: Scaffold Helm Chart Structure, Namespace Templates, and NOTES.txt Skeleton

## Objective
Create the charts/hermes-infra Helm chart directory structure with Chart.yaml, per-environment values files, namespace templates with labels/annotations, and a NOTES.txt skeleton that will be populated as services are added. This is the foundational scaffold all other subtasks build upon.

## Steps
Step-by-step:
1. Create directory `charts/hermes-infra/` with standard Helm layout: `Chart.yaml`, `values.yaml` (defaults), `templates/`, `templates/tests/`, `.helmignore`.
2. In `Chart.yaml`, set name: `hermes-infra`, version: `0.1.0`, appVersion matching project version. Add subchart dependencies placeholder (redis, nats — will be filled in by backing service subtasks).
3. Create `values.yaml` with all parameterized keys: `namespace`, `environment`, `replicaCount`, `minio.dedicated` (boolean, default false), `minio.endpoint`, `minio.bucketName`, `minio.retentionDays`, `minio.bucketQuota`, `minio.accessKeyId`, `minio.secretAccessKey`, `minio.presignExpiry` (default 3600), `resourceQuota.cpu`, `resourceQuota.memory`, `resourceQuota.pods`, `limitRange.defaultCpu`, `limitRange.defaultMemory`, `limitRange.maxCpu`, `limitRange.maxMemory`, `cnpg.password`, `cnpg.storageSize`, `cnpg.backupEnabled` (default false), `redis.auth.password`.
4. Create `values-staging.yaml` overriding: namespace=hermes-staging, environment=staging, minio.bucketName=hermes-staging-artifacts, retentionDays=30, bucketQuota=20Gi, resourceQuota cpu=8/memory=16Gi/pods=20.
5. Create `values-production.yaml` overriding: namespace=hermes-production, environment=production, minio.bucketName=hermes-prod-artifacts, retentionDays=90, bucketQuota=50Gi, resourceQuota cpu=16/memory=32Gi/pods=40.
6. Create `templates/namespace.yaml`: Namespace resource with `metadata.name: {{ .Values.namespace }}`, labels (`app.kubernetes.io/part-of: hermes`, `hermes.io/environment: {{ .Values.environment }}`, `app.kubernetes.io/managed-by: helm`), annotations (`hermes.io/owner: platform-team`, `hermes.io/project: hermes-e2e-pipeline`).
7. Create `templates/NOTES.txt` skeleton with sections for Namespace, Environment, Endpoints (placeholders for ConfigMap keys), MinIO, Secrets, and Usage (`envFrom` example). Will be completed as templates are added.
8. Validate: `helm lint charts/hermes-infra` passes. `helm template hermes-infra charts/hermes-infra -f charts/hermes-infra/values-staging.yaml` renders namespace correctly.

## Validation
`helm lint charts/hermes-infra` passes with no errors. `helm template hermes-infra charts/hermes-infra -f charts/hermes-infra/values-staging.yaml` renders a valid Namespace YAML with name=hermes-staging, all 3 labels, and all 2 annotations. Same for production values. Chart.yaml contains valid metadata. Both values files parse correctly. NOTES.txt skeleton renders without template errors.