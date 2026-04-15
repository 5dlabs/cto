<identity>
You are atlas working on subtask 10002 of task 10.
</identity>

<context>
<scope>
Add Stage 2 test jobs to the CI workflow: anchor-test (running the full Anchor integration test suite with solana-test-validator and JUnit XML output) and receipt-uploader-test (Bun unit tests). These jobs depend on Stage 1 anchor-build.
</scope>
</context>

<implementation_plan>
1. Add `anchor-test` job to ci.yml:
   - Set `needs: [anchor-build]` to depend on Stage 1.
   - Install Solana CLI, Anchor CLI, Rust toolchain (same as anchor-build, consider composite action or reusable workflow to DRY).
   - Download the `cto_billing.so` artifact from anchor-build.
   - Option A: Run `anchor test` which handles localnet lifecycle automatically.
   - Option B: Start `solana-test-validator` as background process, deploy the .so via `solana program deploy`, then run tests with `--skip-build --skip-deploy`.
   - Choose Option A for simplicity unless test suite requires custom validator config.
   - Configure test output as JUnit XML. If using mocha (Anchor default), add `--reporter mocha-junit-reporter --reporter-options mochaFile=./test-results/results.xml`.
   - Upload test-results/*.xml as artifact using `actions/upload-artifact@v4`.
   - Optionally use `dorny/test-reporter@v1` to display test results in PR.
2. Add `receipt-uploader-test` job:
   - Set `needs: [anchor-build]` (or no dependency if independent).
   - Install Bun.
   - Run `cd packages/receipt-uploader && bun install && bun test`.
   - If Irys integration tests exist, skip them in CI (e.g., via env var `SKIP_IRYS_INTEGRATION=true` or test tag).
3. Ensure both test jobs fail the pipeline if any test fails (default behavior with `set -e`).
4. Verify test timeout is reasonable (e.g., 15 minutes max for anchor-test).
</implementation_plan>

<validation>
Push a commit with the test stage configured. anchor-test job runs all 38+ integration tests and produces JUnit XML artifact. Receipt-uploader-test job runs unit tests. Both jobs depend on Stage 1 completion. If any test fails, the CI pipeline reports failure. JUnit XML is uploaded and parseable. Total test stage completes within 10 minutes.
</validation>