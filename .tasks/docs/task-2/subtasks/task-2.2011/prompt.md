Implement subtask 2011: Generate OpenAPI specification for all catalog endpoints

## Objective
Document all API endpoints with an OpenAPI 3.0 specification, either auto-generated from code using utoipa or manually authored, and serve it at a discovery endpoint.

## Steps
1. Add `utoipa` and `utoipa-swagger-ui` crates to Cargo.toml. 2. Annotate all handler functions with #[utoipa::path(...)] macros specifying: operation_id, tags, path params, query params, request bodies, response schemas and status codes. 3. Annotate all request/response structs with #[derive(ToSchema)] from utoipa. 4. Create an OpenAPI doc struct using #[derive(OpenApi)] with #[openapi(paths(...), components(schemas(...)), tags(...))] listing all endpoints and schemas. 5. Serve Swagger UI at /docs and raw OpenAPI JSON at /api/v1/openapi.json. 6. Include descriptions for all endpoints, parameters, and schema fields. 7. Add server URLs for local development and production.

## Validation
GET /api/v1/openapi.json returns valid OpenAPI 3.0 JSON that passes validation (e.g., swagger-cli validate). GET /docs renders Swagger UI. All 6+ endpoints are documented with correct paths, methods, parameters, and response schemas. Try-it-out in Swagger UI successfully calls endpoints.