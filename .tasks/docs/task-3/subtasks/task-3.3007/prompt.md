Implement subtask 3007: Add structured logging for all artifact operations

## Objective
Implement structured JSON logging across all artifact modules, ensuring every upload, delete, presign, and capture operation emits logs with the required fields: `deliberation_id`, `artifact_type`, `storage_key`, `operation`, `duration_ms`, and `error_code`.

## Steps
1. Create or extend a logging utility at `src/modules/hermes/artifacts/artifact-logger.ts` that wraps the existing structured logger with artifact-specific context.
2. Define the standard log shape: `{ deliberation_id: string; artifact_type?: string; storage_key?: string; operation: 'capture' | 'upload' | 'delete' | 'presign' | 'persist'; duration_ms: number; error_code?: string; message: string }`.
3. Instrument all operations in the artifact modules:
   - `minio-client.ts`: log on upload start/complete/failure, presign generation, delete operations
   - `screenshot-capture.ts`: log on capture start/complete/failure with duration
   - `variant-snapshot-generator.ts`: log per-variant capture with variant ID
   - `artifact-generator.ts`: log orchestration steps and status transitions
   - Presigned URL endpoint: log each presign request
4. Use `performance.now()` or `Date.now()` to measure `duration_ms` for each operation.
5. On errors, include `error_code` field (e.g., `CAPTURE_FAILED`, `UPLOAD_FAILED`, `NAVIGATION_FAILED`, `TIMEOUT`) matching the typed error codes from the capture service.
6. Ensure logs are emitted as JSON to stdout for collection by Loki/Promtail.

## Validation
Capture stdout during a test artifact upload and parse the JSON log output. Verify log entries contain all required fields: `deliberation_id`, `operation`, `duration_ms`, and `storage_key`. Error test: trigger a capture failure and verify the log includes `error_code: 'CAPTURE_FAILED'`. Verify that a full deliberation lifecycle (capture → upload → persist → presign) produces at least 4 distinct log entries with correct `operation` values. Logs should be valid JSON parseable by `JSON.parse()`.