Implement subtask 8002: Configure CI credential injection from sigma1-dev namespace secrets

## Objective
Set up the CI pipeline job to inject real credentials (LINEAR_API_KEY, GITHUB_TOKEN, NOUS_API_KEY, PM_SERVER_URL, DISCORD_COLLECTOR_URL) from sigma1-dev Kubernetes secrets into the E2E test runner environment.

## Steps
1. In the CI config (GitHub Actions workflow or equivalent), add a new job `e2e-integration` that runs after all build/deploy jobs.
2. Configure the job to authenticate to the sigma1-dev Kubernetes cluster.
3. Use `kubectl get secret sigma1-dev-credentials -o jsonpath` (or a CI-native secret injection) to extract: LINEAR_API_KEY, GITHUB_TOKEN, NOUS_API_KEY, PM_SERVER_URL.
4. Export each as an environment variable available to the test runner step.
5. Add a step that runs `bun install` in `tests/e2e/` and then `bun test tests/e2e/` with a 10-minute job timeout.
6. Ensure secrets are masked in CI logs.
7. Add a manual trigger option (`workflow_dispatch`) so E2E can be run on-demand outside normal CI.

## Validation
Trigger the CI job manually. Verify it reaches the test execution step without secret injection errors. Confirm secrets are masked in logs. Confirm the placeholder test from 8001 passes in CI.