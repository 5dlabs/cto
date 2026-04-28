# Intake Remote-Repo Handling — Audit

**Scope:** Read-only audit of how the intake pipeline (`intake/workflows/pipeline.lobster.yaml`) handles a remote/target GitHub repository. Goal is to confirm the documented invocation, verify clone/path logic is intact, and flag gaps.

**Branch audited:** `intake-video` (worktree `cto-intake-video`).
**Status:** No regressions found. Two minor portability concerns documented in §4.

---

## 1. Remote-repo invocation

The pipeline accepts **three** repo-related arguments (top of `intake/workflows/pipeline.lobster.yaml`):

| Arg | Default | Purpose |
|---|---|---|
| `repository_url` | `""` | Remote URL of the **target** customer repo. If empty, `setup-repo` (lines 762–792) creates a fresh repo via `scm_create_repo` under `${github_org}/${project_name}`. If set, it is reused. |
| `target_repo_local_path` | `""` | Path to a **pre-existing local clone** of the target repo. The pipeline does **not** clone for you. Consumed by `sync-to-target-repo` (lines 1757–1885), which rsyncs `.tasks/` artifacts into the clone, commits on a fresh branch `intake/${project}-<timestamp>`, pushes, and opens a PR via `gh pr create`. |
| `codebase_repository_url` | `""` | Optional override for the codebase-analysis sub-workflow (lines ~1000–1014). Lets you analyze a different "platform" repo than the target. Falls back to `repository_url` when empty. |

### Standard invocation

```bash
lobster run intake/workflows/pipeline.lobster.yaml --args-json "$ARGS_JSON"
```

`ARGS_JSON` always carries `repository_url` and `target_repo_local_path` — see `intake/scripts/run-quick-intake.sh:32-39` for the canonical builder.

### Wrapper scripts

- **`intake/scripts/run-quick-intake.sh`** — quick smoke run. Positional args: `PROJECT_NAME REPO_URL TARGET_REPO_PATH`. Defaults: `sigma-1 https://github.com/5dlabs/sigma-1 /Users/jonathon/5dlabs/sigma-1`.
- **`intake/scripts/run-full-e2e.sh`** — full deliberate flow. Same positional args + flags `--fresh`, `--skip-deliberation`, `--from <stage>`.
- **`intake/scripts/go-green.sh`** — preflight + checkpoint loop only; **does not invoke a pipeline** and takes no repo args.

### Where remote pulls actually happen

- **`codebase-analysis.lobster.yaml`** (`pack-repo`, lines 23–43) calls `openclaw.invoke --tool repomix --action pack_remote_repository`, which fetches the remote URL **without a local clone** (falls back to OctoCode `githubViewRepoStructure`).
- **`intake.lobster.yaml:2157`** has the only `gh repo clone` in the pipeline. It targets `5dlabs/cto-agents` into a `mktemp -d` workdir for committing the agent package — **unrelated** to the customer/target repo.
- **No step clones the target repo.** The user is expected to pre-clone and pass `target_repo_local_path`.

### Environment / secrets

`intake/local.env.op.defaults` and `discord.env.example` define **no repo-related vars**. All repo selection flows through `--args-json`. Secrets are injected via `op run --env-file=intake/local.env.op` and contain only Linear, Discord, LLM provider, and TTS credentials.

---

## 2. Path resolution

```
 user pre-clones target repo  →  ${target_repo_local_path}
                                          │
 lobster run … --args-json {…}            │
   │                                      │
   ▼                                      ▼
 WORKSPACE=$PWD                    sync-to-target-repo
   .tasks/  (workflow artifacts)      └── rsync .tasks/ into local clone
   .intake/                           └── git checkout -b intake/${project}-<ts>
                                      └── git commit && git push
                                      └── gh pr create
```

- Workflow artifacts always land in `${WORKSPACE}/.tasks/` and `${WORKSPACE}/.intake/`. `WORKSPACE=$PWD` is set by the wrapper scripts before `lobster run`.
- `sync-to-target-repo` (lines 1768–1778) **guards** on `.git` presence. If `target_repo_local_path` is empty or not a git repo, the step short-circuits with `{"synced":false,"reason":"not a git repo"}` instead of erroring — so a missing path is non-fatal.
- `codebase-analysis` consumes the remote over the network; nothing lands on disk outside the workspace.
- `repository_url` empty → `setup-repo` creates a new repo on demand and stores the resulting URL on the artifact for downstream steps.

---

## 3. Regression check

| Check | Result |
|---|---|
| Three repo args still declared in `pipeline.lobster.yaml` | ✅ present, defaults intact |
| `setup-repo` create-if-empty path intact | ✅ lines 762–792 |
| `sync-to-target-repo` guards `.git` presence and skips cleanly | ✅ lines 1774–1778 (commit `35a312b9` "skip workspace PR when target_repo_local_path is set" reinforces this guard) |
| Codebase-analysis still uses remote-only repomix path (no surprise local clone) | ✅ lines 23–43 |
| `gh repo clone` only in agent-package side-channel (`intake.lobster.yaml:2157`) | ✅ unchanged |
| `local.env.op.defaults` / `discord.env.example` introduce no repo vars | ✅ secrets-only |
| Wrapper scripts pass `repository_url` + `target_repo_local_path` consistently | ✅ |

Recent commits touching repo handling (`d8f6056e` add `codebase_repository_url`, `35a312b9` skip workspace PR when `target_repo_local_path` set, `36467a0a` / `9980a8e4` preflight cleanup) all preserve the contract above. **No regression observed.**

### Portability concerns (not regressions)

1. **Hardcoded Jonathon paths.** `run-quick-intake.sh:13` and `run-full-e2e.sh:32` default `TARGET_REPO_PATH=/Users/jonathon/5dlabs/sigma-1`. Other developers must always pass arg #3; the default is otherwise silently broken.
2. **Implicit pre-clone expectation.** The "user must pre-clone the target repo before invoking the pipeline" expectation is not called out clearly in `docs/2026-03/intake-local-prereqs.md` (only `repository_url` is mentioned in the secrets table). A first-time user can read the docs and miss it.

---

## 4. Recommendations

All recommendations are **non-blocking** — the audit found no broken behavior. They reduce friction for new contributors.

1. **Document the pre-clone expectation.** Add a subsection to `docs/2026-03/intake-local-prereqs.md` that lists `target_repo_local_path` alongside `repository_url`, explicitly stating that the pipeline does not clone the target repo and listing the commands a user should run (`gh repo clone <url> <path>`).

2. **Replace hardcoded `/Users/jonathon/...` defaults with env fallbacks.** Both wrapper scripts could honor an `INTAKE_TARGET_REPO_PATH` (or `INTAKE_${PROJECT}_PATH`) env var before falling back to the literal path, e.g.:
   ```bash
   TARGET_REPO_PATH="${3:-${INTAKE_TARGET_REPO_PATH:-$HOME/5dlabs/${PROJECT_NAME}}}"
   ```
   This keeps behavior identical for Jonathon while letting other contributors set a single env var in `~/.zshrc`.

3. **Optional: auto-clone helper.** Consider a helper (e.g. `intake/scripts/ensure-target-repo.sh`) that clones `repository_url` into `${INTAKE_REPOS_HOME:-$HOME/.cto/intake-repos}/${project_name}` when `target_repo_local_path` is unset, and emits the resolved path on stdout. The two wrapper scripts could `eval "$(./intake/scripts/ensure-target-repo.sh "$REPO_URL" "$PROJECT_NAME")"` before invoking lobster. This eliminates the pre-clone footgun without touching the pipeline contract.

4. **Optional: surface the `.git` skip.** When `sync-to-target-repo` short-circuits with `{"synced":false,"reason":"not a git repo"}`, log a clear `WARN` so a misconfigured `target_repo_local_path` doesn't silently drop the PR step.

---

**Reviewer notes:** Pipeline contract is sound. The "pre-clone + pass path" model is intentional (avoids the pipeline managing checkouts the user might already have customized) and aligns with the workspace-relative `.tasks/` artifact strategy. Recommendations 1 and 2 are quick wins; 3 and 4 are larger asks.
