Implement subtask 2003: Implement HermesRepository with PostgreSQL data access layer

## Objective
Implement the IHermesRepository interface with PostgreSQL queries for CRUD operations on deliberations and hermes_artifacts tables.

## Steps
1. In `src/modules/hermes/repository.ts`, implement `HermesRepository` class that satisfies `IHermesRepository`.
2. Use the project's existing database client/ORM pattern (check if Drizzle, Prisma, or raw pg is used) to implement:
   - `createDeliberation(input)` — INSERT into deliberations, return full record
   - `getDeliberationById(id)` — SELECT by UUID
   - `listDeliberations(params)` — SELECT with pagination (ORDER BY created_at DESC, LIMIT/OFFSET or cursor)
   - `updateDeliberationStatus(id, status, resultPayload?)` — UPDATE status and optionally result_payload, set updated_at
   - `createArtifact(artifact)` — INSERT into hermes_artifacts
   - `getArtifactsByDeliberationId(deliberationId)` — SELECT artifacts for a deliberation
3. Read `CNPG_HERMES_URL` from environment variables (provided by `hermes-infra-endpoints` ConfigMap).
4. Implement proper error handling: wrap database errors in domain-specific error types.
5. All methods must be async and return typed results matching the interfaces from types.ts.

## Validation
Unit tests with a test database: `createDeliberation` returns a valid UUID and the record is retrievable via `getDeliberationById`. `listDeliberations` returns paginated results in correct order. `updateDeliberationStatus` changes status and updated_at timestamp. `getArtifactsByDeliberationId` returns only artifacts belonging to the specified deliberation.