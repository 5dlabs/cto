Implement subtask 3014: Generate and validate OpenAPI spec from grpc-gateway annotations

## Objective
Generate the OpenAPI v2 specification from protobuf grpc-gateway annotations and validate it for correctness and completeness.

## Steps
1. Ensure `protoc-gen-openapiv2` is configured in `buf.gen.yaml` with output to `api/openapi/`.
2. Run `buf generate` to produce the OpenAPI spec files.
3. Merge per-proto OpenAPI files into a single `api/openapi/rms.swagger.json` using `swagger-cli bundle` or a custom merge script.
4. Add metadata to the spec: title 'Sigma1 RMS API', version 'v1', description, contact info.
5. Add security definition for Bearer API key auth: `securityDefinitions: {ApiKeyAuth: {type: apiKey, in: header, name: Authorization}}`.
6. Verify all endpoints are present: count should match all REST paths defined across 5 services plus GDPR endpoint.
7. Add Makefile target `openapi-gen` and `openapi-validate`.

## Validation
1) Run `swagger-cli validate api/openapi/rms.swagger.json` → exits 0 with no errors. 2) Parse spec and verify all expected paths exist: /api/v1/opportunities (GET, POST), /api/v1/opportunities/{id} (GET, PUT), /api/v1/opportunities/{id}/approve (POST), /api/v1/opportunities/{id}/convert (POST), etc. — total ~20+ endpoints. 3) Verify request/response schemas reference correct protobuf-generated types.