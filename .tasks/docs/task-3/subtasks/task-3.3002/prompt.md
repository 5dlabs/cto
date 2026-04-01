Implement subtask 3002: Format research memo section and integrate into deliberation output with artifact storage

## Objective
Take filtered Hermes results, format them into a Markdown research memo section, append it to the deliberation output, and persist the raw Hermes response to the deliberation artifacts directory.

## Steps
1. Create or extend `src/deliberation/research-memo.ts`.
2. Export a function `formatHermesMemo(results: HermesResult[]): string` that:
   - Returns an empty string if the results array is empty.
   - Otherwise builds a Markdown section starting with `## Hermes Research Findings\n`.
   - For each result, appends: `- **{title}** ({relevance_score}): {summary} [source]({url})\n`.
3. In the main deliberation pipeline file (e.g., `src/deliberation/index.ts` or equivalent):
   a. After existing deliberation logic, call `fetchHermesResearch({ title: prd.title, description: prd.description })`.
   b. If the result is not null and has length > 0, call `formatHermesMemo(results)` and append the returned string to the deliberation output text before passing it to the task generation stage.
   c. If the result is null (key missing or error), do nothing — deliberation output proceeds unchanged.
4. Implement artifact storage:
   a. Determine the artifacts directory path from config or convention (e.g., `artifacts/deliberation/{sessionId}/`).
   b. If Hermes was called (regardless of filtering), write the raw JSON response to `hermes-raw-response.json` in that directory using `Bun.write()`.
   c. Ensure the directory is created if it doesn't exist (`mkdir -p` equivalent via `fs`).
5. Ensure the deliberation module's overall export/interface remains unchanged for downstream consumers (task generation stage receives the same shape, just with an optional extra Markdown section).

## Validation
Unit test: pass a list of 3 HermesResult objects to formatHermesMemo; verify output contains '## Hermes Research Findings' header and all 3 entries formatted correctly with Markdown links. Unit test: pass empty array; verify empty string returned. Integration test: run the deliberation pipeline with mocked Hermes client returning results; verify the final deliberation output string contains the Hermes section appended. Verify hermes-raw-response.json file is written to the artifacts directory with correct JSON content.