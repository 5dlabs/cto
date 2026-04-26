# PRD — Self-Hosted GitLab Cutover & CTO App Parity

**Owner**: Platform
**Status**: In progress — mirror live (in-cluster CronJob), pipelines running on self-hosted runners; remaining gap is G1 (cosmetic Argo drift) + G9/G15.
**Source of truth for sync**: this doc + `infra/charts/gitlab/`, `infra/charts/gitlab-mirror/`, `infra/gitops/applications/platform/gitlab*.yaml`, `infra/gitops/manifests/gitlab/`, `infra/docs/gitlab-mirror.md`

## Status snapshot

| ID | Item | Status |
|----|------|--------|
| G1 | Argo `gitlab` app drift | **Open** — `ComparisonError` on duplicate `TZ` env in gitaly STS, masked by `ignoreDifferences`. Live pods are healthy. |
| G2 | `gitlab-config` chicken-and-egg | **Moot** — Application no longer exists; not needed with the mirror flow below. |
| G3 | Cloudflare tunnel routes | **Done** — `gitlab.5dlabs.ai` → 302, `registry.5dlabs.ai/v2/` → 401. |
| G4 | External secrets | **Done** — pods running, smtp/registry/runner secrets present. |
| G5 | Registry push smoke test | **Done** — pipelines push to `registry.5dlabs.ai/5dlabs/<svc>` via cargo-chef. |
| G6 | Mirror activation | **Done** — replaced GH Action with in-cluster CronJob (see § Mirror). |
| G7 | Repo-root `.gitlab-ci.yml` | **Done** — full pipeline at repo root + `.gitlab/ci/` includes. |
| G8 | Image-build pipelines | **Done** — `.gitlab/ci/jobs/images.yml` builds + pushes. |
| G9 | Backups | **Open** — no `backups:` section yet. |
| G10 | Webhooks | Open — handler exists; not exercised end-to-end. |
| G11 | Buildkit references | **Done** — chart references resolve. |
| G12 | Repo URL flip | **Done** — `infra/charts/cto/values.yaml` uses GitLab URL. |
| G13 | Image parity (controller, agents-adapter, sidebar) | **Done** for the components covered by `.gitlab/ci/jobs/images.yml`. |
| G14 | Runner verification | **Done** — 5 runners on `gitlab-runners` ns picked up pipelines #1+ on `k8s-runner` tag, end-to-end through registry push. |
| G15 | Runbook | **Open** — partial in `infra/docs/gitlab-mirror.md`; full runbook deferred. |

## Mirror (G6) — final design

**Replaced** the GitHub Actions workflow with an in-cluster Kubernetes CronJob. Cloudflare's 100 MB request limit made HTTPS push of a full clone unworkable; SSH directly to the in-cluster `gitlab-shell` service bypasses the tunnel.

- Helm chart: `infra/charts/gitlab-mirror/`
- ArgoCD Application: `infra/gitops/applications/platform/gitlab-mirror.yaml`
- Schedule: every 5 minutes, `concurrencyPolicy: Forbid`, 15-min deadline.
- Pushes `main` + all tags via SSH (`gitlab-gitlab-shell.gitlab.svc:22`) using deploy key with `can_push: true` registered on `5dlabs/cto`.
- Secret `gl-mirror-key` (in `gitlab` ns) is **not** managed by Helm. Backup of private key is in 1Password Operations vault, item `ee6p5fqlidbn4lhld3eiz57234`. Recreate procedure in `infra/docs/gitlab-mirror.md`.
- GitLab project access token (PAT used for setup): 1Password Operations vault, item `nrlr33fxeppprxkxz3tqir6rqi`.
- Old `.github/workflows/mirror-to-gitlab.yml` is **deleted**.

---

## 1. Why now (urgency)

- **GHCR / GitHub Actions quota is exhausted** for the `5dlabs` org. Any new workflow run that does `docker push ghcr.io/5dlabs/...` (or even consumes minutes) will fail. We need an alternative CI + registry **immediately**, and self-hosted GitLab is the path we already started.
- All net-new image work is currently **build-locally only** (see `.codex/agents/coder-expert.md` "Image Builds — GHCR Quota Exhausted"). That is a stop-gap; the durable fix is GitLab CI + `registry.5dlabs.ai`.
- GitLab is **already largely deployed in-cluster** (webservice, registry, gitaly, postgres, redis, sidekiq, 5 runners up 13–25d; `gitlab.5dlabs.ai` returns 302). This is a *finish-the-cutover* effort, not greenfield.

## 2. Goals

1. **Production-ready self-hosted GitLab** at `gitlab.5dlabs.ai` with healthy registry at `registry.5dlabs.ai`, backups, secrets, and runner capacity.
2. **CTO app parity on GitLab** — every CI pipeline / image build / release flow we run on GitHub today has a working equivalent on GitLab, so we can ship without depending on the GitHub Actions / GHCR quota.
3. **Maintain public presence on GitHub** — GitHub remains the public face; **GitHub → GitLab one-way mirror stays on**. No bidirectional sync, no GitLab-as-public-source.
4. **Zero-downtime cutover** — at no point should ArgoCD apps go un-syncable or controller image pulls fail.

## 3. Non-goals

- Migrating issue tracker / PR review off GitHub. Keep on GitHub for now.
- Bidirectional sync. One-way (GitHub → GitLab) only.
- Public-facing GitLab. `gitlab.5dlabs.ai` is internal/team-only.
- Migrating GitHub-only secrets to GitLab unless required by a ported pipeline.

## 4. Current State

- DNS responds (302 on `gitlab.5dlabs.ai`); core GitLab pods Running 13–25d.
- 5 runners online — token freshness unverified.
- ArgoCD app `gitlab` shows **Unknown / Missing** despite running pods (drift — see G1).
- ArgoCD app `gitlab-config` sources from `gitlab.5dlabs.ai/5dlabs/cto.git` — chicken-and-egg until mirror is live (G2).
- `mirror-to-gitlab.yml` workflow is **gated off** behind repo Variable `MIRROR_TO_GITLAB=true` (commit `3a3c76143`). Re-enable once `GITLAB_PUSH_TOKEN` is set.
- `infra/charts/cto/values.yaml:809-810, 947`, `infra/charts/buildkit/Chart.yaml:21,25` already pre-stage GitLab URLs — ready to be activated.
- `.gitlab/`, `.gitlab-ci-configs/` exist with per-project CI templates; **no** repo-root `.gitlab-ci.yml` yet for `cto`.
- GitHub Repo Variable / Secrets needed: `MIRROR_TO_GITLAB`, `GITLAB_PUSH_TOKEN`.

## 5. Backlog (G1 – G15)

### G1 — ArgoCD `gitlab` app drift
Shows `Unknown / Missing` despite running pods. Investigate sync state vs. chart-source revision; resync; verify generated manifests match what's actually in cluster. **Foundational — do first.**

### G2 — ArgoCD `gitlab-config` self-reference (chicken-and-egg)
Sources from `gitlab.5dlabs.ai/5dlabs/cto.git` (the repo we're not mirroring yet). Options:
- (a) Switch source back to GitHub temporarily until first successful mirror push, **then** flip to GitLab.
- (b) Manually seed the GitLab repo once and let the app catch up.

### G3 — Cloudflare tunnel routes
`infra/gitops/manifests/gitlab/tunnel-binding.yaml` exists. Verify routes for `gitlab.5dlabs.ai` and `registry.5dlabs.ai` are bound and healthy on the tunnel.

### G4 — External secrets fully populated
Verify all 1Password/Vault keys exist and are synced:
- `gitlab-initial-root-password`
- `gitlab-registry-storage` (S3 / minio creds)
- SMTP creds (password reset emails)
- Runner registration tokens (5 runners are running — confirm tokens aren't expired)

Check `infra/gitops/manifests/gitlab/external-secrets.yaml`.

### G5 — Container registry (`registry.5dlabs.ai`)
Verify DNS → tunnel → registry pod. Test push:
```
docker login registry.5dlabs.ai
docker push registry.5dlabs.ai/5dlabs/test:smoke
```

### G6 — `GITLAB_PUSH_TOKEN` for mirror
Create a GitLab project access token with `write_repository` scope. Store as GitHub repo secret `GITLAB_PUSH_TOKEN`. Then set repo Variable `MIRROR_TO_GITLAB=true` to enable `.github/workflows/mirror-to-gitlab.yml`. Watch first mirror succeed.

### G7 — Repo-root `.gitlab-ci.yml` for `cto`
`.gitlab-ci-configs/` has per-project templates but **no** `.gitlab-ci.yml` at repo root. Author one for `cto` itself, mirroring at minimum:
- `cargo build --workspace`
- `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
- `cargo fmt --all -- --check`
- `bun install && bun run build` for sidebar / TS apps
- `helm lint infra/charts/*`

### G8 — CI parity inventory & port
For each GitHub Actions workflow under `.github/workflows/`, decide: **port to GitLab CI**, **drop**, or **accept GitHub-only**. Known categories:
- `ci.yml` (rust) → port to GitLab
- `cto-sidebar-release.yaml` → GitLab Releases analog
- `parity-check.yaml` → port
- `mirror-to-gitlab.yml` → GitHub-only (one-way mirror)
- Image build / publish workflows for controller, agents-adapter, sidebar VSIX, etc. → port; have them push to `registry.5dlabs.ai`

### G9 — Backups
No `backups:` section in `infra/charts/gitlab/values.yaml`. Configure S3/minio object-storage backups + retention before declaring production.

### G10 — Webhooks
`infra/charts/cto/values.yaml:947` already provisions HTTPRoute / Ingress / TunnelBinding for GitLab webhooks → PM handler at `/webhooks/gitlab/events`. Verify the handler exists in the PM crate and exercise end-to-end (push event → PM receives → expected side-effect).

### G11 — Buildkit references
`infra/charts/buildkit/Chart.yaml:21,25` references `gitlab.5dlabs.ai/5dlabs/cto`. Confirm pull works post-cutover (depends on G5 + G6).

### G12 — Repository URL flip
`infra/charts/cto/values.yaml:809-810` already lists `gitlab.5dlabs.ai/5dlabs/cto`. Confirm pre-staging is intentional; either complete the flip after mirror is healthy, or revert to GitHub URL temporarily.

### G13 — CTO application parity on GitLab (image builds + releases)
**This is the durable replacement for the current GHCR-exhausted state.** Move every image we currently push to GHCR over to `registry.5dlabs.ai` via GitLab CI:

- `controller` (`controller-build.Dockerfile`)
- `agents-adapter` (`agents-adapter-patch.Dockerfile`)
- `cto-sidebar` VSIX (release artifact + image if applicable)
- Any other component listed under `infra/charts/*/values.yaml` `image:` keys that today resolves to `ghcr.io/5dlabs/...`

For each:
1. Add a GitLab CI job that builds + tags + pushes to `registry.5dlabs.ai/5dlabs/<component>:<sha>` and `:latest`.
2. Update the corresponding Helm values default to point at the GitLab registry.
3. Update CRD/controller image fields (`crates/controller/src/tasks/code/resources.rs`, `infra/charts/cto/crds/coderun-crd.yaml`, `infra/charts/cto-lite/crds/coderun-crd.yaml`) per the AGENTS.md co-change table.
4. Verify ArgoCD picks up the new image refs and the cluster pulls successfully (no `ImagePullBackOff`).

**Automation goal**: a single merge to `main` (mirrored to GitLab) triggers GitLab CI → registry push → ArgoCD auto-sync → cluster rolls. If we can fully automate this, we should; otherwise document the manual steps.

### G14 — Runner capacity & node selectors
5 runners are deployed but unverified. Confirm:
- Tokens current
- Tags map to the workloads we're scheduling (e.g. `docker`, `kubernetes`, `gpu`)
- Resource requests / node selectors appropriate (avoid GPU node contention with MuseTalk + similar workloads)
- Concurrency tuned for our typical CI fan-out

### G15 — Documentation & ops handoff
- `docs/self-hosted-gitlab-runbook.md` — day-2 ops (restart, restore from backup, rotate root password, add a runner, mint a project token).
- Update `AGENTS.md` co-change table to include `infra/charts/gitlab/` and the GitLab CI files alongside their GitHub equivalents.
- Update `.codex/agents/coder-expert.md` once registry push works to remove the "build locally only" stop-gap.

## 6. Suggested Cutover Order

1. **G1** — fix ArgoCD `gitlab` app drift (so we trust what's deployed)
2. **G4** — verify all secrets present
3. **G3** — verify tunnel routes (`gitlab.5dlabs.ai`, `registry.5dlabs.ai`)
4. **G5** — test registry push
5. **G6** — mint `GITLAB_PUSH_TOKEN`, set `MIRROR_TO_GITLAB=true`, watch first mirror succeed
6. **G2** — re-point `gitlab-config` once mirror is healthy
7. **G14** — verify runner fleet
8. **G7 + G8** — port CI (rust + sidebar + helm lint first; image-publish jobs next)
9. **G13** — CTO app image parity (controller, agents-adapter, sidebar, etc.)
10. **G9** — backups before declaring production
11. **G10 + G11** — webhook + buildkit smoke tests
12. **G12** — final URL flip / cleanup
13. **G15** — runbook + co-change docs

## 7. Acceptance Criteria

- `gitlab.5dlabs.ai` and `registry.5dlabs.ai` both reachable, healthy, backed up.
- ArgoCD apps `gitlab` and `gitlab-config` both `Synced / Healthy`.
- GitHub → GitLab mirror runs green on every push to `main` (and feature branches per workflow config).
- Every image currently pushed to `ghcr.io/5dlabs/*` is also being built + pushed to `registry.5dlabs.ai/5dlabs/*` by GitLab CI on the same trigger, with no manual steps.
- A clean cluster bring-up using only GitLab as the source/registry succeeds end-to-end.
- GitHub remains the public face — `mirror-to-gitlab.yml` stays one-way; no GitLab → GitHub flow.
- `docs/self-hosted-gitlab-runbook.md` exists and covers day-2 ops.

## 8. Risks / Caveats

- **`gitlab-config` chicken-and-egg (G2)** — must be resolved before G2 step or ArgoCD app stays broken. Plan resolves it after first successful mirror.
- **Mirror push token scope** — `write_repository` is enough for branches; if we mirror tags/releases, may need broader. Confirm during G6.
- **Backups before traffic** — do not consider "production" without G9. Losing GitLab right now means losing CI history and registry contents.
- **Runner contention** — GPU node pressure (MuseTalk) and runner workloads may collide. G14 should set node selectors / taints accordingly.
- **Mirror deletes** — verify mirror behavior on force-pushes / branch deletes; we don't want a bad GitHub state to nuke GitLab history.
- **Public presence** — keep all public docs / READMEs / images pointing at GitHub. Internal infra refs may flip to GitLab.

## 9. Open Questions

1. **Mirror trigger surface** — mirror only `main` + tags, or all branches? Current workflow mirrors per its own config; revisit after G6.
2. **Releases on GitLab** — do we need GitLab Releases for the sidebar VSIX, or is GitHub Releases (current) sufficient since GitHub stays public? Probably keep GitHub Releases for VSIX since sidecar pulls from `github.com/5dlabs/cto/releases/...`.
3. **Backup destination** — which S3 / minio bucket? Reuse an existing one or provision new?
4. **Should cluster image pulls in production reference `registry.5dlabs.ai` exclusively, or keep GHCR as a fallback** until proven? Suggest: switch defaults to GitLab once G13 is green for that component, leave GHCR refs only as commented fallback.
5. **GitLab CI runner image-build strategy** — use docker-in-docker, kaniko, or buildah? Coder/Kaniko shim work in the controller is in flight; pick one strategy and document.

## 10. Handoff Notes

- Mirror is **live** as an in-cluster CronJob (`gitlab-mirror` in `gitlab` ns). See `infra/docs/gitlab-mirror.md` for ops.
- All commits should include the Copilot co-author trailer (per `AGENTS.md`):
  `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>`
- Co-change rule still applies: any CRD / Helm change must update both paths per the AGENTS.md table.
- Coder agent is currently instructed to build images locally only. Once G13 ships for a given image, update `.codex/agents/coder-expert.md` to allow GitLab CI + registry push for that image.
