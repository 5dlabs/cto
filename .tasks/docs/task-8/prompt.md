Implement task 8: Validate Hermes Research Memo Content (Tess - Test frameworks)

## Goal
Validate that the deliberation path contains Hermes-sourced research content when NOUS_API_KEY is available, and that the fallback behavior works correctly when it is not. This tests the Hermes integration from Task 3 end-to-end.

## Task Context
- Agent owner: tess
- Stack: Test frameworks
- Priority: medium
- Dependencies: 3, 6

## Implementation Plan
1. Create test file `e2e/hermes-research-validation.test.ts`.
2. Test case 1 — Hermes content present: retrieve the deliberation artifacts from the pipeline run (via API endpoint, e.g., `GET /api/pipeline/runs/{runId}/deliberation`). Assert: at least one research memo has `source: 'hermes'` and a non-empty `content` field.
3. Test case 2 — Content quality: assert the Hermes-sourced content is a string with length > 100 characters (not just a placeholder or error message).
4. Test case 3 — Circuit breaker status: query `GET /api/pipeline/status` and assert the Hermes circuit breaker state is `closed` after a successful run.
5. Test case 4 — Fallback behavior (conditional): if `NOUS_API_KEY` is not set in the test environment, assert the deliberation artifacts contain a fallback memo with `source: 'fallback'` and `reason: 'no_api_key'`. Skip this test if the key is present.
6. Test case 5 — Availability gating log: when running with the key, assert PM server logs contain `stage: 'hermes_research'` entries without `action: 'skipped'`.
7. Design tests to be environment-aware: detect whether `NOUS_API_KEY` is available and adjust assertions accordingly (Hermes content vs. fallback). Both paths must be validated.

## Acceptance Criteria
1. When NOUS_API_KEY is set: deliberation artifacts contain at least one memo with `source: 'hermes'` and `content.length > 100`. Circuit breaker reports `closed` state. No `skipped` log entries for hermes_research stage. 2. When NOUS_API_KEY is not set: deliberation artifacts contain a fallback memo with `source: 'fallback'` and `reason: 'no_api_key'`. Pipeline still completes successfully. 3. At least one of the two paths (with key or without) must execute and pass in the test run, depending on environment configuration.

## Subtasks
- Implement Hermes primary path validation tests (content presence, quality, and circuit breaker): Create the test file `e2e/hermes-research-validation.test.ts` and implement the test cases that validate Hermes-sourced research content when NOUS_API_KEY is available. This includes asserting memo presence with `source: 'hermes'`, content quality (length > 100), circuit breaker `closed` state, and absence of `action: 'skipped'` log entries for the hermes_research stage.
- Implement Hermes fallback path validation tests (no API key scenario): Add test cases that validate fallback behavior when NOUS_API_KEY is not set: assert deliberation artifacts contain a fallback memo with `source: 'fallback'` and `reason: 'no_api_key'`, and that the pipeline still completes successfully.
- Implement environment-aware test orchestration and path coverage guarantee: Add orchestration logic that detects API key availability, conditionally routes to the correct test path, and includes a meta-assertion ensuring at least one of the two paths (primary or fallback) executed and passed in the current run.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.