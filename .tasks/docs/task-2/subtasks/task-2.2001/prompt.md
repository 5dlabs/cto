Implement subtask 2001: Scaffold Hermes module structure and define TypeScript interface contracts

## Objective
Create the src/modules/hermes/ directory structure with all file stubs and define the IHermesService, IHermesRepository, and IHermesArtifactWriter TypeScript interfaces.

## Steps
1. Create directory `src/modules/hermes/`.
2. Create `src/modules/hermes/types.ts` with:
   - `DeliberationStatus` enum: `pending | processing | completed | failed`
   - `DeliberationInput` interface for the request payload
   - `DeliberationResult` interface for the response payload
   - `Deliberation` interface: `id: string (UUID), status: DeliberationStatus, inputPayload: DeliberationInput, resultPayload: DeliberationResult | null, triggeredBy: string, createdAt: Date, updatedAt: Date`
   - `HermesArtifact` interface: `id: string, deliberationId: string, name: string, mimeType: string, storagePath: string, sizeBytes: number, createdAt: Date`
   - Pagination types: `PaginationParams`, `PaginatedResponse<T>`
   - Structured error response type with `error_code` field
3. Create `src/modules/hermes/service.ts` with `IHermesService` interface:
   - `triggerDeliberation(input: DeliberationInput, triggeredBy: string): Promise<Deliberation>`
   - `getDeliberation(id: string): Promise<Deliberation | null>`
   - `listDeliberations(params: PaginationParams): Promise<PaginatedResponse<Deliberation>>`
   - `getDeliberationArtifacts(deliberationId: string): Promise<HermesArtifact[]>`
4. Create `src/modules/hermes/repository.ts` with `IHermesRepository` interface for CRUD operations.
5. Create `src/modules/hermes/artifact-writer.ts` with `IHermesArtifactWriter` interface that abstracts storage backend. Default implementation targets parallel table approach but interface must be swappable.
6. Create stub files: `routes.ts`, `middleware.ts`, `index.ts`.
7. Export all interfaces from `index.ts`.

## Validation
All TypeScript files compile without errors. Interfaces are importable from `src/modules/hermes/index.ts`. Running `bun typecheck` (or `tsc --noEmit`) passes with no errors in the hermes module.