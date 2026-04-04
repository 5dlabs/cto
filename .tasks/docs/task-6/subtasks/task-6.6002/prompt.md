Implement subtask 6002: Implement pipeline execution orchestrator with stage-level checkpoints

## Objective
Build the orchestration function that drives the test PRD through all 5 pipeline stages (intake, deliberation, task generation, agent delegation resolution, Linear issue creation) and captures structured checkpoint results at each stage.

## Steps
1. Create `src/validation/pipeline-runner.ts`.
2. Define a `PipelineRun` type with fields: `runId` (UUID), `stages` (array of `StageResult`), `startedAt`, `completedAt`, `status` ('success' | 'failure').
3. Define `StageResult` with: `name`, `status`, `durationMs`, `output` (unknown), `errors` (string[]).
4. Implement `runPipeline(prd: TestPRD): Promise<PipelineRun>` that sequentially executes:
   - Stage 1 — PRD parsing/intake: call the existing intake module, capture parsed output.
   - Stage 2 — Deliberation: invoke deliberation, pass through research context if available from Task 3 integration. Set `research_included` flag based on whether research memos are non-empty.
   - Stage 3 — Task generation: invoke task generation, capture the generated tasks array. Assert `tasks.length >= 5` at this checkpoint — if assertion fails, mark stage as failed and record error.
   - Stage 4 — Agent delegation resolution: call `resolve_agent_delegates()`, capture mapping results. Assert at least 5 tasks have non-null `delegate_id`. Track any tasks that fall back to `agent:pending`.
   - Stage 5 — Linear issue creation: iterate over tasks, create Linear issues with `assigneeId` set. Implement 100ms backoff between API calls to avoid rate limiting.
5. Each stage wraps execution in try/catch — capture errors but continue to produce partial results where possible.
6. Return the full `PipelineRun` object for downstream consumption.

## Validation
Unit test `runPipeline` with a mocked pipeline (mock each stage module). Verify all 5 stages execute in order. Verify that a stage failure is captured in `StageResult.errors` without crashing the orchestrator. Verify `runId` is a valid UUID. Verify backoff delay is applied between Linear API calls.