## [0.2.9] - 2026-01-15

### 🐛 Bug Fixes
- Preserve local task customizations when syncing from Linear (test_strategy, agent_hint, priority only update if explicitly set in Linear)

## [0.2.14](https://github.com/5dlabs/cto/compare/v0.2.13...v0.2.14) (2026-01-17)


### 🐛 Bug Fixes

* **infra:** add retry logic for network operations in infrastructure builds ([#3860](https://github.com/5dlabs/cto/issues/3860)) ([f2d3d7f](https://github.com/5dlabs/cto/commit/f2d3d7f98f51697c1419b1d0a22565eae0edbb2f))

## [0.2.13](https://github.com/5dlabs/cto/compare/v0.2.12...v0.2.13) (2026-01-17)


### ✨ Features

* add OctoCode MCP integration for semantic code research ([#3848](https://github.com/5dlabs/cto/issues/3848)) ([faac278](https://github.com/5dlabs/cto/commit/faac27896812ab6b0bf45c620bde2cbdc442b616))
* **marketing:** add feature flags to hide app access buttons ([#3843](https://github.com/5dlabs/cto/issues/3843)) ([72017cf](https://github.com/5dlabs/cto/commit/72017cf7aa238fdcfe29652073ef0bda145b7d82))


### 🐛 Bug Fixes

* correct invalid model names and add missing skills categories ([#3849](https://github.com/5dlabs/cto/issues/3849)) ([e86052b](https://github.com/5dlabs/cto/commit/e86052b6f6addb1815338b5a0c4860ac60a1b629))
* **intake:** add JSON string escaping guidance to prevent malformed task output ([#3844](https://github.com/5dlabs/cto/issues/3844)) ([b22fbde](https://github.com/5dlabs/cto/commit/b22fbde0029f642907713765445196ede330b0c6))
* **intake:** skip hallucinated JSON content to find valid task objects ([#3845](https://github.com/5dlabs/cto/issues/3845)) ([1f5a14d](https://github.com/5dlabs/cto/commit/1f5a14ddec61136381cebbf7eb621d7a35c6acc0))


### 🔧 Maintenance

* release 0.2.12 ([#3846](https://github.com/5dlabs/cto/issues/3846)) ([172b137](https://github.com/5dlabs/cto/commit/172b137649ce0c57e1edde3dc87b02c166036c71))

## [0.2.12](https://github.com/5dlabs/cto/compare/v0.2.11...v0.2.12) (2026-01-16)


### ✨ Features

* **marketing:** add feature flags to hide app access buttons ([#3843](https://github.com/5dlabs/cto/issues/3843)) ([72017cf](https://github.com/5dlabs/cto/commit/72017cf7aa238fdcfe29652073ef0bda145b7d82))


### 🐛 Bug Fixes

* **ci:** increase build-runtime timeout from 30m to 90m ([#3840](https://github.com/5dlabs/cto/issues/3840)) ([9518aca](https://github.com/5dlabs/cto/commit/9518aca022d292be8d4cdddf55804f220cd5b821))
* **intake:** add --verbose flag required by Claude CLI with stream-json ([#3841](https://github.com/5dlabs/cto/issues/3841)) ([89389cd](https://github.com/5dlabs/cto/commit/89389cd4b9bf7b68364fa4cad5f9b5eb24fc0003))
* **intake:** add JSON string escaping guidance to prevent malformed task output ([#3844](https://github.com/5dlabs/cto/issues/3844)) ([b22fbde](https://github.com/5dlabs/cto/commit/b22fbde0029f642907713765445196ede330b0c6))


### 🔧 Maintenance

* bump version to 0.2.12 ([#3842](https://github.com/5dlabs/cto/issues/3842)) ([d80ae12](https://github.com/5dlabs/cto/commit/d80ae12029f9fb2bbaa824933d86ab4e632d5f06))
* **runtime:** bump intake CLI to v0.2.11 ([cec7754](https://github.com/5dlabs/cto/commit/cec775468026b9990bdd29b28d7f6ba71f89b3fe))
* **runtime:** bump intake CLI to v0.2.11 ([#3838](https://github.com/5dlabs/cto/issues/3838)) ([dff5581](https://github.com/5dlabs/cto/commit/dff55811d1c7612652ce9ebfd39375bf1cb95da6))

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
