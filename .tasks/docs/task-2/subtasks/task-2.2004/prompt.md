Implement subtask 2004: Implement HermesService business logic layer

## Objective
Implement the IHermesService interface with business logic for triggering deliberations, querying status, and retrieving results, using the repository abstraction.

## Steps
1. In `src/modules/hermes/service.ts`, implement `HermesService` class that satisfies `IHermesService`.
2. Constructor accepts `IHermesRepository` and `IHermesArtifactWriter` via dependency injection.
3. `triggerDeliberation(input, triggeredBy)`:
   - Validate input payload structure
   - Create deliberation record via repository with status `pending`
   - Return the created deliberation object
   - (Note: actual processing/orchestration is a separate concern for future tasks)
4. `getDeliberation(id)` — delegate to repository, return null if not found
5. `listDeliberations(params)` — delegate to repository with pagination
6. `getDeliberationArtifacts(deliberationId)` — verify deliberation exists, then delegate to repository
7. Implement structured error handling:
   - `DeliberationNotFoundError` for missing deliberations
   - `ValidationError` for malformed input
   - All errors include `error_code` string for downstream logging
8. Implement a default `HermesArtifactWriter` that writes to the parallel table via the repository (swappable per D6 decision).

## Validation
Unit tests with mocked repository: `triggerDeliberation` calls `repository.createDeliberation` with correct params and returns the result. `getDeliberation` with non-existent ID throws `DeliberationNotFoundError`. Input validation rejects payloads missing required fields. All error objects include `error_code` field.