Implement subtask 4004: Implement PR creation with formatted body and summary

## Objective
Implement the GitHub API call to create a pull request from the pipeline branch to main, with a title following the convention and a body summarizing the pipeline run.

## Steps
1. Add to `src/design-snapshot/github-ops.ts` a function `createPullRequest(sessionId: string, pipelineOutput: PipelineOutput): Promise<string>` that returns the PR URL.
2. Call `POST /repos/{owner}/{repo}/pulls` with: head `pipeline/<session-id>`, base `main`, title `[Pipeline] <session-id> - Task Scaffolds`.
3. Generate the PR body containing: total task count, a table or list of tasks with their agent assignments, a summary of research memo availability (e.g., '3/5 tasks have research memos'), session ID, and timestamp.
4. Parse the response to extract the `html_url` field as the PR URL.
5. Return the PR URL for inclusion in PRResult.

## Validation
Unit test: Given a PipelineOutput with 5 tasks (3 with research memos, 2 without), verify the PR body contains '5 tasks', lists all agent assignments, and states '3/5 tasks have research memos'. Verify the PR title matches `[Pipeline] <session-id> - Task Scaffolds`.