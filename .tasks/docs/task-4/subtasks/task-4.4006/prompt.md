Implement subtask 4006: Implement pipeline orchestrator that wires scaffold generation, GitHub commit, and PR creation together and stores PR URL in pipeline state

## Objective
Create the top-level orchestration function that invokes scaffold generation, commits files to a new branch, opens a PR, and stores the resulting PR URL (or failure status) in the pipeline state object for downstream tasks.

## Steps
1. Create `src/pr/pipeline-pr-orchestrator.ts`.
2. Implement `executePRStep(pipelineState: PipelineState): Promise<PipelineState>` that:
   a. Reads task list and run metadata from `pipelineState`.
   b. Calls `buildFileTree(...)` from scaffold-generator to produce the file array.
   c. Calls `buildBranchName(runId)` and `createBranch(...)` from github-client.
   d. Calls `commitFileTree(...)` to push all scaffold files.
   e. Calls `openPipelinePR(...)` to create the PR with title, body, and labels.
   f. Writes `pipelineState.prStep = { success: true, prUrl, prNumber }` on success.
   g. On any graceful failure, writes `pipelineState.prStep = { success: false, error }` and continues.
3. Ensure the function is exported as the public entry point for this pipeline step.
4. Wire into the main pipeline execution chain (e.g., called after task delegation, before notifications).
5. Add a Bun-compatible Elysia route (if pipeline is HTTP-triggered) or export for direct invocation.

## Validation
Integration test: call `executePRStep` with a mock pipeline state containing 5+ tasks — verify `pipelineState.prStep.success` is true and `pipelineState.prStep.prUrl` is a valid URL string. Integration test: call `executePRStep` with GitHub client mocked to fail on PR creation — verify `pipelineState.prStep.success` is false and `pipelineState.prStep.error` is populated. Verify the function does not throw in either case.