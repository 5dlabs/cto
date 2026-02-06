# HEARTBEAT.md - PR Health Monitor

## PR Polling Task (every 5 minutes)

Check open PRs in the CTO repo and ensure they're healthy and progressing toward merge.

### 1. List Open PRs
```bash
cd ~/cto && gh pr list --state open --json number,title,url,headRefName,mergeable,statusCheckRollup --author @me
```

If no PRs assigned to you, also check PRs on `ctoapp/*` branches:
```bash
cd ~/cto && gh pr list --state open --head "ctoapp/" --json number,title,url,headRefName,mergeable,statusCheckRollup
```

### 2. For Each Open PR, Check:

**A. Merge Conflicts**
- If `mergeable` is `CONFLICTING`, rebase onto main:
  ```bash
  git fetch origin main
  git checkout <branch>
  git rebase origin/main
  # Fix conflicts
  git push --force-with-lease
  ```

**B. CI Status**
- If any check has `conclusion: "FAILURE"`:
  1. Read the failure logs: `gh run view <run-id> --log-failed`
  2. Identify the fix needed
  3. Make the fix and push
  4. Wait for CI to re-run

**C. Bot Comments (Bugbot & Stitch) - RECURSIVE LOOP**
- **Acceptance Criteria:** All bot issues resolved (no pending Bugbot/Stitch fixes)
- **Loop until clean:**
  1. Check for bot reviews AND comments:
     ```bash
     # Check reviews (where Bugbot posts)
     gh pr view <number> --json reviews --jq '.reviews[] | select(.author.login == "cursor") | select(.body | contains("potential issues")) | {body, submittedAt}'
     
     # Check comments (where Bugbot Autofix reports)
     gh pr view <number> --json comments --jq '.comments[] | select(.author.login == "cursor") | select(.body | contains("Bugbot Autofix")) | {body, createdAt}'
     ```
  2. If Bugbot found issues but Autofix hasn't committed:
     - Read the fix descriptions from Bugbot Autofix comment
     - Apply the fixes manually
     - Commit and push
     - Wait for CI/Bugbot to re-run
     - **REPEAT from step 1**
  3. Exit loop when: Latest Bugbot Autofix comment shows all fixes ✅ AND committed
  4. Only move to next PR when this PR is bot-clean

**D. Review Status**
- If CI is green and no merge conflicts, check if review is needed
- If approved and green, merge: `gh pr merge <number> --squash`

### 3. Work Sequentially
- Process one PR at a time
- Don't start the next PR until current one is either:
  - Green and merged
  - Green and waiting for review
  - Blocked on something you can't fix (note the blocker)

### 4. Report
If any PR needs attention you can't resolve, alert with details.
If all PRs are healthy (green or waiting for human review), reply HEARTBEAT_OK.

---

## Priority Order
1. PRs with merge conflicts (fix first to avoid drift)
2. PRs with failing CI (get them green)
3. PRs with unresolved Bugbot/Stitch comments (address issues)
4. PRs that are green and approved (merge them)
