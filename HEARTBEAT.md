# HEARTBEAT.md - Healer Monitoring Tasks

## Active Monitoring

### CTO Namespace - Bolt & Atlas Failure Monitoring

**Task:** Monitor CTO namespace for Bolt and Atlas workflow failures initiated by Healer and perform FULL remediation with success verification.

**Check Interval:** Every 2 minutes (background monitor running)

**Remediation Process:**
1. Query workflows in CTO namespace with `initiator=healer` label
2. Filter for Bolt (agent=bolt) and Atlas (agent=atlas) workflows
3. Identify Failed or Error status (exclude already remediated)
4. For each failure:
   - Archive failed workflow details to `/tmp/failed-{workflow}-{timestamp}.yaml`
   - Label failed workflow with `remediated=true` and timestamp
   - Extract workflow spec and create new remediation workflow
   - Monitor new workflow for up to 10 minutes
   - Confirm successful completion before marking remediation complete
5. Report all remediation outcomes (success, failure, or timeout)

**Target Agents:** bolt, atlas
**Namespace:** cto
**Auto-remediate:** Yes (with success verification)
**Background Monitor:** session=marine-shell, pid=4711

---

## Fixed Issues

### 2026-02-04: Git Lock File Conflict Fix (IN PROGRESS)

**Symptom:** Git lock file conflicts when multiple CodeRuns share the same PVC:
```
fatal: Unable to create '/workspace/cto/.git/index.lock': File exists.
```

**Root Cause:** Implementation agents (Atlas, Rex, Blaze, Vex) share the same PVC (`workspace-{service}`). When multiple pods run concurrently, they all mount the same PVC at `/workspace/cto` (the repository checkout), causing git lock conflicts.

**Fix Applied:**
1. **Controller Changes** (`crates/controller/src/tasks/code/resources.rs`):
   - Generate unique workspace subdirectory per CodeRun: `/workspace/runs/{coderun-name}-{uid}/`
   - Add `WORKSPACE_DIR` environment variable for agent isolation
   - Update init container to create the unique subdirectory with proper permissions
   - Set container's working directory to the unique subdirectory

2. **Template Changes** (`templates/_shared/partials/git-setup.sh.hbs`):
   - Use `WORKSPACE_DIR` environment variable when set
   - Clone repository to the unique subdirectory: `$WORKSPACE_DIR/{repo_name}/`
   - Falls back to `/workspace` for backward compatibility

**How it works:**
- Each CodeRun gets its own isolated directory: `/workspace/runs/coderun-abc123/`
- Repository is cloned to: `/workspace/runs/coderun-abc123/cto/`
- Multiple concurrent CodeRuns no longer conflict on git operations
- Storage is still shared (single PVC), but workspaces are isolated

**Files Modified:**
- `crates/controller/src/tasks/code/resources.rs`
- `templates/_shared/partials/git-setup.sh.hbs`

**Status:** Code compiles, tests pending deployment

---

### 2026-02-04: GitHub Issue Acceptance Criteria Fix (COMPLETED)

**Symptom:** When Healer creates GitHub issues for CI failures, the issue body says "no acceptance criteria prompt"

**Root Cause:** The `generate_issue_body` function in `github_actions.rs` didn't include acceptance criteria in the issue body

**Fix Applied:**
1. **GitHub Actions Sensor** (`crates/healer/src/sensors/github_actions.rs`):
   - Added `generate_issue_acceptance_criteria` function
   - Updated `generate_issue_body` to include failure-type-specific acceptance criteria
   - Criteria varies by failure type (build, test, lint, security, deploy, etc.)

2. **Escalation Handler** (`crates/healer/src/ci/escalate.rs`):
   - Added acceptance criteria section to escalation messages
   - Provides checklist for manual resolution

**Acceptance Criteria Sections:**
- **Root Cause**: Identify and document failure root cause
- **Fix Requirements**: Type-specific fixes (build errors, test failures, etc.)
- **Verification**: Code compiles, tests pass, changes pushed
- **Completion**: Workflow passes, CI checks green

**Files Modified:**
- `crates/healer/src/sensors/github_actions.rs`
- `crates/healer/src/ci/escalate.rs`

**Symptom:** 1,315+ coderun jobs failing with "Could not read private key from /tmp/github-app-key-15"

**Root Cause:** The `github-auth.sh.hbs` template created a temp key file for JWT generation, then immediately deleted it. Rex and other agents tried to read this file later and failed.

**Fix Applied:** 
- Modified `/templates/_shared/partials/github-auth.sh.hbs` to preserve the temp key file
- Exported `GITHUB_APP_PRIVATE_KEY_FILE` env var pointing to the temp file location
- File is auto-cleaned when container terminates (ephemeral)

**Resolution Steps Taken:**
1. Deleted 5 stuck healer-ci-rex CodeRuns that were spawning failed jobs
2. Cleaned up 1,315+ failed coderun jobs  
3. Deployed fix via ConfigMap overlay (immediate)
4. Commits: 808bb43f, 7429df26 on agents/healer-fix branch

**Status:** ✅ FULLY RESOLVED - No failures since deployment

**Deployment Method:**
- Created ConfigMap `controller-template-fix-github-auth` with fixed template
- Patched Deployment to mount ConfigMap over buggy template file
- Controller restarted with fix at 2026-02-04 14:11 PST
- Verified: `GITHUB_APP_PRIVATE_KEY_FILE` env var now exported

**Final State (2026-02-04 14:40 PST):**
- 0 CodeRuns (down from 5 stuck)
- 0 CodeRun jobs (down from 1,315+ failures)
- 0 healer-initiated workflows (no remediation loops)
- Controller running with fix mounted and verified

**Next Steps:**
- Merge agents/healer-fix branch to main for permanent fix
- Normal CI/CD will rebuild controller image with fix baked in
- ConfigMap overlay can be removed after new controller deploys
