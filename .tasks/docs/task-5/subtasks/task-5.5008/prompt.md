Implement subtask 5008: Add Prometheus metrics and OpenAPI documentation

## Objective
Instrument the vetting service with Prometheus metrics for observability and generate an OpenAPI specification documenting all endpoints.

## Steps
1. Add prometheus and metrics crate dependencies. 2. Define metrics: vetting_pipeline_duration_seconds (histogram, labels: status), vetting_integration_duration_seconds (histogram, labels: provider, status), vetting_requests_total (counter, labels: endpoint, status_code), vetting_integration_errors_total (counter, labels: provider, error_type). 3. Instrument the pipeline and each integration call with timing and error metrics. 4. Expose GET /metrics endpoint in Prometheus exposition format. 5. Use utoipa or aide to generate OpenAPI 3.0 spec from the handler signatures and model structs. 6. Serve the OpenAPI spec at GET /api/v1/vetting/openapi.json. 7. Ensure all request/response models are documented with examples in the spec.

## Validation
Verify /metrics endpoint returns valid Prometheus format with all defined metrics after running a vetting pipeline. Verify /openapi.json returns valid OpenAPI 3.0 spec that passes swagger-cli validation. Check that all three endpoints are documented with request/response schemas.