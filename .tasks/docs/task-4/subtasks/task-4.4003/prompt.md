Implement subtask 4003: Create typed Hermes API client

## Objective
Create `lib/hermes-api.ts` — a typed API client for all Hermes backend endpoints, with SWR hooks for data fetching, error handling, and types mirroring the backend definitions.

## Steps
1. Create `lib/hermes-api.ts` with the following types (mirroring backend):
   - `Deliberation`: `{ id: string; status: 'pending' | 'processing' | 'completed' | 'failed'; targetUrl: string; triggeredBy: string; createdAt: string; completedAt?: string; artifactCount: number }`
   - `Artifact`: `{ id: string; deliberationId: string; artifactType: 'current_site_screenshot' | 'variant_snapshot'; storageKey: string; contentType: string; sizeBytes: number; metadata: Record<string, unknown>; createdAt: string }`
   - `PresignedUrlResponse`: `{ url: string; expiresAt: string; contentType: string; sizeBytes: number }`
2. Implement fetch-based API functions:
   - `fetchDeliberations(page: number, limit: number): Promise<{ data: Deliberation[]; total: number }>`
   - `fetchDeliberation(id: string): Promise<Deliberation>`
   - `fetchDeliberationArtifacts(deliberationId: string): Promise<Artifact[]>`
   - `fetchArtifactUrl(artifactId: string): Promise<PresignedUrlResponse>`
3. Create SWR hooks:
   - `useDeliberations(page, limit)` — returns `{ data, error, isLoading }`
   - `useDeliberation(id)` — with refresh interval when status is `pending` or `processing`
   - `useDeliberationArtifacts(deliberationId)`
4. Error handling: parse API error responses, throw typed errors, and surface user-friendly messages via toast notifications (use a toast library already in the project or a simple custom toast).
5. Base URL: use `NEXT_PUBLIC_API_BASE_URL` env var, or default to relative path `/api`.

## Validation
Unit test: mock fetch responses and verify `fetchDeliberations` returns correctly typed data. Test error handling: mock a 500 response and verify a typed error is thrown. Test SWR hooks using `@testing-library/react`: mock API, render a component using `useDeliberations`, verify loading state then data state. Verify refresh interval is set when deliberation status is 'processing'.