# Intake as GitHub Action (self-hosted)

This document describes the GitHub Actions port of the Argo `intake.lobster.yaml`
workflow. It lives at `.github/workflows/intake.yaml` and is exposed as a
**reusable** workflow (`workflow_call`) plus a manual `workflow_dispatch`
trigger.

## Invocation pattern (from managed repos)

Managed repos adopt intake with a small trigger workflow that calls the
reusable workflow in this repo:

```yaml
# .github/workflows/intake.yaml (in the managed repo)
name: intake
on:
  workflow_dispatch:
  push:
    paths:
      - 'docs/prd.md'

jobs:
  run:
    uses: 5dlabs/cto/.github/workflows/intake.yaml@main
    with:
      target_repo: ${{ github.repository }}
      prd_path: docs/prd.md
      ref: ${{ github.ref_name }}
      open_pr: true
    secrets:
      CONTEXT7_API_KEY: ${{ secrets.CONTEXT7_API_KEY }}
      OPENAI_API_KEY:   ${{ secrets.OPENAI_API_KEY }}
      ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
      GEMINI_API_KEY:   ${{ secrets.GEMINI_API_KEY }}
      ELEVENLABS_API_KEY: ${{ secrets.ELEVENLABS_API_KEY }}
      CTO_INTAKE_PAT:   ${{ secrets.CTO_INTAKE_PAT }}
```

## Inputs

| Input          | Required | Default         | Purpose                                 |
| -------------- | -------- | --------------- | --------------------------------------- |
| `target_repo`  | yes      | —               | `owner/name` of the managed repo.       |
| `prd_path`     | no       | `docs/prd.md`   | PRD path inside `target_repo`.          |
| `ref`          | no       | `main`          | Git ref to check out of `target_repo`.  |
| `voice`        | no       | `alloy`         | TTS voice for `briefing.mp3`.           |
| `project_name` | no       | `intake`        | Slug used in branch + artifact names.   |
| `open_pr`      | no       | `true`          | Open PR against `target_repo`.          |

## Required secrets

| Secret               | Required | Used for                                         |
| -------------------- | -------- | ------------------------------------------------ |
| `CONTEXT7_API_KEY`   | yes      | Catalog query / custom skill generation.         |
| `CTO_INTAKE_PAT`     | rec.     | Cross-repo checkout + PR creation on target.     |
| `OPENAI_API_KEY`     | opt.     | TTS (via `lobster-voice`) and LLM fallback.      |
| `ANTHROPIC_API_KEY`  | opt.     | LLM calls for capability analysis.               |
| `GEMINI_API_KEY`     | opt.     | Default analysis/catalog provider.               |
| `ELEVENLABS_API_KEY` | opt.     | Alternative TTS provider.                        |

If `CTO_INTAKE_PAT` is not provided the workflow falls back to
`GITHUB_TOKEN`, which cannot open PRs against a different repo. Supply a PAT
with `repo` + `workflow` scope for any cross-repo run.

## Self-hosted runner

Jobs run on `runs-on: [k8s-runner]` — the label used by the existing in-cluster
ARC runner set (see `kubectl get runnersets -A`). The workflow assumes the
following are available on the runner image (all optional degradations are
logged as `::warning::`):

- `jq`, `git`, `python3`, `bash`, `curl` (hard requirements, preflight will
  fail if missing).
- `intake-util`, `openclaw`, `lobster-voice`, `npx`, `gh` (soft requirements —
  some phases degrade gracefully when absent, e.g. TTS is skipped without a
  provider key or binary).

## Outputs

Two artifacts are uploaded per run:

- `intake-briefing-<owner>-<repo>-<run_id>` → `briefing.mp3`
- `intake-docs-<owner>-<repo>-<run_id>`    → `docs/` (briefing.md, manifest, phase JSON)

On-runner layout: `_intake_out/briefing.mp3` and `_intake_out/docs/*`.

If `open_pr=true` the docs are copied into `docs/intake/` of the target repo
and a PR is opened on branch `intake/<project_name>-<run_id>`.

## Argo phase mapping

| Argo step                          | Reusable workflow step                      |
| ---------------------------------- | ------------------------------------------- |
| `inventory-effective-tools`        | `Inventory — effective tools`               |
| `inventory-effective-skills`       | `Inventory — effective skills`              |
| `analyze-required-capabilities`    | `Analyze required capabilities`             |
| `compute-capability-gaps`          | `Compute capability gaps`                   |
| `query-tools-catalog`              | `Query tools catalog (Context7)`            |
| `search-and-resolve-gaps`          | `Search and resolve gaps`                   |
| `install-resolved-skills`          | `Install resolved skills`                   |
| `generate-custom-skills`           | `Generate custom skills (Context7-backed)`  |
| `generate-agent-package`           | `Generate agent package`                    |
| `validate-agent-package`           | `Validate agent package`                    |
| *(new)* `generate-briefing-md`     | `Generate briefing.md`                      |
| *(new)* `tts-mp3`                  | `Render briefing.mp3 (TTS)`                 |
| `commit-agent-package`             | `Commit generated docs + open PR`           |

### Known gaps vs. Argo

The following Argo behaviour is **not** faithfully reproduced and must be
covered by the reconciler (Phase C) or a follow-up PR:

1. **Linear activity/plan syncing** (`linear-plan-*`, `linear-activity-*`,
   `sync-linear-issues*`). Not wired here — the reusable workflow focuses on
   artifact generation. Reconciler should post progress events from the CRD.
2. **Task fan-out phases** (`parse-prd`, `analyze-complexity`, `refine-tasks`,
   `fan-out-docs`, `fan-out-prompts`, `generate-scale-tasks`,
   `generate-security-report`, `generate-remediation-tasks`). These rely on
   in-cluster agent pods; the GH Actions port emits a stub capabilities result
   when `openclaw` is unavailable and defers the full fan-out to the
   controller-managed agents.
3. **Checkpoint / resume semantics** (`$root/.intake/checkpoints/*`). Not yet
   ported — each GH Actions run is idempotent-from-scratch.
4. **Breakpoint / verify-step audio gates**. Only the generate/tts phases emit
   TTS; per-step audio is suppressed (`audio_debug=false`).
5. **PRD CRD upsert** — placeholder `TODO(phase-c)` step is present so the
   plug-in point is visible. Phase A supplies the CRDs; Phase C wires the
   reconciler that will consume them.

## Troubleshooting

- **Runner offline / queued forever**: confirm the `k8s-runner` RunnerSet is
  healthy: `kubectl get runnersets -A && kubectl get pods -l app=runner -A`.
- **Checkout of target repo fails**: the caller must pass `CTO_INTAKE_PAT`
  with `repo` scope; `GITHUB_TOKEN` cannot cross repos.
- **`openclaw: command not found`**: the runner image needs the in-cluster
  intake tooling installed. The workflow falls back to stub outputs so the
  run completes, but the resulting manifest will be empty.
- **TTS skipped**: no `OPENAI_API_KEY` / `ELEVENLABS_API_KEY`, or
  `lobster-voice` is not on `PATH`. An empty `briefing.mp3` is still uploaded
  so the artifact contract holds.
- **Context7 lookups silently empty**: check `CONTEXT7_API_KEY` is forwarded
  and that the runner has outbound HTTPS to `context7.com`.

## TTS text rules

The briefing template avoids em-dashes, underscores, and colon-heavy prose
so the TTS output sounds natural. When editing `Generate briefing.md`, keep
sentences plain and punctuated with periods + commas only.
