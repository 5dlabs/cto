# Linear Agent Assignment Checklist

Use this checklist before enabling strict assignment mode (`INTAKE_STRICT_ASSIGNMENTS=true`).

## 1) Workspace users exist for each agent

- Confirm Linear users exist (or app users) for expected agent names:
  - `morgan`, `bolt`, `rex`, `grizz`, `nova`, `blaze`, `tap`, `spark`, `cipher`, `tess`, `cleo`, `atlas`, `stitch`, `angie`
- Ensure naming is predictable (display names and usernames should include the canonical agent token).

## 2) OAuth / API token capability

- Verify token can:
  - read users
  - create issues
  - assign issues
  - create/read labels (optional but recommended)
- If assignment fails with `not valid`, verify the token/app is allowed to assign that user.

## 3) Team membership

- Ensure assignable agent users are members of the destination team (`CTOPA` or configured team key).
- Confirm issue assignment policy in Linear allows assignment to those users.

## 4) Dry-run diagnostics

- Run intake in quick mode and inspect `sync-linear-issues-post-push` logs:
  - `unresolvedAgents`
  - `unassignedIssueCount`
- Keep strict mode off until unresolved count is zero.

## 5) Enforce strict mode

- Enable:
  - `INTAKE_STRICT_ASSIGNMENTS=true`
  - `INTAKE_STRICT_CONTENT_GATES=true`
  - `INTAKE_REQUIRE_SUBTASKS=true`
- Re-run quick intake and verify:
  - no unresolved agents
  - parent/task/subtask hierarchy created
  - links resolve to pushed branch content
