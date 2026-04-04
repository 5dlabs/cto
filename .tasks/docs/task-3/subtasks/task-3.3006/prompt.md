Implement subtask 3006: Integrate research provider into the deliberation path and write research memos

## Objective
Hook the research provider selector into the existing cto-pm deliberation pipeline — invoke it after PRD parsing and before task generation, write results as research memo files to the deliberation output directory.

## Steps
1. Locate the existing deliberation pipeline entry point in the cto-pm codebase (likely in `src/deliberation/` or similar).
2. After the PRD parsing step and before the task generation step, add a call to `ResearchProviderSelector.executeWithFallback()` passing the parsed PRD content as the query.
3. Create `src/research/memo-writer.ts` with function `writeResearchMemo(result: ResearchResult, outputDir: string): Promise<string>` that:
   - Generates a filename like `research-memo-{timestamp}.md`
   - If `result.skipped === false`: writes markdown with provider name, content, response time, and timestamp
   - If `result.skipped === true`: writes markdown stating research was unavailable, the skip reason, and timestamp
   - Returns the path to the written file
4. In the deliberation path, call `writeResearchMemo()` and include the memo path in the deliberation context passed to task generation.
5. Ensure the entire research block is wrapped in a try/catch that logs errors and continues the pipeline — research must never block deliberation.
6. Add structured log: `[research:pipeline] Research phase complete — provider={provider}, skipped={skipped}, duration={ms}`.

## Validation
Integration test: (1) Run the deliberation path with a mock Hermes returning content — verify a research memo file exists in the output directory with Hermes content. (2) Run with Hermes unavailable and NOUS_API_KEY set — verify memo has NOUS content. (3) Run with both unavailable — verify memo exists with skip notice and timestamp. (4) Verify the pipeline completes successfully in all three scenarios (no thrown errors). (5) Verify the research memo path is included in the deliberation context for task generation.