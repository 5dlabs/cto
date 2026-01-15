## [0.2.8] - 2026-01-15

### ✨ Features
- Add `intake update` command to re-parse modified PRD/architecture and generate delta PRs
- Add `intake sync-task` command to sync task files from Linear issue edits
- Add `intake_update` and `intake_sync_task` MCP tools for mid-flight workflow updates
- Add subtask support with execution levels for parallel subagent coordination
- Add `SubtaskSpec` to CodeRun CRD for subtask orchestration
- Add `group_by` Handlebars helper for grouping subtasks by execution level
- Support OpenCode CLI for subagent parallel execution (in addition to Claude Code)

### 🐛 Bug Fixes
- Load `autoAppendDeployTask` from cto-config.json during Intake command
- Fix stale data when syncing tasks from Linear (test_strategy, agent_hint now cleared if missing)
- Remove hardcoded Task 1 = Bolt enforcement; tasks now route by content
- Make `task_id` optional in `intake_sync_task` MCP tool (defaults to Linear issue ID)
- Fix details field not being cleared when Linear acceptance criteria is removed

### ♻️ Refactoring
- Remove `local=true` option from MCP intake tool (intake always runs in-cluster)
- Remove dead code: `workerIsolation` config and related template conditionals
- Remove redundant `roleModels` config (per-agent models already configurable)
- Simplify Cursor-inspired improvements to just Fresh Start mechanism

## [0.2.7] - 2026-01-14

### 🐛 Bug Fixes
- Improve intake CLI JSON parsing for code block responses.

