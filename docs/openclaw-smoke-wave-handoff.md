# OpenClaw Smoke Wave Handoff

## Context

- Workspace: `/Users/jonathon/.codex/worktrees/openclaw-smoke-wave`
- Branch: `codex/openclaw-smoke-wave`
- Current commit: `a2fdbd075` (`Add OpenClaw smoke wave controller fixes`)
- Cluster: `ovh-cluster`
- Namespace: `cto`
- Morgan is intentionally **out of scope** for this smoke wave because Morgan runs as a statefulset, not a `CodeRun`.
- This wave is only for CRD-backed agents using the OpenClaw path.

## What Was Implemented

- Added a provider-aware OpenClaw CLI adapter and supporting controller plumbing.
- Added minimum Pixel prompt support so Pixel no longer falls back to Rex.
- Fixed multiple controller/template issues uncovered while running real `CodeRun` smoke jobs.
- Ran the smoke wave manually against the OVH cluster from a local controller process pointed at `cto`.
- Stopped the local controller after the sweep to avoid background retry churn.

## Key Code Changes

### OpenClaw support

- `crates/controller/src/cli/adapters/openclaw.rs`
  - New OpenClaw adapter implementation.
- `crates/controller/src/cli/adapters/mod.rs`
- `crates/controller/src/cli/adapter_factory.rs`
- `crates/controller/src/cli/discovery.rs`
- `crates/controller/src/cli/types.rs`
- `templates/clis/openclaw.sh.hbs`
  - New OpenClaw harness template.
  - Supports built-in `openai-codex`, OpenAI API-key, and Anthropic API-key lanes.
  - Includes provider-rejection detection for known quota/credit failure strings, although this still needs re-verification with a funded provider.

### Controller/job fixes

- `crates/controller/src/tasks/code/resources.rs`
  - OpenClaw OAuth env injection for `openclaw-api-keys`.
  - Blaze fix: removed broken `blaze-scripts` ConfigMap mount that referenced a non-existent ConfigMap.
- `templates/_shared/partials/github-auth.sh.hbs`
  - Job-local git config fix.
- `templates/_shared/partials/unity-env.sh.hbs`
  - Guarded `ANDROID_HOME` and `UNITY_LICENSE` for `set -u` compatibility.

### Agent mapping / prompt routing

- `crates/controller/src/tasks/code/templates.rs`
  - Added missing GitHub App to agent mappings for:
    - `5DLabs-Grizz`
    - `5DLabs-Nova`
    - `5DLabs-Tap`
    - `5DLabs-Spark`
    - `5DLabs-Stitch`
  - Added Pixel routing so `5DLabs-Pixel` maps to `pixel`.
- `templates/agents/pixel/coder.md.hbs`
  - Added minimal Pixel coder template for the smoke wave.

### Config / secrets wiring

- `infra/charts/cto/values.yaml`
- `infra/gitops/manifests/external-secrets/cto-secrets.yaml`
- `cto-config.json`
- `crates/controller/src/bin/agent_controller.rs`
- `crates/controller/src/tasks/template_paths.rs`
- `infra/images/agents/entrypoint.sh`
- `Cargo.lock`

## Smoke Wave Results

### Clean successes

These CRD-backed agents completed successfully on OVH `cto` with `ghcr.io/5dlabs/agents:latest`:

- `atlas`
- `stitch`
- `rex`
- `blaze`
- `grizz`
- `tess`
- `cleo`
- `cipher`
- `bolt`
- `angie`
- `nova`
- `spark`
- `tap`

Atlas was additionally verified on-disk earlier in the run by mounting the shared workspace PVC and confirming `/workspace/smoke/atlas.txt` existed with the expected contents.

### Vex status

`vex` is **not honestly validated yet**, even though Kubernetes currently shows `CodeRun/openclaw-smoke-vex-codex-v1` as `Succeeded`.

What was observed:

- The Vex pod stopped crashing immediately after the `ANDROID_HOME` / `UNITY_LICENSE` shell fixes.
- However, the actual OpenClaw model call failed on every provider lane tested:
  - OpenAI OAuth lane: ChatGPT usage-limit message
  - Anthropic API-key lane: low credit balance
  - OpenAI API-key lane: quota exceeded
- Despite that, OpenClaw still returned exit code `0`, which caused false-green controller status.
- I mounted the shared workspace PVC `workspace-test-sandbox` after the latest Vex runs and confirmed `/workspace/smoke/vex.txt` was **not** present.
- The workspace currently only shows `rex.txt` under `/workspace/smoke`.

Conclusion:

- Treat Vex as **blocked by provider capacity/quota**, not as a real pass.

### Pixel status

`pixel` was not run.

Blocker:

- `ExternalSecret/github-app-5dlabs-pixel` exists in `cto`, but it still cannot read `github-app-pixel` from OpenBao.
- `kubectl describe externalsecret github-app-5dlabs-pixel -n cto` shows:
  - `Message: could not get secret data from provider`
  - OpenBao `403 permission denied`
- There is still no live `secret/github-app-5dlabs-pixel` in `cto`.

Conclusion:

- Treat Pixel as **blocked by GitHub App credentials / OpenBao permissions**.

## Live Cluster Notes

- `CodeRun`s currently visible in `cto`:
  - all listed successful agents above
  - `openclaw-smoke-vex-codex-v1` also shows `Succeeded`, but this is misleading for the reasons noted above
- The local isolated controller used for the wave has been stopped.
- There should not currently be an active local controller process reconciling further smoke retries from this worktree.

## What The Next Agent Should Do

1. Do **not** include Morgan in the CRD smoke wave.
2. Treat the wave as:
   - `13` confirmed successful CRD-backed agent smoke runs
   - `vex` blocked by exhausted provider lanes / false-green OpenClaw exit behavior
   - `pixel` blocked by missing readable GitHub App secret
3. For Vex, use a funded provider lane before retrying.
4. After getting a funded provider, rerun Vex and verify the workspace file exists:
   - `/workspace/smoke/vex.txt`
5. For Pixel, fix the OpenBao permission / source secret issue so `secret/github-app-5dlabs-pixel` materializes in `cto`.
6. Once Vex and Pixel are unblocked, rerun only those two rather than repeating the whole wave.

## Useful Commands

### Scoreboard

```bash
kubectl --context ovh-cluster -n cto get coderun
```

### Pixel blocker

```bash
kubectl --context ovh-cluster -n cto describe externalsecret github-app-5dlabs-pixel
```

### Inspect shared workspace PVC

```bash
kubectl --context ovh-cluster -n cto run workspace-debug \
  --image=busybox:1.36 \
  --restart=Never \
  --overrides='{"apiVersion":"v1","spec":{"containers":[{"name":"debug","image":"busybox:1.36","command":["sh","-c","sleep 600"],"volumeMounts":[{"name":"workspace","mountPath":"/workspace"}]}],"volumes":[{"name":"workspace","persistentVolumeClaim":{"claimName":"workspace-test-sandbox"}}]}}'
```

### Clean up the debug pod

```bash
kubectl --context ovh-cluster -n cto delete pod workspace-debug --ignore-not-found
```
