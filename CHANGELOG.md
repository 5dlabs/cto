## [0.2.9] - 2026-01-15

### 🐛 Bug Fixes
- Preserve local task customizations when syncing from Linear (test_strategy, agent_hint, priority only update if explicitly set in Linear)

## [0.2.24](https://github.com/5dlabs/cto/compare/v0.2.23...v0.2.24) (2026-01-23)


### ✨ Features

* add Frankfurt cluster kubeconfig to secrets pipeline ([#3919](https://github.com/5dlabs/cto/issues/3919)) ([cec6876](https://github.com/5dlabs/cto/commit/cec68768250cc08aed32a2d24f6b8d77762a5245))
* **tools:** add kubernetes-mcp server to MCP configuration ([#3917](https://github.com/5dlabs/cto/issues/3917)) ([572a7e1](https://github.com/5dlabs/cto/commit/572a7e1b7ea45d78e8fa04b98c974dce03d3b4cc))
* **tools:** add Tavily MCP server for AI-powered web search ([#3915](https://github.com/5dlabs/cto/issues/3915)) ([8a5d409](https://github.com/5dlabs/cto/commit/8a5d40941f2a145a38284a7366c610e3fd346d49))


### 🐛 Bug Fixes

* **controller:** add imagePullSecrets to CodeRun pod specs ([#3920](https://github.com/5dlabs/cto/issues/3920)) ([cead939](https://github.com/5dlabs/cto/commit/cead93922eb9666e98839922a4749e95c810bcd5))
* **tools:** inject FIRECRAWL_API_KEY env var into tools deployment ([#3913](https://github.com/5dlabs/cto/issues/3913)) ([35383e9](https://github.com/5dlabs/cto/commit/35383e97b112e966c626b7f1188e4a1cc5c5c761))


### 🔧 Maintenance

* update claude image tag to dev for v0.2.23 intake fix ([#3912](https://github.com/5dlabs/cto/issues/3912)) ([c822c7e](https://github.com/5dlabs/cto/commit/c822c7e23a732430527e50a0d81c6aa7c3b563bb))
* update play workflow template to use claude:dev ([#3914](https://github.com/5dlabs/cto/issues/3914)) ([c485fc7](https://github.com/5dlabs/cto/commit/c485fc74615310ddf6e40971e4cb3e6070237576))

## [0.2.23](https://github.com/5dlabs/cto/compare/v0.2.22...v0.2.23) (2026-01-23)


### ✨ Features

* **intake:** Generate documentation for subtasks ([#3910](https://github.com/5dlabs/cto/issues/3910)) ([ecc30b4](https://github.com/5dlabs/cto/commit/ecc30b4d3f0fc191a1a7b79e15d2fb06ca3b0e32))


### 🐛 Bug Fixes

* **tools:** consume stderr and increase timeout in connection pool ([#3911](https://github.com/5dlabs/cto/issues/3911)) ([047efb3](https://github.com/5dlabs/cto/commit/047efb3c1ca6ff565ce0fa9900c3c5fd4179a5ff))


### 🔧 Maintenance

* **deps:** bump the npm-minor group in /apps/web with 7 updates ([#3891](https://github.com/5dlabs/cto/issues/3891)) ([9ab8f63](https://github.com/5dlabs/cto/commit/9ab8f63647f3393c9db8c432c4a512bb9ced3f9e))

## [0.2.22](https://github.com/5dlabs/cto/compare/v0.2.21...v0.2.22) (2026-01-22)


### 🔧 Maintenance

* **deps:** bump the actions group with 9 updates ([#3896](https://github.com/5dlabs/cto/issues/3896)) ([ccb5911](https://github.com/5dlabs/cto/commit/ccb5911b7921137ea61ac170c4bea46adf21fdcf))
* **deps:** bump the rust-minor group with 24 updates ([#3898](https://github.com/5dlabs/cto/issues/3898)) ([911b516](https://github.com/5dlabs/cto/commit/911b516ea0ae26a665430cde503d0a44616201f0))

## [0.2.21](https://github.com/5dlabs/cto/compare/v0.2.20...v0.2.21) (2026-01-22)


### 🔧 Maintenance

* **deps:** bump opentelemetry-otlp from 0.17.0 to 0.31.0 ([#3899](https://github.com/5dlabs/cto/issues/3899)) ([59099eb](https://github.com/5dlabs/cto/commit/59099ebd95e35f5c82ddeffcae6b8a7829fa53fe))
* **deps:** bump tiktoken-rs from 0.6.0 to 0.9.1 ([#3901](https://github.com/5dlabs/cto/issues/3901)) ([e748bea](https://github.com/5dlabs/cto/commit/e748bead4c20d40cc9891f6072a5e88a0eb67d0b))

## [0.2.20](https://github.com/5dlabs/cto/compare/v0.2.19...v0.2.20) (2026-01-22)


### 🔧 Maintenance

* **deps:** bump opentelemetry_sdk from 0.24.1 to 0.30.0 ([#3902](https://github.com/5dlabs/cto/issues/3902)) ([76d4703](https://github.com/5dlabs/cto/commit/76d4703a63aae5e78647bca810efd5155b5c1285))

## [0.2.19](https://github.com/5dlabs/cto/compare/v0.2.18...v0.2.19) (2026-01-22)


### ✨ Features

* Add Twingate Operator for Latitude cluster zero-trust access ([#3897](https://github.com/5dlabs/cto/issues/3897)) ([adfd629](https://github.com/5dlabs/cto/commit/adfd629baac34d42f9ca3ae75bccaebaf859fb65))


### 🐛 Bug Fixes

* **ci:** use ubuntu-latest for path filter jobs (ISSUE-010) ([#3904](https://github.com/5dlabs/cto/issues/3904)) ([51d5fb2](https://github.com/5dlabs/cto/commit/51d5fb2535e7bb68daa98f137bd0241a7aa4bc39))


### 🔧 Maintenance

* **monitor:** update tracking for check [#15](https://github.com/5dlabs/cto/issues/15) - system healthy after 17 fixes ([444d191](https://github.com/5dlabs/cto/commit/444d191d95ead3a80caa836e9b0cfde3415c40ad))

## [0.2.18](https://github.com/5dlabs/cto/compare/v0.2.17...v0.2.18) (2026-01-22)


### 🐛 Bug Fixes

* resolve pre-commit hook failures ([#3885](https://github.com/5dlabs/cto/issues/3885)) ([36f59df](https://github.com/5dlabs/cto/commit/36f59df16207cbdd57733c803e690acdbe7c88c0))

## [0.2.17](https://github.com/5dlabs/cto/compare/v0.2.16...v0.2.17) (2026-01-20)


### ✨ Features

* **quality:** setup Ralph code quality workflow and fix unused_self ([#3880](https://github.com/5dlabs/cto/issues/3880)) ([43fbfa3](https://github.com/5dlabs/cto/commit/43fbfa37c676fc2d29f2a98715783846af3abcf8))
* **stitch:** add standalone PR review sensor and configuration ([#3877](https://github.com/5dlabs/cto/issues/3877)) ([794e50e](https://github.com/5dlabs/cto/commit/794e50e0c11bd7e1eaee9e757418679833f22049))


### 📚 Documentation

* update README logo to match web app branding ([#3881](https://github.com/5dlabs/cto/issues/3881)) ([220902e](https://github.com/5dlabs/cto/commit/220902e68aa04f41bff27946c883379d7d34913a))

## [0.2.16](https://github.com/5dlabs/cto/compare/v0.2.15...v0.2.16) (2026-01-18)


### 🐛 Bug Fixes

* extend pod GC retention to 2 hours for debugging ([#3875](https://github.com/5dlabs/cto/issues/3875)) ([4c73d00](https://github.com/5dlabs/cto/commit/4c73d00cd6124830f4c0e9778a2ee6d14bd8bc54))

## [0.2.15](https://github.com/5dlabs/cto/compare/v0.2.14...v0.2.15) (2026-01-18)


### ✨ Features

* **tools:** add octocode MCP server for GitHub code search ([#3872](https://github.com/5dlabs/cto/issues/3872)) ([60d2a5c](https://github.com/5dlabs/cto/commit/60d2a5c334a45919895edaacec9f5efb0afa922d))

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
