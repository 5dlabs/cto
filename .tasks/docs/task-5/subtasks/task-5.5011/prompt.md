Implement subtask 5011: Generate OpenAPI spec with utoipa

## Objective
Add utoipa annotations to all endpoints and data models, and expose the OpenAPI JSON spec at /api/v1/vetting/openapi.json.

## Steps
1. Add `utoipa` and `utoipa-swagger-ui` (optional) to Cargo.toml dependencies.
2. Annotate all request/response structs with `#[derive(utoipa::ToSchema)]`: VettingResult, VettingRequest, RunVettingRequest, CreditSubset, GdprDeleteResponse.
3. Annotate all handlers with `#[utoipa::path(...)]` macros specifying method, path, request_body, responses, params, and tags.
4. Create the OpenApi struct using `#[derive(utoipa::OpenApi)]` with `#[openapi(paths(...), components(schemas(...)), tags(...))]`.
5. Add route: `GET /api/v1/vetting/openapi.json` that returns `Json(ApiDoc::openapi())`.
6. Verify the generated spec is valid OpenAPI 3.0/3.1 JSON.

## Validation
Call GET /api/v1/vetting/openapi.json, verify 200 response with Content-Type application/json. Parse the response as valid OpenAPI spec. Verify all 4 endpoints (POST run, GET result, GET credit, DELETE gdpr) are listed. Verify all schema components are present. Optionally validate with an OpenAPI linter.