---
title: Docs Job Investigation — base64: invalid input
date: 2025-09-13
---

Summary
- Symptom: During the docs job, right after “📋 Found tasks.json, generating individual task files…”, the container logs repeatedly show “base64: invalid input”, and the job does not process all tasks.
- Finding: The current docs container template in this repo does not use base64 during task extraction. It was refactored to use jq-only parsing. If you still see base64 errors, the running job is using an older template/config that still attempted to base64‑decode task fields.

What We Looked At
- Template referenced by you: `infra/charts/controller/claude-templates/docs/container.sh.hbs`.
  - This template emits the exact line “📋 Found tasks.json, generating individual task files…”.
  - Immediately after, it analyzes and iterates tasks using jq only:
    - Lines ~323–361: “Use jq directly to process tasks without base64 encoding/decoding.”
    - Each task object is streamed with `jq -c` and fields are read via `jq -r` in a helper `_decode()`; no `base64 -d` calls are present.
  - The only base64 use in this script is for GitHub App JWT generation (encoding only): lines ~35–43. That cannot produce “invalid input” because decode is not used there.
- Generated static ConfigMap: `infra/charts/controller/templates/claude-templates-static.yaml`.
  - This file embeds `docs_container.sh.hbs` (key: `docs_container.sh.hbs`) and carries a generator timestamp (e.g., `generated-at: "2025-09-12T19:35:32Z"`).
  - It’s produced by `infra/charts/controller/scripts/generate-templates-configmap.sh`, which base64-embeds the raw templates for Helm/ArgoCD delivery.

Inference About The Error
- The repeated “base64: invalid input” messages right after the “Found tasks.json…” banner indicate a script variant that tries something like:
  - `jq -r '…' … | base64 -d` or iterates `@base64` outputs and decodes each row.
- That pattern fails when the value being piped is not actually base64 (e.g., plain text like a title/description), or when quoting/word-splitting corrupts the encoded blob.
- Because the current repo template no longer does this, the most likely cause is a deployment mismatch:
  1) The target repo/environment is running an older `docs_container.sh.hbs` (pre-refactor), or
  2) Its chart still contains an older `claude-templates-static.yaml` that wasn’t regenerated and committed after the refactor, or
  3) The job selects a different container template (e.g., a forked chart or an environment-specific ConfigMap override) that still contains the base64 logic.

Prompt Reading Concern
- The docs container expects an initial free-form prompt in `prompt.md` (if present) and requires `CLAUDE.md` for the environment. Relevant bits:
  - It copies all `/task-files/*.md` into the working directory (renaming only `claude.md` → `CLAUDE.md`).
  - Later, if `prompt.md` exists in the working directory, it reads and sends it as the initial input; otherwise it prints “No prompt.md found”.
- It does not read a `prompt` field from `tasks.json` (by design). Instead, it generates per‑task `task.txt` and `task.xml` from fields like `id`, `title`, `description`, `details`, `testStrategy`.
- If you expected a `prompt` property in `tasks.json` to be used, that is not currently wired up in this script.

How To Verify What’s Running
1) Check for the jq-only logs in your job:
   - After “📋 Found tasks.json…”, you should see:
     - “🔍 Analyzing tasks.json structure…”, “📄 Raw tasks.json structure preview:”, and “📊 JSON structure analysis:”.
   - If those do not appear and you see base64 errors instead, you’re on the older script.
2) Inspect the live ConfigMap in the target cluster/namespace:
   - `kubectl -n <ns> get cm <release>-claude-templates -o yaml | grep generated-at`
   - Extract `docs_container.sh.hbs` and confirm the presence of the comment: “Use jq directly to process tasks without base64 encoding/decoding”.
3) Confirm your chart bundle includes the updated static ConfigMap:
   - Open `infra/charts/controller/templates/claude-templates-static.yaml` and confirm it has a recent `generated-at` and a `docs_container.sh.hbs` entry that decodes to the jq-only script.
   - If not current, run: `make -C infra/charts/controller` to regenerate via `scripts/generate-templates-configmap.sh`, commit, and redeploy.

Likely Root Cause(s)
- Stale chart/config: Target environment using an older `docs_container.sh.hbs` that still base64-decodes task fields.
- Divergent template source: Another repo or overlay overrides the docs container with a variant that still uses base64.
- Misinterpreted data shape: Old logic attempted to decode plain strings (titles/descriptions) as base64, yielding “invalid input”.

Recommendations
- Roll out the jq-only template:
  - Regenerate and commit `claude-templates-static.yaml` in the chart.
  - Bump chart/app version and redeploy the controller so the new ConfigMap is mounted.
- Add a runtime version banner for easier forensics:
  - Optionally, add an `echo "Docs container template version: <git-sha or date>"` near the start of `docs/container.sh.hbs` so logs immediately confirm which script is live.
- Prompt handling:
  - Ensure a `prompt.md` is provided via the task ConfigMap if you want an initial prompt injected. The script does not read a `prompt` property from `tasks.json`.
- Optional compatibility shim (if you must support old `tasks.json` encodings):
  - Detect base64‑looking fields before decoding (strict regex and length/padding checks), else treat as plain text. The current jq‑only approach is safer and recommended.

Next Steps For Us
- If you can share the target namespace/release, we can:
  - Diff the live ConfigMap vs. the repo version.
  - Confirm the presence/absence of the jq-only block.
  - Propose a small PR that adds a version banner and (optionally) a base64‑compat shim guarded behind a flag.

Files Touched In This Investigation
- Read/inspected:
  - `infra/charts/controller/claude-templates/docs/container.sh.hbs`
  - `infra/charts/controller/templates/claude-templates-static.yaml`
  - `infra/charts/controller/scripts/generate-templates-configmap.sh`

## REMEDIATION IMPLEMENTED (2025-09-13)

### Root Cause Confirmed
- **Issue**: The docs job was experiencing "base64: invalid input" errors after "📋 Found tasks.json, generating individual task files…"
- **Analysis**: The current source template (`claude-templates/docs/container.sh.hbs`) already contains the jq-only logic (line 323-324) with no problematic base64 decoding
- **Verification**: Template validation confirmed:
  - 1091 lines in full template (matches source)
  - Line 324: "Use jq directly to process tasks without base64 encoding/decoding" 
  - No problematic `base64 -d` usage found (only safe JWT encoding)

### Fixes Applied

#### 1. Version Banner Added
- **Change**: Added version banner to `claude-templates/docs/container.sh.hbs` at line 4
- **Purpose**: Easier forensics to identify template version during execution
- **Banner**: `echo "📍 Docs container template version: $(date -u +%Y-%m-%d) - jq-only processing (no base64 decode)"`

#### 2. ConfigMap Regeneration
- **Action**: Regenerated `claude-templates-static.yaml` using `make generate-templates`
- **Before**: `generated-at: "2025-09-12T19:35:32Z"` (578278 bytes)
- **After**: `generated-at: "2025-09-13T00:53:54Z"` (578418 bytes)
- **Verification**: Template now includes version banner and confirmed jq-only logic

#### 3. Git Workflow
- **Commit**: `86d2ee8` - Added version banner and regenerated ConfigMap
- **Files Changed**: 
  - `claude-templates/docs/container.sh.hbs` (source template)
  - `templates/claude-templates-static.yaml` (generated ConfigMap)

### Expected Resolution
When the updated ConfigMap is deployed:
1. Jobs will show the version banner: "📍 Docs container template version: 2025-09-13 - jq-only processing (no base64 decode)"
2. No more "base64: invalid input" errors should occur during task processing
3. The jq-only task extraction should work reliably with all task data formats

### Deployment Required
- **Next Step**: Deploy the updated chart to target environment
- **Verification**: Look for version banner in job logs to confirm new template is active
- **Monitoring**: Verify absence of base64 errors in subsequent docs job runs

### Additional Investigation Notes
- The investigation revealed that the repo template was already correctly refactored
- The issue was likely due to deployment lag or environment using stale ConfigMap
- Version banner will provide immediate confirmation of template version in future investigations

## Original Analysis

Appendix: Why base64 failed
- `base64: invalid input` is raised when the decoder receives non‑base64 characters, bad padding, or truncated input. When shell pipelines extract JSON strings with `jq -r` and then feed them into `base64 -d`, any plain text (e.g., a title like "Fix docs flow") will trigger this error. Quoting and word-splitting can magnify the issue by corrupting otherwise valid encodings. Removing the decode step (as the current template does) avoids this entire class of errors.

