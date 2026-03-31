## Implement Hermes Deliberation Path API (Nova - Bun/Elysia)

### Objective
Build the Hermes deliberation path as an internal module within the existing Bun/Elysia service, exposing RESTful endpoints for triggering deliberation, querying deliberation status, and retrieving results. This module forms the core of the intake pipeline validation.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
Step-by-step implementation:

1. **Module structure:** Create `src/modules/hermes/` directory with clear boundary:
   - `src/modules/hermes/index.ts` — module entry point exporting Elysia plugin
   - `src/modules/hermes/routes.ts` — route definitions
   - `src/modules/hermes/service.ts` — business logic (HermesService class/interface)
   - `src/modules/hermes/types.ts` — TypeScript interfaces for deliberation request/response, artifact metadata
   - `src/modules/hermes/repository.ts` — database access abstraction

2. **Interface contracts (per D1):** Define clear TypeScript interfaces:
   - `IHermesService` — deliberation trigger, status query, result retrieval
   - `IHermesRepository` — CRUD for deliberation records and artifact references
   - `IHermesArtifactWriter` — abstraction for artifact storage (per open question #7 on D6, this must accommodate either schema extension or parallel table)

3. **REST endpoints (per D3):** Implement via Elysia route handlers with `@elysiajs/swagger` for OpenAPI generation:
   - `POST /api/hermes/deliberations` — trigger a new deliberation (requires `hermes:trigger` RBAC claim)
   - `GET /api/hermes/deliberations/:id` — get deliberation status and result (requires `hermes:read`)
   - `GET /api/hermes/deliberations` — list deliberations with pagination (requires `hermes:read`)
   - `GET /api/hermes/deliberations/:id/artifacts` — list artifacts for a deliberation (requires `hermes:read`)

4. **Auth integration (per D5):** Extend existing session middleware to check RBAC claims:
   - Create `src/modules/hermes/middleware.ts` with `requireHermesClaim(claim: string)` guard
   - Claims: `hermes:read`, `hermes:trigger` at minimum. Document additional claims as needed.
   - Coordinate claim names with Task 10 (RBAC hardening) via shared types file or ADR.

5. **Database schema:** Create migration for deliberation records:
   - `deliberations` table: `id` (UUID), `status` (enum: pending/processing/completed/failed), `input_payload` (JSONB), `result_payload` (JSONB), `triggered_by` (FK to users), `created_at`, `updated_at`
   - For artifact references: implement behind `IHermesArtifactWriter` abstraction to accommodate pending D6 decision. Default to parallel table approach (`hermes_artifacts` with FK to deliberation) but ensure the interface can swap to schema extension.

6. **Elysia plugin registration:** Register the Hermes module as an Elysia plugin in the main app entry point:
   ```typescript
   app.use(hermesPlugin)
   ```
   Ensure it does not interfere with existing legacy pipeline routes.

7. **OpenAPI documentation:** Ensure all endpoints generate OpenAPI specs via `@elysiajs/swagger`. Tag all Hermes endpoints with `hermes` group.

8. **Environment configuration:** Read all infra connection strings from `hermes-infra-endpoints` ConfigMap (env vars). Feature flag: `HERMES_ENABLED=true|false` env var controls whether the module registers routes.

9. **Error handling:** Return structured JSON error responses with `error_code` field for downstream logging (Task 6 dependency).

### Subtasks
- [ ] Scaffold Hermes module structure and define TypeScript interface contracts: Create the src/modules/hermes/ directory structure with all file stubs and define the IHermesService, IHermesRepository, and IHermesArtifactWriter TypeScript interfaces.
- [ ] Create database migrations for deliberations and hermes_artifacts tables: Write and validate database migration scripts that create the deliberations and hermes_artifacts tables in PostgreSQL without affecting existing tables.
- [ ] Implement HermesRepository with PostgreSQL data access layer: Implement the IHermesRepository interface with PostgreSQL queries for CRUD operations on deliberations and hermes_artifacts tables.
- [ ] Implement HermesService business logic layer: Implement the IHermesService interface with business logic for triggering deliberations, querying status, and retrieving results, using the repository abstraction.
- [ ] Implement RBAC middleware with hermes:read and hermes:trigger claim guards: Create Hermes-specific RBAC middleware that extends the existing session middleware to check for hermes:read and hermes:trigger claims.
- [ ] Implement REST endpoints and register Elysia plugin with feature flag: Implement all four Hermes REST endpoints as Elysia route handlers, wire them to the service layer with RBAC guards, register as a plugin, and implement the HERMES_ENABLED feature flag.
- [ ] Configure OpenAPI/Swagger documentation for all Hermes endpoints: Ensure all Hermes endpoints are properly documented in the OpenAPI spec via @elysiajs/swagger with correct schemas, tags, and descriptions.