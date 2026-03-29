Implement subtask 3008: Define CI/CD pipeline with lint, test, build, and deploy stages

## Objective
Create a GitHub Actions (or equivalent) CI/CD pipeline definition with PR checks (fmt, clippy, test, sqlx prepare check), Docker build+push on merge, Helm deploy to staging, and manual approval for production.

## Steps
1. Create `.github/workflows/ci.yaml` for PR checks:
   - Trigger: on pull_request to main.
   - Jobs:
     a. **lint**: `cargo fmt --check`, `cargo clippy -- -D warnings`.
     b. **test**: Start Postgres service container, set DATABASE_URL, `cargo sqlx prepare --check`, `cargo test`.
     c. **docker-lint**: `hadolint Dockerfile` (optional but recommended).
2. Create `.github/workflows/cd.yaml` for deployment:
   - Trigger: on push to main (after PR merge).
   - Jobs:
     a. **build**: Docker build with multi-stage Dockerfile, tag with git SHA and `latest`, push to container registry (parameterized via secrets: REGISTRY_URL, REGISTRY_USER, REGISTRY_PASSWORD).
     b. **deploy-staging**: Depends on build. Run `helm upgrade --install notifycore infra/notifycore -f values-dev.yaml --set image.tag=$SHA -n notifycore-staging`. Wait for rollout.
     c. **deploy-production**: Depends on deploy-staging. Uses `environment: production` for manual approval gate. Run `helm upgrade --install notifycore infra/notifycore -f values-prod.yaml --set image.tag=$SHA -n notifycore`.
3. Include sqlx offline mode: ensure `cargo sqlx prepare --check` step validates that `.sqlx/` query cache is up to date.
4. Add caching for Cargo registry and target directory to speed up CI.
5. Set appropriate timeouts (build: 15min, deploy: 10min).

## Validation
CI pipeline file `.github/workflows/ci.yaml` exists and contains jobs for fmt check, clippy, sqlx prepare check, and cargo test. CD pipeline file `.github/workflows/cd.yaml` exists and contains build, deploy-staging, and deploy-production jobs. Production deploy job uses `environment: production` for manual approval. Pipeline YAML is valid (can be validated with actionlint or similar tool). sqlx prepare check step is present in the test job.