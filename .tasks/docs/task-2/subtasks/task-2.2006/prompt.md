Implement subtask 2006: Implement S3/R2 image serving and service observability

## Objective
Implement S3/R2 integration for secure and efficient image serving, and add Prometheus metrics, liveness, and readiness probes for service observability.

## Steps
1. Integrate an S3 client library (e.g., `aws-sdk-s3`) to upload, retrieve, and serve product images, using credentials from Kubernetes secrets.2. Implement a dedicated endpoint for image serving (e.g., `GET /images/:id`).3. Add `axum-prometheus` or similar for `/metrics` endpoint.4. Implement `/health/live` and `/health/ready` endpoints that check database and Redis connectivity.

## Validation
1. Upload a test image via an admin endpoint and verify it's stored in S3/R2.2. Retrieve the test image via its serving endpoint.3. Access `/metrics` endpoint and verify Prometheus metrics are exposed.4. Access `/health/live` and `/health/ready` and verify they return 200 OK when dependencies are healthy.