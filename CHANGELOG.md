## [0.2.9] - 2026-01-15

### 🐛 Bug Fixes
- Preserve local task customizations when syncing from Linear (test_strategy, agent_hint, priority only update if explicitly set in Linear)

## [0.2.46](https://github.com/5dlabs/cto/compare/v0.2.45...v0.2.46) (2026-02-07)


### ✨ Features

* **infra:** route GitHub webhooks directly to PM server ([#4384](https://github.com/5dlabs/cto/issues/4384)) ([0d145d8](https://github.com/5dlabs/cto/commit/0d145d83460350ec515375a3e48f66333548b5b1))
* **intake:** Lobster intake migration with 5-model voting ([#4379](https://github.com/5dlabs/cto/issues/4379)) ([0c6b799](https://github.com/5dlabs/cto/commit/0c6b799b87f386fedbb9216c993bf0a76f2cf62b))


### 🐛 Bug Fixes

* **openclaw:** remove unsupported group:memory from agent tools allow lists ([#4386](https://github.com/5dlabs/cto/issues/4386)) ([0ad8144](https://github.com/5dlabs/cto/commit/0ad8144bd54e88549fb85ca2f771768fb7c67067))
* **secrets:** update ExternalSecret with all 16 deployed bot tokens ([#4381](https://github.com/5dlabs/cto/issues/4381)) ([e1dfd3a](https://github.com/5dlabs/cto/commit/e1dfd3a3a1d37d60e53c78f276123bd0ee6ec31f))


### 🔧 Maintenance

* **openclaw:** remove pixel, scout, and pixel-assistant agents ([#4385](https://github.com/5dlabs/cto/issues/4385)) ([86e18d2](https://github.com/5dlabs/cto/commit/86e18d2a50479af921b3008afdf1aae45125a96a))

## [0.2.45](https://github.com/5dlabs/cto/compare/v0.2.44...v0.2.45) (2026-02-07)


### ✨ Features

* **openclaw:** deploy all 14 remaining agents to Kubernetes ([#4375](https://github.com/5dlabs/cto/issues/4375)) ([b6cf4b6](https://github.com/5dlabs/cto/commit/b6cf4b616d633be37466d8ba25c0b5ddeb706175))
* **openclaw:** expand container-builds skill with Docker→Kaniko guide ([#4378](https://github.com/5dlabs/cto/issues/4378)) ([25aaaeb](https://github.com/5dlabs/cto/commit/25aaaeb8684f73e91a2c70845864599d96661c27))


### 🐛 Bug Fixes

* **openclaw:** add RBAC and kubectl for kaniko sidecar exec ([#4376](https://github.com/5dlabs/cto/issues/4376)) ([9e20498](https://github.com/5dlabs/cto/commit/9e204985e537c95f96fc988a0b5fc618fe58d84e))

## [0.2.44](https://github.com/5dlabs/cto/compare/v0.2.43...v0.2.44) (2026-02-07)


### ✨ Features

* **openclaw:** add kaniko container-builds skill ([#4374](https://github.com/5dlabs/cto/issues/4374)) ([8121730](https://github.com/5dlabs/cto/commit/8121730e4dfd64b0e3ed0f842f59f6d29f0713cb))


### 🐛 Bug Fixes

* **loki:** switch from SeaweedFS S3 to filesystem storage ([#4373](https://github.com/5dlabs/cto/issues/4373)) ([818d568](https://github.com/5dlabs/cto/commit/818d568898746c1d6e7ebe74f889c164b33f7546))
* **openclaw:** correct Loki URL and add informational pod labels ([#4370](https://github.com/5dlabs/cto/issues/4370)) ([4fe5ce5](https://github.com/5dlabs/cto/commit/4fe5ce5d2ba9aa0d9826dd688d4301e69c9f4dda))
* **openclaw:** move info fields from labels to pod annotations ([#4372](https://github.com/5dlabs/cto/issues/4372)) ([767fc77](https://github.com/5dlabs/cto/commit/767fc776a443a1aa077ee9fb1c4450da3e483cec))

## [0.2.43](https://github.com/5dlabs/cto/compare/v0.2.42...v0.2.43) (2026-02-07)


### 🐛 Bug Fixes

* **openclaw:** correct kaniko debug image tag to :debug ([#4369](https://github.com/5dlabs/cto/issues/4369)) ([dbd08aa](https://github.com/5dlabs/cto/commit/dbd08aaa70257ea87b4e664fa4b8d7a6a1d874b2))
* **openclaw:** correct repoURL and chart path in ArgoCD applications ([#4365](https://github.com/5dlabs/cto/issues/4365)) ([29e64cd](https://github.com/5dlabs/cto/commit/29e64cdac81396de4ec377c4c557a549bb1838e7))
* **openclaw:** remove unsupported config keys and fix kaniko sidecar ([#4367](https://github.com/5dlabs/cto/issues/4367)) ([91f70ea](https://github.com/5dlabs/cto/commit/91f70eaf547cfcc25af9d4b321b22c167a695f4c))

## [0.2.42](https://github.com/5dlabs/cto/compare/v0.2.41...v0.2.42) (2026-02-07)


### ✨ Features

* **openclaw:** memory optimization, group:memory allowlists, heartbeat autonomy ([#4362](https://github.com/5dlabs/cto/issues/4362)) ([1916f07](https://github.com/5dlabs/cto/commit/1916f073ca5195eac1ab6d8d803c9b863ffa7fe4))

## [0.2.41](https://github.com/5dlabs/cto/compare/v0.2.40...v0.2.41) (2026-02-07)


### ✨ Features

* **openclaw:** Phase 2 golden copy — unified image, tools, skills, secrets ([#4358](https://github.com/5dlabs/cto/issues/4358)) ([04c78d0](https://github.com/5dlabs/cto/commit/04c78d09852c840bdc40b1ec7c988c16dfacd2ea))


### ♻️ Refactoring

* organize research docs and add Grok MCP server ([#4359](https://github.com/5dlabs/cto/issues/4359)) ([7aae2a2](https://github.com/5dlabs/cto/commit/7aae2a2e934fd6c219161c57477a2690148331be))

## [0.2.40](https://github.com/5dlabs/cto/compare/v0.2.39...v0.2.40) (2026-02-06)


### ✨ Features

* **agents:** add OpenClaw image to agent builds Add ([92176c4](https://github.com/5dlabs/cto/commit/92176c40f3b42dec2dd8c517c041d89a93fccb2c))
* **agents:** enable dexter agent build ([1d2b036](https://github.com/5dlabs/cto/commit/1d2b0361f97518d6b44f3bf3d80a8f7543f33e82))
* **clawd:** add clawd agent configuration and identity ([#4349](https://github.com/5dlabs/cto/issues/4349)) ([f0a03a9](https://github.com/5dlabs/cto/commit/f0a03a9853fb20d66f5fe23a35a70181a58fe1b7))
* **cupid:** add cupid agent configuration and identity ([#4347](https://github.com/5dlabs/cto/issues/4347)) ([18da541](https://github.com/5dlabs/cto/commit/18da541cbc58aca9114aa0b380931f2991d9eace))
* **keeper:** keeper agent configuration and identity ([#4355](https://github.com/5dlabs/cto/issues/4355)) ([0f22793](https://github.com/5dlabs/cto/commit/0f22793c00027fe239d86976f83fb3b2204c1120))
* **marketing:** add managed services tech stack section ([#4357](https://github.com/5dlabs/cto/issues/4357)) ([d46b956](https://github.com/5dlabs/cto/commit/d46b956a9e31ead7809fc7ca8a4fbadf3e441052))
* **openclaw:** deploy OpenClaw agents to Kubernetes in bots namespace ([#4335](https://github.com/5dlabs/cto/issues/4335)) ([d21537d](https://github.com/5dlabs/cto/commit/d21537dbe8262b22c0598f3ff2f26dbe18fbd107))
* **openclaw:** use npm package instead of source build ([1126503](https://github.com/5dlabs/cto/commit/1126503e8482f8e39b7f326dab1bf50583ff887b))
* **pixel-assistant:** add pixel-assistant agent configuration ([#4353](https://github.com/5dlabs/cto/issues/4353)) ([35bbccb](https://github.com/5dlabs/cto/commit/35bbccb4dd936f0a9b13faa465d1c0fd4b3861c2))
* **review:** add review agent configuration and identity ([#4352](https://github.com/5dlabs/cto/issues/4352)) ([ce122e9](https://github.com/5dlabs/cto/commit/ce122e909968f8304d0a2c2769a52524e11a8cd8))
* **scout:** add scout agent configuration and identity ([#4344](https://github.com/5dlabs/cto/issues/4344)) ([2cb26cd](https://github.com/5dlabs/cto/commit/2cb26cd1221f9639945588dc5a08a47feb27bb95))
* **webapp:** add webapp agent configuration and identity ([#4354](https://github.com/5dlabs/cto/issues/4354)) ([d0ef4c4](https://github.com/5dlabs/cto/commit/d0ef4c4cf851cc3d02bea1cf001dd089af36720e))


### 🐛 Bug Fixes

* **agents:** fix unbound variable error in build-runtime ([aba9bd7](https://github.com/5dlabs/cto/commit/aba9bd785471b03cfd5b8e970b3e0d88f20f2da3))
* **openclaw:** fix version extraction from release name ([4db156b](https://github.com/5dlabs/cto/commit/4db156b804c9fb737143c445a9e4b06ab8152c23))
* **openclaw:** use correct URL format for OpenClaw releases ([a7d9e28](https://github.com/5dlabs/cto/commit/a7d9e28c35a4a34c827c0be53713129203f33954))
* **release:** add play-monitor binary to release artifacts ([563059d](https://github.com/5dlabs/cto/commit/563059d2443a5805163a9a4e69255a0b1f0bba0a))

## [0.2.39](https://github.com/5dlabs/cto/compare/v0.2.38...v0.2.39) (2026-02-05)


### 🐛 Bug Fixes

* **tools:** correctly handle SYSTEM_CONFIG_PATH as file path ([a5375e6](https://github.com/5dlabs/cto/commit/a5375e62f7cde7014d6e621683af3cd89a25a33b))

## [0.2.38](https://github.com/5dlabs/cto/compare/v0.2.37...v0.2.38) (2026-02-05)


### 🐛 Bug Fixes

* **agents:** Add persistence logic to Atlas and Bolt agents ([#4290](https://github.com/5dlabs/cto/issues/4290)) ([9d6919d](https://github.com/5dlabs/cto/commit/9d6919d412e5a23b9ca2e35049f35c9d4edd31c8))
* **codeql:** correct job condition and increase timeout ([#4325](https://github.com/5dlabs/cto/issues/4325)) ([c99a729](https://github.com/5dlabs/cto/commit/c99a7299640bb3210b638fb93b3bca208369a482))
* **controller:** use TimeoutLayer::with_status_code for consistent 408 responses ([#4317](https://github.com/5dlabs/cto/issues/4317)) ([c329fda](https://github.com/5dlabs/cto/commit/c329fdab64b6c941e70573f1222a4a236e940a7a))
* Preserve GitHub App private key for agent access ([#4316](https://github.com/5dlabs/cto/issues/4316)) ([66396d0](https://github.com/5dlabs/cto/commit/66396d0ee6c6c001a7391f5cb659f9ebf2d39946))
* **tools:** release stdio semaphore permit immediately after init ([#4304](https://github.com/5dlabs/cto/issues/4304)) ([77b70cd](https://github.com/5dlabs/cto/commit/77b70cd83544567e828ce5aac9fadc36d761ff33))

## [0.2.37](https://github.com/5dlabs/cto/compare/v0.2.36...v0.2.37) (2026-02-04)


### ✨ Features

* **cto-app:** React UI Scaffolding - Phase 1 ([#4245](https://github.com/5dlabs/cto/issues/4245)) ([de3fbfe](https://github.com/5dlabs/cto/commit/de3fbfe3db61564fd5c1dbadb3068f51055d7187))
* **intake-agent:** add Anthropic API key support for Claude provider ([fadf37e](https://github.com/5dlabs/cto/commit/fadf37eaed0477ad080ebea44f819341b0d1886c))
* **intake-agent:** implement 3-agent debate pattern with arbiter decision ([d4dff79](https://github.com/5dlabs/cto/commit/d4dff79040fed9bf970acf2a46aa9e33317e5dc4))
* **tasks:** AlertHub intake output with 10 tasks and prompts ([#4242](https://github.com/5dlabs/cto/issues/4242)) ([53fff1c](https://github.com/5dlabs/cto/commit/53fff1c0ac567baf229667bb292152af17cf685c))
* **tools:** add summary log for tool discovery ([#4219](https://github.com/5dlabs/cto/issues/4219)) ([c947dff](https://github.com/5dlabs/cto/commit/c947dff1bd258a48452272ed6d39b12bdc789db7))


### 🐛 Bug Fixes

* **ci:** remove frozen-lockfile flag for bun due to known issues ([#4279](https://github.com/5dlabs/cto/issues/4279)) ([9e24e7e](https://github.com/5dlabs/cto/commit/9e24e7e073178e458e3b566da78be47b90aca780))
* **controller:** make GITHUB_APP_INSTALLATION_ID optional ([#4221](https://github.com/5dlabs/cto/issues/4221)) ([ce985c1](https://github.com/5dlabs/cto/commit/ce985c19fa09cf92699359bd033600f9fcece935))
* **detection:** route Java/Ruby/PHP to Generic instead of Nova ([#4254](https://github.com/5dlabs/cto/issues/4254)) ([5996f56](https://github.com/5dlabs/cto/commit/5996f56cf3651ebd59bf49f6e5369566b200606a))
* **intake-agent:** add --help and --version CLI flag support ([#4266](https://github.com/5dlabs/cto/issues/4266)) ([f62ddd3](https://github.com/5dlabs/cto/commit/f62ddd3184322f347ba993c22f3e08698f8dc9a6))
* **intake-agent:** add missing @streamparser/json dependency ([#4261](https://github.com/5dlabs/cto/issues/4261)) ([e72ef24](https://github.com/5dlabs/cto/commit/e72ef24e1ed8a91a7e2cd53cb8168b480c72bf70))
* **intake-agent:** enforce single-concern subtasks per SUBTASK-SPLITTING-GUIDE ([115f6e2](https://github.com/5dlabs/cto/commit/115f6e24be7ab863495e92bbc051fb5deafc1dc8))
* **intake-agent:** improve single-concern validation accuracy ([8936f07](https://github.com/5dlabs/cto/commit/8936f074e386a7fb0ed7015dce0e081f6a7889b1))
* **pm:** remove unnecessary hashes from raw string literal ([37faa29](https://github.com/5dlabs/cto/commit/37faa29268a8191f2d5669496a2115d96cf33673))
* **pm:** resolve all clippy pedantic warnings in status-sync ([8e9b0f6](https://github.com/5dlabs/cto/commit/8e9b0f6af71d8a0dd03886293e0aa8936f5666c9))
* **pm:** resolve clippy errors in agent_interactions and detection/utils ([#4211](https://github.com/5dlabs/cto/issues/4211)) ([96057d0](https://github.com/5dlabs/cto/commit/96057d01ac0642431bf9e2003df5916b21e13ab8))
* **pm:** use GitHub API instead of gh CLI for CI failure handler ([#4208](https://github.com/5dlabs/cto/issues/4208)) ([9a0f78e](https://github.com/5dlabs/cto/commit/9a0f78eccb15df3eb6cfb844548ad7876acaa3a8))
* **templates:** remove unnecessary DNS checks ([#4262](https://github.com/5dlabs/cto/issues/4262)) ([b72170e](https://github.com/5dlabs/cto/commit/b72170e198368f8be15de182b09a27fc5b978878))
* **tools:** add timeout to stdio MCP init to prevent hanging ([706a17d](https://github.com/5dlabs/cto/commit/706a17d1a60d8cff0ab66a8bcf6a5932b988f497))
* **tools:** limit concurrent stdio init + add smoke tests to prevent regressions ([#4224](https://github.com/5dlabs/cto/issues/4224)) ([e9556ab](https://github.com/5dlabs/cto/commit/e9556ab61a163f50d0d32d69bf4d8b1ee08821bd))
* **tools:** LRU connection pool + lint fixes ([#4218](https://github.com/5dlabs/cto/issues/4218)) ([34ff24b](https://github.com/5dlabs/cto/commit/34ff24bcaea6d1aa34954c724ee039f61eaf2453))


### 📚 Documentation

* **prds:** add Morgan Coordinator PRD and ACP research notes ([#4214](https://github.com/5dlabs/cto/issues/4214)) ([ad3d84c](https://github.com/5dlabs/cto/commit/ad3d84cd3c7c1abe66c5e6bf40f3c2da7e33a3f4))
* **prds:** Agent Capability PRDs - LSP, Self-Diagnosis, Recursive Learning ([#4225](https://github.com/5dlabs/cto/issues/4225)) ([5ddacec](https://github.com/5dlabs/cto/commit/5ddacece9fd645c2b5749151a587ce2e6775c2a7))


### 🔧 Maintenance

* **deps:** bump @types/node from 20.19.28 to 25.2.0 in /apps/web ([#4238](https://github.com/5dlabs/cto/issues/4238)) ([300a220](https://github.com/5dlabs/cto/commit/300a220ae3d28ed740dd6e7be04d7a29ba054953))
* **deps:** bump jsdom from 27.4.0 to 28.0.0 in /apps/web ([#4237](https://github.com/5dlabs/cto/issues/4237)) ([7ce9421](https://github.com/5dlabs/cto/commit/7ce942187b883d334072269dc409a47742cb4d18))
* **deps:** bump octocrab from 0.38.0 to 0.49.5 ([#4234](https://github.com/5dlabs/cto/issues/4234)) ([bb3415d](https://github.com/5dlabs/cto/commit/bb3415da16a1b77cac0c7e90de1a8a06fb713b0c))
* **deps:** bump the marketing-npm group in /marketing with 7 updates ([#4230](https://github.com/5dlabs/cto/issues/4230)) ([2a35dc9](https://github.com/5dlabs/cto/commit/2a35dc951a35fd9c83c2b291f5637e0e7ba17b2b))
* **deps:** bump the rust-minor group across 1 directory with 2 updates ([#4255](https://github.com/5dlabs/cto/issues/4255)) ([8c24c59](https://github.com/5dlabs/cto/commit/8c24c597dca3ea76fcbf125e8d774bece0778888))

## [0.2.36](https://github.com/5dlabs/cto/compare/v0.2.35...v0.2.36) (2026-02-01)


### ✨ Features

* **intake-agent:** subagent-aware task expansion and dual-agent support ([#4136](https://github.com/5dlabs/cto/issues/4136)) ([2e447ac](https://github.com/5dlabs/cto/commit/2e447ace32adae6035a9ead52a1f9d548129eebe))
* **intake:** add file analysis utilities ([#4148](https://github.com/5dlabs/cto/issues/4148)) ([9988fbb](https://github.com/5dlabs/cto/commit/9988fbb6698fab461308515b103ec4642a6204ec))
* **linear-sink:** CLI-agnostic tool/skill discovery for Linear agent dialog ([#4129](https://github.com/5dlabs/cto/issues/4129)) ([f5923e9](https://github.com/5dlabs/cto/commit/f5923e9f3e44dd21735fe1e32001e5ba3807e88e))
* Multi-provider LLM support ([#4191](https://github.com/5dlabs/cto/issues/4191)) ([8a16d02](https://github.com/5dlabs/cto/commit/8a16d02974fc3e7b981c8ea40d60362b3cff64e7))
* **stitch:** Stitch implementation branch merge ([#4204](https://github.com/5dlabs/cto/issues/4204)) ([867b694](https://github.com/5dlabs/cto/commit/867b694869e984fe5e32a0166ee7998c3dbada3f))
* **tests:** CLI agent testing infrastructure with Linear sidecar ([#4180](https://github.com/5dlabs/cto/issues/4180)) ([846a1a8](https://github.com/5dlabs/cto/commit/846a1a851b8e8ec5048614b911f831535fe77ed7))
* **tools:** add per-MCP health status and metrics endpoints ([#4189](https://github.com/5dlabs/cto/issues/4189)) ([b7fe6f5](https://github.com/5dlabs/cto/commit/b7fe6f5a9bd78b4eb93156476d265c815cfe2e8f))
* **tools:** LRU connection pool to prevent MCP subprocess OOM ([#4188](https://github.com/5dlabs/cto/issues/4188)) ([0c95319](https://github.com/5dlabs/cto/commit/0c95319432e380092704bb4d3cebc3e7d7787035))


### 🐛 Bug Fixes

* **tools:** add timeout and error handling for MCP server init ([#4187](https://github.com/5dlabs/cto/issues/4187)) ([b4ab5bd](https://github.com/5dlabs/cto/commit/b4ab5bd34ba63222b9438bb4639d8dc677ebd30f))
* **tools:** remove redis from server config (client-side only) ([#4190](https://github.com/5dlabs/cto/issues/4190)) ([a202ae8](https://github.com/5dlabs/cto/commit/a202ae8802300de7b8c9e0c1f12f60404a775693))


### 📚 Documentation

* **intake:** add subtask splitting guide for intake agent ([#4199](https://github.com/5dlabs/cto/issues/4199)) ([03cca6c](https://github.com/5dlabs/cto/commit/03cca6c690ac753754df1705870c7d226ff53e3a))

## [0.2.35](https://github.com/5dlabs/cto/compare/v0.2.34...v0.2.35) (2026-01-31)


### ✨ Features

* add Stitch agent config + detection-aware review template ([b0dd873](https://github.com/5dlabs/cto/commit/b0dd873b938fab78d062a324ce3e8333db94a7d1))
* **examples:** AlertHub planning documents from intake-agent ([#4163](https://github.com/5dlabs/cto/issues/4163)) ([a3964f5](https://github.com/5dlabs/cto/commit/a3964f5385c35789436dca594165ce030e88cfd4))
* remediation buttons Phase A + config fixes ([#4123](https://github.com/5dlabs/cto/issues/4123)) ([399aa40](https://github.com/5dlabs/cto/commit/399aa408961319294a83ac9fe7fee2aefb3d064d))
* **stitch:** CI failure remediation buttons (webhook-based) ([#4166](https://github.com/5dlabs/cto/issues/4166)) ([3e3cc8d](https://github.com/5dlabs/cto/commit/3e3cc8d9e0372166fe18431e1a1e1d6a2dc9d3c9))


### 🐛 Bug Fixes

* define CURL_MAX_RETRIES before retry loops ([2ead8c3](https://github.com/5dlabs/cto/commit/2ead8c373073b2f2989743a75ba015f3ea328fdf))
* **intake-agent:** Robust JSON parsing with jsonrepair ([#4164](https://github.com/5dlabs/cto/issues/4164)) ([4a8e899](https://github.com/5dlabs/cto/commit/4a8e89930932d1694787e6e15927307a76391fe1))
* Stitch must use gh CLI for reviews to post as correct identity ([cb1d67d](https://github.com/5dlabs/cto/commit/cb1d67d20ce461a28c84784d2201aeb45c2211fc))
* **stitch:** deduplicate CodeRuns using deterministic names ([#4143](https://github.com/5dlabs/cto/issues/4143)) ([dadc8a2](https://github.com/5dlabs/cto/commit/dadc8a248659b9d1f940993a40837629b3dc6491))
* **stitch:** enforce gh CLI for reviews to use correct GitHub App identity ([#4157](https://github.com/5dlabs/cto/issues/4157)) ([015892a](https://github.com/5dlabs/cto/commit/015892a09560e658aa6996a37e732f881e18297b))
* **tools:** bump memory limit 2Gi→4Gi to prevent OOM ([#4186](https://github.com/5dlabs/cto/issues/4186)) ([530fd5b](https://github.com/5dlabs/cto/commit/530fd5b295d913a1630825547346d950aa05d326))


### 📚 Documentation

* add CTO Lite freemium desktop application plan ([#4154](https://github.com/5dlabs/cto/issues/4154)) ([39a1e18](https://github.com/5dlabs/cto/commit/39a1e183032beb698dce43698052884ddc09fd83))
* add remediation buttons implementation status ([#4131](https://github.com/5dlabs/cto/issues/4131)) ([bbcfbde](https://github.com/5dlabs/cto/commit/bbcfbde9afd9bf54b21dd563a01537a2b3005870))
* AlertHub full task/subtask structure (50 tasks, 200 subtasks) ([#4145](https://github.com/5dlabs/cto/issues/4145)) ([cbf462b](https://github.com/5dlabs/cto/commit/cbf462b9de9587eff4968b52b88a450ed7c527ca))

## [0.2.34](https://github.com/5dlabs/cto/compare/v0.2.33...v0.2.34) (2026-01-30)


### ✨ Features

* Add Twingate connector deployment via Argo CD ([#4082](https://github.com/5dlabs/cto/issues/4082)) ([f2f7f8a](https://github.com/5dlabs/cto/commit/f2f7f8ac5e1f81593777a93fcc70c92129b7544e))
* **config:** add AgentSkills to cto-config.json with job-type-based resolution ([#4099](https://github.com/5dlabs/cto/issues/4099)) ([4bbf998](https://github.com/5dlabs/cto/commit/4bbf998466b1bdd721e1921a49d909f7c0e76f81))
* **controller:** add GitHub App installation ID for faster auth ([#4100](https://github.com/5dlabs/cto/issues/4100)) ([23151ea](https://github.com/5dlabs/cto/commit/23151ea162150b2dec05c00a981424186d4617e7))
* **ingress:** add DNS namespace separation for home and Frankfurt clusters ([66f8ba4](https://github.com/5dlabs/cto/commit/66f8ba40586e1b34c9c0d3b2c1bfd62489f97493))
* intake SDK migration and linear-sync crate ([#4096](https://github.com/5dlabs/cto/issues/4096)) ([d674f94](https://github.com/5dlabs/cto/commit/d674f94c7de0ec44ed5803ecd6b41fdad1cdc0c4))
* **tools:** add custom headers support and MiniMax/Cloudflare env vars ([#4092](https://github.com/5dlabs/cto/issues/4092)) ([27eeab2](https://github.com/5dlabs/cto/commit/27eeab27160516ce1263fe0f1908c06869d85901))


### 🐛 Bug Fixes

* Add extraEnv to explicitly set token env vars from secret ([d0f26a3](https://github.com/5dlabs/cto/commit/d0f26a38343a7386ea10116fa1a12b5878ad99ac))
* **cilium:** enable hostNetwork and ClusterFirstWithHostNet DNS for Hubble Relay on Talos ([8bf67d2](https://github.com/5dlabs/cto/commit/8bf67d202dadea74592b0d8c406e188da565621f))
* enable hostNetwork for Hubble Relay to access Cilium agents on Talos ([1136e87](https://github.com/5dlabs/cto/commit/1136e872c5e06451fe17e434b98c13f0d37c1a75))
* **ingress:** enable external-dns for Frankfurt cluster ([ae44ca1](https://github.com/5dlabs/cto/commit/ae44ca196f8206147ca6c76048dc167ad53648a2))
* Move existingSecret to top level ([ec996eb](https://github.com/5dlabs/cto/commit/ec996eb0048f55db97d4014c2bc77cd7cb9d770a))
* **pm:** OAuth hardening - enable auto token refresh ([#4097](https://github.com/5dlabs/cto/issues/4097)) ([57ae4cc](https://github.com/5dlabs/cto/commit/57ae4ccd236eb06f84f24da15971a80b1f743177))
* Reduce CPU request to 50m for scheduling ([891850f](https://github.com/5dlabs/cto/commit/891850fc35a9bff1c7b4bede168ba8748bf04b40))
* **seaweedfs:** configure for single-node cluster ([#4084](https://github.com/5dlabs/cto/issues/4084)) ([c1819b1](https://github.com/5dlabs/cto/commit/c1819b1ed8508c876db79f639f27327a4a1d6ea0))
* **templates:** add security and documents categories to skills-setup ([#4103](https://github.com/5dlabs/cto/issues/4103)) ([6fd0d36](https://github.com/5dlabs/cto/commit/6fd0d3609752b07c280db292036424a635069f15))
* temporarily disable presync validation hook ([#4083](https://github.com/5dlabs/cto/issues/4083)) ([7f34191](https://github.com/5dlabs/cto/commit/7f34191f60dabc25fb8ff25ec9f1f4660692f631))
* **tools:** add missing env vars for MCP servers ([#4091](https://github.com/5dlabs/cto/issues/4091)) ([552c9b6](https://github.com/5dlabs/cto/commit/552c9b6b6d91187d69796eced83c9e6d5335ecd6))
* **tools:** use @iflow-mcp/firecrawl-mcp package ([5357882](https://github.com/5dlabs/cto/commit/53578829bbbf4ad3c6137d6fee6204ea6da2d0bd))
* Use correct env var names for Twingate connector tokens ([753c725](https://github.com/5dlabs/cto/commit/753c72500dbd8ce8e64d7d251cac47d8cc5a3f48))
* Use existingSecret as string (secret name only) ([147bd54](https://github.com/5dlabs/cto/commit/147bd54478123e54228b8ad64e482fcd1e4b500a))
* Use existingSecret format for Twingate connector tokens ([7d1825b](https://github.com/5dlabs/cto/commit/7d1825b900401558926e58d43e9351f9832738a9))
* Use valid semver constraint for Twingate connector chart ([d47be5f](https://github.com/5dlabs/cto/commit/d47be5f2459de53d906c25a09906f9dd0763f16c))


### 📚 Documentation

* add DNS configuration guide for multi-cluster setup ([6f93e80](https://github.com/5dlabs/cto/commit/6f93e8084631d8595da4f03253e4ded5e32470df))


### 🔧 Maintenance

* **deps:** bump kube-derive from 0.93.1 to 0.98.0 ([#3900](https://github.com/5dlabs/cto/issues/3900)) ([72c4dfc](https://github.com/5dlabs/cto/commit/72c4dfc17d1ecdb8e9524ad8142991dc6f08135a))
* **infra:** enable controller with 1 replica ([#4104](https://github.com/5dlabs/cto/issues/4104)) ([ae5b1ac](https://github.com/5dlabs/cto/commit/ae5b1ac20e9f28a66534194c8b702b90d51a0f72))

## [0.2.33](https://github.com/5dlabs/cto/compare/v0.2.32...v0.2.33) (2026-01-27)


### ✨ Features

* **research:** auto-install skills and MCP servers from research items ([#4066](https://github.com/5dlabs/cto/issues/4066)) ([0660026](https://github.com/5dlabs/cto/commit/06600260daad9a2f0e960a05a92b04c172d89fed))


### 🔧 Maintenance

* **deps:** Bump uuid in the rust-minor group across 1 directory ([#4079](https://github.com/5dlabs/cto/issues/4079)) ([916a25d](https://github.com/5dlabs/cto/commit/916a25da673e28c2ac1f58d3860fa03e2c5042eb))

## [0.2.32](https://github.com/5dlabs/cto/compare/v0.2.31...v0.2.32) (2026-01-27)


### 🐛 Bug Fixes

* **cloudflare:** support existing tunnels in ClusterTunnel ([#4065](https://github.com/5dlabs/cto/issues/4065)) ([cdc4ac4](https://github.com/5dlabs/cto/commit/cdc4ac422798d05c6bfe84ffa54e1d29a7df2513))


### 🔧 Maintenance

* **deps:** Bump actions/checkout from 4 to 6 in the actions group ([#4077](https://github.com/5dlabs/cto/issues/4077)) ([c24cbcc](https://github.com/5dlabs/cto/commit/c24cbccaeb88daf70e959887051da1ebb519c675))
* **deps:** Bump indicatif from 0.17.11 to 0.18.3 ([#4075](https://github.com/5dlabs/cto/issues/4075)) ([bbec339](https://github.com/5dlabs/cto/commit/bbec3390afd89ac6394b91b406996c58084d7b26))
* **deps:** Bump jsdom from 26.1.0 to 27.4.0 in /apps/web ([#4074](https://github.com/5dlabs/cto/issues/4074)) ([9f10095](https://github.com/5dlabs/cto/commit/9f1009522e313c373109f161aa85599edea3e61a))
* **deps:** Bump the npm-minor group in /apps/web with 6 updates ([#4069](https://github.com/5dlabs/cto/issues/4069)) ([94a14bc](https://github.com/5dlabs/cto/commit/94a14bc6fc41bdfbbfbb303964784d2f2be0a0be))
* **deps:** Bump vitest from 3.2.4 to 4.0.18 in /apps/web ([#4076](https://github.com/5dlabs/cto/issues/4076)) ([40190cf](https://github.com/5dlabs/cto/commit/40190cf4192c7af1b2320d23880aa3595958ee3a))
* **infra:** increase minRunners to 8 to avoid CI queue buildup ([272b3a6](https://github.com/5dlabs/cto/commit/272b3a69feace5574106c8b68e15dec0136132bb))

## [0.2.31](https://github.com/5dlabs/cto/compare/v0.2.30...v0.2.31) (2026-01-25)


### 🐛 Bug Fixes

* **stitch:** add EventBus and webhook secret for PR reviews ([#4061](https://github.com/5dlabs/cto/issues/4061)) ([b3282c4](https://github.com/5dlabs/cto/commit/b3282c41c00ce9d11d33d847e8dc1486d8501057))

## [0.2.30](https://github.com/5dlabs/cto/compare/v0.2.29...v0.2.30) (2026-01-25)


### 🐛 Bug Fixes

* **tools:** upgrade Node.js to v22 LTS for firecrawl-mcp compatibility ([#4058](https://github.com/5dlabs/cto/issues/4058)) ([199f5d7](https://github.com/5dlabs/cto/commit/199f5d7ed033f5992c7d9c5d89f1de9edead441c))

## [0.2.29](https://github.com/5dlabs/cto/compare/v0.2.28...v0.2.29) (2026-01-25)


### ✨ Features

* **mcp:** add add_skills tool to import skills from GitHub repos ([#4057](https://github.com/5dlabs/cto/issues/4057)) ([3d95e70](https://github.com/5dlabs/cto/commit/3d95e7096c7d06a0819d013f23465d25d944a4b5))
* **tools:** add CONTEXT7_API_KEY to context7 MCP server config ([#4055](https://github.com/5dlabs/cto/issues/4055)) ([e802eaf](https://github.com/5dlabs/cto/commit/e802eaf8513e6d3e2a4c14622537167369e401d2))
* **tools:** add MCP Prompts server for prompt management ([#4056](https://github.com/5dlabs/cto/issues/4056)) ([9a50027](https://github.com/5dlabs/cto/commit/9a500275ac08cb2b4124234d53550b5057be53ab))
* **tools:** add Tavily API key to OpenBao seed script ([#4052](https://github.com/5dlabs/cto/issues/4052)) ([2c78c7a](https://github.com/5dlabs/cto/commit/2c78c7add31cde154171c30d0ad4490ed4b5bb69))


### 🐛 Bug Fixes

* **tools:** add supplementalGroups for Docker socket access ([#4054](https://github.com/5dlabs/cto/issues/4054)) ([c17e6fb](https://github.com/5dlabs/cto/commit/c17e6fbd626ec5451b04f238afbbede6f7ffe365))

## [0.2.28](https://github.com/5dlabs/cto/compare/v0.2.27...v0.2.28) (2026-01-24)


### ✨ Features

* **pm:** Add Linear OAuth client credentials flow and fix sidecar activity serialization ([#4025](https://github.com/5dlabs/cto/issues/4025)) ([44836d2](https://github.com/5dlabs/cto/commit/44836d23a4240c601cad77355b549dc29e1a89ec))


### 📚 Documentation

* **research:** add 1 new research entries ([#3972](https://github.com/5dlabs/cto/issues/3972)) ([491f6de](https://github.com/5dlabs/cto/commit/491f6de98f53ed65576db5d3ef075fddea3958fc))
* **research:** add 1 new research entries ([#3996](https://github.com/5dlabs/cto/issues/3996)) ([759fdbb](https://github.com/5dlabs/cto/commit/759fdbb086672d9f2b0c9d13f73d85aaf4325b6f))
* **research:** add 1 new research entries ([#4002](https://github.com/5dlabs/cto/issues/4002)) ([de1cc7a](https://github.com/5dlabs/cto/commit/de1cc7a24271f4993d9464ef6b0ff0b624a08b58))
* **research:** add 1 new research entries ([#4003](https://github.com/5dlabs/cto/issues/4003)) ([3caa2cd](https://github.com/5dlabs/cto/commit/3caa2cdf04017ceb4d32b2c3a155483c06ca353d))
* **research:** add 1 new research entries ([#4007](https://github.com/5dlabs/cto/issues/4007)) ([0e77822](https://github.com/5dlabs/cto/commit/0e77822390cdbcdfba36959a0ac3e179996ed756))
* **research:** add 1 new research entries ([#4008](https://github.com/5dlabs/cto/issues/4008)) ([dfc28bd](https://github.com/5dlabs/cto/commit/dfc28bd0368caccc5346a148e09cd8a6d3c54847))
* **research:** add 1 new research entries ([#4010](https://github.com/5dlabs/cto/issues/4010)) ([e8a7f59](https://github.com/5dlabs/cto/commit/e8a7f59065902c023ce3560422573c8e6b4fd307))
* **research:** add 1 new research entries ([#4011](https://github.com/5dlabs/cto/issues/4011)) ([59d4470](https://github.com/5dlabs/cto/commit/59d4470b45cb4faea8b68d6a4f9835ebe0465929))
* **research:** add 1 new research entries ([#4014](https://github.com/5dlabs/cto/issues/4014)) ([28681af](https://github.com/5dlabs/cto/commit/28681afb2f065ed5853b274802e30971f0670715))
* **research:** add 1 new research entries ([#4015](https://github.com/5dlabs/cto/issues/4015)) ([d4878bc](https://github.com/5dlabs/cto/commit/d4878bc8c553d15db83552dea7ceb4def4ff8517))
* **research:** add 1 new research entries ([#4016](https://github.com/5dlabs/cto/issues/4016)) ([60a171d](https://github.com/5dlabs/cto/commit/60a171dca98a3ab174caba1bf6f7806f224fb6aa))
* **research:** add 1 new research entries ([#4023](https://github.com/5dlabs/cto/issues/4023)) ([a4fbbaa](https://github.com/5dlabs/cto/commit/a4fbbaa2c059f029bb7ff48e41248e3ab75b59ab))
* **research:** add 1 new research entries ([#4024](https://github.com/5dlabs/cto/issues/4024)) ([df4ed38](https://github.com/5dlabs/cto/commit/df4ed38224884b3479d91857a65860b37f068dc0))
* **research:** add 1 new research entries ([#4026](https://github.com/5dlabs/cto/issues/4026)) ([05f539b](https://github.com/5dlabs/cto/commit/05f539b106cad1e14cbc7137e77d8a7dfdfe8d92))
* **research:** add 1 new research entries ([#4030](https://github.com/5dlabs/cto/issues/4030)) ([34bca85](https://github.com/5dlabs/cto/commit/34bca85b511fdbc3ef8cee3a6644fcdbd059924b))
* **research:** add 1 new research entries ([#4032](https://github.com/5dlabs/cto/issues/4032)) ([71e09b8](https://github.com/5dlabs/cto/commit/71e09b89d996a970628335a9ec77b58599091ecf))
* **research:** add 1 new research entries ([#4038](https://github.com/5dlabs/cto/issues/4038)) ([772b320](https://github.com/5dlabs/cto/commit/772b320a03cf3851d1d638f26c5f417ee4158ba7))
* **research:** add 1 new research entries ([#4039](https://github.com/5dlabs/cto/issues/4039)) ([5d71039](https://github.com/5dlabs/cto/commit/5d710395c90f627402c8549885a65f26f2cecae4))
* **research:** add 1 new research entries ([#4042](https://github.com/5dlabs/cto/issues/4042)) ([a0b5870](https://github.com/5dlabs/cto/commit/a0b5870fb7be171730e71cadc6d88a5bed59307d))
* **research:** add 1 new research entries ([#4045](https://github.com/5dlabs/cto/issues/4045)) ([9da1b90](https://github.com/5dlabs/cto/commit/9da1b900f9248f64c06d490a0ab57b2dceee95ae))
* **research:** add 1 new research entries ([#4049](https://github.com/5dlabs/cto/issues/4049)) ([b031741](https://github.com/5dlabs/cto/commit/b031741712936009b8506b77456268929511f97f))
* **research:** add 1 new research entries ([#4050](https://github.com/5dlabs/cto/issues/4050)) ([6505f6c](https://github.com/5dlabs/cto/commit/6505f6c75cef4fd94aee8351672184744a89f998))
* **research:** add 1 new research entries ([#4051](https://github.com/5dlabs/cto/issues/4051)) ([6433c09](https://github.com/5dlabs/cto/commit/6433c09873b8272cd552c863cc104aff45e2ad0a))
* **research:** add 2 new research entries ([#3990](https://github.com/5dlabs/cto/issues/3990)) ([334fdc1](https://github.com/5dlabs/cto/commit/334fdc1b5e261c1ddc59b82f06ec9a5020c1ad37))
* **research:** add 2 new research entries ([#3993](https://github.com/5dlabs/cto/issues/3993)) ([99f0c7d](https://github.com/5dlabs/cto/commit/99f0c7d0a7cc5fa256b8eb70c4cc0649b05edb1d))
* **research:** add 2 new research entries ([#3998](https://github.com/5dlabs/cto/issues/3998)) ([2aa49f2](https://github.com/5dlabs/cto/commit/2aa49f2ea5967029c60436dcbc4d870e0b7d667c))
* **research:** add 2 new research entries ([#3999](https://github.com/5dlabs/cto/issues/3999)) ([172bf51](https://github.com/5dlabs/cto/commit/172bf51dca060c35591ca6bb81fb5e07376d3a7b))
* **research:** add 2 new research entries ([#4000](https://github.com/5dlabs/cto/issues/4000)) ([5a8bd46](https://github.com/5dlabs/cto/commit/5a8bd46d9b63732656ca1788d10202663cb999c8))
* **research:** add 2 new research entries ([#4004](https://github.com/5dlabs/cto/issues/4004)) ([a7f8c0c](https://github.com/5dlabs/cto/commit/a7f8c0c7c960aca6149481bc35ab01a19f99e834))
* **research:** add 2 new research entries ([#4006](https://github.com/5dlabs/cto/issues/4006)) ([1b7bb9d](https://github.com/5dlabs/cto/commit/1b7bb9d9a58e782c8ebca69545b9e4505c10e8de))
* **research:** add 2 new research entries ([#4009](https://github.com/5dlabs/cto/issues/4009)) ([febbc3a](https://github.com/5dlabs/cto/commit/febbc3ad832f256c300dba06fbc98140aa67334b))
* **research:** add 2 new research entries ([#4012](https://github.com/5dlabs/cto/issues/4012)) ([7f36923](https://github.com/5dlabs/cto/commit/7f36923a03998d724fbb671d5fb4acd776ba60d2))
* **research:** add 2 new research entries ([#4013](https://github.com/5dlabs/cto/issues/4013)) ([9d94609](https://github.com/5dlabs/cto/commit/9d94609f4fc8890abc70fff54c059ee00843a776))
* **research:** add 2 new research entries ([#4035](https://github.com/5dlabs/cto/issues/4035)) ([bef3c4b](https://github.com/5dlabs/cto/commit/bef3c4b22646f57b8463aed195a86e4f14c7104b))
* **research:** add 2 new research entries ([#4037](https://github.com/5dlabs/cto/issues/4037)) ([cedbddc](https://github.com/5dlabs/cto/commit/cedbddcbd7432ab4b2781d37c1b39804f48a324c))
* **research:** add 2 new research entries ([#4040](https://github.com/5dlabs/cto/issues/4040)) ([d8ac601](https://github.com/5dlabs/cto/commit/d8ac6015d9fe433fb04eb9fd6357c86a7d85c233))
* **research:** add 2 new research entries ([#4041](https://github.com/5dlabs/cto/issues/4041)) ([af39647](https://github.com/5dlabs/cto/commit/af396476722124272266367a55c2abc2363ed2e3))
* **research:** add 2 new research entries ([#4046](https://github.com/5dlabs/cto/issues/4046)) ([02e18e5](https://github.com/5dlabs/cto/commit/02e18e5a6bb4bf56d7e2e49092123cbcf71e98e5))
* **research:** add 2 new research entries ([#4048](https://github.com/5dlabs/cto/issues/4048)) ([83e2648](https://github.com/5dlabs/cto/commit/83e2648158a9018421715c80289e572f6d34160a))
* **research:** add 3 new research entries ([#3975](https://github.com/5dlabs/cto/issues/3975)) ([717d57f](https://github.com/5dlabs/cto/commit/717d57f15d676eaea85e8693633acabd535571c8))
* **research:** add 3 new research entries ([#3983](https://github.com/5dlabs/cto/issues/3983)) ([89264f5](https://github.com/5dlabs/cto/commit/89264f533e1d04b133311e9393f1588195e930db))
* **research:** add 3 new research entries ([#3984](https://github.com/5dlabs/cto/issues/3984)) ([ba94a02](https://github.com/5dlabs/cto/commit/ba94a02c804bc195176cbb7f30f4dc714d9ab1b3))
* **research:** add 3 new research entries ([#3995](https://github.com/5dlabs/cto/issues/3995)) ([6868fb5](https://github.com/5dlabs/cto/commit/6868fb501a78e1c3c90bad17297b45db65eec9ea))
* **research:** add 3 new research entries ([#3997](https://github.com/5dlabs/cto/issues/3997)) ([b411610](https://github.com/5dlabs/cto/commit/b41161068b5cc1252a114d713dae3149bb3e3f91))
* **research:** add 3 new research entries ([#4001](https://github.com/5dlabs/cto/issues/4001)) ([46d6964](https://github.com/5dlabs/cto/commit/46d6964873df0318510cdf5904957f6afc3922c9))
* **research:** add 3 new research entries ([#4005](https://github.com/5dlabs/cto/issues/4005)) ([7ab5154](https://github.com/5dlabs/cto/commit/7ab5154565f5ce91d4d00f0b0a0fd7a7b413fbcc))
* **research:** add 3 new research entries ([#4021](https://github.com/5dlabs/cto/issues/4021)) ([d8bf0d9](https://github.com/5dlabs/cto/commit/d8bf0d9cc2b20e83a2ecdbe9cd823c0daa411d17))
* **research:** add 3 new research entries ([#4022](https://github.com/5dlabs/cto/issues/4022)) ([f555e6f](https://github.com/5dlabs/cto/commit/f555e6f0f03e93b8318a4960dc4e523f33ef94d2))
* **research:** add 3 new research entries ([#4027](https://github.com/5dlabs/cto/issues/4027)) ([1555042](https://github.com/5dlabs/cto/commit/15550420c99dc8fc3a96f0ea72f9a53f3670b053))
* **research:** add 3 new research entries ([#4036](https://github.com/5dlabs/cto/issues/4036)) ([7c7c543](https://github.com/5dlabs/cto/commit/7c7c54321001e3d9f9f2c142ee7a7d5a7fa81351))
* **research:** add 4 new research entries ([#3964](https://github.com/5dlabs/cto/issues/3964)) ([04415ff](https://github.com/5dlabs/cto/commit/04415ffb252a3383a056264b9dc5afbcb50ec980))
* **research:** add 4 new research entries ([#3967](https://github.com/5dlabs/cto/issues/3967)) ([91f78f4](https://github.com/5dlabs/cto/commit/91f78f4bceb12827364d52723ceec2d85a0766fa))
* **research:** add 4 new research entries ([#3973](https://github.com/5dlabs/cto/issues/3973)) ([653d35d](https://github.com/5dlabs/cto/commit/653d35da66cc89fd0f8488ceaa28a3c9eb463c0f))
* **research:** add 4 new research entries ([#3974](https://github.com/5dlabs/cto/issues/3974)) ([2444579](https://github.com/5dlabs/cto/commit/24445799560d846d2a480073f029474f95430b60))
* **research:** add 4 new research entries ([#3977](https://github.com/5dlabs/cto/issues/3977)) ([dfeb2f4](https://github.com/5dlabs/cto/commit/dfeb2f457e113c5e24c70a6aef1b5a6ad5883184))
* **research:** add 4 new research entries ([#3981](https://github.com/5dlabs/cto/issues/3981)) ([b9e4bca](https://github.com/5dlabs/cto/commit/b9e4bca362138bcb55d793d60767ca267a53da1e))
* **research:** add 4 new research entries ([#3985](https://github.com/5dlabs/cto/issues/3985)) ([f3f1fd5](https://github.com/5dlabs/cto/commit/f3f1fd5eba0a9cf9f50125f6ef03d6cabbd1195c))
* **research:** add 4 new research entries ([#3987](https://github.com/5dlabs/cto/issues/3987)) ([48956dd](https://github.com/5dlabs/cto/commit/48956dd438669bf9d8bf6da4d81e4c85da5fa659))
* **research:** add 4 new research entries ([#3991](https://github.com/5dlabs/cto/issues/3991)) ([c9b8e01](https://github.com/5dlabs/cto/commit/c9b8e0110b1be06fa6a837d5f3a47ed26920c8d8))
* **research:** add 4 new research entries ([#3992](https://github.com/5dlabs/cto/issues/3992)) ([43156f9](https://github.com/5dlabs/cto/commit/43156f9beb0122ef61b0199dc2b0d4040d072b05))
* **research:** add 4 new research entries ([#3994](https://github.com/5dlabs/cto/issues/3994)) ([141e1b4](https://github.com/5dlabs/cto/commit/141e1b4bd008bc73350bd89a696c9f48565a46a3))
* **research:** add 4 new research entries ([#4018](https://github.com/5dlabs/cto/issues/4018)) ([8d6e322](https://github.com/5dlabs/cto/commit/8d6e322da5045d7692e747fb18140b7ba7f1dd1b))
* **research:** add 4 new research entries ([#4029](https://github.com/5dlabs/cto/issues/4029)) ([ab7b5ae](https://github.com/5dlabs/cto/commit/ab7b5aebe66f38f1c651b399c6943c31f692b208))
* **research:** add 4 new research entries ([#4033](https://github.com/5dlabs/cto/issues/4033)) ([0f08e6d](https://github.com/5dlabs/cto/commit/0f08e6de033d5c7668c9b65cd06f8a3408680bdd))
* **research:** add 4 new research entries ([#4047](https://github.com/5dlabs/cto/issues/4047)) ([575e02b](https://github.com/5dlabs/cto/commit/575e02be37633ad47c416496bb2c054cafb8af20))
* **research:** add 5 new research entries ([#3963](https://github.com/5dlabs/cto/issues/3963)) ([78ffca2](https://github.com/5dlabs/cto/commit/78ffca24ac6a88db26d7225fb97eed04055431e2))
* **research:** add 5 new research entries ([#3966](https://github.com/5dlabs/cto/issues/3966)) ([1e08dae](https://github.com/5dlabs/cto/commit/1e08daec3e5cce5c38514bb7b19d9758fd332ca6))
* **research:** add 5 new research entries ([#3968](https://github.com/5dlabs/cto/issues/3968)) ([981c94e](https://github.com/5dlabs/cto/commit/981c94e828c9f4a97c23408c3b784a87874891d0))
* **research:** add 5 new research entries ([#3976](https://github.com/5dlabs/cto/issues/3976)) ([984faf7](https://github.com/5dlabs/cto/commit/984faf788d265f18ec2caad29f8d9415d0b335f6))
* **research:** add 5 new research entries ([#3986](https://github.com/5dlabs/cto/issues/3986)) ([4556b7b](https://github.com/5dlabs/cto/commit/4556b7bad88dbb368e12beb71538d8afc7a8286b))
* **research:** add 5 new research entries ([#4020](https://github.com/5dlabs/cto/issues/4020)) ([1ecf48d](https://github.com/5dlabs/cto/commit/1ecf48da6f6264003da68664cf6cc460bbcd7ad4))
* **research:** add 5 new research entries ([#4034](https://github.com/5dlabs/cto/issues/4034)) ([e7c9dc0](https://github.com/5dlabs/cto/commit/e7c9dc0f98bda32ba7b819ff5c4b91a6b0c04ee2))
* **research:** add 6 new research entries ([#3962](https://github.com/5dlabs/cto/issues/3962)) ([1eb2374](https://github.com/5dlabs/cto/commit/1eb237400887933cb7219a3c56abd4c27f4ce1e5))
* **research:** add 6 new research entries ([#3969](https://github.com/5dlabs/cto/issues/3969)) ([b58c259](https://github.com/5dlabs/cto/commit/b58c259fb24ef9c59419fdf130f895f9bc174aec))
* **research:** add 6 new research entries ([#3970](https://github.com/5dlabs/cto/issues/3970)) ([d7a6827](https://github.com/5dlabs/cto/commit/d7a6827c21399f9026382c1d9b34ee769f7fa1b0))
* **research:** add 6 new research entries ([#3971](https://github.com/5dlabs/cto/issues/3971)) ([ff16ddf](https://github.com/5dlabs/cto/commit/ff16ddf52dc9db64c38fe893ae6f06e1f237c90f))
* **research:** add 6 new research entries ([#3978](https://github.com/5dlabs/cto/issues/3978)) ([6154eb1](https://github.com/5dlabs/cto/commit/6154eb1b0c9c6d547577595b287f74be2cfb5fb3))
* **research:** add 6 new research entries ([#3980](https://github.com/5dlabs/cto/issues/3980)) ([2a6cc9e](https://github.com/5dlabs/cto/commit/2a6cc9ec0516467e8cadd14d6fa89e9ef83ff2ae))
* **research:** add 6 new research entries ([#4019](https://github.com/5dlabs/cto/issues/4019)) ([20870dc](https://github.com/5dlabs/cto/commit/20870dcabc9027a8c9267d313d169b5841347914))
* **research:** add 6 new research entries ([#4028](https://github.com/5dlabs/cto/issues/4028)) ([2319cde](https://github.com/5dlabs/cto/commit/2319cdea60858519e2653ae0d1387ffee7a913a8))
* **research:** add 6 new research entries ([#4031](https://github.com/5dlabs/cto/issues/4031)) ([14b1c8b](https://github.com/5dlabs/cto/commit/14b1c8bea9cf2a01008f46760f826fe6634adf59))
* **research:** add 6 new research entries ([#4043](https://github.com/5dlabs/cto/issues/4043)) ([7fde605](https://github.com/5dlabs/cto/commit/7fde6050a087797dbfa1387426d18bfeef5d5253))
* **research:** add 6 new research entries ([#4044](https://github.com/5dlabs/cto/issues/4044)) ([9633704](https://github.com/5dlabs/cto/commit/9633704b387c1f0858d79d1af9f8a86637752763))
* **research:** add 7 new research entries ([#3982](https://github.com/5dlabs/cto/issues/3982)) ([9cbbaf3](https://github.com/5dlabs/cto/commit/9cbbaf367d99cb80b2cdfa81a155436d8c9504b4))
* **research:** add 7 new research entries ([#3989](https://github.com/5dlabs/cto/issues/3989)) ([b665997](https://github.com/5dlabs/cto/commit/b6659979016c34f0ebbb43cc3b8a20de0cbbf11c))
* **research:** add 7 new research entries ([#4017](https://github.com/5dlabs/cto/issues/4017)) ([c06d93e](https://github.com/5dlabs/cto/commit/c06d93e04f9d8bacd934a079f60fbaa4b92cad02))
* **research:** add 8 new research entries ([#3979](https://github.com/5dlabs/cto/issues/3979)) ([36387eb](https://github.com/5dlabs/cto/commit/36387eb9bf8a0390e955d81b36264a158864b355))
* **research:** add 8 new research entries ([#3988](https://github.com/5dlabs/cto/issues/3988)) ([b56b17c](https://github.com/5dlabs/cto/commit/b56b17cf42bb8bb26b825cdad8fc3a4c59554d13))

## [0.2.27](https://github.com/5dlabs/cto/compare/v0.2.26...v0.2.27) (2026-01-24)


### 📚 Documentation

* **research:** add 2 new research entries ([#3952](https://github.com/5dlabs/cto/issues/3952)) ([527fdc2](https://github.com/5dlabs/cto/commit/527fdc27b9866c86088e2b4c3fabc74c19fb70e7))
* **research:** add 2 new research entries ([#3956](https://github.com/5dlabs/cto/issues/3956)) ([1f47ac3](https://github.com/5dlabs/cto/commit/1f47ac3dfd13f83470fc30646c4a109f9fc2ebc3))
* **research:** add 4 new research entries ([#3959](https://github.com/5dlabs/cto/issues/3959)) ([4683cee](https://github.com/5dlabs/cto/commit/4683cee4e7742c1c3407a45d4fe9cffec64bca42))
* **research:** add 5 new research entries ([#3954](https://github.com/5dlabs/cto/issues/3954)) ([77c5be5](https://github.com/5dlabs/cto/commit/77c5be53d821c78d1844e6f82129466c0ad8af77))
* **research:** add 5 new research entries ([#3955](https://github.com/5dlabs/cto/issues/3955)) ([328f35f](https://github.com/5dlabs/cto/commit/328f35f4858cdf2cd0a0a90b1803c8a477767302))
* **research:** add 5 new research entries ([#3960](https://github.com/5dlabs/cto/issues/3960)) ([ef9c9e8](https://github.com/5dlabs/cto/commit/ef9c9e89c9da1abd02d3eac78377127a3fdf04f3))
* **research:** add 6 new research entries ([#3957](https://github.com/5dlabs/cto/issues/3957)) ([1347ffc](https://github.com/5dlabs/cto/commit/1347ffc5deb8d39eee758c5bc8c319658d5b01fa))
* **research:** add 6 new research entries ([#3958](https://github.com/5dlabs/cto/issues/3958)) ([bfba91f](https://github.com/5dlabs/cto/commit/bfba91f4714e4705f9621d006a6ffed5c176f6fb))
* **research:** add 7 new research entries ([#3953](https://github.com/5dlabs/cto/issues/3953)) ([bdd8f2a](https://github.com/5dlabs/cto/commit/bdd8f2adbedd951c8691abbf020c687a6a0b3bf1))

## [0.2.26](https://github.com/5dlabs/cto/compare/v0.2.25...v0.2.26) (2026-01-24)


### ✨ Features

* **marketing:** update landing page messaging based on competitor analysis ([#3940](https://github.com/5dlabs/cto/issues/3940)) ([0f6ee00](https://github.com/5dlabs/cto/commit/0f6ee0022158cf3023abbc50c8d443626409b16e))
* **research:** add platform-aware analysis and multi-tool enrichment ([#3933](https://github.com/5dlabs/cto/issues/3933)) ([6f0d272](https://github.com/5dlabs/cto/commit/6f0d272137444c3c28ea7345eadeed685008d2bc))


### 🐛 Bug Fixes

* **research:** update model to valid Claude Sonnet 4.5 ([#3932](https://github.com/5dlabs/cto/issues/3932)) ([3350c09](https://github.com/5dlabs/cto/commit/3350c09cc4e5633427b163ec37c9ce5e9504de7a))


### 📚 Documentation

* **research:** add 10 new research entries ([#3923](https://github.com/5dlabs/cto/issues/3923)) ([fc4c4db](https://github.com/5dlabs/cto/commit/fc4c4dba7afffac88071b91101f356ff69b203b6))
* **research:** add 3 new research entries ([#3930](https://github.com/5dlabs/cto/issues/3930)) ([5dfe744](https://github.com/5dlabs/cto/commit/5dfe744b5b70e0534ac164be1655df2bfaaa47ff))
* **research:** add 3 new research entries ([#3934](https://github.com/5dlabs/cto/issues/3934)) ([ac657c2](https://github.com/5dlabs/cto/commit/ac657c20be51dd7a70ba16c225f7eebf43aed2df))
* **research:** add 3 new research entries ([#3939](https://github.com/5dlabs/cto/issues/3939)) ([9e96f06](https://github.com/5dlabs/cto/commit/9e96f069f94084b6ef24f86806f3a3b1117139c2))
* **research:** add 4 new research entries ([#3928](https://github.com/5dlabs/cto/issues/3928)) ([c89abbf](https://github.com/5dlabs/cto/commit/c89abbf73a0ed16b3e80425f128c5ad5e3686ca2))
* **research:** add 4 new research entries ([#3941](https://github.com/5dlabs/cto/issues/3941)) ([e628433](https://github.com/5dlabs/cto/commit/e628433ec77bcf706c927f271729e3219a78914c))
* **research:** add 4 new research entries ([#3944](https://github.com/5dlabs/cto/issues/3944)) ([2deff56](https://github.com/5dlabs/cto/commit/2deff566f65f24ddf07827d58e8699f8ae1d818b))
* **research:** add 5 new research entries ([#3927](https://github.com/5dlabs/cto/issues/3927)) ([8639290](https://github.com/5dlabs/cto/commit/863929094fdd2d433fd0e427db35926ff668b811))
* **research:** add 5 new research entries ([#3931](https://github.com/5dlabs/cto/issues/3931)) ([4e2d52d](https://github.com/5dlabs/cto/commit/4e2d52d621b27cc692b59f8d707385c2dcdd5fe5))
* **research:** add 5 new research entries ([#3936](https://github.com/5dlabs/cto/issues/3936)) ([cf3a96b](https://github.com/5dlabs/cto/commit/cf3a96bdd11410300e47f275cc7852fbad08d89e))
* **research:** add 5 new research entries ([#3937](https://github.com/5dlabs/cto/issues/3937)) ([f55f2c7](https://github.com/5dlabs/cto/commit/f55f2c72d4448126ccad7065eb88ffebac34b34c))
* **research:** add 5 new research entries ([#3938](https://github.com/5dlabs/cto/issues/3938)) ([c96893d](https://github.com/5dlabs/cto/commit/c96893d76802917f8aae5ca396c3eeaa268335fe))
* **research:** add 5 new research entries ([#3942](https://github.com/5dlabs/cto/issues/3942)) ([a961a0b](https://github.com/5dlabs/cto/commit/a961a0beadd71b7e1a16706fa6de6de863d8b618))
* **research:** add 5 new research entries ([#3946](https://github.com/5dlabs/cto/issues/3946)) ([48feedb](https://github.com/5dlabs/cto/commit/48feedb2b8e5ff77730689e03ed51159e9cdc0f4))
* **research:** add 6 new research entries ([#3924](https://github.com/5dlabs/cto/issues/3924)) ([46f4a8e](https://github.com/5dlabs/cto/commit/46f4a8e8dfb521b999735013701948d9f02e8fed))
* **research:** add 6 new research entries ([#3926](https://github.com/5dlabs/cto/issues/3926)) ([0a83881](https://github.com/5dlabs/cto/commit/0a83881163db3bad2a90748e65c72fe774187de2))
* **research:** add 7 new research entries ([#3929](https://github.com/5dlabs/cto/issues/3929)) ([4e8f120](https://github.com/5dlabs/cto/commit/4e8f12052befc7c2b753903a70045a7d5fcc7126))
* **research:** add 8 new research entries ([#3935](https://github.com/5dlabs/cto/issues/3935)) ([4e87426](https://github.com/5dlabs/cto/commit/4e87426eed754e354b26626ef6adaa484f860d5a))
* **research:** add 8 new research entries ([#3943](https://github.com/5dlabs/cto/issues/3943)) ([2a13f1c](https://github.com/5dlabs/cto/commit/2a13f1c011e3640cd465ac1861463ecbc0169ced))
* **research:** add 8 new research entries ([#3945](https://github.com/5dlabs/cto/issues/3945)) ([cb70356](https://github.com/5dlabs/cto/commit/cb70356483c13d35f9b1e05cb37645f742c75cf6))

## [0.2.25](https://github.com/5dlabs/cto/compare/v0.2.24...v0.2.25) (2026-01-24)


### 🐛 Bug Fixes

* **tools:** add ${VAR} env substitution and readiness probes ([#3921](https://github.com/5dlabs/cto/issues/3921)) ([f0ffa79](https://github.com/5dlabs/cto/commit/f0ffa7971bf0ff465adcc5d46570700cbce9d925))

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
