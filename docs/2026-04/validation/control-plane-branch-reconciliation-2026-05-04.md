# Control Plane Branch Reconciliation — 2026-05-04

Timestamp: 2026-05-04T02:02:40Z

Heartbeat refresh: 2026-05-04T02:51:35Z

## Purpose

Capture the mergeability/evidence handoff state for the local CTO Discord control-plane work-loop without doing broad rebase or force-push surgery from the autonomous heartbeat.

## Current branch state

Command evidence:

```bash
git fetch origin main --prune
git status --short --branch
git log --oneline HEAD..origin/main
git log --oneline origin/main..HEAD
```

Observed state at 2026-05-04T02:02:40Z:

```text
## main...origin/main [ahead 12, behind 16]
?? .hermes/
```

Refreshed state at 2026-05-04T02:26:34Z after this note itself was committed and `git fetch origin main --prune` was rerun:

```text
## main...origin/main [ahead 13, behind 16]
?? .hermes/
```

Refreshed state at 2026-05-04T02:51:35Z after `git fetch origin main --prune` and the safety-branch push:

```text
## main...origin/main [ahead 14, behind 16]
?? .hermes/
```

A safety branch was created locally at the current stack tip before rebase/force-push surgery:

```text
control-plane-presence-hardening-2026-05-04
```

The safety branch was pushed to origin in the 2026-05-04T02:51Z heartbeat without rebasing or force-pushing local `main`:

```text
origin/control-plane-presence-hardening-2026-05-04
```

Local-only commits not on `origin/main`:

```text
3fb33da6 docs(control-plane): refresh reconciliation safety branch evidence
da69ed87 docs(control-plane): record branch reconciliation handoff
ede6c98c test(presence): propagate addressing provenance
ec748efc test(control-plane): preserve attachment-only Discord text
ad287af0 docs(control-plane): reconcile smoke and roadmap artifacts
8f8faa4c test(control-plane): preserve Discord reply references
42c577cd docs(control-plane): record Morgan Hermes policy evidence
c51350c9 docs(control-plane): add operator runbook skeleton
6ce1d71b test(control-plane): harden attachment metadata contract
e4e482f9 docs(control-plane): record Hermes CodeRun dry-run smoke evidence
d7e346ae test(control-plane): harden agent coordination envelope enums
ac6fbc37 feat(control-plane): add agent coordination contract skeleton
be4dcc12 ci: add Hermes presence adapter image publish workflow
abcf0286 feat(control-plane): preserve Hermes presence home metadata
```

Remote-only commits not in the local branch:

```text
6f5292bd fix: disable duplicate Coder Discord gateway
fe3ec30c fix: remove stale codex plugin from Hermes coder gateway
d91e65eb fix: run Hermes coder gateway as main process
000d8b9d fix: install Hermes coder Discord plugin
e2195a30 fix: include Discord plugin when channel is enabled
3cd5e692 fix: enable Hermes coder Discord gateway
07d45887 Add Bclaws Hermes gateway
d79ddb58 Use runtime state for Hermes gateway health
ecb35121 Harden Hermes control-plane gateway health probes
6f226185 fix(hermes): sync Coder voice command to CTO guild
a3a40b03 feat(hermes): add Coder voice slash command (#4923)
6ba5a826 feat(hermes): enable ElevenLabs TTS for Coder voice (#4922)
fec28eb5 fix(intake): persist Copilot ACP routing via GitOps (#4921)
88c678f2 fix(intake): resolve ACP adapter path independent of WORKSPACE
6273468e feat(intake): standardize LLM calls on ACPX backend
72d32f51 fix(intake): enforce provider routing and Sigma artifact generation
```

## Merge-base and path overlap

Merge base: `85ee0d503f8c` (`feat: add runtime-neutral Discord presence control plane`).

A path-overlap check between `85ee0d503f8c..HEAD` and `85ee0d503f8c..origin/main` found:

```text
local_count 30
remote_count 48
overlap_count 0
```

Interpretation: the divergent stacks currently touch disjoint paths. The local stack is mostly presence contract hardening, validation docs/smoke harnesses, the Hermes adapter publish workflow, the isolated agent coordination skeleton, and this reconciliation note. The remote stack is mostly intake/ACPX work plus Hermes gateway/GitOps fixes. This should make reconciliation relatively low-conflict, but it still needs an explicit human- or PR-reviewed rebase/merge because the local branch name is `main` and is 14 commits ahead of the remote default branch.

## Local-only changed artifact groups

Presence contracts and tests:

- `apps/discord-bridge/src/discord-normalizer.ts`
- `apps/discord-bridge/src/discord-normalizer.test.ts`
- `apps/discord-bridge/src/presence-router.ts`
- `apps/discord-bridge/src/presence-router.test.ts`
- `apps/discord-bridge/src/presence-types.ts`
- `apps/discord-bridge/src/presence-types.test.ts`
- `apps/hermes-presence-adapter/src/hermes-client.ts`
- `apps/hermes-presence-adapter/src/index.ts`
- `apps/hermes-presence-adapter/src/index.test.ts`
- `apps/hermes-presence-adapter/src/types.ts`

Isolated Wave 2A coordination skeleton:

- `apps/agent-coordination-plane/`
- `docs/2026-04/design/agent-coordination-plane.md`

Validation, smoke, and roadmap evidence:

- `docs/2026-04/validation/control-plane-validation-matrix.md`
- `docs/2026-04/validation/control-plane-operator-runbook.md`
- `docs/2026-04/validation/hermes-presence-coderun-e2e.md`
- `docs/2026-04/plans/control-plane-completion-roadmap.md`
- `docs/2026-04/plans/hermes-parity-centralized-discord-control-plane.md`
- `docs/2026-04/research/hermes-control-plane-behavior-inventory.md`
- `docs/2026-04/research/hermes-github-source-supply.md`
- `scripts/presence-morgan-task-smoke.py`
- `scripts/presence-smoke-hermes-coderun.py`

Ops/CI candidate:

- `.github/workflows/hermes-presence-adapter-publish.yml`

Morgan policy/design evidence:

- `docs/2026-04/design/morgan-hermes-agent.md`
- `docs/2026-04/design/morgan-memory-skills-policy.md`

## Open PR state

`gh pr list --state open --head main` returned no open PR for the local `main` head.

Search for open PRs with `control plane OR presence OR discord bridge OR hermes adapter` returned no dedicated control-plane PR; the results were unrelated release/dependabot PRs.

## Recommended next safe action

Completed in the 2026-05-04T02:26Z heartbeat:

- Created local safety branch `control-plane-presence-hardening-2026-05-04` at the divergent stack tip.
- Re-ran merge-base/path-overlap analysis after fetch: local 30 paths, remote 48 paths, overlap 0.
- Re-ran GitHub PR checks: no open PR for local `main`; search still found only unrelated release/dependabot style PRs.

Completed in the 2026-05-04T02:51Z heartbeat:

- Pushed safety branch `origin/control-plane-presence-hardening-2026-05-04` so the divergent stack now has a remote handoff point.
- Re-ran branch-state inspection: local `main` is ahead 14 / behind 16; no open PR exists for `main`, and the only open control-plane/presence search hit remains unrelated release PR #4896.
- Re-ran merge-base/path-overlap analysis after fetch: local 30 paths, remote 48 paths, overlap 0.
- Re-ran no-mutation antenna checks: `git diff --check`, Python smoke harness `py_compile`, Hermes CodeRun smoke dry-run, and package-scoped tests/builds for `apps/discord-bridge`, `apps/hermes-presence-adapter`, and `apps/agent-coordination-plane` all passed.

Remaining safe sequence:

1. Open a PR from `control-plane-presence-hardening-2026-05-04` or create a PR branch derived from it; do not push local `main` directly.
2. Rebase that branch onto `origin/main` or merge `origin/main` into it, preserving the 14 local commits as reviewable chunks.
3. Rerun package-scoped validation after reconciliation:
   - `git diff --check`
   - `python3 -m py_compile scripts/presence-smoke-hermes-coderun.py scripts/presence-morgan-task-smoke.py`
   - `python3 scripts/presence-smoke-hermes-coderun.py --mode dry-run`
   - `cd apps/discord-bridge && npm test && npm run build`
   - `cd apps/hermes-presence-adapter && npm test && npm run build`
   - `cd apps/agent-coordination-plane && npm test && npm run build`
4. Open/update the PR against `main` with this reconciliation note, the validation matrix, and the smoke/runbook docs as the evidence handoff.

## Live validation blocker observed in this heartbeat

Kubernetes context exists (`in-cluster`), but the current service account cannot read the prerequisites needed for live/semi-live smoke evidence. Safe read checks failed with RBAC errors for pods/secrets/services/CRDs due to missing `cto-hermes-gateway` ClusterRole. No token values were read or printed.

Until RBAC is restored or a human runs the smoke from an authorized context, the next percentage jump into the 45–60% ladder remains blocked on live Hermes CodeRun/Discord evidence rather than more unit coverage.

Refreshed 2026-05-04T02:26Z checks:

```bash
for res in 'get secrets' 'create coderuns' 'delete coderuns' 'get pods' 'get pods/log'; do
  kubectl auth can-i $res -n cto
done
```

All checks still returned `no` with the same missing `cto-hermes-gateway` ClusterRole error class. No secret values were read or printed.
