Implement subtask 2009: Generate OpenAPI specification and documentation

## Objective
Generate a comprehensive OpenAPI 3.0 specification documenting all Equipment Catalog Service endpoints, request/response schemas, and error codes.

## Steps
1. Add utoipa crate as dependency for OpenAPI generation from Rust code.
2. Annotate all handler functions with #[utoipa::path(...)] macros specifying:
   - HTTP method and path
   - Request parameters (query, path)
   - Request body schemas (for POST endpoints)
   - Response schemas with status codes (200, 400, 404, 409, 429, 500)
3. Annotate all DTO structs with #[derive(ToSchema)].
4. Create an ApiDoc struct with #[derive(OpenApi)] aggregating all paths and schemas.
5. Add a GET /api/v1/docs endpoint serving the OpenAPI JSON.
6. Optionally add Swagger UI at /api/v1/docs/ui using utoipa-swagger-ui.
7. Verify the generated spec is valid using an OpenAPI validator.

## Validation
GET /api/v1/docs returns a valid OpenAPI 3.0 JSON document. The spec includes all 6 endpoint paths with correct schemas. Validate the spec using an OpenAPI linter (e.g., spectral or swagger-editor) with no errors.