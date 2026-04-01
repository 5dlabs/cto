Implement subtask 8001: Set up E2E test framework and project scaffolding

## Objective
Initialize the sigma1-e2e test project with the chosen test framework (vitest or bun:test), configure TypeScript, create the `sigma1-e2e.test.ts` entry file, and establish shared utility modules for HTTP requests, polling, and assertions.

## Steps
1. Create directory `tests/e2e/` in the sigma-1 repo.
2. Add `vitest.config.ts` (or equivalent bun:test config) with a 10-minute global timeout.
3. Create `tests/e2e/helpers/http-client.ts` — a lightweight fetch wrapper that reads `PM_SERVER_URL` from env and provides typed `get`/`post` helpers.
4. Create `tests/e2e/helpers/poll.ts` — a generic async poller: takes a URL, predicate function, interval (default 5s), and max timeout (default 5min); throws on timeout with last response.
5. Create `tests/e2e/fixtures/sample-prd.json` — a minimal but realistic PRD payload that exercises all pipeline stages.
6. Create `tests/e2e/helpers/env.ts` — reads and validates required env vars (PM_SERVER_URL, LINEAR_API_KEY, GITHUB_TOKEN, DISCORD_COLLECTOR_URL, NOUS_API_KEY optional) and exports them typed.
7. Verify `bun test` or `vitest` can discover and run a trivial placeholder test in the new directory.

## Validation
Run `bun test tests/e2e/` (or vitest equivalent) and confirm the placeholder test passes. Verify all helper modules export correctly with no TypeScript errors. Confirm env.ts throws a descriptive error when required vars are missing.