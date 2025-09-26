# Task 2.2 – Secrets & GitOps Updates for Cursor API Key

## Dependencies
- Task 2.1 (Helm values referencing Cursor secret placeholders).

## Parallelization Guidance
- Can run alongside Group 3 preparation as long as secret names are finalised before job specs rely on them.

## Task Prompt
Plumb the Cursor API key through our secrets infrastructure so controller jobs receive `CURSOR_API_KEY` without hardcoding sensitive values.

Steps:
1. External Secrets:
   - Update `infra/gitops/resources/**/secret.yaml` or equivalent ExternalSecret manifests to include `cursor-api-key` entries sourced from the existing secret store (`agent-platform-secrets` namespace, as used for Codex/OpenAI keys).
   - Ensure naming follows convention (e.g., `cursor-api-key` with key `CURSOR_API_KEY`).
2. Controller Helm chart (`infra/charts/controller/templates/secret.yaml`):
   - Add map entries so the controller deployment exposes `CURSOR_API_KEY` environment variables (parallel to `OPENAI_API_KEY`).
   - Confirm any placeholders or template loops pick up the new key automatically.
3. GitOps validation:
   - Run `make -C infra/gitops validate` to ensure yamllint/security checks pass (line-length warnings are known; focus on new errors).
   - If new warnings arise, adjust manifests (e.g., break long lines, add `safe-string` annotations).
4. Documentation:
   - Note in `infra/README.md` (or Group 4 tasks) how to provision the Cursor key, referencing the ExternalSecret path.

## Acceptance Criteria
- ExternalSecret manifests reconcile without errors (dry-run via `kubectl neat` or rendering templates in CI).
- Controller deployment pod spec contains `env:
  - name: CURSOR_API_KEY
    valueFrom: ...` in `infra/charts/controller/templates/task-controller-config.yaml` output.
- `make -C infra/gitops validate` succeeds.
- No accidental exposure of secrets in repository (check `git diff` for literal API keys).

## Implementation Notes / References
- Follow the pattern used for `OPENAI_API_KEY` (Codex) to keep operations simple.
- Remember staging vs production secret naming conventions—document both if they differ.
- Update any CI workflows reference (e.g., `scripts/test-argocd-local.sh`) if they need the new env var to avoid false negatives.
