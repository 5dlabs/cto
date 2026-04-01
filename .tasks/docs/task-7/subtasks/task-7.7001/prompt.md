Implement subtask 7001: Create GitHubClient wrapper with authenticated fetch and error handling

## Objective
Implement a reusable GitHubClient class/module that wraps fetch calls to the GitHub API with authentication headers, base URL resolution, JSON parsing, and structured error handling. This client will be used by all subsequent GitHub API operations.

## Steps
1. Create `src/services/github-client.ts`.
2. Read `GITHUB_PAT` from environment (injected from secret) and `GITHUB_API_BASE` from environment (injected from ConfigMap), defaulting to `https://api.github.com`.
3. Implement a class `GitHubClient` with methods: `get(path)`, `post(path, body)`, `patch(path, body)`.
4. Each method adds `Authorization: Bearer {PAT}`, `Accept: application/vnd.github+json`, and `X-GitHub-Api-Version: 2022-11-28` headers.
5. Parse response JSON; for non-2xx responses, throw a typed `GitHubApiError` with status code, message, and endpoint for debugging.
6. For 404 specifically, throw a distinguishable error subtype so callers can detect repo-inaccessible scenarios.
7. Export the client as a singleton or factory function.

## Validation
Unit test: mock global fetch; verify correct headers are sent including Authorization and Accept. Unit test: verify 200 response returns parsed JSON. Unit test: verify 404 throws GitHubApiError with correct status. Unit test: verify missing GITHUB_PAT throws a clear configuration error at construction time.