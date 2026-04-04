Implement subtask 3005: Integrate Hermes research call into the deliberation pipeline stage

## Objective
Wire the Hermes research orchestrator (client + circuit breaker + availability gate + fallback) into the existing deliberation pipeline, invoking it after PRD parsing and before task generation, and persisting the research memo in the deliberation artifacts.

## Steps
1. Locate the deliberation pipeline entry point (likely `src/pipeline/deliberation.ts` or similar).
2. After the PRD parsing step and before the task generation step, add a new stage: `hermes_research`.
3. Derive a research query from the parsed PRD content — extract the PRD title + key requirements as a condensed query string (keep under 2000 chars).
4. Call the research orchestrator function (which internally checks availability → circuit breaker → Hermes client → fallback).
5. Store the returned research memo (whether from Hermes, fallback, or skip) in the deliberation artifacts object under a `researchMemo` key so downstream stages and Task 8 can access it.
6. Ensure the pipeline does not fail if the research stage returns a fallback or skip memo — the memo is informational/supplementary, not blocking.
7. Pass the research memo content (when available) into the task generation stage context so it can influence task breakdown quality.
8. Persist the deliberation artifacts (including researchMemo) to the DB or pipeline state store, depending on the existing persistence mechanism.

## Validation
Integration tests: (1) Run the deliberation pipeline with a mocked Hermes returning valid content; assert the deliberation output contains `researchMemo` with `source: 'hermes'` and non-null content. (2) Run the pipeline with Hermes mocked to fail; assert the output contains a fallback memo and the pipeline completes successfully (no thrown errors). (3) Run the pipeline with no NOUS_API_KEY; assert the output contains a skip memo and pipeline completes. (4) Assert the researchMemo is persisted and retrievable from the pipeline state/DB. (5) Assert the task generation stage receives the research content in its context when Hermes succeeds.