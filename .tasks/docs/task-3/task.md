## Integrate Hermes Research for Deliberation Path (Nova - Bun/Elysia)

### Objective
Enable Hermes research integration in the deliberation path of the PM server pipeline. When NOUS_API_KEY is available, the deliberation stage should call the Hermes research endpoint and include sourced content in the research memo that feeds into task generation.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. In the deliberation module of the PM server, add a conditional check for the `NOUS_API_KEY` environment variable (sourced from the `nous-api-key` secret via ConfigMap).
2. When the key is present, call the Hermes research endpoint (`NOUS_API_BASE/research`) with:
   - The PRD title and description as the research query.
   - A `max_results` parameter of 10.
   - A timeout of 30 seconds.
3. Parse the Hermes response which returns an array of `{ title, summary, url, relevance_score }` objects.
4. Filter results with `relevance_score >= 0.5`.
5. Format filtered results into a structured research memo section:
   ```
   ## Hermes Research Findings
   - **{title}** ({relevance_score}): {summary} [source]({url})
   ```
6. Append this section to the deliberation output before it is passed to the task generation stage.
7. When `NOUS_API_KEY` is not available, log an info message and skip research integration gracefully (no errors).
8. Store the raw Hermes response in the deliberation artifacts directory for audit purposes.

### Subtasks
- [ ] Implement Hermes API client with conditional NOUS_API_KEY check and timeout: Create a TypeScript module that checks for the NOUS_API_KEY environment variable, constructs requests to the Hermes research endpoint, handles the 30-second timeout, and parses/filters the response by relevance score.
- [ ] Format research memo section and integrate into deliberation output with artifact storage: Take filtered Hermes results, format them into a Markdown research memo section, append it to the deliberation output, and persist the raw Hermes response to the deliberation artifacts directory.
- [ ] Write comprehensive unit and integration tests for Hermes research integration: Create test files covering all branches: API available with valid results, API key missing, timeout handling, low-relevance filtering, empty results, and artifact persistence.