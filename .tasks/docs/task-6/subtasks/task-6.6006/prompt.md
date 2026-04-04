Implement subtask 6006: Write end-to-end integration test for full pipeline validation

## Objective
Create an integration test that invokes the pipeline runner with the test PRD fixture, runs all stages, verifies Linear assignees, generates the report, and asserts all acceptance criteria from the test strategy.

## Steps
1. Create `src/validation/__tests__/pipeline-e2e.test.ts`.
2. This test exercises the real pipeline (or a near-real version with only the Linear API mocked if running in CI without credentials).
3. Test cases:
   a. `it('completes all 5 stages without fatal error')` — run pipeline, assert `pipelineRun.status === 'success'` and all 5 stages have `status !== 'fatal'`.
   b. `it('generates at least 5 tasks')` — assert `report.total_tasks >= 5`.
   c. `it('assigns at least 5 tasks with delegate_id')` — assert `report.assigned_tasks >= 5`.
   d. `it('Linear issues have correct assignees')` — for each issue in the report, assert the verification result `matched === true` for at least 5 issues.
   e. `it('no known agent hints remain as agent:pending')` — cross-reference agent mapping, assert no issues with known mappings are stuck at pending.
   f. `it('report endpoint returns valid JSON')` — call `GET /api/validation/report/{run_id}`, assert 200 and schema compliance.
   g. `it('research_included reflects deliberation state')` — if research was available, assert `research_included === true` and deliberation contains non-empty memos.
4. Use `beforeAll` to run the pipeline once and share the report across all assertions.
5. Set a generous test timeout (60s) since this involves multiple stages and possible API calls.

## Validation
All 7 test cases pass. The test suite completes within 60s. No unhandled exceptions. The test can run in CI with Linear API mocked (environment variable toggle for real vs. mocked Linear).