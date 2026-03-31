Implement task 3: Integrate Hermes Path with Snapshot Artifact Generation (Nova - Bun/Elysia)

## Goal
Implement snapshot artifact generation within the Hermes deliberation flow — capturing current-site screenshots and generated variant snapshots, writing them to the dedicated MinIO bucket, and linking artifact metadata to deliberation records.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1, 2

## Implementation Plan
Step-by-step implementation:

1. **Artifact generation service:** Create `src/modules/hermes/artifacts/` subdirectory:
   - `artifact-generator.ts` — orchestrates screenshot capture and variant snapshot generation
   - `minio-client.ts` — S3-compatible client using `@aws-sdk/client-s3` (works with MinIO) configured from env vars (`MINIO_HERMES_ENDPOINT`, `MINIO_HERMES_BUCKET`, credentials from mounted secrets)
   - `artifact-metadata.ts` — metadata model for artifact records

2. **Screenshot capture:** Implement current-site screenshot capture:
   - Use a headless browser library (Playwright's `chromium.launch()` in headless mode, or `puppeteer-core` with Bun compatibility) to capture the current state of the target URL
   - Output: PNG screenshot stored as `{deliberation_id}/current-site/{timestamp}.png` in MinIO
   - Capture metadata: URL, viewport dimensions, timestamp, capture duration

3. **Variant snapshot generation:** After deliberation completes, generate variant snapshots:
   - Capture each generated variant as a screenshot
   - Store as `{deliberation_id}/variants/{variant_id}.png` in MinIO
   - Store comparison metadata linking current-site to each variant

4. **MinIO write operations:**
   - Use multipart upload for images > 5MB
   - Set content-type headers (`image/png`)
   - Apply object tags for lifecycle management: `retention-class: hermes`, `environment: ${ENVIRONMENT}`
   - Generate presigned URLs for frontend retrieval (1-hour TTL, configurable)

5. **Artifact metadata persistence:** Write artifact records via the `IHermesArtifactWriter` abstraction from Task 2:
   - Each artifact record: `id`, `deliberation_id` (FK), `artifact_type` (enum: `current_site_screenshot` | `variant_snapshot`), `storage_key` (MinIO object key), `content_type`, `size_bytes`, `metadata` (JSONB — viewport, URL, capture duration), `created_at`
   - Abstraction layer must work regardless of D6 resolution (parallel table vs schema extension)

6. **Deliberation lifecycle integration:** Wire artifact generation into the deliberation flow:
   - On `POST /api/hermes/deliberations`: after creating the deliberation record, enqueue screenshot capture
   - On deliberation completion: trigger variant snapshot generation
   - Update deliberation status to `completed` only after all artifacts are stored
   - On failure: update status to `failed`, log error with structured fields

7. **Presigned URL endpoint:** Add to Hermes routes:
   - `GET /api/hermes/artifacts/:id/url` — returns a presigned S3 URL for the artifact (requires `hermes:read`)

8. **Structured logging:** All artifact operations must emit structured JSON logs with fields: `deliberation_id`, `artifact_type`, `storage_key`, `operation` (upload/delete/presign), `duration_ms`, `error_code` (if applicable).

## Acceptance Criteria
1. Integration test: Triggering a deliberation via `POST /api/hermes/deliberations` with a valid target URL results in at least one `current_site_screenshot` artifact record in the database within 30 seconds.
2. MinIO verification: After artifact generation completes, `aws s3 ls s3://hermes-artifacts-dev/{deliberation_id}/current-site/` (using MinIO endpoint) returns at least one PNG object with size > 0 bytes.
3. Presigned URL: `GET /api/hermes/artifacts/:id/url` returns a URL that, when fetched via HTTP GET, returns a 200 with `content-type: image/png`.
4. Variant snapshots: After deliberation completion, `GET /api/hermes/deliberations/:id/artifacts` returns both `current_site_screenshot` and `variant_snapshot` type artifacts.
5. Failure handling: When screenshot capture fails (invalid URL), the deliberation record status is `failed` and a structured log with `error_code: CAPTURE_FAILED` is emitted (queryable in Loki).
6. Object tags: MinIO objects created by the service have `retention-class=hermes` tag verified via `aws s3api get-object-tagging`.

## Subtasks
- Implement MinIO S3 client with multipart upload and object tagging: Create `src/modules/hermes/artifacts/minio-client.ts` — an S3-compatible client using `@aws-sdk/client-s3` configured from environment variables, supporting multipart upload for large images, content-type headers, object tagging, and presigned URL generation.
- Implement headless browser screenshot capture service: Create `src/modules/hermes/artifacts/screenshot-capture.ts` — a service that uses a headless browser to capture PNG screenshots of a given URL, returning the image buffer along with capture metadata (viewport, duration, URL).
- Implement variant snapshot generation pipeline: Create `src/modules/hermes/artifacts/variant-snapshot-generator.ts` — a pipeline that takes completed deliberation variants and captures each as a PNG screenshot, producing variant snapshot buffers and metadata ready for storage.
- Implement artifact metadata persistence via IHermesArtifactWriter: Create `src/modules/hermes/artifacts/artifact-metadata.ts` — the artifact metadata model and persistence logic that writes artifact records through the `IHermesArtifactWriter` abstraction, supporting both `current_site_screenshot` and `variant_snapshot` artifact types.
- Wire artifact generation into the deliberation lifecycle: Create `src/modules/hermes/artifacts/artifact-generator.ts` — the orchestrator that integrates screenshot capture, variant snapshot generation, MinIO uploads, and metadata persistence into the deliberation flow, managing status transitions and error handling.
- Implement presigned URL endpoint for artifact retrieval: Add `GET /api/hermes/artifacts/:id/url` Elysia route that looks up an artifact record by ID, generates a presigned S3 URL for the corresponding MinIO object, and returns it to authenticated callers with the `hermes:read` permission.
- Add structured logging for all artifact operations: Implement structured JSON logging across all artifact modules, ensuring every upload, delete, presign, and capture operation emits logs with the required fields: `deliberation_id`, `artifact_type`, `storage_key`, `operation`, `duration_ms`, and `error_code`.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.