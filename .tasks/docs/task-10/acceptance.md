## Acceptance Criteria

- [ ] Push a commit to the repository — CI pipeline triggers and all stages complete successfully: (1) Stage 1: `anchor build` produces `cto_billing.so` (file size > 100KB). `cargo clippy` exits with 0 warnings. `cargo fmt --check` exits with code 0. Dashboard builds without error. (2) Stage 2: All 38+ Anchor integration tests from Task 8 pass (JUnit XML artifact uploaded). Receipt uploader unit tests pass. (3) Stage 3 (on manual trigger): Program deploys to devnet — `solana program show <program_id> --url devnet` returns program info with 'Deployed' status. Smoke test completes: operator initialized, customer account created, small deposit verified on-chain. (4) Full CI pipeline completes in under 15 minutes. (5) `make test` locally reproduces the same test results as CI. Makefile targets `build`, `test`, `lint`, `deploy-devnet` all work without error.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.