Implement subtask 8005: Create Architecture Decision Records (ADRs) for decisions D1-D5 and D7-D9

## Objective
Create the ADR directory and write 8 individual ADR documents covering each resolved architectural decision, following the standard ADR format with Status, Context, Decision, and Consequences sections.

## Steps
1. Create `docs/hermes/adr/` directory.
2. Create 8 ADR files following the naming convention `NNN-title.md`:
   - `001-d1-<decision-topic>.md` through `008-d9-<decision-topic>.md` (skipping D6 if it was not a resolved decision, or including it — adjust based on the actual D1-D5, D7-D9 mapping)
3. Each ADR must follow this template:
   ```markdown
   # ADR-NNN: <Title>
   
   ## Status
   Accepted
   
   ## Date
   <YYYY-MM-DD>
   
   ## Context
   <What is the issue that we're seeing that is motivating this decision or change?>
   
   ## Decision
   <What is the change that we're proposing and/or doing?>
   
   ## Consequences
   ### Positive
   - <benefit 1>
   ### Negative
   - <trade-off 1>
   ### Neutral
   - <observation 1>
   ```
4. Specific ADRs to create (map to actual decisions from project context):
   - D1: Storage backend choice (MinIO for artifacts)
   - D2: Headless browser technology for screenshot capture
   - D3: API authentication strategy (session-based with RBAC claims)
   - D4: Database schema approach (additive tables, no ALTER TABLE)
   - D5: Feature flag implementation (HERMES_ENABLED ConfigMap)
   - D7: Artifact presigned URL strategy
   - D8: CI/CD promotion workflow (ArgoCD with E2E gating)
   - D9: Monitoring and observability stack choice
5. Each ADR should be 150-400 words, concise but complete.
6. Create an `index.md` in the ADR directory listing all ADRs with links.

## Validation
Verify at least 8 ADR files exist in `docs/hermes/adr/`. For each file, verify it contains the headings 'Status' (with value 'Accepted'), 'Context', 'Decision', and 'Consequences'. Verify index.md exists and links to all 8 ADRs. Check that no ADR is a stub (minimum 150 words per ADR).