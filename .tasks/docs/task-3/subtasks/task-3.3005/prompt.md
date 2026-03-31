Implement subtask 3005: Wire artifact generation into the deliberation lifecycle

## Objective
Create `src/modules/hermes/artifacts/artifact-generator.ts` — the orchestrator that integrates screenshot capture, variant snapshot generation, MinIO uploads, and metadata persistence into the deliberation flow, managing status transitions and error handling.

## Steps
1. Create `artifact-generator.ts` as the top-level orchestrator class `ArtifactGenerator` that composes the MinIO client (3001), screenshot capture service (3002), variant snapshot generator (3003), and artifact persistence (3004).
2. Implement `captureCurrentSite(deliberationId: string, targetUrl: string): Promise<ArtifactRecord>`:
   - Call screenshot capture service with the target URL
   - Upload the PNG buffer to MinIO at key `{deliberation_id}/current-site/{timestamp}.png` with tags `retention-class=hermes` and `environment=${ENVIRONMENT}`
   - Save artifact metadata via persistence layer with type `current_site_screenshot`
   - Return the created artifact record
3. Implement `captureVariants(deliberationId: string, variants: Variant[]): Promise<ArtifactRecord[]>`:
   - Call variant snapshot generator
   - Upload each variant PNG to MinIO
   - Save each artifact metadata with type `variant_snapshot`
   - Return all created artifact records
4. Integrate into the deliberation flow in the existing Hermes route handler:
   - After `POST /api/hermes/deliberations` creates a deliberation record, call `captureCurrentSite()` (can be async/enqueued depending on decision point)
   - On deliberation completion callback/hook, call `captureVariants()`
   - Only transition deliberation status to `completed` after all artifacts are successfully stored
   - On any failure: set deliberation status to `failed`, log structured error
5. Handle the full error flow: if current-site capture fails, mark deliberation as `failed` immediately. If variant capture partially fails, still mark as `completed` but with a warning in metadata.
6. Export the orchestrator for route-level consumption.

## Validation
Integration test: trigger `POST /api/hermes/deliberations` with a valid URL, verify that within 30 seconds the deliberation has at least one `current_site_screenshot` artifact in the DB and the corresponding object exists in MinIO. End-to-end test: after deliberation completes, verify both `current_site_screenshot` and `variant_snapshot` artifacts exist. Failure test: trigger with an invalid URL, verify deliberation status is `failed` and no artifacts are created. Partial failure test: mock one variant capture to fail, verify deliberation still completes and successful variant artifacts are persisted.