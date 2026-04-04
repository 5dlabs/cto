## Integrate Hermes Research Content in Deliberation Path (Nova - Bun/Elysia)

### Objective
Add a Hermes research integration module to the PM server's deliberation stage. When NOUS_API_KEY is available, the module calls the Hermes/NOUS API to generate research memos that are embedded directly on each task's research_memo field. The integration must gracefully degrade when the API key is unavailable or the Hermes service is unreachable.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Create a new internal module `hermes-research` within the PM server codebase with a clean interface: `async function fetchResearchMemo(taskContext: TaskContext): Promise<ResearchMemo | null>`.
2. Define the `ResearchMemo` type: `{ content: string, source: string, timestamp: Date }`.
3. Extend the task entity/type definition to include `research_memo: ResearchMemo | null`.
4. In the deliberation pipeline stage, after initial task context is assembled, call `fetchResearchMemo()` for each task.
5. The Hermes API call should: read `NOUS_API_KEY` from environment (injected from `sigma-1-secrets`), send the task description/context as the research query, parse the response into the `ResearchMemo` structure, store the raw Hermes response verbatim in `content`.
6. Implement graceful degradation: if `NOUS_API_KEY` is not set, log an info message ('Hermes integration skipped: NOUS_API_KEY not configured') and set `research_memo` to null. If the Hermes API returns an error or times out (30s timeout), log a warning and set `research_memo` to null. Pipeline must never fail due to Hermes unavailability.
7. Ensure the module interface is clean enough for future extraction into a separate service per D1.
8. Write unit tests for: successful memo fetch, missing API key skip, API timeout handling, API error handling.
9. Write an integration test that verifies research memos appear in the deliberation output when the API key is provided.

### Subtasks
- [ ] Define ResearchMemo type and extend task entity type: Create the ResearchMemo TypeScript type definition and extend the existing task entity/type to include the research_memo field as ResearchMemo | null.
- [ ] Implement Hermes API client with NOUS_API_KEY reading and 30s timeout: Create the core hermes-research module with the fetchResearchMemo function that reads NOUS_API_KEY from environment, calls the Hermes/NOUS API with the task context, and parses the response into a ResearchMemo.
- [ ] Implement graceful degradation for missing API key, timeouts, and API errors: Add all error handling paths to fetchResearchMemo: missing NOUS_API_KEY skip with info log, 30s timeout handling with warning log, and HTTP error handling with warning log. None of these should throw.
- [ ] Integrate fetchResearchMemo into the deliberation pipeline stage: Wire the hermes-research module into the existing deliberation pipeline so that fetchResearchMemo is called for each task after initial task context assembly, and the returned memo is stored on the task's research_memo field.
- [ ] Write comprehensive unit and integration tests for hermes-research module: Create the full test suite covering all fetchResearchMemo paths and the pipeline integration, using Bun's test runner and mocked HTTP responses.