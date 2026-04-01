Implement task 7: Generate PR in Sigma-1 Repo with Task Scaffolds (Nova - Bun/Elysia)

## Goal
Automate the creation of a pull request in the 5dlabs/sigma-1 private repository containing generated task scaffold files for the E2E pipeline run. This includes directory structure, task metadata files, and a summary README.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1, 2

## Implementation Plan
1. Create a `PRGenerator` service in the PM server.
2. Read `GITHUB_PAT` from secret and `GITHUB_API_BASE` from ConfigMap.
3. After task generation completes, collect all generated tasks with their metadata (id, title, agent, stack, description, details).
4. Create a new branch `pipeline/{runId}` from the default branch of `5dlabs/sigma-1` using the GitHub API:
   a. GET `/repos/5dlabs/sigma-1/git/ref/heads/main` to get the latest SHA.
   b. POST `/repos/5dlabs/sigma-1/git/refs` to create the branch.
5. For each task, create a scaffold file at `tasks/{taskId}-{slug}/README.md` containing:
   - Task title, agent, stack.
   - Description and implementation details.
   - Test strategy.
6. Create a root `tasks/SUMMARY.md` with a table of all tasks, their agents, priorities, and dependency graph.
7. Commit all files in a single commit using the Git tree API (create blobs → create tree → create commit → update ref).
8. Create a PR via POST `/repos/5dlabs/sigma-1/pulls` with:
   - Title: `[Pipeline {runId}] Task Scaffolds`.
   - Body: Summary of tasks generated, link to Linear session.
   - Base: main, Head: pipeline/{runId}.
9. Store the PR URL and number in the pipeline run metadata for use by notifications and frontend.
10. Handle errors: if repo is inaccessible, log error and mark PR step as failed without crashing the pipeline.

## Acceptance Criteria
1. Unit test: mock GitHub API; verify branch creation, blob/tree/commit sequence, and PR creation calls are made in correct order. 2. Unit test: verify scaffold file content for a sample task contains all required sections (title, agent, stack, description, details, test strategy). 3. Unit test: verify SUMMARY.md contains a markdown table with correct task count and agent assignments. 4. Unit test: mock 404 on repo access; verify error is logged and pipeline continues. 5. Integration test: run pipeline against 5dlabs/sigma-1; verify PR exists with correct branch name, file count matches task count + 1 (summary), and PR body contains run ID.

## Subtasks
- Create GitHubClient wrapper with authenticated fetch and error handling: Implement a reusable GitHubClient class/module that wraps fetch calls to the GitHub API with authentication headers, base URL resolution, JSON parsing, and structured error handling. This client will be used by all subsequent GitHub API operations.
- Implement branch creation from latest main SHA via GitHub Refs API: Implement a function that fetches the latest commit SHA from the default branch of 5dlabs/sigma-1 and creates a new branch `pipeline/{runId}` pointing to that SHA using the GitHub Git Refs API.
- Implement scaffold file content generators for per-task README and SUMMARY.md: Create pure functions that generate markdown content for each task's scaffold README.md and the root SUMMARY.md table. These are pure content generators with no API dependencies.
- Implement Git tree API commit flow (blobs → tree → commit → update ref): Implement the function that takes generated file contents and commits them to the pipeline branch in a single commit using GitHub's low-level Git Data API: creating blobs, assembling a tree, creating a commit object, and updating the branch ref.
- Implement PR creation via GitHub Pulls API and store PR metadata: Implement the function that creates a pull request from the pipeline branch to main, formats the PR body with run context and task summary, and stores the resulting PR URL and number in the pipeline run metadata.
- Implement PRGenerator orchestrator service with error handling: Create the top-level PRGenerator service that orchestrates the full flow: collect tasks → create branch → generate scaffolds → commit files → create PR, with error handling that logs failures and marks the PR step as failed without crashing the pipeline.
- Write comprehensive unit and integration tests for the PR generation pipeline: Create a dedicated test suite covering all GitHub API interaction sequences, scaffold content validation, error scenarios, and an end-to-end integration test verifying PR creation against the real 5dlabs/sigma-1 repo.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.