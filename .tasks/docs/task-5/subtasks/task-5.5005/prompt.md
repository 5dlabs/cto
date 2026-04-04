Implement subtask 5005: Integrate notify() calls into the pipeline lifecycle

## Objective
Wire the notify() function into the existing pipeline orchestration code, calling it at pipeline start, successful completion, and on fatal error.

## Steps
1. At pipeline initiation (after session creation and task decomposition begins), call `notify({ event: 'pipeline.start', pipeline_id, status: 'started', task_count, assigned_count: 0, timestamp: new Date().toISOString() })`.
2. After all Linear issues are created and the PR is submitted successfully, call `notify({ event: 'pipeline.complete', pipeline_id, status: 'completed', task_count, assigned_count, pr_url, linear_session_url, timestamp })`.
3. In the pipeline's top-level error handler (catch block for fatal errors), call `notify({ event: 'pipeline.error', pipeline_id, status: 'error', task_count, assigned_count, timestamp })`.
4. Ensure notify() calls do not add latency to the critical path — they are fire-and-forget (await them but they never throw).
5. Pass the pr_url and linear_session_url from the pipeline context into the complete event payload.

## Validation
Integration test: With mocked bridge HTTP endpoints (using Bun's test server), run the full pipeline lifecycle and verify: (1) both Discord and Linear bridges receive a POST with event='pipeline.start' at the beginning, (2) both receive a POST with event='pipeline.complete' including pr_url and linear_session_url at the end. Verify exactly 2 calls per bridge (start + complete).