Implement subtask 1007: Create sigma1-infra-endpoints ConfigMap aggregating all connection strings

## Objective
Create the central 'sigma1-infra-endpoints' ConfigMap in the sigma1 namespace that aggregates all infrastructure connection strings and endpoints, enabling downstream services to consume them via envFrom.

## Steps
1. Author a ConfigMap 'sigma1-infra-endpoints' in the sigma1 namespace under infra/configmap/sigma1-infra-endpoints.yaml.
2. Include the following keys:
   - POSTGRES_URL: postgresql://<role>:<password-ref>@<cluster-service>.databases.svc:5432/<dbname>
   - POSTGRES_HOST, POSTGRES_PORT, POSTGRES_DB (individual components)
   - REDIS_URL: redis://:<password>@redis-sigma1.databases.svc:6379/0
   - REDIS_HOST, REDIS_PORT
   - S3_ENDPOINT: the R2/S3 endpoint URL
   - S3_BUCKET_IMAGES: sigma1-product-images
   - S3_BUCKET_EVENTS: sigma1-event-photos
   - S3_CDN_BASE_URL: public CDN URL for image reads
   - SIGNAL_CLI_URL: http://signal-cli.sigma1.svc:8080
3. For secrets referenced in URLs, use a pattern where the ConfigMap provides the template and services combine it with Secret values at runtime (do NOT embed passwords in the ConfigMap).
4. Add labels: app.kubernetes.io/part-of=sigma1, sigma1.io/type=infra-endpoints.
5. Replicate or mirror the ConfigMap to other namespaces (databases, social, web, openclaw) using a kustomize overlay or manual copy so each namespace can use envFrom locally.

## Validation
kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml contains all expected keys; verify the ConfigMap exists in all required namespaces; deploy a test pod with envFrom referencing the ConfigMap and verify all env vars are populated.