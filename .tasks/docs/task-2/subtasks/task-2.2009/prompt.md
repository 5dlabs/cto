Implement subtask 2009: Implement machine-readable equipment API endpoint and OpenAPI spec generation

## Objective
Build the GET /api/v1/equipment-api/catalog endpoint returning a flat JSON structure optimized for AI agent consumption, and generate the OpenAPI spec using utoipa served at /api/v1/catalog/openapi.json.

## Steps
1. Implement `GET /api/v1/equipment-api/catalog`:
   - Return a flat JSON array of all active products with denormalized category names.
   - Structure: `[{"id": "...", "name": "...", "category": "Cameras", "day_rate": 150.00, "specs": {...}, "available": true}]`.
   - Include a top-level `available` boolean based on whether the product has any availability in the next 30 days.
   - Support query param `category` for filtering.
   - No pagination — return all products (optimized for AI agent single-call consumption).
2. Integrate `utoipa` crate:
   - Add `#[derive(utoipa::ToSchema)]` to all request/response DTOs.
   - Add `#[utoipa::path(...)]` annotations to all endpoint handlers.
   - Create the OpenAPI doc struct with `#[derive(OpenApi)]` collecting all paths.
   - Serve the spec at `GET /api/v1/catalog/openapi.json`.
3. Include API metadata: title, version, description, contact info, server URLs.
4. Add `utoipa-swagger-ui` for optional Swagger UI at `/swagger-ui` (dev only, behind feature flag).

## Validation
Integration test: seed products across categories, call GET /api/v1/equipment-api/catalog and verify flat structure with denormalized category names. Filter by category and verify correct subset. Verify `available` field reflects actual availability data. Fetch /api/v1/catalog/openapi.json and validate it with `swagger-cli validate` or equivalent. Verify all endpoints are documented in the spec.