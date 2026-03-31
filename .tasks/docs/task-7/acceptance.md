## Acceptance Criteria

- [ ] 1. API lifecycle test passes: A deliberation created via POST transitions to `completed` status within 60 seconds and has at least 2 artifacts (1 current_site_screenshot + 1 variant_snapshot).
- [ ] 2. Auth tests pass: All 3 auth scenarios (no session, wrong claim, correct claim) return expected HTTP status codes.
- [ ] 3. Browser dashboard test passes: `/hermes` page renders deliberation cards matching the count returned by the API.
- [ ] 4. Browser comparison test passes: Artifact comparison view renders two non-zero-dimension images side by side for a completed deliberation.
- [ ] 5. Accessibility audit passes: axe-core reports zero critical and zero serious violations on all Hermes pages.
- [ ] 6. CI pipeline: GitHub Actions workflow completes with exit code 0 on a staging deployment with all test suites passing; HTML report artifact is downloadable from the workflow run.
- [ ] 7. Cross-browser: All browser tests pass in Chromium, Firefox, and WebKit without modification.
- [ ] 8. Flakiness: Test suite achieves 100% pass rate over 3 consecutive CI runs (no flaky tests).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.