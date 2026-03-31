Implement subtask 3004: Implement artifact metadata persistence via IHermesArtifactWriter

## Objective
Create `src/modules/hermes/artifacts/artifact-metadata.ts` — the artifact metadata model and persistence logic that writes artifact records through the `IHermesArtifactWriter` abstraction, supporting both `current_site_screenshot` and `variant_snapshot` artifact types.

## Steps
1. Define the artifact metadata types in `artifact-metadata.ts`:
   - `ArtifactType` enum: `'current_site_screenshot' | 'variant_snapshot'`
   - `ArtifactRecord`: `{ id: string; deliberationId: string; artifactType: ArtifactType; storageKey: string; contentType: string; sizeBytes: number; metadata: Record<string, unknown>; createdAt: Date }`
   - `CreateArtifactInput`: omit `id` and `createdAt` from `ArtifactRecord`
2. Implement `ArtifactPersistence` class that takes an `IHermesArtifactWriter` dependency (from Task 2's abstraction) in its constructor.
3. Methods:
   - `saveArtifact(input: CreateArtifactInput): Promise<ArtifactRecord>` — writes to DB via the writer abstraction, returns the created record with generated `id` and `createdAt`.
   - `getArtifactsByDeliberation(deliberationId: string): Promise<ArtifactRecord[]>` — fetches all artifacts for a deliberation.
   - `getArtifactById(id: string): Promise<ArtifactRecord | null>` — fetches a single artifact record.
4. The `metadata` JSONB field stores capture-specific data: viewport dimensions, source URL, capture duration, variant ID (for variant snapshots).
5. Ensure the abstraction works regardless of whether the underlying storage is a parallel table or schema extension (per D6 decision from Task 2).
6. Add input validation: `storageKey` must match expected patterns, `sizeBytes` must be > 0, `contentType` must be a valid MIME type.

## Validation
Unit test: mock `IHermesArtifactWriter`, call `saveArtifact` with valid input, verify the writer's insert method is called with correct fields and the returned record has an `id` and `createdAt`. Test `getArtifactsByDeliberation` returns only artifacts matching the given deliberation ID. Validation test: calling `saveArtifact` with `sizeBytes: 0` or empty `storageKey` throws a validation error.