## Decision Points

- Should the E2E test use a live Linear API with a dedicated test project/team, or recorded mock responses? Live provides higher fidelity but requires test project provisioning, cleanup logic, and a valid API token in the test environment. Mocks are deterministic but won't catch API contract drift.
- Should the test create real PRs on sigma-1 (e.g., on a `test/e2e-*` branch prefix) or mock the GitHub API? Real PRs validate the full flow including branch creation and file content, but risk rate limits and leave artifacts if cleanup fails.
- How should Discord notification delivery be verified? Options: rely on the in-cluster bridge returning 2xx and inspect pipeline logs, use an HTTP interceptor/spy to capture outbound requests, or query a dedicated test Discord channel for message presence.

## Coordination Notes

- Agent owner: tess
- Primary stack: Test frameworks