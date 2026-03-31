Implement subtask 7007: Create GitHub Actions CI workflow for Hermes E2E tests

## Objective
Create the GitHub Actions workflow file that runs the full E2E test suite against the staging environment on PR and push triggers, uploads reports, and gates ArgoCD production promotion.

## Steps
1. Create `.github/workflows/hermes-e2e.yml`:
   - Name: `Hermes E2E Tests`
   - Triggers:
     a. `pull_request` targeting `main` branch (paths: `src/hermes/**`, `tests/e2e/hermes/**`)
     b. `push` to `staging` branch
   - Runs-on: `self-hosted` runner (in-cluster via `actions.github.com` ARC)
   - Environment variables: `E2E_BASE_URL` from repository secrets

2. Job steps:
   a. Checkout code
   b. Setup Node.js (match project version)
   c. Install dependencies (`npm ci`)
   d. Install Playwright browsers (`npx playwright install --with-deps chromium firefox webkit`)
   e. Run API tests: `npx playwright test --project=api`
   f. Run browser tests: `npx playwright test --project=chromium --project=firefox --project=webkit`
   g. Upload HTML report: `actions/upload-artifact@v4` with `playwright-report/` directory
   h. Upload trace files: `actions/upload-artifact@v4` with `test-results/` directory (on failure only)
   i. Upload JUnit XML: for CI dashboard integration

3. ArgoCD promotion gating:
   - Add a final step that creates a GitHub deployment status or updates a commit status check
   - Document that ArgoCD's `argocd-notifications` or a sync hook should check for the `hermes-e2e` check status before promoting to production
   - The workflow exit code (0 = all tests pass) is the gate signal

4. Add concurrency group to prevent parallel runs against the same staging environment:
   ```yaml
   concurrency:
     group: hermes-e2e-staging
     cancel-in-progress: true
   ```

## Validation
Push a test commit to the staging branch and verify: (1) the workflow triggers, (2) all 4 projects run, (3) HTML report artifact is downloadable from the Actions run, (4) JUnit XML is uploaded, (5) workflow completes with exit code 0 if all tests pass, (6) concurrency group prevents parallel runs.