Implement subtask 6004: Build validation report generator with edge case tracking

## Objective
Consume the PipelineRun output and LinearVerificationResults to produce the structured validation report JSON with all required fields, including warnings for edge cases like agent mapping failures and fallback to agent:pending.

## Steps
1. Create `src/validation/report-generator.ts`.
2. Define the `ValidationReport` type matching the schema from the task details:
   ```typescript
   interface ValidationReport {
     run_id: string;
     total_tasks: number;
     assigned_tasks: number;
     pending_tasks: number;
     linear_session_url: string;
     issues: Array<{ id: string; title: string; agent: string; delegate_id: string; linear_url: string }>;
     research_included: boolean;
     warnings: string[];
     stages: Array<{ name: string; status: string; durationMs: number }>;
   }
   ```
3. Implement `generateReport(pipelineRun: PipelineRun, verificationResults: LinearVerificationResult[]): ValidationReport`:
   - Extract `run_id` from `pipelineRun.runId`.
   - Count `total_tasks` from Stage 3 output, `assigned_tasks` from tasks with non-null `delegate_id`, `pending_tasks` as the remainder.
   - Map verification results into the `issues` array.
   - Set `research_included` based on Stage 2 deliberation output.
   - Populate `warnings` array: include any agents that failed resolution, any issues that fell back to `agent:pending`, any verification mismatches.
   - Include stage timing summaries.
4. Implement `storeReport(report: ValidationReport): Promise<void>` that persists the report to pipeline state (in-memory store or file-based, depending on existing patterns in cto-pm).
5. Implement `getReport(runId: string): Promise<ValidationReport | null>` for retrieval by the API endpoint and downstream tasks (Task 5, Task 4, Task 8).

## Validation
Unit test `generateReport` with mock PipelineRun and verification data. Assert output matches the ValidationReport schema. Assert `warnings` array contains entries when agent mappings fail. Assert `research_included` is true when deliberation has non-empty research memos and false otherwise. Assert `storeReport` + `getReport` round-trips correctly.