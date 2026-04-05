Implement subtask 10016: CI/CD: GitHub Actions PR workflow (lint, build, test, security scan)

## Objective
Create a GitHub Actions workflow that runs on every PR: linting for all languages, building all projects, running tests with 80% coverage enforcement, and security scanning.

## Steps
Step-by-step:
1. Create `.github/workflows/pr.yaml` triggered on `pull_request` to `main`.
2. Define jobs (can run in parallel):
   a. **rust-checks**:
      - Checkout, install Rust toolchain, cache cargo registry
      - `cargo clippy --all-targets --all-features -- -D warnings`
      - `cargo build --workspace`
      - `cargo test --workspace` with `cargo-llvm-cov` for coverage, fail if < 80%
   b. **go-checks**:
      - Checkout, setup Go, cache modules
      - `golangci-lint run ./...`
      - `go build ./...`
      - `go test ./... -coverprofile=coverage.out`, parse coverage and fail if < 80%
   c. **node-checks**:
      - Checkout, setup Bun
      - `bun run lint` (Biome)
      - `bun build`
      - `bun test --coverage`, fail if < 80%
   d. **nextjs-checks**:
      - Checkout, setup Node.js
      - `npx eslint .`
      - `next build`
      - `vitest run --coverage`, fail if < 80%
   e. **security-scan**:
      - Semgrep scan with `--config auto`
      - CodeQL analysis (setup + analyze)
      - Check Dependabot alerts (or `npm audit` / `cargo audit`)
   f. **container-build** (depends on all above passing):
      - Build Docker images for all services (no push)
      - Verify images build successfully
3. Set `concurrency` group to cancel stale PR runs.

## Validation
Push a branch with an intentional Clippy warning, open a PR, verify the rust-checks job fails with the lint error. Fix the warning, push again, verify all jobs pass. Verify the workflow YAML is valid by running `actionlint` locally. Check that coverage thresholds are enforced by temporarily reducing test coverage and verifying failure.