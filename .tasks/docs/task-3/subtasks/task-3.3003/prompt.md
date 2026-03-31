Implement subtask 3003: Implement variant snapshot generation pipeline

## Objective
Create `src/modules/hermes/artifacts/variant-snapshot-generator.ts` — a pipeline that takes completed deliberation variants and captures each as a PNG screenshot, producing variant snapshot buffers and metadata ready for storage.

## Steps
1. Create `variant-snapshot-generator.ts` with interface: `generateVariantSnapshots(deliberationId: string, variants: Variant[]): Promise<VariantSnapshotResult[]>`.
2. For each variant, determine how to render it for capture. Variants will need to be served or rendered as HTML — implement a temporary local HTTP server (using Bun's built-in `Bun.serve`) that serves variant HTML on a random port, then capture via the screenshot service.
3. Call `captureScreenshot(`http://localhost:${port}/variant/${variantId}`)` from subtask 3002 for each variant.
4. `VariantSnapshotResult`: `{ variantId: string; buffer: Buffer; metadata: CaptureMetadata; storageKey: string }` where `storageKey` follows pattern `{deliberation_id}/variants/{variant_id}.png`.
5. Generate comparison metadata linking the current-site capture to each variant: `{ deliberationId, currentSiteKey, variantKey, variantId }`.
6. Process variants sequentially to avoid overwhelming the browser instance (or in controlled parallel batches of 2-3 if the browser pool supports it).
7. Clean up the temporary HTTP server after all variants are captured.
8. Handle partial failures: if one variant capture fails, continue with remaining variants, collect errors, and report which variants succeeded/failed.

## Validation
Unit test with mocked screenshot service: provide 3 mock variants, verify `captureScreenshot` is called 3 times with correct localhost URLs, and 3 `VariantSnapshotResult` objects are returned with correct storage key patterns. Integration test: create a simple HTML variant, serve it via the temp server, capture it, verify the resulting buffer is a valid PNG. Partial failure test: mock one variant capture to throw, verify the other variants still produce results and the error is collected.