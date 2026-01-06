## Incident: CodeRun created but no Job (controller fails pre-Job during template render)

### Summary
We observed CodeRuns being created successfully (CR accepted by the API server), but the controller failed *before Job creation*, leaving:

- `.status.jobName` unset (missing)
- `.status.phase` unset or stuck
- no corresponding Kubernetes `Job`/`Pod`
- no Linear sidecar activity updates (because nothing ever ran)

The primary root cause was a **template file layout mismatch inside the controller container**: the running controller binary attempted to load shared Handlebars partials using a *flat* filename layout at `/app/templates/*.hbs`, but the controller image shipped templates in the repository directory structure (`/app/templates/_shared/partials/...`). This caused `CLAUDE.md` rendering to fail, and the controller aborted reconciliation *before* creating the Job.

---

### Primary symptom(s)
- **CodeRun exists** in `cto` namespace (e.g. `intake-prd-...`), and has valid `spec.*` fields.
- **No Job is created** and `.status.jobName` is missing/empty.
- Controller logs show reconciliation for that CodeRun, but it never progresses past template generation.
- Downstream user-facing symptoms:
  - no agent Pod, so no logs
  - no Linear agent dialog activity, because the sidecar never runs

Additionally, in earlier attempts we saw a *related symptom*:

- CodeRuns/Jobs named with `...-unknown-...` segments, and Pods attempting to pull `ghcr.io/5dlabs/agent:latest` (fallback image) and failing with `ImagePullBackOff` / GHCR `403 Forbidden`.

---

### Concrete reproduction instance
The most useful repro CodeRun:

- **CodeRun**: `intake-prd-alerthub-e2e-tes-k4ff9`
- **Namespace**: `cto`
- **runType**: `intake`
- **githubApp**: `5DLabs-Morgan`
- **cli_config**: `None` (expected to be auto-populated)
- **linearIntegration.enabled**: `true`
- **status**: `None` (no `.status.jobName`)

From `kubectl describe coderun` (abridged):

- `Spec.Github App: 5DLabs-Morgan`
- `Spec.Env.INTAKE_CLI: claude`
- `Spec.Env.INTAKE_CONFIGMAP: intake-linear-ctopa-644-...`
- `Events: <none>`

This is a classic “controller failed early” signature: no events, no status updates, no job.

---

### Key evidence (controller logs)
For `intake-prd-alerthub-e2e-tes-k4ff9`, the controller failed during `CLAUDE.md` rendering:

- **Hard error**:
  - `Failed to render CLAUDE.md: Error rendering "claude_memory" ... Partial not found infrastructure-operators`

- **Earlier warning showing why the partial couldn’t be registered**:
  - `Failed to load infrastructure partial infrastructure-operators from ConfigMap (path: infrastructure-operators.md.hbs): ... tried: /app/templates/infrastructure-operators.md.hbs ... No such file or directory`

This happens **before** Job creation, so the CodeRun never receives a `.status.jobName`.

---

### Root cause
#### What the controller expected
The running controller binary attempted to load shared partials using **flat filenames** under `AGENT_TEMPLATES_PATH`:

- `/app/templates/infrastructure-operators.md.hbs`
- `/app/templates/frontend-toolkits.md.hbs`
- `/app/templates/tanstack-stack.md.hbs`
- `/app/templates/shadcn-stack.md.hbs`
- (and more)

This expectation is also visible in the warning (controller tries `direct_path` and then a “configmap key format” fallback, but both paths resolve to `/app/templates/<flat-file>` for this particular case).

#### What the controller image actually contained
The controller image is built from the repository `templates/` directory (see `infra/images/controller/Dockerfile`), and it copies templates *preserving the repo structure*:

- `COPY templates/ /app/templates/`

So the partials are actually located at:

- `/app/templates/_shared/partials/infrastructure-operators.md.hbs`
- `/app/templates/_shared/partials/frontend-toolkits.md.hbs`
- `/app/templates/_shared/partials/tanstack-stack.md.hbs`
- `/app/templates/_shared/partials/shadcn-stack.md.hbs`

#### Why this prevented Job creation
The controller creates Jobs only after it generates the per-job ConfigMap payload containing:

- `container.sh`
- `CLAUDE.md`
- settings/config files (depending on CLI)

`CLAUDE.md` is rendered from the agent system prompt template (for intake, it uses `agents/morgan/intake.md.hbs`) and relies on shared partials like:

- `{{> infrastructure-operators }}`

Because `infrastructure-operators` could not be registered (file not found), `CLAUDE.md` render fails, the reconcile returns a `ConfigError`, and the controller stops before Job creation.

---

### Immediate mitigation (cluster hotfix)
This was a “hotfix to restore service”, not a long-term architectural change.

#### Step 1: ensure controller config has agent mappings (CLI resolution)
We patched `task-controller-config` to include an `agents:` section mapping GitHub App names to CLI + model defaults (example shown in the configmap):

- `5DLabs-Morgan → cli: claude`
- `5DLabs-Rex → cli: claude`
- etc.

This enables the controller’s `populate_cli_config_if_needed()` path to set `spec.cli_config` for CodeRuns that only provide `spec.githubApp`.

#### Step 2: ensure templates exist where the controller expects them
We solved the *template-path mismatch* without waiting for a new controller image by:

1) patching `controller-templates-shared` to contain the needed template contents as flat keys:
   - `infrastructure-operators.md.hbs`
   - `frontend-toolkits.md.hbs`
   - `tanstack-stack.md.hbs`
   - `shadcn-stack.md.hbs`
   - plus added missing runtime templates used by some prompts:
     - `infrastructure-setup.sh.hbs`
     - `infrastructure-verify.sh.hbs`

2) adding backward-compat alias keys for older loaders:
   - `_shared_partials_infrastructure-operators.md.hbs`
   - `_shared_partials_frontend-toolkits.md.hbs`
   - `_shared_partials_tanstack-stack.md.hbs`
   - `_shared_partials_shadcn-stack.md.hbs`
   - `_shared_partials_infrastructure-setup.sh.hbs`
   - `_shared_partials_infrastructure-verify.sh.hbs`

3) patching the **controller Deployment** to mount those ConfigMap keys into the controller container as *files* at:
   - `/app/templates/<flat-filename>`

This resolves the “No such file or directory” issue because the file literally exists at the path the controller tries to open.

#### Deployment patch details (what was applied)
We patched `deployment/cto-controller` to:

- add a volume:
  - `templates-shared` → `configMap: controller-templates-shared`
- add volume mounts (subPath):
  - `/app/templates/infrastructure-operators.md.hbs` ← `infrastructure-operators.md.hbs`
  - `/app/templates/frontend-toolkits.md.hbs` ← `frontend-toolkits.md.hbs`
  - `/app/templates/tanstack-stack.md.hbs` ← `tanstack-stack.md.hbs`
  - `/app/templates/shadcn-stack.md.hbs` ← `shadcn-stack.md.hbs`
  - `/app/templates/infrastructure-setup.sh.hbs` ← `infrastructure-setup.sh.hbs`
  - `/app/templates/infrastructure-verify.sh.hbs` ← `infrastructure-verify.sh.hbs`

Then restarted `deployment/cto-controller` so the new mounts took effect.

---

### Verification (proof it worked)
After applying the hotfix and forcing a reconcile retry (by annotating the CodeRun), we observed:

#### 1) CodeRun status updated and Job created
For `intake-prd-alerthub-e2e-tes-k4ff9`:

- `.status.phase = Running`
- `.status.jobName = play-coderun-t0-morgan-claude-8afa6d24-v1`
- `.status.message = Code implementation started`

#### 2) Pod created with correct CLI runtime image
The Pod for `play-coderun-t0-morgan-claude-8afa6d24-v1` shows these images:

- `busybox:1.36` (init/utility)
- `ghcr.io/5dlabs/claude:latest` (**correct runtime image**)
- `ghcr.io/5dlabs/linear-sidecar:latest` (sidecar)

This confirms:
- template rendering succeeded
- Job creation succeeded
- CLI image selection succeeded

---

### Why we previously saw “unknown” and wrong images
There were two overlapping issues in the overall debugging session:

#### A) “cto-dev” is not a real GitHub App name
Some CodeRuns were created with:

- `spec.githubApp: cto-dev`

The controller’s CLI auto-population uses `spec.githubApp` to look up an agent CLI config. If the GitHub App name is unknown, the controller can’t enrich `cli_config`, which leads to:

- job names containing `...-unknown-...`
- selection of the fallback default image (`agent.image.repository/tag`) instead of the CLI-specific image (`agent.cliImages[cli]`)

#### B) Even when githubApp was correct, the controller failed before Job creation
Even after we got a CodeRun with:

- `spec.githubApp: 5DLabs-Morgan`

the controller still failed to create the Job because template partials were missing at the file paths it tried to read.

The hotfix addressed both:
- controller config now contains agent mappings (`agents:`)
- controller can load the needed partials at `/app/templates/*`

---

### Secondary contributing issue (earlier)
We also fixed a separate but important runtime correctness issue for Play submissions:

- Argo play workflow templates require a `github-app` parameter.
- Rust submission code had stopped passing `github-app`, which would have caused runtime failure:
  - `ERROR: github-app parameter is empty or not set!`

That was fixed by restoring `github-app` parameter passing in the PM workflow submission paths.

Mitigations applied:
- Controller config (`task-controller-config`) was patched to include an `agents:` section mapping GitHub Apps (e.g. `5DLabs-Morgan`) to CLI + model defaults so the controller can populate `spec.cli_config`.
- PM workflow submission code was corrected to consistently pass `github-app` where required by Argo workflow templates.

---

### Reproduction / debugging checklist (if this happens again)
#### A) Confirm the “CodeRun but no Job” signature
1) Identify the CodeRun:
   - `kubectl get coderuns -n cto --sort-by=.metadata.creationTimestamp | tail -20`
2) Check status is missing jobName:
   - `kubectl get coderun <name> -n cto -o jsonpath='{.status.phase} {.status.jobName} {.status.message}'`
   - If it prints blanks, the controller likely failed very early.
3) Check controller logs for render failures:
   - `kubectl logs deployment/cto-controller -n cto --since=15m | grep -E 'ConfigError\\(|Failed to render CLAUDE\\.md|Partial not found'`

#### B) Confirm template-path mismatch quickly
If you see:
- `Failed to load ... tried: /app/templates/<flat-file> ... No such file`

then the controller expects flat filenames but the files are not present at `/app/templates`.

---

### Recommended long-term fix (code + Helm)
The cluster patch is a stopgap. The durable solution should be:

1. **Controller template resolution should support both layouts**
   - In `load_template()`, try repo-structured paths *and* flat paths, e.g.:
     - `_shared/partials/<name>` and `<name>`
   - Or ensure the controller image includes a deterministic flat directory layout at `/app/templates` (symlinks or a build step that copies specific files flat).

2. **Helm: mount templates ConfigMaps**
   - Instead of relying solely on baked-in image templates, mount `controller-templates-*` ConfigMaps into the controller at `AGENT_TEMPLATES_PATH` so:
     - template updates do not require rebuilding the controller
     - prod/staging can patch templates safely
   - Prefer a single `templates` volume with a consistent key naming convention.

3. **Add diagnostics**
   - Emit a K8s Event and/or set `.status.message` when template rendering fails, including:
     - template path attempted
     - missing partial name
     - hint: “templates volume not mounted / wrong layout”
   - Add a debug endpoint or log line at startup printing:
     - `AGENT_TEMPLATES_PATH`
     - count of shared partials successfully registered
     - list of missing shared partials (if any)

4. **Make image selection robust**
   - Ensure “unknown” CLI never silently selects `ghcr.io/5dlabs/agent:latest` for production tasks.
   - If `cli_config` can’t be resolved, fail fast with a high-signal error that includes:
     - CodeRun name
     - githubApp value
     - known agent keys in config

---

### Rollback plan (for the hotfix)
If the controller deployment patch causes issues, rollback options:

1) Rollback controller deployment to previous ReplicaSet:
   - `kubectl rollout undo deployment/cto-controller -n cto`

2) Remove the templates volume mounts:
   - Patch the deployment to remove `templates-shared` volume and mounts (or undo).

3) Restore original `controller-templates-shared` ConfigMap:
   - Re-apply via Helm/ArgoCD sync (preferred) or revert the patched keys.

---

### Notes
This incident’s distinguishing characteristic is: **CodeRun exists, but `.status.jobName` is empty**, and controller logs show template rendering failures. When that happens, focus immediately on **controller template loading and partial registration** rather than Argo workflow templates or Job scheduling.

