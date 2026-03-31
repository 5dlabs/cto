Implement subtask 2007: Configure OpenAPI/Swagger documentation for all Hermes endpoints

## Objective
Ensure all Hermes endpoints are properly documented in the OpenAPI spec via @elysiajs/swagger with correct schemas, tags, and descriptions.

## Steps
1. Verify `@elysiajs/swagger` is installed and configured in the main app.
2. Add Elysia `detail` metadata to each Hermes route:
   - `summary` — short description
   - `description` — detailed description including auth requirements
   - `tags: ['hermes']` — group all Hermes endpoints
   - Request body schema with examples for POST endpoint
   - Response schemas for 200/201, 400, 401, 403, 404 status codes
3. Define reusable schema components for `Deliberation`, `HermesArtifact`, `PaginatedResponse`, and error response types.
4. Verify the generated spec at `GET /api/swagger/json` includes all four endpoints with correct schemas.
5. Test that the Swagger UI at the configured path renders the Hermes group correctly.

## Validation
`GET /api/swagger/json` returns an OpenAPI spec containing all four Hermes endpoints under the 'hermes' tag. Each endpoint has correct request/response schemas. The spec validates against OpenAPI 3.x schema validator without errors.