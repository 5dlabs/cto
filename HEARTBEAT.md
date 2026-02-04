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

### 2026-02-04: GitHub App Key File Deletion (✅ RESOLVED)

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
