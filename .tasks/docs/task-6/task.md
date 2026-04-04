## Validate Task Generation with Agent Assignment (Nova - Bun/Elysia)

### Objective
Run the full pipeline from PRD intake through deliberation, task generation, and Linear issue creation. Validate that at least 5 tasks are generated with valid agent assignments, Linear issues have delegate_id set, and the pipeline completes through all stages without fatal errors.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 2, 3

### Implementation Details
Step-by-step implementation:

1. Prepare a test PRD input that exercises the full pipeline:
   - Use the Sigma-1 E2E PRD itself or a well-structured test PRD
   - Ensure it references at least 5 distinct agent types (Bolt, Nova, Blaze, Tess, and one more)

2. Execute the pipeline end-to-end within cto-pm:
   - Stage 1: PRD parsing and intake
   - Stage 2: Deliberation (with research from Task 3 integration)
   - Stage 3: Task generation — verify output includes agent hints
   - Stage 4: Agent delegation resolution — `resolve_agent_delegates()` maps hints to Linear user IDs
   - Stage 5: Linear issue creation with `assigneeId` set

3. Validation checkpoints at each stage:
   - After task generation: assert `tasks.length >= 5`
   - After delegation resolution: assert at least 5 tasks have non-null `delegate_id`
   - After issue creation: for each created issue, query Linear API to confirm `assignee` is set (not null, not just `agent:pending` label)

4. Handle edge cases:
   - If fewer than 5 agent mappings exist, this is a blocker — log which agents failed resolution
   - If Linear API rate-limits issue creation, implement backoff (100ms between creates)
   - Track which tasks fell back to `agent:pending` and report as warnings

5. Produce a validation report:
   ```json
   {
     "run_id": "...",
     "total_tasks": N,
     "assigned_tasks": N,
     "pending_tasks": N,
     "linear_session_url": "...",
     "issues": [
       { "id": "...", "title": "...", "agent": "...", "delegate_id": "...", "linear_url": "..." }
     ],
     "research_included": true/false,
     "warnings": [...]
   }
   ```

6. Store the validation report in pipeline state — it will be consumed by Task 5 (notifications), Task 4 (PR content), and Task 8 (E2E test).

7. Expose the report via `GET /api/validation/report/{run_id}`.

### Subtasks
- [ ] Prepare test PRD fixture with 5+ distinct agent type references: Create a well-structured test PRD document that references at least 5 distinct agent types (Bolt, Nova, Blaze, Tess, and at least one more such as Rex or Grizz). This PRD will serve as the canonical input for the full pipeline validation run.
- [ ] Implement pipeline execution orchestrator with stage-level checkpoints: Build the orchestration function that drives the test PRD through all 5 pipeline stages (intake, deliberation, task generation, agent delegation resolution, Linear issue creation) and captures structured checkpoint results at each stage.
- [ ] Implement Linear API verification for issue assignees: After Linear issues are created in Stage 5, query the Linear API for each created issue to independently confirm that the `assignee.id` matches the expected `delegate_id` from the delegation resolution stage.
- [ ] Build validation report generator with edge case tracking: Consume the PipelineRun output and LinearVerificationResults to produce the structured validation report JSON with all required fields, including warnings for edge cases like agent mapping failures and fallback to agent:pending.
- [ ] Expose GET /api/validation/report/{run_id} Elysia endpoint: Create the Elysia HTTP endpoint that serves the validation report by run_id, returning 200 with the full JSON report or 404 if the run_id is not found.
- [ ] Write end-to-end integration test for full pipeline validation: Create an integration test that invokes the pipeline runner with the test PRD fixture, runs all stages, verifies Linear assignees, generates the report, and asserts all acceptance criteria from the test strategy.