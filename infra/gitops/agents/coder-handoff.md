# Session Handoff — 2026-04-15

## Summary

This session covered: Qdrant client version fix, Morgan git auth awareness, `cto-play` launcher utility, release-please pipeline fix, CTO config schema alignment, and binaries-release pipeline fixes.

---

## Completed Work

### 1. Qdrant Client Version Fix
- **Problem:** Morgan's mem0 plugin bundled `@qdrant/js-client-rest@1.13.0` but the Qdrant server is `v1.17.1`. Client enforces major match + minor diff ≤ 1.
- **Fix:** Added npm `overrides` in package.json to force `@qdrant/js-client-rest@1.17.0`. Added version-check logic in init scripts to detect stale client and reinstall.
- **Files:** `infra/charts/openclaw-agent/templates/deployment.yaml`, openclaw-platform `statefulset.yaml` (both mem0 sections), `deployment.yaml`
- **Commits:** `fb2dc1057` (cto), `cfa3b72` (openclaw-platform)
- **Status:** ✅ Verified working — qdrant client 1.17.0, no errors

### 2. Morgan Git Auth Awareness
- **Problem:** Morgan wasted 10+ turns trying to authenticate git when it was already pre-configured by the init container.
- **Fix:** Added "GIT and REPO ACCESS" section to Morgan's `customInstructions` in both `morgan-values.yaml` and the ArgoCD inline values (patched directly).
- **Important:** ArgoCD uses **inline values** in `.spec.source.helm.values`, NOT the file from the CTO repo. Changes to `infra/gitops/agents/morgan-values.yaml` don't auto-propagate.
- **Status:** ✅ Verified instructions in `/workspace/.openclaw/openclaw.json`

### 3. `cto-play` Launcher Utility
- **Crate:** `crates/play-launcher/` — standalone Rust binary
- **What it does:** Reads per-repo `.tasks/play-config.yaml` + CTO `cto-config.json`, merges with CLI overrides, invokes `lobster run` with `--args-json`
- **Config merge priority:** CLI flags > play-config.yaml > cto-config.json > hardcoded fallbacks
- **$ref resolution:** Resolves `{"$ref": "model-providers.json#/agentHarness"}` etc. at load time
- **Fields passed through:**
  - `repo_url`, `namespace`, `base_branch`, `cli`, `provider`, `model`
  - `harness_agent`, `github_app_prefix`, `working_directory`, `auto_merge`
  - `enable_docker`, `discord_enabled`, `discord_bridge_url`
  - `linear_session_id`, `linear_team_id`, `docs_repository_url`
  - `agent_harness_json` — full per-agent CLI+model map from model-providers.json
  - `openclaw_json` — OpenClaw provider config
  - `acp_json` — ACP entries
- **Skill:** `skills/play-launcher/SKILL.md` — Morgan skill with triggers
- **Docker:** Added `play-builder` stage to `infra/images/agents/Dockerfile`
- **Status:** ✅ Built locally, dry-run verified against both test repos

### 4. Test Repo Play Configs
- **test-sandbox:** `.tasks/play-config.yaml` — Todo API project (local clone at `/tmp/test-sandbox/`)
- **test-sandbox-2:** `.tasks/play-config.yaml` — Bookmarks API project (local clone at `/tmp/test-sandbox-2/`)
- Both pushed to main
- **Status:** ✅ Committed and pushed

### 5. Dockerfile `BASE_IMAGE` Fix
- **Problem:** Adding `play-builder` as first `FROM` pushed `ARG BASE_IMAGE` into stage scope instead of global scope, breaking the main stage's `FROM ${BASE_IMAGE}`.
- **Fix:** Moved `ARG BASE_IMAGE=...` before all `FROM` statements.
- **Commit:** `97f0ffba9`
- **Status:** ✅ Agent image CI green (run #24463334069)

### 6. Release-Please Pipeline Fix
- **Problem:** Tag `v0.2.53` already existed (from prior binaries-release), causing `already_exists` error when release-please tried to create the GitHub Release.
- **Fix:** Deleted stale `v0.2.53` release and tag. Release-please moved forward and created PR #4633 for v0.2.54, which was merged creating `v0.2.54`.
- **Status:** ✅ v0.2.54 release exists, PR #4634 ("chore: release 0.2.55") is now the open release PR

### 7. Binaries Release Pipeline Fixes
- **Added `cto-play`** to `binaries-release.yaml` build matrix (all platforms except Windows)
- **Fixed job skip cascade:** Docker build jobs now use `always()` so they don't skip when unrelated matrix entries (healer) fail
- **Updated release job condition** to tolerate docker/healer failures
- **Commit:** `a06225665`
- **Status:** ⚠️ See outstanding items below

---

## Outstanding / Needs Attention

### 🔴 Binaries Release Not Publishing Assets
- **v0.2.54 release exists but has 0 assets** — no binaries attached
- Run `24464831891` (first attempt): healer builds failed → docker jobs skipped → release job skipped (this is what the `always()` fix addresses)
- Run `24465610586` (second attempt with fix): `startup_failure` — likely a k8s-runner issue, not code
- **Action needed:** Re-dispatch: `gh workflow run binaries-release.yaml --repo 5dlabs/cto --field version=0.2.54`
- If it fails again with `startup_failure`, the k8s runner pool may need attention

### 🟡 Healer Build Failures
- Healer crate is failing to build on all 4 platforms in the binaries-release matrix
- This is a pre-existing issue but it blocks docker image builds too
- **Action needed:** Check `crates/healer/` for build errors, fix or exclude from release

### 🟡 Release PR #4634 (v0.2.55)
- Open and ready to merge: https://github.com/5dlabs/cto/pull/4634
- Contains all the changes from this session
- Merging will create v0.2.55 release and auto-dispatch binaries-release + agents-build

### 🟡 play.lobster.yaml Template Updates
- The `play.lobster.yaml` in both test repos still has **hardcoded** OpenClaw provider blocks (just Fireworks)
- Now that `openclaw_json` and `agent_harness_json` flow through as lobster inputs, the play.lobster.yaml CRD templates should be updated to use these instead of hardcoded values
- The per-agent CLI/model from `agent_harness_json` isn't yet being used in the CRD `cliConfig` blocks

### 🟡 Morgan Invocation Mechanism
- Deferred: how does Morgan get invoked to start a play?
- The `cto-play` binary exists and Morgan has the skill, but the trigger path hasn't been implemented

### 🟡 Full E2E Test
- Both test repos have play-config.yaml and play.lobster.yaml ready
- Need to actually run both plays concurrently to validate the full pipeline

### 🟡 ArgoCD Values Sync
- `infra/gitops/agents/morgan-values.yaml` does NOT auto-sync to ArgoCD (inline values used)
- Must patch directly: `kubectl -n argocd patch app openclaw-morgan --type json --patch-file <patch>`

---

## Key Architecture Notes

### ArgoCD Values (Critical)
Morgan's ArgoCD app uses **inline values** in `.spec.source.helm.values`. The file `infra/gitops/agents/morgan-values.yaml` is for reference only.

### cto-play Config Discovery
```
--cto-config flag > CTO_CONFIG env > /etc/cto/config.json > ./cto-config.json
```
Play config: `.tasks/play-config.yaml` > `.tasks/docs/play-config.yaml`
Play YAML: `.tasks/docs/play.lobster.yaml` > `.tasks/play.lobster.yaml` > `play.lobster.yaml`

### $ref Resolution
`cto-config.json` uses `$ref` pointers like `{"$ref": "model-providers.json#/agentHarness"}`. The play-launcher resolves these at load time relative to the config file location.

---

## Commits This Session

### CTO repo (5dlabs/cto) — pushed to main
| SHA | Message |
|-----|---------|
| `fb2dc1057` | fix: qdrant client override + morgan git awareness |
| `7ac87d690` | feat: cto-play launcher + skill + Dockerfile |
| `4537b8afc` | fix: simplified Dockerfile build stage |
| `97f0ffba9` | fix: move BASE_IMAGE ARG before first FROM |
| `551cc5de9` | feat(play-launcher): pass agentHarness, openclaw, acp from CTO config |
| `a06225665` | fix(ci): add cto-play to binaries release, fix job skip cascade |

### openclaw-platform — pushed to main
| SHA | Message |
|-----|---------|
| `cfa3b72` | fix: qdrant client override in statefulset + deployment |

### test-sandbox — pushed to main
| SHA | Message |
|-----|---------|
| `4cc5e65` | feat: add play-config.yaml |
| `8035988` | feat: add workingDirectory and autoMerge |

### test-sandbox-2 — pushed to main
| SHA | Message |
|-----|---------|
| `bd73a7d` | feat: add play-config.yaml |
| `a953e3d` | feat: add workingDirectory and autoMerge |
