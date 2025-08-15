# GitHub Workflow Guidelines

## CI must be green before PR or marking tasks done

- Run local quality gates before pushing:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
  cargo test --all-features
  ```
- Push to your feature branch and wait for GitHub Actions to complete
- Only open a PR when all jobs are green; if a deploy job exists, it must also succeed

## Branching & PR
- Work on a feature branch; never push directly to `main`
- Keep PRs small and focused; re-run CI until green
- Use descriptive titles and detailed PR descriptions

## Taskmaster docs
- Apply the CI gating to all tasks in `.taskmaster/docs/*` for consistent acceptance criteria
