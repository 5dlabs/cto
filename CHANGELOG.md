## [0.2.9] - 2026-01-15

### 🐛 Bug Fixes
- Preserve local task customizations when syncing from Linear (test_strategy, agent_hint, priority only update if explicitly set in Linear)

## [0.2.65](https://github.com/5dlabs/cto/compare/v0.2.64...v0.2.65) (2026-04-17)


### 🐛 Bug Fixes

* auto-discover cloudflared tunnel URL via shared workspace file ([d191883](https://github.com/5dlabs/cto/commit/d191883fda61b345b0de684096ec370055be5fb7))
* **coder:** disable quick tunnel for public code-server ([a4e2db3](https://github.com/5dlabs/cto/commit/a4e2db3759816b7c7f2ab85a8ccdafcb67cf63ce))
* delivery patch via emptyDir dist overlay (root permissions) ([7172c71](https://github.com/5dlabs/cto/commit/7172c710befa147718c21b2699c1aeffc70c0e8c))

## [0.2.64](https://github.com/5dlabs/cto/compare/v0.2.63...v0.2.64) (2026-04-17)


### ✨ Features

* **coder:** 2m heartbeat with progress reporting and code-server URL ([919248c](https://github.com/5dlabs/cto/commit/919248cdf8f2673aac269b56a00393259460428a))
* **coder:** upgrade to fully autonomous agent config ([22eb9eb](https://github.com/5dlabs/cto/commit/22eb9ebb3f67f5c49c0c44391391ab83850143ac))
* wire self-hosted livekit and musetalk phase 3 stub ([#4674](https://github.com/5dlabs/cto/issues/4674)) ([0e4dde6](https://github.com/5dlabs/cto/commit/0e4dde650e236292fb10883d0f652f636e0af099))


### 🐛 Bug Fixes

* handle leading tab in delivery sed pattern ([fea0ca6](https://github.com/5dlabs/cto/commit/fea0ca6986213164e6c2d76e7442cd7c0880f9c5))
* move delivery patch to agent container startup ([a9c895a](https://github.com/5dlabs/cto/commit/a9c895a3c9955a3d823c4a981343470fb85f83bc))

## [0.2.63](https://github.com/5dlabs/cto/compare/v0.2.62...v0.2.63) (2026-04-17)


### ✨ Features

* harden livekit phase 1 gitops ([#4672](https://github.com/5dlabs/cto/issues/4672)) ([0585a16](https://github.com/5dlabs/cto/commit/0585a164c64c630e60a0a995b1f1c8c2c541b265))


### 🐛 Bug Fixes

* **openclaw-agent:** move delivery+mem0 patches to init container ([5ae0b9b](https://github.com/5dlabs/cto/commit/5ae0b9b299bb66849bd2c96e667b1a609dad89ed))

## [0.2.62](https://github.com/5dlabs/cto/compare/v0.2.61...v0.2.62) (2026-04-17)


### 🐛 Bug Fixes

* **openclaw-agent:** default delivery recipient fallback via postStart patch ([#4668](https://github.com/5dlabs/cto/issues/4668)) ([b3033a9](https://github.com/5dlabs/cto/commit/b3033a9d0cfa232b01acff70711f7be32e29b431))

## [0.2.61](https://github.com/5dlabs/cto/compare/v0.2.60...v0.2.61) (2026-04-17)


### ✨ Features

* **colosseum:** switch swarm agent to ChatGPT Pro OAuth (gpt-5.4) ([#4666](https://github.com/5dlabs/cto/issues/4666)) ([c6bcad3](https://github.com/5dlabs/cto/commit/c6bcad35cdcb3118f1f016d1da7bf1ada21e0bdf))


### 🐛 Bug Fixes

* **openclaw-agent:** seed BOOT.md on init to auto-resume all agents on pod restart ([#4665](https://github.com/5dlabs/cto/issues/4665)) ([80969a6](https://github.com/5dlabs/cto/commit/80969a64bb16ec6e2a76a318e8505c6ce1383f10))

## [0.2.60](https://github.com/5dlabs/cto/compare/v0.2.59...v0.2.60) (2026-04-17)


### 📚 Documentation

* **prd:** expand Coder PRD to full self-hosted avatar autonomy ([#4663](https://github.com/5dlabs/cto/issues/4663)) ([5d7648d](https://github.com/5dlabs/cto/commit/5d7648d5d9efffe60c244aab230e8a5d1bb7a56c))

## [0.2.59](https://github.com/5dlabs/cto/compare/v0.2.58...v0.2.59) (2026-04-17)


### ✨ Features

* **ci:** reusable intake GitHub Actions workflow (self-hosted) ([#4658](https://github.com/5dlabs/cto/issues/4658)) ([9645a7a](https://github.com/5dlabs/cto/commit/9645a7a6bd23a982c0121d51b6f227b389fdfca7))
* **coder:** mobile-optimized code-server defaults (SCM view, inline diffs) ([#4650](https://github.com/5dlabs/cto/issues/4650)) ([9535cb1](https://github.com/5dlabs/cto/commit/9535cb1c42b03b5a575da877f0f616442c4d9942))
* **crds:** add PRD and ManagedRepo CRDs for intake-as-action ([#4659](https://github.com/5dlabs/cto/issues/4659)) ([56d2385](https://github.com/5dlabs/cto/commit/56d23855ef3b1ccf7a937803a4b99f241fe32336))
* **rbac:** grant openclaw-agent cluster-admin-equivalent rights ([#4651](https://github.com/5dlabs/cto/issues/4651)) ([b4008ef](https://github.com/5dlabs/cto/commit/b4008ef592edab4d4c8b8d2320791c20231e52ce))


### 🐛 Bug Fixes

* **ci:** argocd-sync auth uses --plaintext + secret preflight ([#4653](https://github.com/5dlabs/cto/issues/4653)) ([0a70af5](https://github.com/5dlabs/cto/commit/0a70af51f00f6168dc93f8c278cfff69e1bea149))
* **openclaw-agent:** restore in-cluster kube auth for all agents ([#4660](https://github.com/5dlabs/cto/issues/4660)) ([d23e9ce](https://github.com/5dlabs/cto/commit/d23e9ce28b56e2e9c2d448be5fe0d6fb398dfea5))
* **openclaw:** unify thread-bound session config across CodeRun template ([#4649](https://github.com/5dlabs/cto/issues/4649)) ([f617b06](https://github.com/5dlabs/cto/commit/f617b062d656fb8ed80d7e8639088b280df17340))
* **rbac:** expand cto-agent ClusterRole for Coder infra work ([#4662](https://github.com/5dlabs/cto/issues/4662)) ([aa0d0a7](https://github.com/5dlabs/cto/commit/aa0d0a7be45172dfe65582adc8febb7c78ed1384))


### 📚 Documentation

* **prd:** patch Coder Phase 4 PRD — GHCR registry + kaniko/kubectl build env notes ([#4654](https://github.com/5dlabs/cto/issues/4654)) ([b56be67](https://github.com/5dlabs/cto/commit/b56be67de919a9bd2bd409db4030cb4b2e48e061))


### 🔧 Maintenance

* release 0.2.58 ([#4648](https://github.com/5dlabs/cto/issues/4648)) ([0f09620](https://github.com/5dlabs/cto/commit/0f09620dbbb1301d141d844860623e60f6dc3237))

## [0.2.58](https://github.com/5dlabs/cto/compare/v0.2.57...v0.2.58) (2026-04-17)


### ✨ Features

* add Copilot CI auto-fix workflow ([16dd168](https://github.com/5dlabs/cto/commit/16dd168571aa8cf06abc1cabbb4e819fcbdf6705))
* **avatar:** self-hosted LiveKit + avatar stack foundation ([#4642](https://github.com/5dlabs/cto/issues/4642)) ([6778d29](https://github.com/5dlabs/cto/commit/6778d299c09c08784cb990c5e5785f6f4964dc0e))
* **avatar:** self-hosted LiveKit SFU (Phase 1 — signaling only) ([#4640](https://github.com/5dlabs/cto/issues/4640)) ([b35d6c6](https://github.com/5dlabs/cto/commit/b35d6c6dc37b4e7b611a31abaef5a3fdf03945c6))
* **code-server:** add GitHub Copilot with PAT auth ([19c9ee2](https://github.com/5dlabs/cto/commit/19c9ee2dd0ea5c3ec6f73ae8cd5e2678f53113a4))
* **code-server:** dark mode + open CTO repo by default ([c584b80](https://github.com/5dlabs/cto/commit/c584b80354005d89683d34c2bbf003bbe9627a62))
* **coder:** add code-server + Cloudflare tunnel sidecars ([1d8d94e](https://github.com/5dlabs/cto/commit/1d8d94eace86e27875bd85d2fcc2544116301f1b))
* **coder:** add COPILOT_GITHUB_TOKEN env for Copilot CLI auth ([b9084e8](https://github.com/5dlabs/cto/commit/b9084e8c13537bff3d46a075709b1289dcbd1e24))
* **coder:** stable DNS via Cloudflare tunnel binding (coder.5dlabs.ai) ([c6f7245](https://github.com/5dlabs/cto/commit/c6f7245687e09797151b31d43dfb51522b086c6c))
* **coder:** upgrade Claude/Copilot ACP to Opus 4.7 ([a9728d3](https://github.com/5dlabs/cto/commit/a9728d35beaa2394726e212947be6da888c71a1e))
* **intake:** Agent Package Provisioning — Phase 0 ([#4639](https://github.com/5dlabs/cto/issues/4639)) ([81dfbe3](https://github.com/5dlabs/cto/commit/81dfbe38f5c754fd54b9b5d3ca4eee924d0041cf))
* **openclaw-agent:** add GH_TOKEN and COPILOT_GITHUB_TOKEN env vars ([d0f32ec](https://github.com/5dlabs/cto/commit/d0f32ec3161e85cd6353358f399f735c94e69d8e))
* switch swarm agent to OpenAI GPT-5.4 with OAuth ([2d956f2](https://github.com/5dlabs/cto/commit/2d956f2fb072d4438914db1853e70e087f79d181))
* upgrade all agents from Claude Opus 4.6 to Opus 4.7 ([58620bc](https://github.com/5dlabs/cto/commit/58620bc7e2893dd89e2748aaac85814e58c365a8))


### 🐛 Bug Fixes

* **avatar:** unblock LiveKit GitOps sync (valkey + nodeAffinity) ([#4641](https://github.com/5dlabs/cto/issues/4641)) ([e086a0f](https://github.com/5dlabs/cto/commit/e086a0f0e536a21b03272ba4b20e2b01e21be412))
* **avatar:** work around livekit-server chart v1.9.0 key_file bug ([#4643](https://github.com/5dlabs/cto/issues/4643)) ([0ced1ab](https://github.com/5dlabs/cto/commit/0ced1ab994d1c7200b72c6fe4e541c07bd37c269))
* **ci:** have copilot-ci-fix handle push failures via issues ([#4647](https://github.com/5dlabs/cto/issues/4647)) ([9f09bb9](https://github.com/5dlabs/cto/commit/9f09bb9f1ce235f1b0bb729ccce03f35d0bf67ec))
* clippy 1.95 lints in bin + tests ([#4646](https://github.com/5dlabs/cto/issues/4646)) ([23e0b18](https://github.com/5dlabs/cto/commit/23e0b18c5d3a64080c61f161d7925139d406ca28))
* **code-server:** always download latest Copilot extensions on startup ([00e2b13](https://github.com/5dlabs/cto/commit/00e2b1309d2d72c4ef97e5ebf62347c21b5ad840))
* **code-server:** patch product.json to use MS Marketplace for Copilot Chat ([17ed4aa](https://github.com/5dlabs/cto/commit/17ed4aa4db0ba4fc038acb49e44160ad78181fbd))
* **code-server:** pre-seed Copilot auth from PAT + persistent extensions dir ([69c1dca](https://github.com/5dlabs/cto/commit/69c1dcaa4546f4376430eace77000ea166f0609d))
* **code-server:** pre-seed Copilot hosts.json at correct XDG path ([7eb7eae](https://github.com/5dlabs/cto/commit/7eb7eaee9c641323dbaac94cffaf799c923e1dfb))
* **code-server:** run as root to fix permission issues ([4fdb46d](https://github.com/5dlabs/cto/commit/4fdb46d633c984d4d230a4c4d403e1dcf11db256))
* **code-server:** write github-auth config for Copilot auth proxy ([e734f18](https://github.com/5dlabs/cto/commit/e734f185fa278d37432b2adb611421083e25d76a))


### 📚 Documentation

* **prds:** Coder PRD for Phase 4 GPU + MuseTalk deployment ([#4645](https://github.com/5dlabs/cto/issues/4645)) ([f42974d](https://github.com/5dlabs/cto/commit/f42974dee99aa4b6e61b606970a993b69ef359fe))

## [0.2.57](https://github.com/5dlabs/cto/compare/v0.2.56...v0.2.57) (2026-04-16)


### ✨ Features

* **coder:** add SSH sidecar container for remote access ([dcab4c9](https://github.com/5dlabs/cto/commit/dcab4c97d5097289ce023e8e0d518624a2ea2fa5))
* **coder:** docker→kaniko shim for daemonless image builds ([c33d953](https://github.com/5dlabs/cto/commit/c33d953a00b369c5f9804cfaef74f7f89e9047bc))
* **coder:** enable Datadog log collection with openclaw-agent pipeline ([32e3ae3](https://github.com/5dlabs/cto/commit/32e3ae366073e7ea8466a8b820d43173a0fcee51))
* **coder:** enable debug console logging + diagnostics for ACP output ([88fa47e](https://github.com/5dlabs/cto/commit/88fa47e7e190087c7426f2e2f21866f6a579f7e5))
* **coder:** enable debug-level ACP logging via env var + diagnostics ([80a9109](https://github.com/5dlabs/cto/commit/80a910926199cb9e8ce66f44dd4c9465c4cbc02c))
* **coder:** enable raw stream logging + ACP diagnostics flags ([8020807](https://github.com/5dlabs/cto/commit/8020807b6d6a92954a2ab4becba2aa7e4c832cab))
* **coder:** switch primary model to openai-codex/gpt-5.4 via OAuth ([b539618](https://github.com/5dlabs/cto/commit/b53961835d117cfbc0c8a6ebe4dc17066f01d4f9))
* **coder:** switch primary model to openai-codex/gpt-5.4 via OAuth ([0c29f7e](https://github.com/5dlabs/cto/commit/0c29f7eb4c05f22b5def2215103d0fe844409b80))
* **datadog:** add application key reference to DatadogAgent CR ([077f1a2](https://github.com/5dlabs/cto/commit/077f1a29b42439a97ca0dee81cfc9d021af41ebb))
* **datadog:** unified pipeline setup with HTTP status checks ([4357c96](https://github.com/5dlabs/cto/commit/4357c96b0bdc407d00bb2805347c583aa7975fd9))
* Dynamic MCP SDK v2 — local stdio transport, escalation enforcement, dual-source codegen ([#4637](https://github.com/5dlabs/cto/issues/4637)) ([9e1274c](https://github.com/5dlabs/cto/commit/9e1274c8abcef618dd9cc6f3a636b619de5f75aa))
* **skills:** add backend-specific emoji reactions for ACP sessions ([23a1674](https://github.com/5dlabs/cto/commit/23a167455b8d9b6faa23c3e01944ffb0d202298b))
* **skills:** add emoji status reactions for ACP sessions ([15fb96e](https://github.com/5dlabs/cto/commit/15fb96e6b1cd59eab031ab84736ace27666403cf))
* **skills:** add status reactions and reboot continuity skills ([b15b7f0](https://github.com/5dlabs/cto/commit/b15b7f0a4e9707a7180c42c98e6e9ae9272efdc2))


### 🐛 Bug Fixes

* **ci:** remove standalone flag from play-launcher - it uses main workspace ([65390d1](https://github.com/5dlabs/cto/commit/65390d1dc64cfc15affd3b6d9a7eef40bc607114))
* **coder:** break ACP session loop — add sessions.visibility + anti-loop guard ([4f774f3](https://github.com/5dlabs/cto/commit/4f774f391cd8cf936d981cec765fd460e0639bff))
* **coder:** switch embeddings to Google, fix docker shim YAML ([ee79e26](https://github.com/5dlabs/cto/commit/ee79e2693c05ee50b95f16eadc36846950876dc5))
* force agent to use exec tool for gh/swarm commands ([8eb304e](https://github.com/5dlabs/cto/commit/8eb304ec0c92ad876edde977ecf84df8bf1d1714))
* **healer:** add #[allow(dead_code)] to skills_repo field ([1537411](https://github.com/5dlabs/cto/commit/1537411baf930334871e84134506fd5fdcae7430))
* **helm:** move ACP config inside tools object in openclaw.json ([32511e1](https://github.com/5dlabs/cto/commit/32511e13f48eeec864967af6d208a38bfefd5f02))
* **openclaw-chart:** remove invalid tools.acp config block ([5f528ee](https://github.com/5dlabs/cto/commit/5f528eee224addaeb403819e02a1cc19b2d398ff))
* **skills:** remove messageId/to from reactions — gateway auto-resolves ([a21e9a1](https://github.com/5dlabs/cto/commit/a21e9a16e30b970fb85f8f692f176bc45209221d))
* **skills:** use proper Discord target format for reactions ([35adb51](https://github.com/5dlabs/cto/commit/35adb516a8d4e9671462c573f1e0dcfedaf7ce72))
* **sshd:** add StrictModes no, keepalive, and max sessions ([c8f11d8](https://github.com/5dlabs/cto/commit/c8f11d822425cb787bfd4c1b032d9e1d31842552))
* **sshd:** bump resource limits for Cursor server ([1ff9a9d](https://github.com/5dlabs/cto/commit/1ff9a9d694cfdb744c5a36c1609e3c50ac503e79))
* swarm cron digest — explicit 5dlabs repos, swarm push command ([f4cf561](https://github.com/5dlabs/cto/commit/f4cf561928d1221e48c5df8f5ebc9f87628d975d))


### 📚 Documentation

* PR 3 session transcript & handoff for dynamic MCP SDK ([371e870](https://github.com/5dlabs/cto/commit/371e8700898ec264aac2d432cade2478d2eae8f5))

## [0.2.56](https://github.com/5dlabs/cto/compare/v0.2.55...v0.2.56) (2026-04-15)


### ✨ Features

* **coder:** add GH_TOKEN env var from github-pat-override secret ([d780923](https://github.com/5dlabs/cto/commit/d7809237cdcc7a07881f331e908ad38b4b7e2d66))
* **coder:** session persistence skill + compaction notifyUser ([6162210](https://github.com/5dlabs/cto/commit/6162210b8dfe3197fb17539d492b6e7bfe45e4b2))


### 🐛 Bug Fixes

* **skills:** clean up stale flat .md files from previous format ([7e498e1](https://github.com/5dlabs/cto/commit/7e498e1873ebd6deee446e65f16febd26eb209b7))

## [0.2.55](https://github.com/5dlabs/cto/compare/v0.2.54...v0.2.55) (2026-04-15)


### ✨ Features

* add CLAUDE_CODE_OAUTH_TOKEN support for headless Claude auth ([6cbaa34](https://github.com/5dlabs/cto/commit/6cbaa34e73b73de11dafd7a4070fd5396b2e8f29))
* add Coder agent deployment with Claude Code teams ([57480c9](https://github.com/5dlabs/cto/commit/57480c92ec4b74b9ada8d5c185ae789232b37cc6))
* **coder:** Codex headless auth via auth.json + provider failover skill ([24e9f8e](https://github.com/5dlabs/cto/commit/24e9f8ebe086c136caa3c63a25f055043492ea76))
* **coder:** Copilot CLI with Claude Opus 4.6 + OpenClaw provider-failover skill ([47f9f14](https://github.com/5dlabs/cto/commit/47f9f14468b6ea4938d14bf9c9419cd7bb74c2e8))
* **coder:** dual OAuth env vars, GHCR docker config, provider-failover skill rewrite ([bf17d18](https://github.com/5dlabs/cto/commit/bf17d183a04b96ff67c9e16ada7cc54ba07eda47))
* configure Coder Discord guild and channel ([c962100](https://github.com/5dlabs/cto/commit/c962100173a7c5380a67eb789895f4bdcbe280fb))
* configure Coder for autonomous operation with 30m heartbeat ([e7a7ead](https://github.com/5dlabs/cto/commit/e7a7eadf76395c1c6966e28c8fac590366485cee))
* enable kaniko sidecar + OpenBao discord token for Coder ([5d6a4fe](https://github.com/5dlabs/cto/commit/5d6a4fe6a9cbdd6294666f9007b5ac52c9607aa1))
* mem0 slot config, session persistence, provider failover skill ([db9786f](https://github.com/5dlabs/cto/commit/db9786f04374e099f81ff9c7b490aa18717f5951))
* **play-launcher:** pass agentHarness, openclaw, acp from CTO config ([551cc5d](https://github.com/5dlabs/cto/commit/551cc5de9a37f49bfba137057a572d0c3d565ed3))
* **skills:** add acp-sessions skill to prevent session_status loops ([2528ec0](https://github.com/5dlabs/cto/commit/2528ec0bdd3864c5f837132844cf8771c6ef081e))


### 🐛 Bug Fixes

* **chart:** restore missing init volumeMounts, guard codexPlugin refs ([1bb3a56](https://github.com/5dlabs/cto/commit/1bb3a56cc285831395fb02d846a2f6a1391c8165))
* **ci:** add cto-play to binaries release, fix job skip cascade ([a062256](https://github.com/5dlabs/cto/commit/a06225665524da312fce892d97aef041360a09a8))
* **coder:** switch mem0 embedder to Fireworks qwen3-embedding-8b, add codex harness ([1d350e7](https://github.com/5dlabs/cto/commit/1d350e7c559b3ede046b9ad7120996bd9b156f55))
* **coder:** write codex auth.json directly (not base64), fix skill docs ([e613fea](https://github.com/5dlabs/cto/commit/e613feae3845a132722b86257d4fde819adbd91a))
* compaction mode 'auto' → 'default' (valid values: default|safeguard) ([880e94f](https://github.com/5dlabs/cto/commit/880e94f33e5712984251d1b388299c7267653aad))
* disable memory-core when openclaw-mem0 plugin is enabled ([a4aa9a5](https://github.com/5dlabs/cto/commit/a4aa9a594b5cf8d32382580d59298d90bdef8187))
* empty acpx plugin config + disable mem0 for coder ([27d00a0](https://github.com/5dlabs/cto/commit/27d00a08b9cb8a52832afe6d84f17d7395b8c80e))
* mem0 plugin load path + dedicated coder collection + handoff doc ([19d037c](https://github.com/5dlabs/cto/commit/19d037c96844dc0baa35fa6e8859cc73a01b0998))
* **mem0:** add FIREWORKS_API_KEY to init container + fix baseURL casing ([9832e52](https://github.com/5dlabs/cto/commit/9832e5290ab20227fc958753baf6059ef34445d5))
* **mem0:** share cto_memory collection across all agents ([e0a95ac](https://github.com/5dlabs/cto/commit/e0a95aca5e5a1fa9199bfc9f11db20eeb9a7e9f6))
* memorySearch source 'session' → 'sessions' ([1e3cb8a](https://github.com/5dlabs/cto/commit/1e3cb8a07304e5296b2f2869c32b67750d9f5654))
* re-enable mem0 plugin for coder agent ([c509667](https://github.com/5dlabs/cto/commit/c509667b2bb615301dc5c61bdacb72312785b709))
* set Kimi K2.5 as agent.model, disable expired Anthropic/OpenAI ([1afccd3](https://github.com/5dlabs/cto/commit/1afccd39fb3336aed7f208a7c8ef703622ed763b))
* **skills:** restructure to directory/SKILL.md format (AgentSkills convention) ([6c93f43](https://github.com/5dlabs/cto/commit/6c93f439d3b0970c764fa10f3b7191671b9f93cb))
* swarm agent — Fireworks embeddings, explicit delivery channel, 9PM cron ([b802a65](https://github.com/5dlabs/cto/commit/b802a654a80151f4bd4802020f1ce0c22dc5daf6))
* **swarm:** remove memory block from config (not in openclaw schema) ([96e2535](https://github.com/5dlabs/cto/commit/96e25352588085ebc3cccd968cfe5bb1bd91e5a2))
* use full Fireworks model paths in fallback chain + add qwen3p6 ([204a12e](https://github.com/5dlabs/cto/commit/204a12ecbdf33f628650a8e386c9a86f667d2d56))
* use GitHub URL for coder ArgoCD app source ([672542e](https://github.com/5dlabs/cto/commit/672542e99d0796bd6ec28082ce51ff85f9e03415))
* use GitHub URLs for coder workspace repo clones ([43f94bd](https://github.com/5dlabs/cto/commit/43f94bdbdab347cb62033fbd55ecf222a8357cd7))
* use Kimi K2.5 Turbo as primary model for Coder gateway ([24ed298](https://github.com/5dlabs/cto/commit/24ed29872179386a0c89d0625ddb728b90e00273))
* use openai-completions API type for Fireworks provider ([dc1cc40](https://github.com/5dlabs/cto/commit/dc1cc4070e80d510cf8814b705595d88d924c0d7))

## [0.2.54](https://github.com/5dlabs/cto/compare/v0.2.53...v0.2.54) (2026-04-15)


### ✨ Features

* **acp:** clean image build + controller retry race fix ([accd120](https://github.com/5dlabs/cto/commit/accd1202eef8caf7f8de39c94938e0f7e858eda7))
* add activity summary table with commit/PR metrics to digest ([95c571b](https://github.com/5dlabs/cto/commit/95c571b6d9cd1d82e19c57387f06a70f81e06410))
* add AI, GPU, and Nvidia operators; enable twingate-operator ([6ec364a](https://github.com/5dlabs/cto/commit/6ec364a1aa2be73f3c151c2e544f8e0e9ad53096))
* add cloud parity callout to services page hero ([#4500](https://github.com/5dlabs/cto/issues/4500)) ([f8fb48a](https://github.com/5dlabs/cto/commit/f8fb48aa3c9ac8501aca5a66f8817cabed8acbf2))
* add cto-agents repo to monitored workspace ([efc43af](https://github.com/5dlabs/cto/commit/efc43af01e3d338b228c72b11bbacb2767be74c3))
* add cto-play launcher utility + Morgan skill ([7ac87d6](https://github.com/5dlabs/cto/commit/7ac87d690a7c3690550df4661be50b88aa864500))
* add kotal-operator ArgoCD app (5dlabs fork with Reth/NEAR support) ([78a5da3](https://github.com/5dlabs/cto/commit/78a5da3321d7e0bf3d8a9baf2aa17451119fe57d))
* add projectId to CodeRun CRD + memory isolation config ([412d28b](https://github.com/5dlabs/cto/commit/412d28bd1731d001ebc7af6c1d368fe034d667b2))
* add Swarm agent (Canteen hackathon) to CTO platform ([ff969a3](https://github.com/5dlabs/cto/commit/ff969a3765af127d00eec3a6c0b51fd5b1894000))
* **agents:** add Block as Solana specialist agent ([b484a18](https://github.com/5dlabs/cto/commit/b484a18bb27c17b974ad133eb55fa2c936f2c48c))
* **agents:** add kubectl/argocd to agent image, RBAC for task pods ([abfb0f9](https://github.com/5dlabs/cto/commit/abfb0f99ea5169e7724b680b7f83a48c78763374))
* **agents:** add OpenClaw image + fix build-runtime errors ([#4332](https://github.com/5dlabs/cto/issues/4332)) ([3d4e2ce](https://github.com/5dlabs/cto/commit/3d4e2ce076b448731acc27adff502fd530be1882))
* **agents:** add OpenClaw image to agent builds Add ([92176c4](https://github.com/5dlabs/cto/commit/92176c40f3b42dec2dd8c517c041d89a93fccb2c))
* **agents:** cto-tools CLI + Deno runtime + tools-server sidecar for dynamic MCP tool access ([#4607](https://github.com/5dlabs/cto/issues/4607)) ([89f4b4b](https://github.com/5dlabs/cto/commit/89f4b4bbd2e484c056393a032d34c11a7544aa11))
* **agents:** enable dexter agent build ([1d2b036](https://github.com/5dlabs/cto/commit/1d2b0361f97518d6b44f3bf3d80a8f7543f33e82))
* **avatar:** consolidate Morgan avatar PoC into CTO repo ([#4508](https://github.com/5dlabs/cto/issues/4508)) ([2668ea9](https://github.com/5dlabs/cto/commit/2668ea98d3cec9bac180556440d070f03eac6bbd))
* **block:** add controller templates for Block agent ([b68f66a](https://github.com/5dlabs/cto/commit/b68f66a3e2f1a5b43dfeb6240bdfb570a8fbaa92))
* **block:** add Solana CLI, Helius RPC, and expertise docs ([b52e88d](https://github.com/5dlabs/cto/commit/b52e88d944f5c15de50baaf8170457d1cfa7a030))
* **block:** add solana skills — node-ops config + ClawHub dev skills ([149f616](https://github.com/5dlabs/cto/commit/149f6168c33797c2f36438fffaa48585e9a8c4ea))
* **block:** expand to multi-chain blockchain agent ([fe4ba49](https://github.com/5dlabs/cto/commit/fe4ba49f11a728a507a1d24bd0a9eb1344bc723c))
* brand refresh, narrative realignment, and services catalog ([#4497](https://github.com/5dlabs/cto/issues/4497)) ([8f027f9](https://github.com/5dlabs/cto/commit/8f027f987998906e10a9da13cb87a9ad7f088a6a))
* **clawd:** add clawd agent configuration and identity ([#4349](https://github.com/5dlabs/cto/issues/4349)) ([f0a03a9](https://github.com/5dlabs/cto/commit/f0a03a9853fb20d66f5fe23a35a70181a58fe1b7))
* CLI matrix fixes — Codex promtail, Gemini env, model registry, tslog deps, clippy lint ([ee212f8](https://github.com/5dlabs/cto/commit/ee212f8c9f04b426f3fa4db743bde8b6cadf2107))
* complete Angie OpenClaw-first rollout ([#4515](https://github.com/5dlabs/cto/issues/4515)) ([5f6f121](https://github.com/5dlabs/cto/commit/5f6f12145c077455488c74e92c8c062ea81ff308))
* **conductor:** add conductor agent configuration and identity ([#4338](https://github.com/5dlabs/cto/issues/4338)) ([690b8d5](https://github.com/5dlabs/cto/commit/690b8d5e88c0723d039a3e91835917ab8e676e47))
* configure optimist/pessimist Discord + fix gp3 storage on OVH ([aee795a](https://github.com/5dlabs/cto/commit/aee795a8cd4cb3e48d97edf6100008cbc23e5a64))
* **controller:** CRD overhaul, skills pipeline, Discord resilience & Datadog telemetry ([#4613](https://github.com/5dlabs/cto/issues/4613)) ([42f1bdc](https://github.com/5dlabs/cto/commit/42f1bdca34575587731487cbfd5cb56ecbd8dd4f))
* **controller:** CRD provider refactor — explicit provider + providerBaseUrl on CLIConfig ([3340cbf](https://github.com/5dlabs/cto/commit/3340cbf9d74a9406a327787c1c2740245de3a31a))
* **controller:** thread EscalationPolicy from CRD through MCP config ([#4594](https://github.com/5dlabs/cto/issues/4594)) ([638412d](https://github.com/5dlabs/cto/commit/638412d285a13895ddb19399471fcb1ea9163198))
* **crd:** Multi-agent CodeRun with ACP, OpenClaw config, and new naming convention ([#4603](https://github.com/5dlabs/cto/issues/4603)) ([04b76c6](https://github.com/5dlabs/cto/commit/04b76c6c63304b112cc10510d70990614ba84a55))
* **cupid:** add cupid agent configuration and identity ([#4347](https://github.com/5dlabs/cto/issues/4347)) ([18da541](https://github.com/5dlabs/cto/commit/18da541cbc58aca9114aa0b380931f2991d9eace))
* **deliberation:** one meeting room architecture ([b9183b7](https://github.com/5dlabs/cto/commit/b9183b71312ff8b56db027a8d5facf0e8efe20e2))
* **desktop:** OpenClaw-centric UI redesign with agent integration ([#4382](https://github.com/5dlabs/cto/issues/4382)) ([220d757](https://github.com/5dlabs/cto/commit/220d75797e122c000d3fc63155786a11538298ca))
* **docs:** add docs agent configuration and identity ([#4348](https://github.com/5dlabs/cto/issues/4348)) ([4a70406](https://github.com/5dlabs/cto/commit/4a70406840eb715753ddf6cb54229f6ba80a04df))
* enable daily-digest cron job at 8:45 PM PDT (test) ([9a63ab4](https://github.com/5dlabs/cto/commit/9a63ab42d770655c7d5ab502ada4ad45ee3b5b95))
* Fireworks-only providers, embeddings, IfNotPresent pullPolicy ([6b2eb43](https://github.com/5dlabs/cto/commit/6b2eb43fcdeff7025f4eb251acf6aed9bbf9f124))
* **forge:** agent identity and configuration ([#4336](https://github.com/5dlabs/cto/issues/4336)) ([e316ffe](https://github.com/5dlabs/cto/commit/e316ffe3643b79c1f5b8a3afca16af5073ba14e6))
* **grok:** replace Rust research CronJob with Bun-based Grok pipeline ([#4364](https://github.com/5dlabs/cto/issues/4364)) ([f69de5d](https://github.com/5dlabs/cto/commit/f69de5dac20ab8f5fbe9dd6a254ca88eafdaa829))
* **helm:** add cloud provider support (Bedrock, Vertex AI, Foundry) ([#4396](https://github.com/5dlabs/cto/issues/4396)) ([252b168](https://github.com/5dlabs/cto/commit/252b168d9d669d6c9bb5a888e7b8d18063905c24))
* Hermes harness agent integration ([#4630](https://github.com/5dlabs/cto/issues/4630)) ([f0760d9](https://github.com/5dlabs/cto/commit/f0760d968072b698e1a177e618eb2849b180b755))
* **holt:** add holt agent configuration and identity ([#4346](https://github.com/5dlabs/cto/issues/4346)) ([ad7c801](https://github.com/5dlabs/cto/commit/ad7c801d077a00addc0a6551f70b109d998b59b5))
* **infra:** add ArgoCD sync workflow for automatic deployments on merge ([#4616](https://github.com/5dlabs/cto/issues/4616)) ([504dede](https://github.com/5dlabs/cto/commit/504dede0dce0451902f646dff127e8c3d3c23549))
* **infra:** add Qdrant Datadog metrics + Cloudflare dashboard egress ([#4619](https://github.com/5dlabs/cto/issues/4619)) ([537be5e](https://github.com/5dlabs/cto/commit/537be5eb0bdfe8952f668f38e4bea729633e7362))
* **infra:** add Qdrant vector database for Mem0 agent memory ([#4617](https://github.com/5dlabs/cto/issues/4617)) ([f00a38d](https://github.com/5dlabs/cto/commit/f00a38d311546445efaa5fd95ead97c6704545ce))
* **infra:** add switchable webhook exposure and re-enable Stitch Argo app ([#4419](https://github.com/5dlabs/cto/issues/4419)) ([db88b6d](https://github.com/5dlabs/cto/commit/db88b6dd7146ba51ed2760e9ed9a85d270d18466))
* **infra:** AWS cluster bootstrap – OpenClaw bots disabled, storage overlay app ([#4416](https://github.com/5dlabs/cto/issues/4416)) ([5dd793f](https://github.com/5dlabs/cto/commit/5dd793f1d69994ce66ed86ef1d6ef6e46587dd86))
* **infra:** AWS cluster bootstrap with skip-reconcile overlay strategy ([#4415](https://github.com/5dlabs/cto/issues/4415)) ([25afd22](https://github.com/5dlabs/cto/commit/25afd220e73400cbb2c07f3c056c5cf613a52b96))
* **infra:** disable OpenMemory in favor of Mem0 + Qdrant ([#4618](https://github.com/5dlabs/cto/issues/4618)) ([7b82021](https://github.com/5dlabs/cto/commit/7b820216b0199e01066152fdcd4f0593be270e06))
* **infra:** route GitHub webhooks directly to PM server ([#4384](https://github.com/5dlabs/cto/issues/4384)) ([0d145d8](https://github.com/5dlabs/cto/commit/0d145d83460350ec515375a3e48f66333548b5b1))
* **intake:** add codebase_repository_url arg for cross-repo analysis ([d8f6056](https://github.com/5dlabs/cto/commit/d8f6056ecd682148b9e53975e7791647f6e75a34))
* **intake:** add intake agent configuration and identity ([#4345](https://github.com/5dlabs/cto/issues/4345)) ([4ccc8bb](https://github.com/5dlabs/cto/commit/4ccc8bb639d31ccf202ab1650590796a4d4745db))
* **intake:** Anthropic metaprompt-compliant intake prompts ([cff597d](https://github.com/5dlabs/cto/commit/cff597dc0ec1e50a126ece1109270d48be9eb097))
* **intake:** attach deliberation MP3 to GitHub Release on target repo ([8e31ecb](https://github.com/5dlabs/cto/commit/8e31ecb8f0e471ffa638d77264ad41ce740ca2c7))
* **intake:** deliberation system — debate agents + committee voting ([#4434](https://github.com/5dlabs/cto/issues/4434)) ([6c95735](https://github.com/5dlabs/cto/commit/6c9573584afbe24c70c08eabbe71ebd73230b1e0))
* **intake:** enable AI prompt generation (Session 2) by default ([d6578bd](https://github.com/5dlabs/cto/commit/d6578bdcb291cab0afb41362b63e39db5531d6fe))
* **intake:** generate AlertHub e2e test prompts ([c329f14](https://github.com/5dlabs/cto/commit/c329f143b92096e139a5cbb5152c371157358ca4))
* **intake:** Lobster intake migration with 5-model voting ([#4379](https://github.com/5dlabs/cto/issues/4379)) ([0c6b799](https://github.com/5dlabs/cto/commit/0c6b799b87f386fedbb9216c993bf0a76f2cf62b))
* **intake:** notifycore-e2e task breakdown ([#4552](https://github.com/5dlabs/cto/issues/4552)) ([d43b686](https://github.com/5dlabs/cto/commit/d43b686d0c2207f843849d12f6cdc76a3fb51679))
* **intake:** research step, Discord reporting, PR creation ([#4462](https://github.com/5dlabs/cto/issues/4462)) ([bff007d](https://github.com/5dlabs/cto/commit/bff007d25d4bba3dcbf65fa924c5c63ee3943b51))
* **intake:** sigma-1 task breakdown ([#4589](https://github.com/5dlabs/cto/issues/4589)) ([2760d2c](https://github.com/5dlabs/cto/commit/2760d2cab509350945d42157d72ce7d12252c072))
* **intake:** wire deliberation into real intake pipeline before task generation ([f6715c7](https://github.com/5dlabs/cto/commit/f6715c76fb74ed704318931a72c750fe4d8cbbe6))
* **keeper:** add keeper agent configuration and identity ([03a2973](https://github.com/5dlabs/cto/commit/03a29738a6b44a2dd3143f994a079f2ecb31690d))
* **keeper:** keeper agent configuration and identity ([#4355](https://github.com/5dlabs/cto/issues/4355)) ([0f22793](https://github.com/5dlabs/cto/commit/0f22793c00027fe239d86976f83fb3b2204c1120))
* **linear-bridge:** Loki activity stream for ACP agent dialog ([#4491](https://github.com/5dlabs/cto/issues/4491)) ([904af87](https://github.com/5dlabs/cto/commit/904af87206654539e29e4235a8242a379348866c))
* **lobster-voice:** add TTS pronunciation dictionary for natural technical speech ([507797f](https://github.com/5dlabs/cto/commit/507797fc962c5a6d4dddae4f12d2fa1954aede7c))
* Main Team (Lex, Hype, Tally, Chase) on splash and CTO ([#4487](https://github.com/5dlabs/cto/issues/4487)) ([9d990bd](https://github.com/5dlabs/cto/commit/9d990bdb0e4cc0f1dd67657f3e676bef2acec7fb))
* **marketing:** add 5D Git as platform service under Source Control ([0099151](https://github.com/5dlabs/cto/commit/00991510e31ebc6060dfa815f1195f5547618718))
* **marketing:** add 9 bare metal providers to infrastructure section ([#4446](https://github.com/5dlabs/cto/issues/4446)) ([109b55c](https://github.com/5dlabs/cto/commit/109b55c5ac7f8a13f7a357dff4fa1de1f1bf1c75))
* **marketing:** add ACP harness and multi-provider model routing ([81d5d11](https://github.com/5dlabs/cto/commit/81d5d1157417979cc33b38a2a98dd5959c3965c5))
* **marketing:** add Blockchain & AI Teams section ([d565ba9](https://github.com/5dlabs/cto/commit/d565ba902f8061ae4c480b14e061c9271d32ed92))
* **marketing:** add GitLab/Gitea support, align header behavior ([41f13d0](https://github.com/5dlabs/cto/commit/41f13d02bc81659ee3ebf50bb38229b81b624c04))
* **marketing:** add Glitch agent avatar ([#4486](https://github.com/5dlabs/cto/issues/4486)) ([3bd33a9](https://github.com/5dlabs/cto/commit/3bd33a9dac9c95a9cae4092f6fa53006f192edba))
* **marketing:** add managed services tech stack section ([#4357](https://github.com/5dlabs/cto/issues/4357)) ([d46b956](https://github.com/5dlabs/cto/commit/d46b956a9e31ead7809fc7ca8a4fbadf3e441052))
* **marketing:** add Vex and Block agent avatars ([#4485](https://github.com/5dlabs/cto/issues/4485)) ([0135a4d](https://github.com/5dlabs/cto/commit/0135a4dd48443d985ea613a4b232cc671432a65a))
* **marketing:** agent card overhaul + content corrections ([#4482](https://github.com/5dlabs/cto/issues/4482)) ([bb62d9d](https://github.com/5dlabs/cto/commit/bb62d9d97d79b6973ddbf067b82ed7232a8c825a))
* **marketing:** clarify GPU compute optionality with 3-tier breakdown ([00eb520](https://github.com/5dlabs/cto/commit/00eb52025707ee461bb1b67eac5da934c171616e))
* **marketing:** expand Bolt to Infrastructure & SRE, add Healer agent ([#4445](https://github.com/5dlabs/cto/issues/4445)) ([46a6818](https://github.com/5dlabs/cto/commit/46a6818a2bc2ba5f21cc94f7bfd1b599a2a4be27))
* **marketing:** infra rewrite, new agents, blockchain refresh, security tooling ([485f3e9](https://github.com/5dlabs/cto/commit/485f3e948d668ab78acc92d3b005d5051addc3c2))
* **marketing:** integrations section, Block/Vex agents, Cipher pen testing ([6e51966](https://github.com/5dlabs/cto/commit/6e519666376bcd5e21772877f8a4a32556e50677))
* **marketing:** merge Specialists squad, refresh self-hosted model list to March 2026 ([e682eac](https://github.com/5dlabs/cto/commit/e682eacad1715e887a7b6a4fba74f63258dfb53e))
* **marketing:** Morgan hero effects, card flip fix, LemonSlice inline, golden retriever avatar ([a40d988](https://github.com/5dlabs/cto/commit/a40d98871ebfc1e46004c5f0452dc7b07398ad72))
* **marketing:** Morgan prominence across CTO site ([d706194](https://github.com/5dlabs/cto/commit/d7061947968bc26b16cae4411bfe742a088b5d37))
* **mcp:** add toggle_app tool for enabling/disabling ArgoCD applications ([#4403](https://github.com/5dlabs/cto/issues/4403)) ([57b8384](https://github.com/5dlabs/cto/commit/57b83848b41220a3badb26d3690cc6bd37b286ed))
* **mem0:** consolidate Mem0 plugin into CTO chart ([1309404](https://github.com/5dlabs/cto/commit/1309404940625f2fa5e4d2d7eb0d3092d4c3fa40))
* **mem0:** wire Mem0 plugin into CRD task pods ([a4e527b](https://github.com/5dlabs/cto/commit/a4e527b849b6919d16cf9c9a8c2b2fa2f546801b))
* **memory:** enhanced payload schema with metadata, categories, indexes ([88371a3](https://github.com/5dlabs/cto/commit/88371a3334ab646ff0a8d6ea83adebca67f15f43))
* **memory:** Phase 2-4 memory isolation — task scoping, skills, dashboard ([6169d25](https://github.com/5dlabs/cto/commit/6169d2542431b3fe6a8a66c6b2edf937e54ea7e1))
* **metal:** add metal agent configuration and identity ([#4339](https://github.com/5dlabs/cto/issues/4339)) ([96f6cbd](https://github.com/5dlabs/cto/commit/96f6cbdffcfb796e2f1dcb1e3768025687d5c2de))
* Morgan RBAC + git fix + Mem0 plugin config ([db07994](https://github.com/5dlabs/cto/commit/db07994e2241be0b63bcecdc24b63cd7f38cd92e))
* **morgan:** add lean Morgan agent for avatar PoC ([#4504](https://github.com/5dlabs/cto/issues/4504)) ([a5ed834](https://github.com/5dlabs/cto/commit/a5ed8341a397c57e3f380cf9ef00ad7edc81c005))
* **morgan:** marketing page, avatar, LemonSlice embed, and desktop wiring ([917e2cf](https://github.com/5dlabs/cto/commit/917e2cf73f68afbc9cacf75646933bc51368194d))
* **nats:** add discovery protocol, agent roster, and ping-pong guard ([4542490](https://github.com/5dlabs/cto/commit/4542490e60ce0596291b6dd8812cb464b9bbc83c))
* **nats:** NATS inter-agent messaging plugin and infrastructure ([#4387](https://github.com/5dlabs/cto/issues/4387)) ([7c13be1](https://github.com/5dlabs/cto/commit/7c13be1f1d8ae6d80e20ff32b3bc8b049b55e661))
* **observability:** add local Grafana/Loki stack for intake pipeline ([c446e72](https://github.com/5dlabs/cto/commit/c446e7266c2ad00d9523d7128f8b3d8ffd83ad7d))
* **observability:** add log level label to Loki + Cluster Logs dashboard ([8189d2a](https://github.com/5dlabs/cto/commit/8189d2a33b330070383f45b5e4dcc33a958a691d))
* **observability:** add prominent status tiles to Solana Validator dashboard ([6c6c399](https://github.com/5dlabs/cto/commit/6c6c3993de4989188a73d690d51aaecd92620a93))
* **openclaw-agent:** add nodeSelector support and remove CPU limits ([89f7360](https://github.com/5dlabs/cto/commit/89f7360a3bad32991ae5c4479c34b2b03db86ed3))
* **openclaw:** add kaniko container-builds skill ([#4374](https://github.com/5dlabs/cto/issues/4374)) ([8121730](https://github.com/5dlabs/cto/commit/8121730e4dfd64b0e3ed0f842f59f6d29f0713cb))
* **openclaw:** deploy all 14 remaining agents to Kubernetes ([#4375](https://github.com/5dlabs/cto/issues/4375)) ([b6cf4b6](https://github.com/5dlabs/cto/commit/b6cf4b616d633be37466d8ba25c0b5ddeb706175))
* **openclaw:** deploy OpenClaw agents to Kubernetes in bots namespace ([#4335](https://github.com/5dlabs/cto/issues/4335)) ([d21537d](https://github.com/5dlabs/cto/commit/d21537dbe8262b22c0598f3ff2f26dbe18fbd107))
* **openclaw:** expand container-builds skill with Docker→Kaniko guide ([#4378](https://github.com/5dlabs/cto/issues/4378)) ([25aaaeb](https://github.com/5dlabs/cto/commit/25aaaeb8684f73e91a2c70845864599d96661c27))
* **openclaw:** memory optimization, group:memory allowlists, heartbeat autonomy ([#4362](https://github.com/5dlabs/cto/issues/4362)) ([1916f07](https://github.com/5dlabs/cto/commit/1916f073ca5195eac1ab6d8d803c9b863ffa7fe4))
* **openclaw:** Phase 2 golden copy — unified image, tools, skills, secrets ([#4358](https://github.com/5dlabs/cto/issues/4358)) ([04c78d0](https://github.com/5dlabs/cto/commit/04c78d09852c840bdc40b1ec7c988c16dfacd2ea))
* **openclaw:** use npm package instead of source build ([1126503](https://github.com/5dlabs/cto/commit/1126503e8482f8e39b7f326dab1bf50583ff887b))
* **pitch-deck:** rewrite deck with PAS framework, plain language, 13 slides ([a0980c0](https://github.com/5dlabs/cto/commit/a0980c0685b6fb826c67a7e43b4a63c853005a99))
* **pitch:** add pitch agent configuration and identity ([#4350](https://github.com/5dlabs/cto/issues/4350)) ([c16c1d4](https://github.com/5dlabs/cto/commit/c16c1d4bc9a889a085299f6ba4cb7df3fca19fa0))
* **pixel-assistant:** add pixel-assistant agent configuration ([#4353](https://github.com/5dlabs/cto/issues/4353)) ([35bbccb](https://github.com/5dlabs/cto/commit/35bbccb4dd936f0a9b13faa465d1c0fd4b3861c2))
* **pixel:** add pixel agent configuration and identity ([#4341](https://github.com/5dlabs/cto/issues/4341)) ([cd15f9c](https://github.com/5dlabs/cto/commit/cd15f9c2f8efd457b41328b0503e2d062eb9113b))
* **planner:** add planner agent configuration and identity ([#4343](https://github.com/5dlabs/cto/issues/4343)) ([78250b9](https://github.com/5dlabs/cto/commit/78250b92b80cc79d20115411259a1d4e5597ad45))
* platform naming, logo, hiring philosophy, CLI additions, abstraction language ([#4498](https://github.com/5dlabs/cto/issues/4498)) ([e7cfffd](https://github.com/5dlabs/cto/commit/e7cfffdc888b55b61d7604ddc38d5bad67a79706))
* **playmon:** add playmon agent configuration and identity ([#4351](https://github.com/5dlabs/cto/issues/4351)) ([470a808](https://github.com/5dlabs/cto/commit/470a8084b8e50a14c5c12c81eae29fe8a6ea2c47))
* **pm:** add unified GitHub webhook handler with HMAC verification ([#4383](https://github.com/5dlabs/cto/issues/4383)) ([d71e727](https://github.com/5dlabs/cto/commit/d71e72779f824356893b0747b5f7c06f623df2b5))
* **pm:** agent delegation, token endpoint, and config hardening ([#4568](https://github.com/5dlabs/cto/issues/4568)) ([e56df9f](https://github.com/5dlabs/cto/commit/e56df9f66753623f6a68f4a3a0b61a2491aa0432))
* remote skills + agent persona distribution via cto-agent-personas ([#4605](https://github.com/5dlabs/cto/issues/4605)) ([6cd880c](https://github.com/5dlabs/cto/commit/6cd880c6efa971cf53b0f42107866db864445db2))
* **research:** research agent tools, findings, and MCP server ([#4337](https://github.com/5dlabs/cto/issues/4337)) ([5bf561e](https://github.com/5dlabs/cto/commit/5bf561e21af193e6c110eae796a80a0aa0f9f2c4))
* **review:** add review agent configuration and identity ([#4352](https://github.com/5dlabs/cto/issues/4352)) ([ce122e9](https://github.com/5dlabs/cto/commit/ce122e909968f8304d0a2c2769a52524e11a8cd8))
* **runtime:** add steipete CLI tools to baseline image ([#4423](https://github.com/5dlabs/cto/issues/4423)) ([4525b9e](https://github.com/5dlabs/cto/commit/4525b9e2a89d97886c3b7a3e62b2cf4067163066))
* **scm:** dual GitHub + GitLab SCM provider support ([#4536](https://github.com/5dlabs/cto/issues/4536)) ([a43c572](https://github.com/5dlabs/cto/commit/a43c572e7ba95fe52fd10c8cc020fa931e43c204))
* **scout:** add scout agent configuration and identity ([#4344](https://github.com/5dlabs/cto/issues/4344)) ([2cb26cd](https://github.com/5dlabs/cto/commit/2cb26cd1221f9639945588dc5a08a47feb27bb95))
* **skills:** add 10 ClawHub community skills to templates ([#4424](https://github.com/5dlabs/cto/issues/4424)) ([2f0f66e](https://github.com/5dlabs/cto/commit/2f0f66e4510a84923fe77846ea1bc6de3244b839))
* **splash,marketing:** boost grid visibility and restore perspective breathing ([#4473](https://github.com/5dlabs/cto/issues/4473)) ([391a535](https://github.com/5dlabs/cto/commit/391a53589c0262631be1d84561dc32bc075c6d4f))
* **splash,marketing:** Grid Pulse + Shift Dimensions, deploy via 1Password ([#4461](https://github.com/5dlabs/cto/issues/4461)) ([aa8dc6f](https://github.com/5dlabs/cto/commit/aa8dc6f08ea0eeef43fec6b3e01cbfe54d84e914))
* **splash,marketing:** match grid and shift values to bg-preview demo ([#4474](https://github.com/5dlabs/cto/issues/4474)) ([b0e6c7a](https://github.com/5dlabs/cto/commit/b0e6c7a71f1fede6767f92cfb9ec9b48c4eece78))
* **splash,marketing:** restore entrance animations with CSS-only approach ([#4472](https://github.com/5dlabs/cto/issues/4472)) ([dc1ff7c](https://github.com/5dlabs/cto/commit/dc1ff7cdcb3ffe86821e63e30862d65ec1225897))
* **splash:** add 5D Labs homepage app for 5dlabs.io ([#4425](https://github.com/5dlabs/cto/issues/4425)) ([5aabffb](https://github.com/5dlabs/cto/commit/5aabffb356b65f1cfdcf282889ce7dde30df86e5))
* **splash:** add Chase (Sales) business agent, increase avatar size ([ca74c70](https://github.com/5dlabs/cto/commit/ca74c70d7425b20a61af0db5717f5228b92f5e4d))
* **splash:** add consulting services page ([#4435](https://github.com/5dlabs/cto/issues/4435)) ([9061ee2](https://github.com/5dlabs/cto/commit/9061ee29ffe6abca3042b8a1b3c61b595dc0d410))
* **splash:** add downloadable investor one-pager on investors page ([#4459](https://github.com/5dlabs/cto/issues/4459)) ([470ffab](https://github.com/5dlabs/cto/commit/470ffabf9419602dba215f4c7e002c071fb7b5f3))
* **splash:** add Polygon to trading chain coverage ([3e5755c](https://github.com/5dlabs/cto/commit/3e5755cf1e53fcea037b6c7f848e09d1e1f529bd))
* **splash:** expand Bolt from Deployment Specialist to Infrastructure & SRE ([#4444](https://github.com/5dlabs/cto/issues/4444)) ([0e04918](https://github.com/5dlabs/cto/commit/0e049186c5f48fdd3f3550cee9b88f9d6d846ced))
* **splash:** move Opportunities to footer, add Hivelocity provider ([#4441](https://github.com/5dlabs/cto/issues/4441)) ([86ab681](https://github.com/5dlabs/cto/commit/86ab681efcf545ca0d060cfbe8d27ebfff622248))
* **splash:** remove Sanctuary venture ([f1183fb](https://github.com/5dlabs/cto/commit/f1183fbfd94dbd5010e113caa98148f7e213ed42))
* **splash:** rename The Designers to The Product on team page ([#4442](https://github.com/5dlabs/cto/issues/4442)) ([075060a](https://github.com/5dlabs/cto/commit/075060a56a9becc022604daf10c5cd75e6722dab))
* **splash:** Rex, Grizz, Nova as peers — Rust/Go/Node Engineer ([#4443](https://github.com/5dlabs/cto/issues/4443)) ([3f17f38](https://github.com/5dlabs/cto/commit/3f17f381f086780a726a74905f12bbf7b81d92d9))
* **splash:** scroll-hide header ([#4480](https://github.com/5dlabs/cto/issues/4480)) ([de86689](https://github.com/5dlabs/cto/commit/de866891b181fa605939fd343b4da481ec6ccfa3))
* **splash:** Sui chain, Servers.com/PhoenixNAP/i3D.net providers, model optimization ([#4439](https://github.com/5dlabs/cto/issues/4439)) ([c4f5381](https://github.com/5dlabs/cto/commit/c4f538193509794205ebd9b59633ca419a98bb08))
* **stitch:** add stitch agent configuration and identity ([#4340](https://github.com/5dlabs/cto/issues/4340)) ([b7a0c61](https://github.com/5dlabs/cto/commit/b7a0c613312d20f5c70f557a1e8d94fba4f79b22))
* **swarm:** add Colosseum Copilot PAT + correct env vars ([ddccaa1](https://github.com/5dlabs/cto/commit/ddccaa16807f0ba9f1480fb038a3fcdbadd820fd))
* **swarm:** add Colosseum PAT env vars + sigma-1, lab-eco, big-balls repos ([cae7c9d](https://github.com/5dlabs/cto/commit/cae7c9dfd0fe5dbdcd7114c84bc669a4afeb9631))
* **swarm:** ArgoCD-managed StatefulSet with OpenClaw native cron ([c10e691](https://github.com/5dlabs/cto/commit/c10e691ea4163d0a58e3597ab21beedf4db62c81))
* **swarm:** enable memory, session maintenance, compaction model ([ac40abe](https://github.com/5dlabs/cto/commit/ac40abed9071e4c9ca17154eb4ceefdd25cb049a))
* **swarm:** enable SWARM CLI pre-auth + profile config ([dfe999b](https://github.com/5dlabs/cto/commit/dfe999b365937034321d1e090805756124772fa8))
* **swarm:** upgrade digest workflow, add repos, progress journal ([e92872d](https://github.com/5dlabs/cto/commit/e92872d5d96751218f5596cf2d1ce09fcc5d0cf6))
* switch swarm ArgoCD to public chart (5dlabs/swarm-openclaw-agent) ([3be6247](https://github.com/5dlabs/cto/commit/3be6247e7b655e95698993ee5271e964bd0caf86))
* **task-controller:** add research CLI image + switch intake to research image ([2eac6de](https://github.com/5dlabs/cto/commit/2eac6de8734b354430686f68a7db11b8145dbf06))
* **telemetry:** enrich Loki labels with cli_name, session_type, activity_type ([d2393bd](https://github.com/5dlabs/cto/commit/d2393bdc0a96e8e7461947b05a1617255207b485))
* **test:** neon gauntlet — multi-CLI E2E test suite ([#4614](https://github.com/5dlabs/cto/issues/4614)) ([4aa1187](https://github.com/5dlabs/cto/commit/4aa11874ecc5bc5faaeb7a606c2a6646d28c8ec5))
* **tests:** alerthub e2e intake output — subtask prompts (146 subtasks) ([#4453](https://github.com/5dlabs/cto/issues/4453)) ([d793712](https://github.com/5dlabs/cto/commit/d793712cd7d4ec208d2ec5bcfb6a139f8e21703c))
* **tools:** Dynamic MCP TypeScript SDK for code-execution mode ([#4610](https://github.com/5dlabs/cto/issues/4610)) ([1d66057](https://github.com/5dlabs/cto/commit/1d6605793894e14ed98f2ac5bf7595b7e268770c))
* **tools:** escalation policy engine + tools_request_capability built-in ([#4593](https://github.com/5dlabs/cto/issues/4593)) ([9f293e0](https://github.com/5dlabs/cto/commit/9f293e0e1834b23db26b27e54baa92c93af9aeb3))
* **trader:** add trader agent configuration and identity ([#4342](https://github.com/5dlabs/cto/issues/4342)) ([38598eb](https://github.com/5dlabs/cto/commit/38598ebb962cd5c9ac691e9f7c056c67e4c65f56))
* **trading:** add Yellowstone gRPC geyser plugin to Agave RPC node ([7647831](https://github.com/5dlabs/cto/commit/7647831d51814145c48213dd2e1484fed3cbf369))
* **trading:** build Yellowstone gRPC from source, fix ABI crash ([9b6bfeb](https://github.com/5dlabs/cto/commit/9b6bfeb4852889bca9b9b4ebbd48c45d738d8f98))
* **trading:** deploy 8 trading agents to K8s via ArgoCD ([#4530](https://github.com/5dlabs/cto/issues/4530)) ([1105794](https://github.com/5dlabs/cto/commit/1105794e49a0aaf2a1e79cdd028df91cd341b9f6))
* **trading:** deploy solana exporter, production dashboards, and ArgoCD integration for Cherry cluster ([16dee06](https://github.com/5dlabs/cto/commit/16dee0685f892358a5f09617850204d604c1373f))
* **trading:** Solana DEX indexer + Agave flag fix ([#4535](https://github.com/5dlabs/cto/issues/4535)) ([d51edb9](https://github.com/5dlabs/cto/commit/d51edb9696b7843959a01aa6f20966f7bfbd9db9))
* update cilium values for k3s/OVH - tunnel mode, k3s API, correct pod CIDR ([b323e1e](https://github.com/5dlabs/cto/commit/b323e1e1c245cf02890c3fb4a78f37d0ca3004af))
* v0-inspired 'Build at the Speed of Thought' hero rewrite ([#4503](https://github.com/5dlabs/cto/issues/4503)) ([18c22ad](https://github.com/5dlabs/cto/commit/18c22aded104c252dea2eec80cc12621a94fe882))
* **webapp:** add webapp agent configuration and identity ([#4354](https://github.com/5dlabs/cto/issues/4354)) ([d0ef4c4](https://github.com/5dlabs/cto/commit/d0ef4c4cf851cc3d02bea1cf001dd089af36720e))
* **website:** Morgan page — Meta Ray-Ban Display + WhatsApp ([5bcb6f9](https://github.com/5dlabs/cto/commit/5bcb6f9cec58e77b9dcbb9420f15de6521fbd00c))
* **website:** Morgan page — Rokid Glasses + Vuzix Z100 sections ([a229dde](https://github.com/5dlabs/cto/commit/a229ddec9a069b7f9566917403f0758e89cb2e1e))


### 🐛 Bug Fixes

* add colosseum API URL construction warning to AGENTS.md ([b76de08](https://github.com/5dlabs/cto/commit/b76de085cde82126228c2ebe916c1e94d3a141d2))
* add custom ArgoCD Lua health checks for ClusterTunnel, TunnelBinding, HTTPRoute ([93f341b](https://github.com/5dlabs/cto/commit/93f341bab89fc6ff4323bb8a33f30bf12741213b))
* add githubSecret to committee agent values (was causing empty secretKeyRef) ([fe521e0](https://github.com/5dlabs/cto/commit/fe521e0fc43c59c335c25b0d38e1b5e637a57024))
* add headers field to tools mcp-servers.json configmap template ([83f65df](https://github.com/5dlabs/cto/commit/83f65df8ef9cb4081e67530db2332973bb1abdd4))
* add ipv4NativeRoutingCIDR for Cilium native routing ([#4397](https://github.com/5dlabs/cto/issues/4397)) ([dd51e3a](https://github.com/5dlabs/cto/commit/dd51e3afd5af7c2eeb17fbd1aa91b0fbbb0a166b))
* add kube-dns service alias for RKE2 compatibility ([9c0ecb0](https://github.com/5dlabs/cto/commit/9c0ecb0c174f7be76c7e9e84a713d0001e3b07e8))
* add token leak prevention + absolute path rules to AGENTS.md ([24fe5c7](https://github.com/5dlabs/cto/commit/24fe5c7822edf442b42ac22ed02c2bacfb6f87c1))
* add tool usage rules + missing repos to swarm AGENTS.md ([1b0c137](https://github.com/5dlabs/cto/commit/1b0c137c1e8be51f9e1c71282c32be1845274624))
* **agents:** add --legacy-peer-deps for openclaw tslog resolution ([547b4dd](https://github.com/5dlabs/cto/commit/547b4dd9d8ef7cc5d72d623de4aea8aa9361147e))
* **agents:** Add persistence logic to Atlas and Bolt agents ([#4290](https://github.com/5dlabs/cto/issues/4290)) ([9d6919d](https://github.com/5dlabs/cto/commit/9d6919d412e5a23b9ca2e35049f35c9d4edd31c8))
* **agents:** explicitly install tslog in openclaw dir ([feb4596](https://github.com/5dlabs/cto/commit/feb4596012254dd07621c443d0196a2f767a106b))
* **agents:** fix unbound variable error in build-runtime ([aba9bd7](https://github.com/5dlabs/cto/commit/aba9bd785471b03cfd5b8e970b3e0d88f20f2da3))
* **agents:** use official openclaw install without --ignore-scripts ([6406761](https://github.com/5dlabs/cto/commit/640676107844fdf741545572f8bc5a40c6ae8a2e))
* allow intake script to continue when deliberation fails ([3c894d1](https://github.com/5dlabs/cto/commit/3c894d1c0492e7f4f616f43514e91adf75317379))
* **angie:** replace placeholder avatar with final lynx asset ([#4518](https://github.com/5dlabs/cto/issues/4518)) ([7e35337](https://github.com/5dlabs/cto/commit/7e35337cd37e4382e3b44ac7654f0b88c1dc2fce))
* **bots:** add imagePullSecrets to discord-bridge and linear-bridge ([#4495](https://github.com/5dlabs/cto/issues/4495)) ([f9b5c85](https://github.com/5dlabs/cto/commit/f9b5c856bc9a4d9cb853cd96fbad9aed857b2622))
* chown git-credentials so agent can clone repos ([d2013f3](https://github.com/5dlabs/cto/commit/d2013f364b6824a20b354660d5adb8e001432182))
* **ci:** add --branch=main to wrangler deploy for production release ([9ef233e](https://github.com/5dlabs/cto/commit/9ef233e1ee58fa072980466c4890c5dc094db746))
* **ci:** correct malformed if condition in healer-ci workflow ([#4286](https://github.com/5dlabs/cto/issues/4286)) ([bbb8e3b](https://github.com/5dlabs/cto/commit/bbb8e3bdbbf786256e8221eaa6a4e06f61ae29eb))
* **cilium:** disable bpf masquerade and enable nodeport for k3s ([5f0c9e6](https://github.com/5dlabs/cto/commit/5f0c9e6fbd0830716924e364ef5043f39be53197))
* **ci:** read workspace version from root Cargo.toml ([3211c33](https://github.com/5dlabs/cto/commit/3211c338a87281a86017847c43407b8f67351404))
* **ci:** remove all develop branch references from workflow triggers ([#4278](https://github.com/5dlabs/cto/issues/4278)) ([b0c3471](https://github.com/5dlabs/cto/commit/b0c34712fbc99341d50c052f6e75de19f1d076fc))
* **ci:** remove develop branch from CodeQL and Healer CI workflow triggers ([#4283](https://github.com/5dlabs/cto/issues/4283)) ([1867591](https://github.com/5dlabs/cto/commit/1867591bb84ba851e12ad76f7890e07c3605f1b2))
* **ci:** remove develop branch references from healer-ci workflow ([#4282](https://github.com/5dlabs/cto/issues/4282)) ([8612ec2](https://github.com/5dlabs/cto/commit/8612ec2c67bd31672dbbb078dbad12aba3ae7403))
* **ci:** stop redirecting stderr into scan-results.json ([1ed08aa](https://github.com/5dlabs/cto/commit/1ed08aa990b612b6dc549852231f152bf896ddf2))
* **ci:** switch agents-build to ubuntu-latest runners ([#4577](https://github.com/5dlabs/cto/issues/4577)) ([83e613e](https://github.com/5dlabs/cto/commit/83e613e62176daf7df2d868c64d551893591c87a))
* **ci:** use controller-build.Dockerfile for Docker build ([9e6985b](https://github.com/5dlabs/cto/commit/9e6985b2d602a6284fca5f4d3c8716201c85ced7))
* **ci:** use ubuntu-latest for controller build-and-push ([395c732](https://github.com/5dlabs/cto/commit/395c73253a1b6c0985fb2e872101c0fb8d6027a0))
* **codeql:** correct job condition and increase timeout ([#4325](https://github.com/5dlabs/cto/issues/4325)) ([c99a729](https://github.com/5dlabs/cto/commit/c99a7299640bb3210b638fb93b3bca208369a482))
* **codeql:** remove develop branch + exclude rust/non-https-url ([#4268](https://github.com/5dlabs/cto/issues/4268)) ([ae0b191](https://github.com/5dlabs/cto/commit/ae0b191ddc0299f9036b07d8a59db531bd290249))
* **codeql:** remove develop branch from workflow triggers ([#4284](https://github.com/5dlabs/cto/issues/4284)) ([52a638f](https://github.com/5dlabs/cto/commit/52a638f79b6ad3da56454d49fa49ca6f0d4dd055))
* **codeql:** remove nonexistent develop branch from workflow triggers ([#4269](https://github.com/5dlabs/cto/issues/4269)) ([4ed9c30](https://github.com/5dlabs/cto/commit/4ed9c30bd37064f5ab86f2e8b9d55999cdc3ee98))
* configure gh CLI auth via config file, not env vars ([0dcf4d2](https://github.com/5dlabs/cto/commit/0dcf4d227bd352ce68a7d4dfbfafc670e0faa5d5))
* **controller-template:** fix REPOSITORY_URL set-u crash, workspace persistence, intake-files fallback ([6f04f47](https://github.com/5dlabs/cto/commit/6f04f478d2153c7bc1677fc5c5c9ae093bddfdeb))
* **controller:** add grok cliImage alias pointing to research image ([fdf902a](https://github.com/5dlabs/cto/commit/fdf902aa14cc6aebe25ae74ce572437210c3d2c7))
* **controller:** chown /workspace/runs so research user can clean workspace ([540f225](https://github.com/5dlabs/cto/commit/540f2256ef27b7d9c8b04e7c55ba803e80996fe1))
* **controller:** preserve image node_modules in openclaw-deps init container ([f4a5763](https://github.com/5dlabs/cto/commit/f4a5763091795b485a399dc29ba6301bf8fec858))
* **controller:** use TimeoutLayer::with_status_code for consistent 408 responses ([#4317](https://github.com/5dlabs/cto/issues/4317)) ([c329fda](https://github.com/5dlabs/cto/commit/c329fdab64b6c941e70573f1222a4a236e940a7a))
* **deliberation:** add dmPolicy=block to optimist/pessimist agents ([d2c1483](https://github.com/5dlabs/cto/commit/d2c148382dfa1423d0eafebb0fc6e753e5c11df0))
* **deliberation:** AgentMessage format, NATS reply inbox, bedrock→claude model configs ([#4456](https://github.com/5dlabs/cto/issues/4456)) ([d55f7f4](https://github.com/5dlabs/cto/commit/d55f7f4a81204cc3f3b43e770703ac6dbb995ef3))
* **deliberation:** close critical gaps in committee voting and prompts ([d663b21](https://github.com/5dlabs/cto/commit/d663b212283236dca12f0a16ad4d25374ff37e78))
* **deliberation:** fix dmPolicy from invalid 'block' to 'disabled' ([ac152dc](https://github.com/5dlabs/cto/commit/ac152dc3286fa388eb596933dd4953f2b3e84214))
* **deliberation:** set heartbeat=30s on all 7 debate agents ([43387e2](https://github.com/5dlabs/cto/commit/43387e2700b1e10aef057ba2b3053423f475af0a))
* disable acp plugin install for deliberation agents ([4eea796](https://github.com/5dlabs/cto/commit/4eea79669aea5c7be5d80ed063a3e7b77b362300))
* disable WireGuard to unblock Mayastor hostNetwork connectivity ([#4398](https://github.com/5dlabs/cto/issues/4398)) ([fca9a40](https://github.com/5dlabs/cto/commit/fca9a40b88136b842fbe7f141854852b0810611a))
* disable xAI plugin + web search, add SWARM path hints ([607aae5](https://github.com/5dlabs/cto/commit/607aae51af2841b1af0d72ac7f6e2c1a8e12be5d))
* **dns:** switch CoreDNS to public DNS after Talos firewall blocked port 53 ([8907965](https://github.com/5dlabs/cto/commit/89079657e1d3268c7e02470dd0500338e98250ba))
* enable Cilium native routing for hostNetwork pod connectivity ([#4395](https://github.com/5dlabs/cto/issues/4395)) ([631a32a](https://github.com/5dlabs/cto/commit/631a32a80017c7d17d9f0a4d6510e4f386403e73))
* exclude twingate apps from platform-apps glob (managed by networking-apps) ([f2f1d4f](https://github.com/5dlabs/cto/commit/f2f1d4f4cffd6c1b77ceee51b95e0a79e50dd74d))
* headscale TunnelBinding uses noTlsVerify directly in spec (not under originRequest) ([9565f80](https://github.com/5dlabs/cto/commit/9565f8069e57214dfd84eccbfe2418c7abaa9e87))
* heartbeat "off" invalid duration — use 87600h across all deliberation agents ([7b9849c](https://github.com/5dlabs/cto/commit/7b9849c5597b031ae581fc2e9aa53a16cf0c21e0))
* heartbeat 87600h overflows 32-bit setTimeout — use 500h (~3 weeks) ([9ed6487](https://github.com/5dlabs/cto/commit/9ed6487da9757f532a3319e911f8a5b36167ff1c))
* ignore ClusterPolicy spec/status drift in nvidia-gpu-operator ([f13d989](https://github.com/5dlabs/cto/commit/f13d98913321e8cc8080637c4a8092db27733f92))
* ignore HTTPRoute status in cto app (public-gateway not yet deployed) ([01dd660](https://github.com/5dlabs/cto/commit/01dd6608beef860f6a112e59900a2a9419dbdf33))
* ignore image-updater helm.parameters drift in platform-apps and networking-apps ([3e52d81](https://github.com/5dlabs/cto/commit/3e52d813be4c4081118213266c264915a916afac))
* ignore namespace label/annotation drift in platform-namespaces ([d2ccfa1](https://github.com/5dlabs/cto/commit/d2ccfa11bc6fc9ce86f4e1b1f73a8b5afe0279c5))
* ignore normalized twingate access crd fields ([#4506](https://github.com/5dlabs/cto/issues/4506)) ([39395b6](https://github.com/5dlabs/cto/commit/39395b68b3c341b034b7dbefb37022c16c91ad8b))
* **images:** replace broken nodesource with official Node.js tarballs, update glab ([#4575](https://github.com/5dlabs/cto/issues/4575)) ([77e8cb0](https://github.com/5dlabs/cto/commit/77e8cb03fb47fa8b66fd2b88f7c0e632dbdb1f5d))
* increase MCP server initialization timeout from 8-10s to 45-60s ([faa9787](https://github.com/5dlabs/cto/commit/faa978767712404e2777ebf5c81f261b4ffb8c50))
* **infra:** remove Claude Code from runtime, add 1Password to minimal, restore ARC runners for EKS ([#4431](https://github.com/5dlabs/cto/issues/4431)) ([4617ab5](https://github.com/5dlabs/cto/commit/4617ab5bc05636bacb155cf7931ebdf62d25acd1))
* **intake-agent:** add NATS_URL env fallback for CodeRun pod execution ([4be0680](https://github.com/5dlabs/cto/commit/4be0680e37dbc921dd463de09354332ce1ee4049))
* **intake-agent:** error_type, Operation union, dependencies, expand-task retry ([#4451](https://github.com/5dlabs/cto/issues/4451)) ([7362f6e](https://github.com/5dlabs/cto/commit/7362f6eaa84749c2dd07562b5067d8b44a8fe832))
* **intake-agent:** lazy-initialize Claude provider to fix --help output ([#4272](https://github.com/5dlabs/cto/issues/4272)) ([d0d3f1c](https://github.com/5dlabs/cto/commit/d0d3f1c8ff475c8dc9afcf919abb676b490fca74))
* **intake-agent:** resolve all 53 TypeScript type errors ([db19d8f](https://github.com/5dlabs/cto/commit/db19d8f90e92e27aa8886f5cbd7fe8eb9acd4a1d))
* **intake:** add .intake/checkpoints/ to preflight cleanup ([9980a8e](https://github.com/5dlabs/cto/commit/9980a8e40611a1debf6c3b6faef2b72f59aa952b))
* **intake:** add 120s timeout to search-patterns step ([d1c483b](https://github.com/5dlabs/cto/commit/d1c483bb973a351d2ab9bc10a1e0db47615d5546))
* **intake:** add timeout + stub fallback for create-linear-project ([15e2be3](https://github.com/5dlabs/cto/commit/15e2be3c47350d9eb2a01a3e094bcf86d97980e9))
* **intake:** auto-truncate large repomix output + fix curl E2BIG ([fe5a6ef](https://github.com/5dlabs/cto/commit/fe5a6ef62f23e478e5ae9ae1931ce4978c73d7d1))
* **intake:** clean ALL .tasks/ subdirs in preflight + cross-project guard ([1896261](https://github.com/5dlabs/cto/commit/189626101f27835487e1a1c29dc93167adb3f59c))
* **intake:** clean stale artifacts in preflight to prevent cross-run contamination ([36467a0](https://github.com/5dlabs/cto/commit/36467a0a65aebdf72ab93a7245a9953eadd64f82))
* **intake:** correct file names for disk-based artifact counting ([4bb45ea](https://github.com/5dlabs/cto/commit/4bb45ea9dfd1231933c70269496c91080ce65d76))
* **intake:** extract truncation to separate Python script ([9c6afc1](https://github.com/5dlabs/cto/commit/9c6afc1d6c396e3d057b799327b6d7b5809fa8f3))
* **intake:** harden e2e quick-run workflow ([#4548](https://github.com/5dlabs/cto/issues/4548)) ([39b27ef](https://github.com/5dlabs/cto/commit/39b27ef565f1563832abc512844d79dc0e9f4924))
* **intake:** include transcript JSON in release when MP3 unavailable ([702a7c5](https://github.com/5dlabs/cto/commit/702a7c5dfce7fee22c90e93864168e5989d33620))
* **intake:** make workspace cleanup fault-tolerant (chmod before rm) ([#4460](https://github.com/5dlabs/cto/issues/4460)) ([d8057b7](https://github.com/5dlabs/cto/commit/d8057b70a8ab9d77214e531e017f23eb0427cef8))
* **intake:** morgan agent → grok cli + claude-sonnet-4-20250514 model for e2e test ([88ff498](https://github.com/5dlabs/cto/commit/88ff4982fbbf7d7e570bc08234b17bfbc39ce689))
* **intake:** parser misses DECISION_POINT blocks wrapped in markdown bold ([d17a85e](https://github.com/5dlabs/cto/commit/d17a85e217c685732bdccd4a9f516fd90bb22148))
* **intake:** pipeline resilience — run_id propagation, quality-gate fallback, gateway restart ([#4629](https://github.com/5dlabs/cto/issues/4629)) ([6c0af8f](https://github.com/5dlabs/cto/commit/6c0af8f30fdddd4d79b5cdf0357f2dcd6bc09bcd))
* **intake:** prevent full source code leak into deliberation context ([d44259a](https://github.com/5dlabs/cto/commit/d44259ac98361ea5a7fa5a99609d048421fcffe2))
* **intake:** resilient gateway health check + disk-based step outputs ([877d8f3](https://github.com/5dlabs/cto/commit/877d8f3ea8ae35f03efed56efee175f04020bc01))
* **intake:** respect design_brief_path when deliberate=true; hard error on missing brief ([000e741](https://github.com/5dlabs/cto/commit/000e74154af03410c1cbc15b52e51157f767a214))
* **intake:** set ALL model tiers + committee to github-copilot/claude-opus-4.6 ([e2adcd8](https://github.com/5dlabs/cto/commit/e2adcd877b0569731af72161d1150a66ddafcad7))
* **intake:** skip workspace PR when target_repo_local_path is set ([#4632](https://github.com/5dlabs/cto/issues/4632)) ([35a312b](https://github.com/5dlabs/cto/commit/35a312b9b17410b4eb9a8ef578e54e3be8c3800a))
* **intake:** split acceptance criteria + reference acceptance.md ([7aa5536](https://github.com/5dlabs/cto/commit/7aa5536583ba5cdd7949785e0c4dd171646e4ccd))
* **intake:** use temp files for large repo analysis to avoid E2BIG ([d2accfe](https://github.com/5dlabs/cto/commit/d2accfe108a607b9ae28dc766390fcc7a4e862a4))
* **intake:** write pack-repo/search to disk files, not stdout ([3fa604e](https://github.com/5dlabs/cto/commit/3fa604eedbf1beadb5df8de7849a419613f20d61))
* **intake:** write step outputs to disk to avoid E2BIG in verify-artifact-gates ([83675b1](https://github.com/5dlabs/cto/commit/83675b165f224b2f77ab7bb4d104f1319ce89929))
* **linear-bridge:** add LINEAR_TEAM_ID and fix missing env vars ([#4496](https://github.com/5dlabs/cto/issues/4496)) ([29b1624](https://github.com/5dlabs/cto/commit/29b16248c11196fa3217bb252d8800df8b150044))
* **lobster-voice:** add getCachedPath for render-transcript file path resolution ([0339a8f](https://github.com/5dlabs/cto/commit/0339a8fde1359797703bcf1ff07f600e1f007381))
* **loki:** switch from SeaweedFS S3 to filesystem storage ([#4373](https://github.com/5dlabs/cto/issues/4373)) ([818d568](https://github.com/5dlabs/cto/commit/818d568898746c1d6e7ebe74f889c164b33f7546))
* make gh CLI primary data source for digest, local repos fallback only ([80c259d](https://github.com/5dlabs/cto/commit/80c259dcb5ee7a61868319aa5aa5cb42bf1b6612))
* **marketing,splash:** mobile view regressions + Business Team tools/skills ([5ec9eb4](https://github.com/5dlabs/cto/commit/5ec9eb469780647a03f64a191c2ccac557b41b8d))
* **marketing,splash:** normalize pathname for nav link highlighting ([573ebd7](https://github.com/5dlabs/cto/commit/573ebd7960f5bef640ff944313f20f3e85c57417))
* **marketing:** add OpenGraph and Twitter splash image assets ([009d87a](https://github.com/5dlabs/cto/commit/009d87a77bff556ec31dbb169edaa6b2a6b3a4f6))
* **marketing:** bust social card cache with versioned OG/Twitter images ([2e62dca](https://github.com/5dlabs/cto/commit/2e62dcab0bd6f514e5079834bb8e65546935f067))
* **marketing:** cache-bust Angie avatar URL ([#4519](https://github.com/5dlabs/cto/issues/4519)) ([140c3d2](https://github.com/5dlabs/cto/commit/140c3d2d261984f17bef509485c07815df0130a7))
* **marketing:** center Meet Morgan section, remove empty right space ([7d6e25a](https://github.com/5dlabs/cto/commit/7d6e25a03072f25e24ce6bdcd1c0aaa5998a9df6))
* **marketing:** customer-facing provider copy, remove implementation details ([#4447](https://github.com/5dlabs/cto/issues/4447)) ([7f284b2](https://github.com/5dlabs/cto/commit/7f284b2edbf976c7d16e77454b6064fd7275d4a4))
* **marketing:** grid-pulse z-index so it stays behind hero content ([2b6e7ff](https://github.com/5dlabs/cto/commit/2b6e7ffee547b10992fe61e2d17ee64f32ac3d1f))
* **marketing:** improve Telegram social preview rendering ([#4448](https://github.com/5dlabs/cto/issues/4448)) ([ff412a7](https://github.com/5dlabs/cto/commit/ff412a75b932a925072b7fdb0b8056a1863eb9b0))
* **marketing:** LemonSlice widget 9:14 aspect ratio, responsive sizing, no head crop ([06d4c73](https://github.com/5dlabs/cto/commit/06d4c73be44b98ad632a1aefa631897d03040764))
* **marketing:** make social image routes static-export compatible ([#4449](https://github.com/5dlabs/cto/issues/4449)) ([306f097](https://github.com/5dlabs/cto/commit/306f097dfb97f1ea1fb684894b28b9924b50f4b7))
* **marketing:** remove on-premises provider listing ([#4450](https://github.com/5dlabs/cto/issues/4450)) ([bd8d3c0](https://github.com/5dlabs/cto/commit/bd8d3c04008d821bc1226419007cb24ea18015ee))
* **marketing:** remove Sidero/Omni infra cards ([#4426](https://github.com/5dlabs/cto/issues/4426)) ([614eefd](https://github.com/5dlabs/cto/commit/614eefd0fe2c10ab67617bcd7c4d7eca5ad21c2a))
* **mem0:** remove invalid plugin config properties ([e9f8a73](https://github.com/5dlabs/cto/commit/e9f8a73fa04b925e0d8ddec3b86fd73c83a8b2c3))
* metrics-server containerPort and defaultArgs for port 4443 ([#4393](https://github.com/5dlabs/cto/issues/4393)) ([8f46b47](https://github.com/5dlabs/cto/commit/8f46b47f5d5819fa14bc907369c0529730307902))
* metrics-server port conflict on hostNetwork ([#4392](https://github.com/5dlabs/cto/issues/4392)) ([5783b43](https://github.com/5dlabs/cto/commit/5783b438f3051611d6b31d0656fa94c131af72f9))
* move BASE_IMAGE ARG before first FROM for Docker multi-stage ([97f0ffb](https://github.com/5dlabs/cto/commit/97f0ffba94676838e1d1dcd6c77236c6001e02f6))
* move discord config into agent.messaging (not root messaging) ([020af4d](https://github.com/5dlabs/cto/commit/020af4d970752eb1ea80f1c962be12aa906d3efd))
* **nats:** add nats tool to agent allowlists and update skill guidance ([a1d35b8](https://github.com/5dlabs/cto/commit/a1d35b8dfd8c87a3641e1e553872067fb179bc9d))
* **nats:** align plugin with OpenClaw SDK conventions ([4754b84](https://github.com/5dlabs/cto/commit/4754b84748973451e99cfe443f1bad6861e2bce4))
* **nats:** use standalone createInbox() import from nats module ([6a18a38](https://github.com/5dlabs/cto/commit/6a18a38a7152794f3f486fc0e40198c073f7ce71))
* nfd master nodeSelector to use worker nodes; twingate retry with GHCR creds ([e3d20fb](https://github.com/5dlabs/cto/commit/e3d20fb3ef11199b842cab36f7dfcc06bdb12520))
* **ntfy:** update HTTP listen port to 8080 in configmap and deployment manifests ([bd735f5](https://github.com/5dlabs/cto/commit/bd735f5d12dcd88b69d8022382758ac86ce68c0b))
* **observability:** add cilium-operator Prometheus scrape target ([1c781dc](https://github.com/5dlabs/cto/commit/1c781dc20c81c113c1d4482b700534c56d3741ce))
* **observability:** fix Solana dashboard logs panel and sync ETA ([9c9baa1](https://github.com/5dlabs/cto/commit/9c9baa1458577f4444f6ec091ff1aa757f5a0283))
* **observability:** raise Agave memory limit and fix Cilium dashboards ([6d4de40](https://github.com/5dlabs/cto/commit/6d4de400ccd91a68ff4164108ff30082fd79e278))
* **observability:** remove Label_Keys from Fluent Bit Loki output ([0d080f3](https://github.com/5dlabs/cto/commit/0d080f35378151398e4276bea807891855a0b96e))
* **observability:** split Solana logs into Agave and Exporter panels ([6f94a7b](https://github.com/5dlabs/cto/commit/6f94a7b926e83ef254ca3943bb03388419a6c6ca))
* openclaw-agent image tag v prefix (2026.2.12 → v2026.2.12) ([b3031f1](https://github.com/5dlabs/cto/commit/b3031f1144a4cdf54918bf35593fb80c91da2f29))
* **openclaw-agent:** add gateway.controlUi to configmap template ([a3b4b8a](https://github.com/5dlabs/cto/commit/a3b4b8ad254fe0cfc925f86893f8686f14406753))
* **openclaw-agent:** fix git auth and mem0 telemetry for long-running agents ([159d6ff](https://github.com/5dlabs/cto/commit/159d6ffb43daf6408ae9ec06350afdc4e2d7f424))
* **openclaw-agent:** nil-safe controlUi template access ([1481b91](https://github.com/5dlabs/cto/commit/1481b91042e7787004beeafd051dfada2b01ab1e))
* **openclaw-agent:** reduce CPU requests to fit 14 agents on worker ([a1ff714](https://github.com/5dlabs/cto/commit/a1ff71460d3edef003ddef29d7533f229030441e))
* **openclaw:** add RBAC and kubectl for kaniko sidecar exec ([#4376](https://github.com/5dlabs/cto/issues/4376)) ([9e20498](https://github.com/5dlabs/cto/commit/9e204985e537c95f96fc988a0b5fc618fe58d84e))
* **openclaw:** correct kaniko debug image tag to :debug ([#4369](https://github.com/5dlabs/cto/issues/4369)) ([dbd08aa](https://github.com/5dlabs/cto/commit/dbd08aaa70257ea87b4e664fa4b8d7a6a1d874b2))
* **openclaw:** correct Loki URL and add informational pod labels ([#4370](https://github.com/5dlabs/cto/issues/4370)) ([4fe5ce5](https://github.com/5dlabs/cto/commit/4fe5ce5d2ba9aa0d9826dd688d4301e69c9f4dda))
* **openclaw:** correct repoURL and chart path in ArgoCD applications ([#4365](https://github.com/5dlabs/cto/issues/4365)) ([29e64cd](https://github.com/5dlabs/cto/commit/29e64cdac81396de4ec377c4c557a549bb1838e7))
* **openclaw:** fix version extraction from release name ([4db156b](https://github.com/5dlabs/cto/commit/4db156b804c9fb737143c445a9e4b06ab8152c23))
* **openclaw:** move info fields from labels to pod annotations ([#4372](https://github.com/5dlabs/cto/issues/4372)) ([767fc77](https://github.com/5dlabs/cto/commit/767fc776a443a1aa077ee9fb1c4450da3e483cec))
* **openclaw:** remove unsupported config keys and fix kaniko sidecar ([#4367](https://github.com/5dlabs/cto/issues/4367)) ([91f70ea](https://github.com/5dlabs/cto/commit/91f70eaf547cfcc25af9d4b321b22c167a695f4c))
* **openclaw:** remove unsupported group:memory from agent tools allow lists ([#4386](https://github.com/5dlabs/cto/issues/4386)) ([0ad8144](https://github.com/5dlabs/cto/commit/0ad8144bd54e88549fb85ca2f771768fb7c67067))
* **openclaw:** use correct URL format for OpenClaw releases ([a7d9e28](https://github.com/5dlabs/cto/commit/a7d9e28c35a4a34c827c0be53713129203f33954))
* otel skipSchemaValidation, seaweedfs nodeSelector removed, twingate OCI repo in AppProject ([c564d79](https://github.com/5dlabs/cto/commit/c564d79468a1d576537cd5b506a58efb4bfaf986))
* patch TunnelBinding CRD to preserve unknown fields (headscale originRequest) ([f1059aa](https://github.com/5dlabs/cto/commit/f1059aaa90427ec85ee26bbdde857a2472e9aa23))
* per-agent messaging secrets so DISCORD_TOKEN env var carries correct token ([e5486b7](https://github.com/5dlabs/cto/commit/e5486b741bea10823c108ea612399d3831bfd1b8))
* permanent home dir fix in init container + Dockerfile ([f04f09a](https://github.com/5dlabs/cto/commit/f04f09a2c9b98501878b902f1045ecf2fec56dc7))
* persist ArgoCD ignoreDifferences for cert/CRD/namespace drift (OVH cluster) ([fae8dc6](https://github.com/5dlabs/cto/commit/fae8dc64749fe4693e28639162d66c41042ba0a4))
* **pitch-deck:** accurate traction — pilot phase, pipeline not closed revenue ([4cc7e10](https://github.com/5dlabs/cto/commit/4cc7e10f7c3f5b6a5c8c50821c8f3a5c09dc3007))
* **pitch-deck:** move Model mix from Problem to CTO slide ([#4558](https://github.com/5dlabs/cto/issues/4558)) ([61e780f](https://github.com/5dlabs/cto/commit/61e780fa18c5ad233daae5518d77f0b1ac4266d0))
* **pitch-deck:** replace "tax" metaphor with markup/overhead/fees ([#4560](https://github.com/5dlabs/cto/issues/4560)) ([b984429](https://github.com/5dlabs/cto/commit/b98442953a34ea98e301eb49d6bb0bafadd98ef2))
* **pm:** scale PM server back to one replica ([#4420](https://github.com/5dlabs/cto/issues/4420)) ([caad801](https://github.com/5dlabs/cto/commit/caad801ad74f158150c2524ff41d5275e44b7757))
* **pm:** use claude as default intake CLI (was research→grok which has no configured image) ([e6649f1](https://github.com/5dlabs/cto/commit/e6649f19ce31b1cccac864c6259932009bd1b577))
* Preserve GitHub App private key for agent access ([#4316](https://github.com/5dlabs/cto/issues/4316)) ([66396d0](https://github.com/5dlabs/cto/commit/66396d0ee6c6c001a7391f5cb659f9ebf2d39946))
* qdrant client version override + git awareness instructions ([fb2dc10](https://github.com/5dlabs/cto/commit/fb2dc1057df999118039c1b5e13e43bcc12eeaf5))
* **release:** add play-monitor binary to release artifacts ([563059d](https://github.com/5dlabs/cto/commit/563059d2443a5805163a9a4e69255a0b1f0bba0a))
* remove 'Proprietary Operating Stack' from splash hero badge ([#4499](https://github.com/5dlabs/cto/issues/4499)) ([1aedb28](https://github.com/5dlabs/cto/commit/1aedb286e8da73b875eb0e93931dee3fa5cbd69c))
* remove backslashes from single-quoted jq filter in deliberation ([de3d2c7](https://github.com/5dlabs/cto/commit/de3d2c79743daaffad3a797c998dda262fae2789))
* remove broken Twingate CRD manifests (TwingateRemoteNetwork doesn't exist) ([#4454](https://github.com/5dlabs/cto/issues/4454)) ([6e4c1a4](https://github.com/5dlabs/cto/commit/6e4c1a42c530a77507fe3cf24c526b902fb5f46e))
* remove duplicate COLOSSEUM_COPILOT_API_BASE from extraSecretEnv ([da3a3b4](https://github.com/5dlabs/cto/commit/da3a3b417032aa9afe1854efd58668a8f938a23d))
* remove redundant Team nav link — agents already on homepage ([#4505](https://github.com/5dlabs/cto/issues/4505)) ([9b34bd7](https://github.com/5dlabs/cto/commit/9b34bd70b4073e8471b7071a62e44be28caaa89d))
* remove zhipu provider from gateway config ([f85b3b5](https://github.com/5dlabs/cto/commit/f85b3b5309c564ddc51e32377f6ff943732a27fd))
* rename Managed Foundation → 5D Runtime, rewrite cloud parity with confidence ([#4501](https://github.com/5dlabs/cto/issues/4501)) ([684a9d3](https://github.com/5dlabs/cto/commit/684a9d3127187f784fdd9e7324d7595ddb5e366b))
* replace 'proprietary' with 'in-house', remove negative framing ([#4502](https://github.com/5dlabs/cto/issues/4502)) ([11c53ae](https://github.com/5dlabs/cto/commit/11c53aec263a72d3721b86077e81b052267f4bf8))
* **research:** add gh CLI to image for intake workflow ([640f36c](https://github.com/5dlabs/cto/commit/640f36c28896ce455a6d8a0f92d2aeaee99c86e6))
* **research:** add intake CLI binary to research image ([0ad592c](https://github.com/5dlabs/cto/commit/0ad592cdf9d4a69fe13cef605cf008d6ac6c860c))
* **research:** add jq and curl to runtime image ([d283cb8](https://github.com/5dlabs/cto/commit/d283cb87f7c95c228f22600c9869f666ace92897))
* **research:** add python3 and nodejs to runtime-base image ([9a0cf4e](https://github.com/5dlabs/cto/commit/9a0cf4eea5af2a3f6d3182802bd504a351880c89))
* **research:** bake default NATS_URL into image for OVH CodeRun pods ([2b22832](https://github.com/5dlabs/cto/commit/2b2283293d9386751f79112d61d6659116605d3b))
* **research:** make intake verify non-fatal, add ldd diagnostic ([1f013e4](https://github.com/5dlabs/cto/commit/1f013e4f1463e5bded1aa71aa079a4d39833008e))
* resolve CSS layer specificity, blur variants, and missing magenta theme color ([cd6cdb9](https://github.com/5dlabs/cto/commit/cd6cdb990ba0f4b7df23c62f70117066606e27b8))
* resolve duplicate --prd flag and stderr corruption in intake script ([280e9e5](https://github.com/5dlabs/cto/commit/280e9e5738b7c012ffcc1ee4bba91a2d33a6a573))
* restore lifecycle-test dir and enable deliberation for alerthub e2e ([#4452](https://github.com/5dlabs/cto/issues/4452)) ([7d15e4c](https://github.com/5dlabs/cto/commit/7d15e4cce708c3f115ab104612a2ac4038d34427))
* revert discord to top-level messaging (agent.messaging causes dm.policy panic) ([80520e7](https://github.com/5dlabs/cto/commit/80520e75c0b81acdaa22fbffd901671eb8962e8a))
* seaweedfs tolerations type, otel featureGates schema, twingate-operator OCI chart ref ([cec0b6d](https://github.com/5dlabs/cto/commit/cec0b6d9f9a08f90b467642527c902c96d180dc5))
* seaweedfs volume 100Gi-&gt;50Gi (mayastor pools max ~60GiB free per node) ([a7e44d8](https://github.com/5dlabs/cto/commit/a7e44d8f745518f2c46aab2fbe6093b3845217c3))
* **secrets:** update ExternalSecret with all 16 deployed bot tokens ([#4381](https://github.com/5dlabs/cto/issues/4381)) ([e1dfd3a](https://github.com/5dlabs/cto/commit/e1dfd3a3a1d37d60e53c78f276123bd0ee6ec31f))
* set node user homedir to /workspace in /etc/passwd ([5b427c3](https://github.com/5dlabs/cto/commit/5b427c3fc66354bbb2ec1c058a26ec3a0b942f38))
* simplify cto-play Docker build stage ([4537b8a](https://github.com/5dlabs/cto/commit/4537b8afc2e335cd78335bec8a2ab0dd2cba2e29))
* skip-reconcile aws-load-balancer-controller on OVH bare metal ([138234f](https://github.com/5dlabs/cto/commit/138234f6f2f1b0994edc439e7d81051ce5b2404f))
* skip-reconcile Mayastor on OVH - using local-path storage overlay ([e89b2dc](https://github.com/5dlabs/cto/commit/e89b2dc2d9887fdb912365d14a3c92a8979ecafb))
* **splash,marketing:** boost GridPulse visibility + match preview intensity ([30ca3a9](https://github.com/5dlabs/cto/commit/30ca3a9b772fc0dc6e701d88d63b6270e64dff77))
* **splash,marketing:** eliminate element flashing and fix LCP performance ([#4466](https://github.com/5dlabs/cto/issues/4466)) ([95b2488](https://github.com/5dlabs/cto/commit/95b24882899aba2dc88c86b0b160a9c12900b96d))
* **splash,marketing:** eliminate GridPulse flashing and 3D depth sorting ([9e90c89](https://github.com/5dlabs/cto/commit/9e90c89c4023a38e8f7295fdad56b66cb69c3f4e))
* **splash,marketing:** fix GridPulse visibility, remove Healer agent, SSR perf ([b401e9b](https://github.com/5dlabs/cto/commit/b401e9ba8d7740d2d9e131d74f7f8ed6df0141f4))
* **splash,marketing:** GridPulse z-index stacking + will-change GPU hints ([ccb860f](https://github.com/5dlabs/cto/commit/ccb860f9073ffe358eb9d35cc3b5fd464d3c39aa))
* **splash,marketing:** keep GridPulse lines in the background ([#4468](https://github.com/5dlabs/cto/issues/4468)) ([875eb72](https://github.com/5dlabs/cto/commit/875eb7216e1b6b328b3a552e8a94578562e92f6e))
* **splash,marketing:** move GridPulse inside ShiftDimensionsWrapper ([9107f6e](https://github.com/5dlabs/cto/commit/9107f6e668bd7879bc8c4f53b0738c8b7d38846a))
* **splash,marketing:** prevent grid foreground overlay from hiding content tiles ([#4470](https://github.com/5dlabs/cto/issues/4470)) ([b074809](https://github.com/5dlabs/cto/commit/b074809a3f266318f09400121f1f5c40dcace0f8))
* **splash,marketing:** remove foreground grid lines, restore pulse animation ([#4478](https://github.com/5dlabs/cto/issues/4478)) ([5e4ab65](https://github.com/5dlabs/cto/commit/5e4ab655819795c9ebf12e2c6a1b7f9a96b50d5b))
* **splash,marketing:** remove pulse, soften grid, fix investor page ([#4479](https://github.com/5dlabs/cto/issues/4479)) ([d7b6cad](https://github.com/5dlabs/cto/commit/d7b6cadcbb0a8e0931cadb160999db9d887a4a6d))
* **splash,marketing:** remove scroll animation and fix compositing ([#4476](https://github.com/5dlabs/cto/issues/4476)) ([a5c5cf0](https://github.com/5dlabs/cto/commit/a5c5cf0a19f11fef1ef0e46ea734ef109283fe81))
* **splash,marketing:** restore grid presence with rare foreground passes ([#4469](https://github.com/5dlabs/cto/issues/4469)) ([644fd23](https://github.com/5dlabs/cto/commit/644fd23f2afcc5e81dccb54509666564df371099))
* **splash,marketing:** stop post-render hiding and stale-cache flicker ([#4467](https://github.com/5dlabs/cto/issues/4467)) ([9970a80](https://github.com/5dlabs/cto/commit/9970a8065ffe33a232e06753798286ebc632cdf2))
* **splash,marketing:** strip all content animations to eliminate flashing ([#4477](https://github.com/5dlabs/cto/issues/4477)) ([0de36b5](https://github.com/5dlabs/cto/commit/0de36b5067038d4f32c4ea5520f4a640c0fe4d2e))
* **splash:** add Telegram-friendly OG image ([#4428](https://github.com/5dlabs/cto/issues/4428)) ([917137c](https://github.com/5dlabs/cto/commit/917137c242ee66261e70fe26d5520ad4025ed9de))
* **splash:** button import from @radix-ui/react-slot not radix-ui ([6f90751](https://github.com/5dlabs/cto/commit/6f907511f79e2d0da2757f3538b54af7df9d1c14))
* **splash:** consistent investor CTAs + Motion hover expand ([#4481](https://github.com/5dlabs/cto/issues/4481)) ([5269dc5](https://github.com/5dlabs/cto/commit/5269dc548aacbbedefb20e112c2b32b87867883a))
* **splash:** replace [@apply](https://github.com/apply) glass-bg with inline CSS for Tailwind compat ([95bf65d](https://github.com/5dlabs/cto/commit/95bf65d501e40d1549682213dfb637aff9d7688a))
* **splash:** restore dark theme so text is visible ([39694ed](https://github.com/5dlabs/cto/commit/39694edd8f23377870a0156c58e7cd3dd098a217))
* **splash:** strip Framer Motion from all sub-pages to eliminate flash ([#4475](https://github.com/5dlabs/cto/issues/4475)) ([02d8245](https://github.com/5dlabs/cto/commit/02d8245d26402696727590cb911e422cd52c39cf))
* **splash:** use Slot not Slot.Root from @radix-ui/react-slot ([a57cf1a](https://github.com/5dlabs/cto/commit/a57cf1ae8c33dc698ff51844a75ddf0e03b82f2c))
* **swarm:** add Fireworks provider + OpenAI OAuth for model access ([7cfed7e](https://github.com/5dlabs/cto/commit/7cfed7ec609738f132f9eb58c505b16a78316f08))
* **swarm:** align ArgoCD values with OpenClaw schema ([63e80b8](https://github.com/5dlabs/cto/commit/63e80b841324685be30679a92f0939f2f2c3266f))
* **swarm:** correct cliBackends schema (command+args+output) ([d09a509](https://github.com/5dlabs/cto/commit/d09a509627bbc74e120ee4a3441c5d0a3ca2ec20))
* **swarm:** map provider API key secret keys to actual secret names ([9f30f39](https://github.com/5dlabs/cto/commit/9f30f393dcc7f51acdffebf66945557a7c26c85d))
* **swarm:** move discord config to chart's expected path ([6bbbe70](https://github.com/5dlabs/cto/commit/6bbbe704c882f301a0bc33b95d02d2c67c42cf89))
* switch loki MCP from ghcr.io image (GHCR auth denied) to npx @elad12390/loki-mcp ([1a5ac57](https://github.com/5dlabs/cto/commit/1a5ac5750f137a873ed3b9c1913fd53156eaad1e))
* switch storage overlay from AWS EBS to local-path for OVH deployment ([c398ca4](https://github.com/5dlabs/cto/commit/c398ca4c82d3490f8c88d085794e8ff8bb0b31b4))
* teach agent to use gh CLI for any repo, not just cloned ones ([41eb16b](https://github.com/5dlabs/cto/commit/41eb16b255f6907cf7b97e170e122e7d716d8ec7))
* **telemetry:** add acp-cli promtail jobs for all CLI log directories ([#4494](https://github.com/5dlabs/cto/issues/4494)) ([93595a0](https://github.com/5dlabs/cto/commit/93595a0199479da6d52b539a1c0c0c0485553642))
* **template:** make workspace cleanup non-fatal (permission denied fallback) ([dc8125f](https://github.com/5dlabs/cto/commit/dc8125f56884c34b17bc43d9ddc380496edbf9d5))
* **tools:** correctly handle SYSTEM_CONFIG_PATH as file path ([a5375e6](https://github.com/5dlabs/cto/commit/a5375e62f7cde7014d6e621683af3cd89a25a33b))
* **tools:** disable broken MCP servers causing initialization failures ([#4331](https://github.com/5dlabs/cto/issues/4331)) ([c2b0502](https://github.com/5dlabs/cto/commit/c2b05023305faf53ef87fed096c85f5f6ffd1c9c))
* **tools:** release stdio semaphore permit immediately after init ([#4304](https://github.com/5dlabs/cto/issues/4304)) ([77b70cd](https://github.com/5dlabs/cto/commit/77b70cd83544567e828ce5aac9fadc36d761ff33))
* **trading:** align secrets structure with remote openclaw chart ([0225cd4](https://github.com/5dlabs/cto/commit/0225cd4a44aeb47e0b4c1b9adca98ce5aa4b11ff))
* **trading:** bump memory limits to 1Gi and fix Mode A heartbeat instructions ([7aaee97](https://github.com/5dlabs/cto/commit/7aaee97555bbcbfad37131a4f19773623a70f8c9))
* **trading:** bump memory limits to 2Gi — 1Gi still OOMKilling Mode A agents ([a5bf166](https://github.com/5dlabs/cto/commit/a5bf1664e0c6d3899febf282549ade3888c61ba3))
* **trading:** correct repo URLs, OpenBao paths, disable hooks ([eb77c03](https://github.com/5dlabs/cto/commit/eb77c03169fdce5898c391bfc6d56bdbf68ad97f))
* **trading:** disable Yellowstone gRPC geyser plugin pending crash fix ([2f438a3](https://github.com/5dlabs/cto/commit/2f438a3f54f1011f6d0c6f26dbbeaaa0677aa1d7))
* **trading:** fix secrets structure blocking ArgoCD sync ([d2c5af3](https://github.com/5dlabs/cto/commit/d2c5af301cbe90da3f2358827d7d9b19a59210af))
* **trading:** mirror openclaw-api-keys pattern, add 1Password token ([c1a7250](https://github.com/5dlabs/cto/commit/c1a7250075252a7641ffe301f8f8138a3c6ea015))
* **trading:** override heartbeat model to Haiku (chart defaults to minimax) ([9bdaf61](https://github.com/5dlabs/cto/commit/9bdaf619572d26ab56cfdd6d54e7723b42dda320))
* **trading:** register Haiku 4.5 model in anthropic provider + fix fallback IDs ([fdda146](https://github.com/5dlabs/cto/commit/fdda146dbf8d4a5b3630cdc7affbaa1d56892391))
* **trading:** resolve PR [#4531](https://github.com/5dlabs/cto/issues/4531) conflicts — add simmer/helius/polymarket secrets and env vars ([#4532](https://github.com/5dlabs/cto/issues/4532)) ([57052a9](https://github.com/5dlabs/cto/commit/57052a93364abaa1205741ab38621b0f68b4add1))
* **trading:** set Helm releaseNames in ArgoCD apps to match existing releases ([78bca01](https://github.com/5dlabs/cto/commit/78bca010a9ae31fad3375e9d5fec07f8f32fba0c))
* **trading:** use pre-built asymmetric-research solana-exporter image ([a46fb8c](https://github.com/5dlabs/cto/commit/a46fb8c9a07608ffd49edfb97edae254ec0e7102))
* treat pending WaitForFirstConsumer PVCs as Healthy in ArgoCD ([11f002f](https://github.com/5dlabs/cto/commit/11f002f05cc3679021a2587b13a768fe60e37f61))
* twingate-operator version v0.28.0 -&gt; 0.28.0 (OCI tags have no v prefix) ([ffda650](https://github.com/5dlabs/cto/commit/ffda65007b7dde4116fc577a28c49ec837987fe1))
* **twingate:** allowEmpty=true, ignoreDifferences for stale v1alpha1 resources ([e8696d1](https://github.com/5dlabs/cto/commit/e8696d138a89cb310a089d25e6efd868b9579c91))
* **twingate:** empty kustomization - remove broken v1alpha1 CRD manifests ([2ed83c2](https://github.com/5dlabs/cto/commit/2ed83c204d5d695e01caa49748ca2f4f0516a01c))
* update ingress-nginx externalIPs to OVH node IPs ([92f0572](https://github.com/5dlabs/cto/commit/92f0572f656be778f962d85cf94be17755a0ffd4))
* use plugins.deny to block xAI plugin at runtime level ([fc2d6d8](https://github.com/5dlabs/cto/commit/fc2d6d8b50f3d5926cf2c59af4bc2393e9e1cfcc))


### ⚡ Performance

* **splash,marketing:** add Service Worker for instant cached loads ([af17a9d](https://github.com/5dlabs/cto/commit/af17a9dd66a6bfe971334650e92ddf21e75c96fb))
* **splash,marketing:** compositor-only GridPulse animations ([3d944ca](https://github.com/5dlabs/cto/commit/3d944ca5cff56573f2caf8149c90ec8773b549ca))


### ♻️ Refactoring

* organize research docs and add Grok MCP server ([#4359](https://github.com/5dlabs/cto/issues/4359)) ([7aae2a2](https://github.com/5dlabs/cto/commit/7aae2a2e934fd6c219161c57477a2690148331be))


### 📚 Documentation

* add Cloudflare tunnel setup guide for intake testing agent ([3235987](https://github.com/5dlabs/cto/commit/32359870b59dae249eb2cb3e85c9934c4ea6a7d1))
* add Solana K8s architecture and operator spec ([94d332a](https://github.com/5dlabs/cto/commit/94d332a3a9cd5c3c894af3b5853ba0f749b3fea1))
* add Talos + Cilium responsibilities & overlap section ([c524d7e](https://github.com/5dlabs/cto/commit/c524d7e7ef0ef34783e6d5fa87a8a3aba190e225))
* **skill:** add bridge communication section and new intake-util commands to intake-pipeline skill ([cf678f5](https://github.com/5dlabs/cto/commit/cf678f525181b30803d7ced6490c84636a75694d))
* Solana hackathon on-chain agent ideas brainstorm ([#4604](https://github.com/5dlabs/cto/issues/4604)) ([835f3ca](https://github.com/5dlabs/cto/commit/835f3cac992dacc715d6688552bc3aa5b894392d))
* **swarm:** note SWARM CLI pre-authenticated as [@kaseonedge](https://github.com/kaseonedge) ([438e0dd](https://github.com/5dlabs/cto/commit/438e0ddeed8c405845aca64bd447f3c84702301f))
* trading cluster — clarify dual-homed direct public IP model ([ceeaadf](https://github.com/5dlabs/cto/commit/ceeaadf0a98fd2e6581dec4a30757bab24107c46))
* trading cluster — Cloudflare tunnels out, Twingate throughout ([3dc5469](https://github.com/5dlabs/cto/commit/3dc54697d47167fa9d215091dd8495325a6a78e4))
* trading cluster — resolve open decisions, add Transit implementation ([c2426cc](https://github.com/5dlabs/cto/commit/c2426cc431095aa74dd60eed6a7a47eff93a7482))
* trading cluster architecture & security plan ([d6c91a0](https://github.com/5dlabs/cto/commit/d6c91a0905c32802df5d8145bc00b4d4bd6b7563))


### 🔧 Maintenance

* add development notice and clean up stale root markdown files ([91d84d6](https://github.com/5dlabs/cto/commit/91d84d67b4512a0822d380b59bb8e18384b95a9e))
* archive Argo Workflows and Argo Events infrastructure ([#4389](https://github.com/5dlabs/cto/issues/4389)) ([fc7d6ce](https://github.com/5dlabs/cto/commit/fc7d6ce4f13190783954139ed71425b5d1acae95))
* **cilium:** re-enable argocd management ([6ea452f](https://github.com/5dlabs/cto/commit/6ea452fbabd9d3c733ba4511756cbed9392252a5))
* **ci:** restore marketing and splash deploy workflows ([#4516](https://github.com/5dlabs/cto/issues/4516)) ([5f202e0](https://github.com/5dlabs/cto/commit/5f202e0751d41142468a8f3f9a70566af9aaadec))
* consolidate worktrees and merge trading-agents infra ([#4541](https://github.com/5dlabs/cto/issues/4541)) ([6e5c20a](https://github.com/5dlabs/cto/commit/6e5c20a187152672bb45e9a32ac578b6383e940c))
* **deps:** bump dialoguer from 0.11.0 to 0.12.0 ([#4410](https://github.com/5dlabs/cto/issues/4410)) ([3012767](https://github.com/5dlabs/cto/commit/301276714ff4db7c596233eef716e4b451560279))
* **deps:** bump kube from 0.93.1 to 0.98.0 ([#4233](https://github.com/5dlabs/cto/issues/4233)) ([81c654d](https://github.com/5dlabs/cto/commit/81c654d084b2967db35f8312c3c8a66dcb832ab3))
* **deps:** bump mockall from 0.13.1 to 0.14.0 ([#4232](https://github.com/5dlabs/cto/issues/4232)) ([e87d787](https://github.com/5dlabs/cto/commit/e87d787431414794ac82b7c8e362ad4cb09e68f1))
* **deps:** bump opentelemetry from 0.30.0 to 0.31.0 ([#4414](https://github.com/5dlabs/cto/issues/4414)) ([6063ee4](https://github.com/5dlabs/cto/commit/6063ee4bfb8bd0ff39722403d64b1b38808edf69))
* **deps:** bump scraper from 0.24.0 to 0.25.0 ([#4413](https://github.com/5dlabs/cto/issues/4413)) ([803145c](https://github.com/5dlabs/cto/commit/803145c99fd7e55525c43679e4cb88cc26f52fe5))
* **deps:** bump the actions group with 4 updates ([#4239](https://github.com/5dlabs/cto/issues/4239)) ([c8726ef](https://github.com/5dlabs/cto/commit/c8726ef432b7f60363fd8ba7972fb12af6c52316))
* **deps:** bump the npm-minor group across 1 directory with 12 updates ([#4432](https://github.com/5dlabs/cto/issues/4432)) ([ecd32b2](https://github.com/5dlabs/cto/commit/ecd32b200eb2a254615d37af801e351a19f7a1b0))
* **deps:** bump the npm-minor group in /apps/web with 7 updates ([#4236](https://github.com/5dlabs/cto/issues/4236)) ([410b197](https://github.com/5dlabs/cto/commit/410b197e8fce0eabbefea86995da06d651175dab))
* **deps:** bump the rust-minor group across 1 directory with 14 updates ([#4512](https://github.com/5dlabs/cto/issues/4512)) ([2df6b16](https://github.com/5dlabs/cto/commit/2df6b16e1f55cb8ceeec181574fc97f78a967fa7))
* **deps:** bump tower-http from 0.5.2 to 0.6.8 ([#4235](https://github.com/5dlabs/cto/issues/4235)) ([264c20d](https://github.com/5dlabs/cto/commit/264c20df8e31946384f4b2b859e4089af2a69275))
* **discord-bridge:** trigger GHCR rebuild after package relink ([7ae8a26](https://github.com/5dlabs/cto/commit/7ae8a26504ceefa1c3e365bbd035df1c87fde33a))
* **docker:** pin OpenClaw to 2026.2.12, fix Gemini CLI build ([#4422](https://github.com/5dlabs/cto/issues/4422)) ([64afa7b](https://github.com/5dlabs/cto/commit/64afa7b491d382f559299946474fc807470e2549))
* **docker:** update Dockerfiles for agents and Gemini CLI ([#4421](https://github.com/5dlabs/cto/issues/4421)) ([ad53801](https://github.com/5dlabs/cto/commit/ad53801ad578465ba2cd5de4a8e4902269b96aad))
* enable all operators - remove skip-reconcile, fix otel issuerRef, set aws-lb clusterName ([c30fe8a](https://github.com/5dlabs/cto/commit/c30fe8a8c521280307a25898fcf12c84c8ffd152))
* **gitops:** disable bots namespace apps by default ([#4417](https://github.com/5dlabs/cto/issues/4417)) ([a2d39dc](https://github.com/5dlabs/cto/commit/a2d39dc20883299c1fc2ba49d5477380338a405b))
* **marketing:** remove Even G2 HUD images, keep product shot only ([dc8cb43](https://github.com/5dlabs/cto/commit/dc8cb4333b24f39fb35a83a14e3c25d598897a97))
* **openclaw:** remove pixel, scout, and pixel-assistant agents ([#4385](https://github.com/5dlabs/cto/issues/4385)) ([86e18d2](https://github.com/5dlabs/cto/commit/86e18d2a50479af921b3008afdf1aae45125a96a))
* **prd:** expand Hivelocity provider implementation plan ([#4418](https://github.com/5dlabs/cto/issues/4418)) ([071b881](https://github.com/5dlabs/cto/commit/071b88156f7455ed3b5b31b7a7b3c37e7ac0b23e))
* re-add skip-reconcile to twingate-operator (GHCR OCI auth unresolved) ([38b624a](https://github.com/5dlabs/cto/commit/38b624aed4daffbf26d8c33d0c1b3d2f199cad7d))
* re-disable cilium - requires k3s --flannel-backend=none (CNI conflict with flannel) ([dd390d9](https://github.com/5dlabs/cto/commit/dd390d9f450070a4d3626f66e7d17589375f6a04))
* release 0.2.37 ([#4210](https://github.com/5dlabs/cto/issues/4210)) ([3ccd534](https://github.com/5dlabs/cto/commit/3ccd534aa5d2103f6d6287aa8291bfae5c495f18))
* release 0.2.38 ([#4314](https://github.com/5dlabs/cto/issues/4314)) ([2a4867c](https://github.com/5dlabs/cto/commit/2a4867c312cecbfdc2fa00109f56759e24102e0d))
* release 0.2.39 ([#4330](https://github.com/5dlabs/cto/issues/4330)) ([17559c4](https://github.com/5dlabs/cto/commit/17559c469e5763ce0b5ba56983c8c009a3d543c7))
* release 0.2.40 ([#4333](https://github.com/5dlabs/cto/issues/4333)) ([f9ea9c2](https://github.com/5dlabs/cto/commit/f9ea9c27dd0407aa2440100cc947f607d2b589bb))
* release 0.2.41 ([#4360](https://github.com/5dlabs/cto/issues/4360)) ([c568640](https://github.com/5dlabs/cto/commit/c5686403978ae18bf365e6c7ca0f01d0590f3b90))
* release 0.2.42 ([#4363](https://github.com/5dlabs/cto/issues/4363)) ([bb2b6e3](https://github.com/5dlabs/cto/commit/bb2b6e33fc42184bddd7affe271325169936e412))
* release 0.2.43 ([#4366](https://github.com/5dlabs/cto/issues/4366)) ([08335ba](https://github.com/5dlabs/cto/commit/08335ba40ff79d3f6978fb4ff7b7a3a249e91605))
* release 0.2.44 ([#4371](https://github.com/5dlabs/cto/issues/4371)) ([08df896](https://github.com/5dlabs/cto/commit/08df896d10bcb9aaba4d9b95f73b2a602706ed0c))
* release 0.2.45 ([#4377](https://github.com/5dlabs/cto/issues/4377)) ([e445604](https://github.com/5dlabs/cto/commit/e4456047fbd11cc173b1145cbdbf64cfe3213faa))
* release 0.2.46 ([#4380](https://github.com/5dlabs/cto/issues/4380)) ([9af523c](https://github.com/5dlabs/cto/commit/9af523cbe37783272381be00b5ec2a8099d53be2))
* release 0.2.47 ([#4388](https://github.com/5dlabs/cto/issues/4388)) ([85263ea](https://github.com/5dlabs/cto/commit/85263eab34a35781f6d6813ec238b75d3f011e8e))
* release 0.2.48 ([#4390](https://github.com/5dlabs/cto/issues/4390)) ([7b04638](https://github.com/5dlabs/cto/commit/7b046380aaa60691e4c8d61d1bcd7c304446e199))
* release 0.2.49 ([#4404](https://github.com/5dlabs/cto/issues/4404)) ([720a5d4](https://github.com/5dlabs/cto/commit/720a5d4f3375d09b2ba757ee909518edd04de628))
* release 0.2.50 ([#4406](https://github.com/5dlabs/cto/issues/4406)) ([39534da](https://github.com/5dlabs/cto/commit/39534dabd9a9006fcbd6c9d70672ff3db372d183))
* release 0.2.51 ([#4440](https://github.com/5dlabs/cto/issues/4440)) ([383e339](https://github.com/5dlabs/cto/commit/383e339478b55225c145b2b06d4793ec889572df))
* release 0.2.52 ([#4570](https://github.com/5dlabs/cto/issues/4570)) ([5d1a7c5](https://github.com/5dlabs/cto/commit/5d1a7c572f0c127891fb3fcd0ba00140c44686dd))
* release 0.2.53 ([#4576](https://github.com/5dlabs/cto/issues/4576)) ([e10f89e](https://github.com/5dlabs/cto/commit/e10f89e05161f0951000491374397dd96767077c))
* remove all hardcoded agent counts across both apps ([c849531](https://github.com/5dlabs/cto/commit/c849531763a999d5f64746c22b45616f32eb9646))
* remove openclaw agent apps from CTO cluster gitops ([d4a7e70](https://github.com/5dlabs/cto/commit/d4a7e70a3377bb5ce068ce0dbae90dbd0d69a6c9))
* remove unused swarm-values.yaml (inline values used) ([15cd9a0](https://github.com/5dlabs/cto/commit/15cd9a0159a47359edb45c5a69afeda22609ddf2))
* restore pages deploy workflows + batch platform updates ([#4520](https://github.com/5dlabs/cto/issues/4520)) ([b6a1c10](https://github.com/5dlabs/cto/commit/b6a1c10b73d04fc724dec4c46c74278cf1599f4d))
* **swarm:** disable cron until manual testing complete ([c40d41a](https://github.com/5dlabs/cto/commit/c40d41a3a65ed1df4f1264a1185c66f735f411e9))
* twingate-operator - re-disable, document OCI 3-level path ArgoCD limitation ([c040d30](https://github.com/5dlabs/cto/commit/c040d303f4c371e64c1c1642c8c5d4525ff4226e))
* untrack local build artifacts and add to gitignore ([6aabbac](https://github.com/5dlabs/cto/commit/6aabbac1cb4e0c8e58e0c2140f53bae4d416afdd))
* update cron comments — daily-digest registered and enabled at 5 PM PST ([f9afbfa](https://github.com/5dlabs/cto/commit/f9afbfa875c1d205a02a3205804b1e9852ee168e))
* upgrade OpenClaw to 2026.2.6-3, set imagePullPolicy Always ([#4391](https://github.com/5dlabs/cto/issues/4391)) ([e2e3960](https://github.com/5dlabs/cto/commit/e2e396040d218e8eb9bad3ef22078541c5fac05e))


### 👷 CI/CD

* add deploy-splash workflow for Cloudflare Pages (5dlabs.ai) ([19396bb](https://github.com/5dlabs/cto/commit/19396bb53ba617bae42ec9156af4b4001912756c))
* add skills security scanning via mcp-scan on every push ([146d1b8](https://github.com/5dlabs/cto/commit/146d1b88d445fa4683e8bb44b1c3dd39f937886d))
* **controller:** temp disable cancel-in-progress to allow build to complete ([1927dc3](https://github.com/5dlabs/cto/commit/1927dc325c5527b7b65b4b1c1657cb3893fd53a8))

## [0.2.53](https://github.com/5dlabs/cto/compare/v0.2.52...v0.2.53) (2026-04-15)


### ✨ Features

* **acp:** clean image build + controller retry race fix ([accd120](https://github.com/5dlabs/cto/commit/accd1202eef8caf7f8de39c94938e0f7e858eda7))
* add activity summary table with commit/PR metrics to digest ([95c571b](https://github.com/5dlabs/cto/commit/95c571b6d9cd1d82e19c57387f06a70f81e06410))
* add cto-agents repo to monitored workspace ([efc43af](https://github.com/5dlabs/cto/commit/efc43af01e3d338b228c72b11bbacb2767be74c3))
* add cto-play launcher utility + Morgan skill ([7ac87d6](https://github.com/5dlabs/cto/commit/7ac87d690a7c3690550df4661be50b88aa864500))
* add projectId to CodeRun CRD + memory isolation config ([412d28b](https://github.com/5dlabs/cto/commit/412d28bd1731d001ebc7af6c1d368fe034d667b2))
* add Swarm agent (Canteen hackathon) to CTO platform ([ff969a3](https://github.com/5dlabs/cto/commit/ff969a3765af127d00eec3a6c0b51fd5b1894000))
* **agents:** add Block as Solana specialist agent ([b484a18](https://github.com/5dlabs/cto/commit/b484a18bb27c17b974ad133eb55fa2c936f2c48c))
* **agents:** add kubectl/argocd to agent image, RBAC for task pods ([abfb0f9](https://github.com/5dlabs/cto/commit/abfb0f99ea5169e7724b680b7f83a48c78763374))
* **agents:** cto-tools CLI + Deno runtime + tools-server sidecar for dynamic MCP tool access ([#4607](https://github.com/5dlabs/cto/issues/4607)) ([89f4b4b](https://github.com/5dlabs/cto/commit/89f4b4bbd2e484c056393a032d34c11a7544aa11))
* **block:** add controller templates for Block agent ([b68f66a](https://github.com/5dlabs/cto/commit/b68f66a3e2f1a5b43dfeb6240bdfb570a8fbaa92))
* **block:** add Solana CLI, Helius RPC, and expertise docs ([b52e88d](https://github.com/5dlabs/cto/commit/b52e88d944f5c15de50baaf8170457d1cfa7a030))
* **block:** add solana skills — node-ops config + ClawHub dev skills ([149f616](https://github.com/5dlabs/cto/commit/149f6168c33797c2f36438fffaa48585e9a8c4ea))
* **block:** expand to multi-chain blockchain agent ([fe4ba49](https://github.com/5dlabs/cto/commit/fe4ba49f11a728a507a1d24bd0a9eb1344bc723c))
* CLI matrix fixes — Codex promtail, Gemini env, model registry, tslog deps, clippy lint ([ee212f8](https://github.com/5dlabs/cto/commit/ee212f8c9f04b426f3fa4db743bde8b6cadf2107))
* **controller:** CRD overhaul, skills pipeline, Discord resilience & Datadog telemetry ([#4613](https://github.com/5dlabs/cto/issues/4613)) ([42f1bdc](https://github.com/5dlabs/cto/commit/42f1bdca34575587731487cbfd5cb56ecbd8dd4f))
* **controller:** CRD provider refactor — explicit provider + providerBaseUrl on CLIConfig ([3340cbf](https://github.com/5dlabs/cto/commit/3340cbf9d74a9406a327787c1c2740245de3a31a))
* **controller:** thread EscalationPolicy from CRD through MCP config ([#4594](https://github.com/5dlabs/cto/issues/4594)) ([638412d](https://github.com/5dlabs/cto/commit/638412d285a13895ddb19399471fcb1ea9163198))
* **crd:** Multi-agent CodeRun with ACP, OpenClaw config, and new naming convention ([#4603](https://github.com/5dlabs/cto/issues/4603)) ([04b76c6](https://github.com/5dlabs/cto/commit/04b76c6c63304b112cc10510d70990614ba84a55))
* enable daily-digest cron job at 8:45 PM PDT (test) ([9a63ab4](https://github.com/5dlabs/cto/commit/9a63ab42d770655c7d5ab502ada4ad45ee3b5b95))
* Fireworks-only providers, embeddings, IfNotPresent pullPolicy ([6b2eb43](https://github.com/5dlabs/cto/commit/6b2eb43fcdeff7025f4eb251acf6aed9bbf9f124))
* Hermes harness agent integration ([#4630](https://github.com/5dlabs/cto/issues/4630)) ([f0760d9](https://github.com/5dlabs/cto/commit/f0760d968072b698e1a177e618eb2849b180b755))
* **infra:** add ArgoCD sync workflow for automatic deployments on merge ([#4616](https://github.com/5dlabs/cto/issues/4616)) ([504dede](https://github.com/5dlabs/cto/commit/504dede0dce0451902f646dff127e8c3d3c23549))
* **infra:** add Qdrant Datadog metrics + Cloudflare dashboard egress ([#4619](https://github.com/5dlabs/cto/issues/4619)) ([537be5e](https://github.com/5dlabs/cto/commit/537be5eb0bdfe8952f668f38e4bea729633e7362))
* **infra:** add Qdrant vector database for Mem0 agent memory ([#4617](https://github.com/5dlabs/cto/issues/4617)) ([f00a38d](https://github.com/5dlabs/cto/commit/f00a38d311546445efaa5fd95ead97c6704545ce))
* **infra:** disable OpenMemory in favor of Mem0 + Qdrant ([#4618](https://github.com/5dlabs/cto/issues/4618)) ([7b82021](https://github.com/5dlabs/cto/commit/7b820216b0199e01066152fdcd4f0593be270e06))
* **intake:** add codebase_repository_url arg for cross-repo analysis ([d8f6056](https://github.com/5dlabs/cto/commit/d8f6056ecd682148b9e53975e7791647f6e75a34))
* **intake:** Anthropic metaprompt-compliant intake prompts ([cff597d](https://github.com/5dlabs/cto/commit/cff597dc0ec1e50a126ece1109270d48be9eb097))
* **intake:** attach deliberation MP3 to GitHub Release on target repo ([8e31ecb](https://github.com/5dlabs/cto/commit/8e31ecb8f0e471ffa638d77264ad41ce740ca2c7))
* **intake:** sigma-1 task breakdown ([#4589](https://github.com/5dlabs/cto/issues/4589)) ([2760d2c](https://github.com/5dlabs/cto/commit/2760d2cab509350945d42157d72ce7d12252c072))
* **lobster-voice:** add TTS pronunciation dictionary for natural technical speech ([507797f](https://github.com/5dlabs/cto/commit/507797fc962c5a6d4dddae4f12d2fa1954aede7c))
* **mem0:** consolidate Mem0 plugin into CTO chart ([1309404](https://github.com/5dlabs/cto/commit/1309404940625f2fa5e4d2d7eb0d3092d4c3fa40))
* **mem0:** wire Mem0 plugin into CRD task pods ([a4e527b](https://github.com/5dlabs/cto/commit/a4e527b849b6919d16cf9c9a8c2b2fa2f546801b))
* **memory:** enhanced payload schema with metadata, categories, indexes ([88371a3](https://github.com/5dlabs/cto/commit/88371a3334ab646ff0a8d6ea83adebca67f15f43))
* **memory:** Phase 2-4 memory isolation — task scoping, skills, dashboard ([6169d25](https://github.com/5dlabs/cto/commit/6169d2542431b3fe6a8a66c6b2edf937e54ea7e1))
* Morgan RBAC + git fix + Mem0 plugin config ([db07994](https://github.com/5dlabs/cto/commit/db07994e2241be0b63bcecdc24b63cd7f38cd92e))
* remote skills + agent persona distribution via cto-agent-personas ([#4605](https://github.com/5dlabs/cto/issues/4605)) ([6cd880c](https://github.com/5dlabs/cto/commit/6cd880c6efa971cf53b0f42107866db864445db2))
* **swarm:** add Colosseum Copilot PAT + correct env vars ([ddccaa1](https://github.com/5dlabs/cto/commit/ddccaa16807f0ba9f1480fb038a3fcdbadd820fd))
* **swarm:** add Colosseum PAT env vars + sigma-1, lab-eco, big-balls repos ([cae7c9d](https://github.com/5dlabs/cto/commit/cae7c9dfd0fe5dbdcd7114c84bc669a4afeb9631))
* **swarm:** ArgoCD-managed StatefulSet with OpenClaw native cron ([c10e691](https://github.com/5dlabs/cto/commit/c10e691ea4163d0a58e3597ab21beedf4db62c81))
* **swarm:** enable memory, session maintenance, compaction model ([ac40abe](https://github.com/5dlabs/cto/commit/ac40abed9071e4c9ca17154eb4ceefdd25cb049a))
* **swarm:** enable SWARM CLI pre-auth + profile config ([dfe999b](https://github.com/5dlabs/cto/commit/dfe999b365937034321d1e090805756124772fa8))
* **swarm:** upgrade digest workflow, add repos, progress journal ([e92872d](https://github.com/5dlabs/cto/commit/e92872d5d96751218f5596cf2d1ce09fcc5d0cf6))
* switch swarm ArgoCD to public chart (5dlabs/swarm-openclaw-agent) ([3be6247](https://github.com/5dlabs/cto/commit/3be6247e7b655e95698993ee5271e964bd0caf86))
* **test:** neon gauntlet — multi-CLI E2E test suite ([#4614](https://github.com/5dlabs/cto/issues/4614)) ([4aa1187](https://github.com/5dlabs/cto/commit/4aa11874ecc5bc5faaeb7a606c2a6646d28c8ec5))
* **tools:** Dynamic MCP TypeScript SDK for code-execution mode ([#4610](https://github.com/5dlabs/cto/issues/4610)) ([1d66057](https://github.com/5dlabs/cto/commit/1d6605793894e14ed98f2ac5bf7595b7e268770c))
* **tools:** escalation policy engine + tools_request_capability built-in ([#4593](https://github.com/5dlabs/cto/issues/4593)) ([9f293e0](https://github.com/5dlabs/cto/commit/9f293e0e1834b23db26b27e54baa92c93af9aeb3))


### 🐛 Bug Fixes

* add colosseum API URL construction warning to AGENTS.md ([b76de08](https://github.com/5dlabs/cto/commit/b76de085cde82126228c2ebe916c1e94d3a141d2))
* add token leak prevention + absolute path rules to AGENTS.md ([24fe5c7](https://github.com/5dlabs/cto/commit/24fe5c7822edf442b42ac22ed02c2bacfb6f87c1))
* add tool usage rules + missing repos to swarm AGENTS.md ([1b0c137](https://github.com/5dlabs/cto/commit/1b0c137c1e8be51f9e1c71282c32be1845274624))
* **agents:** add --legacy-peer-deps for openclaw tslog resolution ([547b4dd](https://github.com/5dlabs/cto/commit/547b4dd9d8ef7cc5d72d623de4aea8aa9361147e))
* **agents:** explicitly install tslog in openclaw dir ([feb4596](https://github.com/5dlabs/cto/commit/feb4596012254dd07621c443d0196a2f767a106b))
* **agents:** use official openclaw install without --ignore-scripts ([6406761](https://github.com/5dlabs/cto/commit/640676107844fdf741545572f8bc5a40c6ae8a2e))
* chown git-credentials so agent can clone repos ([d2013f3](https://github.com/5dlabs/cto/commit/d2013f364b6824a20b354660d5adb8e001432182))
* **ci:** read workspace version from root Cargo.toml ([3211c33](https://github.com/5dlabs/cto/commit/3211c338a87281a86017847c43407b8f67351404))
* **ci:** switch agents-build to ubuntu-latest runners ([#4577](https://github.com/5dlabs/cto/issues/4577)) ([83e613e](https://github.com/5dlabs/cto/commit/83e613e62176daf7df2d868c64d551893591c87a))
* **ci:** use controller-build.Dockerfile for Docker build ([9e6985b](https://github.com/5dlabs/cto/commit/9e6985b2d602a6284fca5f4d3c8716201c85ced7))
* **ci:** use ubuntu-latest for controller build-and-push ([395c732](https://github.com/5dlabs/cto/commit/395c73253a1b6c0985fb2e872101c0fb8d6027a0))
* configure gh CLI auth via config file, not env vars ([0dcf4d2](https://github.com/5dlabs/cto/commit/0dcf4d227bd352ce68a7d4dfbfafc670e0faa5d5))
* **controller:** preserve image node_modules in openclaw-deps init container ([f4a5763](https://github.com/5dlabs/cto/commit/f4a5763091795b485a399dc29ba6301bf8fec858))
* disable xAI plugin + web search, add SWARM path hints ([607aae5](https://github.com/5dlabs/cto/commit/607aae51af2841b1af0d72ac7f6e2c1a8e12be5d))
* **images:** replace broken nodesource with official Node.js tarballs, update glab ([#4575](https://github.com/5dlabs/cto/issues/4575)) ([77e8cb0](https://github.com/5dlabs/cto/commit/77e8cb03fb47fa8b66fd2b88f7c0e632dbdb1f5d))
* **intake:** add .intake/checkpoints/ to preflight cleanup ([9980a8e](https://github.com/5dlabs/cto/commit/9980a8e40611a1debf6c3b6faef2b72f59aa952b))
* **intake:** add 120s timeout to search-patterns step ([d1c483b](https://github.com/5dlabs/cto/commit/d1c483bb973a351d2ab9bc10a1e0db47615d5546))
* **intake:** add timeout + stub fallback for create-linear-project ([15e2be3](https://github.com/5dlabs/cto/commit/15e2be3c47350d9eb2a01a3e094bcf86d97980e9))
* **intake:** auto-truncate large repomix output + fix curl E2BIG ([fe5a6ef](https://github.com/5dlabs/cto/commit/fe5a6ef62f23e478e5ae9ae1931ce4978c73d7d1))
* **intake:** clean ALL .tasks/ subdirs in preflight + cross-project guard ([1896261](https://github.com/5dlabs/cto/commit/189626101f27835487e1a1c29dc93167adb3f59c))
* **intake:** clean stale artifacts in preflight to prevent cross-run contamination ([36467a0](https://github.com/5dlabs/cto/commit/36467a0a65aebdf72ab93a7245a9953eadd64f82))
* **intake:** correct file names for disk-based artifact counting ([4bb45ea](https://github.com/5dlabs/cto/commit/4bb45ea9dfd1231933c70269496c91080ce65d76))
* **intake:** extract truncation to separate Python script ([9c6afc1](https://github.com/5dlabs/cto/commit/9c6afc1d6c396e3d057b799327b6d7b5809fa8f3))
* **intake:** include transcript JSON in release when MP3 unavailable ([702a7c5](https://github.com/5dlabs/cto/commit/702a7c5dfce7fee22c90e93864168e5989d33620))
* **intake:** parser misses DECISION_POINT blocks wrapped in markdown bold ([d17a85e](https://github.com/5dlabs/cto/commit/d17a85e217c685732bdccd4a9f516fd90bb22148))
* **intake:** pipeline resilience — run_id propagation, quality-gate fallback, gateway restart ([#4629](https://github.com/5dlabs/cto/issues/4629)) ([6c0af8f](https://github.com/5dlabs/cto/commit/6c0af8f30fdddd4d79b5cdf0357f2dcd6bc09bcd))
* **intake:** prevent full source code leak into deliberation context ([d44259a](https://github.com/5dlabs/cto/commit/d44259ac98361ea5a7fa5a99609d048421fcffe2))
* **intake:** resilient gateway health check + disk-based step outputs ([877d8f3](https://github.com/5dlabs/cto/commit/877d8f3ea8ae35f03efed56efee175f04020bc01))
* **intake:** set ALL model tiers + committee to github-copilot/claude-opus-4.6 ([e2adcd8](https://github.com/5dlabs/cto/commit/e2adcd877b0569731af72161d1150a66ddafcad7))
* **intake:** skip workspace PR when target_repo_local_path is set ([#4632](https://github.com/5dlabs/cto/issues/4632)) ([35a312b](https://github.com/5dlabs/cto/commit/35a312b9b17410b4eb9a8ef578e54e3be8c3800a))
* **intake:** split acceptance criteria + reference acceptance.md ([7aa5536](https://github.com/5dlabs/cto/commit/7aa5536583ba5cdd7949785e0c4dd171646e4ccd))
* **intake:** use temp files for large repo analysis to avoid E2BIG ([d2accfe](https://github.com/5dlabs/cto/commit/d2accfe108a607b9ae28dc766390fcc7a4e862a4))
* **intake:** write pack-repo/search to disk files, not stdout ([3fa604e](https://github.com/5dlabs/cto/commit/3fa604eedbf1beadb5df8de7849a419613f20d61))
* **intake:** write step outputs to disk to avoid E2BIG in verify-artifact-gates ([83675b1](https://github.com/5dlabs/cto/commit/83675b165f224b2f77ab7bb4d104f1319ce89929))
* **lobster-voice:** add getCachedPath for render-transcript file path resolution ([0339a8f](https://github.com/5dlabs/cto/commit/0339a8fde1359797703bcf1ff07f600e1f007381))
* make gh CLI primary data source for digest, local repos fallback only ([80c259d](https://github.com/5dlabs/cto/commit/80c259dcb5ee7a61868319aa5aa5cb42bf1b6612))
* **mem0:** remove invalid plugin config properties ([e9f8a73](https://github.com/5dlabs/cto/commit/e9f8a73fa04b925e0d8ddec3b86fd73c83a8b2c3))
* move BASE_IMAGE ARG before first FROM for Docker multi-stage ([97f0ffb](https://github.com/5dlabs/cto/commit/97f0ffba94676838e1d1dcd6c77236c6001e02f6))
* **openclaw-agent:** fix git auth and mem0 telemetry for long-running agents ([159d6ff](https://github.com/5dlabs/cto/commit/159d6ffb43daf6408ae9ec06350afdc4e2d7f424))
* permanent home dir fix in init container + Dockerfile ([f04f09a](https://github.com/5dlabs/cto/commit/f04f09a2c9b98501878b902f1045ecf2fec56dc7))
* qdrant client version override + git awareness instructions ([fb2dc10](https://github.com/5dlabs/cto/commit/fb2dc1057df999118039c1b5e13e43bcc12eeaf5))
* remove duplicate COLOSSEUM_COPILOT_API_BASE from extraSecretEnv ([da3a3b4](https://github.com/5dlabs/cto/commit/da3a3b417032aa9afe1854efd58668a8f938a23d))
* remove zhipu provider from gateway config ([f85b3b5](https://github.com/5dlabs/cto/commit/f85b3b5309c564ddc51e32377f6ff943732a27fd))
* set node user homedir to /workspace in /etc/passwd ([5b427c3](https://github.com/5dlabs/cto/commit/5b427c3fc66354bbb2ec1c058a26ec3a0b942f38))
* simplify cto-play Docker build stage ([4537b8a](https://github.com/5dlabs/cto/commit/4537b8afc2e335cd78335bec8a2ab0dd2cba2e29))
* **swarm:** add Fireworks provider + OpenAI OAuth for model access ([7cfed7e](https://github.com/5dlabs/cto/commit/7cfed7ec609738f132f9eb58c505b16a78316f08))
* **swarm:** align ArgoCD values with OpenClaw schema ([63e80b8](https://github.com/5dlabs/cto/commit/63e80b841324685be30679a92f0939f2f2c3266f))
* **swarm:** correct cliBackends schema (command+args+output) ([d09a509](https://github.com/5dlabs/cto/commit/d09a509627bbc74e120ee4a3441c5d0a3ca2ec20))
* **swarm:** map provider API key secret keys to actual secret names ([9f30f39](https://github.com/5dlabs/cto/commit/9f30f393dcc7f51acdffebf66945557a7c26c85d))
* **swarm:** move discord config to chart's expected path ([6bbbe70](https://github.com/5dlabs/cto/commit/6bbbe704c882f301a0bc33b95d02d2c67c42cf89))
* teach agent to use gh CLI for any repo, not just cloned ones ([41eb16b](https://github.com/5dlabs/cto/commit/41eb16b255f6907cf7b97e170e122e7d716d8ec7))
* use plugins.deny to block xAI plugin at runtime level ([fc2d6d8](https://github.com/5dlabs/cto/commit/fc2d6d8b50f3d5926cf2c59af4bc2393e9e1cfcc))


### 📚 Documentation

* Solana hackathon on-chain agent ideas brainstorm ([#4604](https://github.com/5dlabs/cto/issues/4604)) ([835f3ca](https://github.com/5dlabs/cto/commit/835f3cac992dacc715d6688552bc3aa5b894392d))
* **swarm:** note SWARM CLI pre-authenticated as [@kaseonedge](https://github.com/kaseonedge) ([438e0dd](https://github.com/5dlabs/cto/commit/438e0ddeed8c405845aca64bd447f3c84702301f))


### 🔧 Maintenance

* remove unused swarm-values.yaml (inline values used) ([15cd9a0](https://github.com/5dlabs/cto/commit/15cd9a0159a47359edb45c5a69afeda22609ddf2))
* **swarm:** disable cron until manual testing complete ([c40d41a](https://github.com/5dlabs/cto/commit/c40d41a3a65ed1df4f1264a1185c66f735f411e9))
* update cron comments — daily-digest registered and enabled at 5 PM PST ([f9afbfa](https://github.com/5dlabs/cto/commit/f9afbfa875c1d205a02a3205804b1e9852ee168e))

## [0.2.52](https://github.com/5dlabs/cto/compare/v0.2.51...v0.2.52) (2026-04-01)


### ✨ Features

* **intake:** notifycore-e2e task breakdown ([#4552](https://github.com/5dlabs/cto/issues/4552)) ([d43b686](https://github.com/5dlabs/cto/commit/d43b686d0c2207f843849d12f6cdc76a3fb51679))
* **marketing:** add 5D Git as platform service under Source Control ([0099151](https://github.com/5dlabs/cto/commit/00991510e31ebc6060dfa815f1195f5547618718))
* **marketing:** add ACP harness and multi-provider model routing ([81d5d11](https://github.com/5dlabs/cto/commit/81d5d1157417979cc33b38a2a98dd5959c3965c5))
* **marketing:** add GitLab/Gitea support, align header behavior ([41f13d0](https://github.com/5dlabs/cto/commit/41f13d02bc81659ee3ebf50bb38229b81b624c04))
* **marketing:** Morgan hero effects, card flip fix, LemonSlice inline, golden retriever avatar ([a40d988](https://github.com/5dlabs/cto/commit/a40d98871ebfc1e46004c5f0452dc7b07398ad72))
* **marketing:** Morgan prominence across CTO site ([d706194](https://github.com/5dlabs/cto/commit/d7061947968bc26b16cae4411bfe742a088b5d37))
* **morgan:** marketing page, avatar, LemonSlice embed, and desktop wiring ([917e2cf](https://github.com/5dlabs/cto/commit/917e2cf73f68afbc9cacf75646933bc51368194d))
* **observability:** add local Grafana/Loki stack for intake pipeline ([c446e72](https://github.com/5dlabs/cto/commit/c446e7266c2ad00d9523d7128f8b3d8ffd83ad7d))
* **observability:** add log level label to Loki + Cluster Logs dashboard ([8189d2a](https://github.com/5dlabs/cto/commit/8189d2a33b330070383f45b5e4dcc33a958a691d))
* **observability:** add prominent status tiles to Solana Validator dashboard ([6c6c399](https://github.com/5dlabs/cto/commit/6c6c3993de4989188a73d690d51aaecd92620a93))
* **pitch-deck:** rewrite deck with PAS framework, plain language, 13 slides ([a0980c0](https://github.com/5dlabs/cto/commit/a0980c0685b6fb826c67a7e43b4a63c853005a99))
* **pm:** agent delegation, token endpoint, and config hardening ([#4568](https://github.com/5dlabs/cto/issues/4568)) ([e56df9f](https://github.com/5dlabs/cto/commit/e56df9f66753623f6a68f4a3a0b61a2491aa0432))
* **scm:** dual GitHub + GitLab SCM provider support ([#4536](https://github.com/5dlabs/cto/issues/4536)) ([a43c572](https://github.com/5dlabs/cto/commit/a43c572e7ba95fe52fd10c8cc020fa931e43c204))
* **splash:** add Polygon to trading chain coverage ([3e5755c](https://github.com/5dlabs/cto/commit/3e5755cf1e53fcea037b6c7f848e09d1e1f529bd))
* **trading:** add Yellowstone gRPC geyser plugin to Agave RPC node ([7647831](https://github.com/5dlabs/cto/commit/7647831d51814145c48213dd2e1484fed3cbf369))
* **trading:** build Yellowstone gRPC from source, fix ABI crash ([9b6bfeb](https://github.com/5dlabs/cto/commit/9b6bfeb4852889bca9b9b4ebbd48c45d738d8f98))
* **trading:** deploy 8 trading agents to K8s via ArgoCD ([#4530](https://github.com/5dlabs/cto/issues/4530)) ([1105794](https://github.com/5dlabs/cto/commit/1105794e49a0aaf2a1e79cdd028df91cd341b9f6))
* **trading:** deploy solana exporter, production dashboards, and ArgoCD integration for Cherry cluster ([16dee06](https://github.com/5dlabs/cto/commit/16dee0685f892358a5f09617850204d604c1373f))
* **trading:** Solana DEX indexer + Agave flag fix ([#4535](https://github.com/5dlabs/cto/issues/4535)) ([d51edb9](https://github.com/5dlabs/cto/commit/d51edb9696b7843959a01aa6f20966f7bfbd9db9))
* **website:** Morgan page — Meta Ray-Ban Display + WhatsApp ([5bcb6f9](https://github.com/5dlabs/cto/commit/5bcb6f9cec58e77b9dcbb9420f15de6521fbd00c))
* **website:** Morgan page — Rokid Glasses + Vuzix Z100 sections ([a229dde](https://github.com/5dlabs/cto/commit/a229ddec9a069b7f9566917403f0758e89cb2e1e))


### 🐛 Bug Fixes

* **intake:** harden e2e quick-run workflow ([#4548](https://github.com/5dlabs/cto/issues/4548)) ([39b27ef](https://github.com/5dlabs/cto/commit/39b27ef565f1563832abc512844d79dc0e9f4924))
* **marketing,splash:** mobile view regressions + Business Team tools/skills ([5ec9eb4](https://github.com/5dlabs/cto/commit/5ec9eb469780647a03f64a191c2ccac557b41b8d))
* **marketing,splash:** normalize pathname for nav link highlighting ([573ebd7](https://github.com/5dlabs/cto/commit/573ebd7960f5bef640ff944313f20f3e85c57417))
* **marketing:** center Meet Morgan section, remove empty right space ([7d6e25a](https://github.com/5dlabs/cto/commit/7d6e25a03072f25e24ce6bdcd1c0aaa5998a9df6))
* **marketing:** grid-pulse z-index so it stays behind hero content ([2b6e7ff](https://github.com/5dlabs/cto/commit/2b6e7ffee547b10992fe61e2d17ee64f32ac3d1f))
* **marketing:** LemonSlice widget 9:14 aspect ratio, responsive sizing, no head crop ([06d4c73](https://github.com/5dlabs/cto/commit/06d4c73be44b98ad632a1aefa631897d03040764))
* **observability:** add cilium-operator Prometheus scrape target ([1c781dc](https://github.com/5dlabs/cto/commit/1c781dc20c81c113c1d4482b700534c56d3741ce))
* **observability:** fix Solana dashboard logs panel and sync ETA ([9c9baa1](https://github.com/5dlabs/cto/commit/9c9baa1458577f4444f6ec091ff1aa757f5a0283))
* **observability:** raise Agave memory limit and fix Cilium dashboards ([6d4de40](https://github.com/5dlabs/cto/commit/6d4de400ccd91a68ff4164108ff30082fd79e278))
* **observability:** remove Label_Keys from Fluent Bit Loki output ([0d080f3](https://github.com/5dlabs/cto/commit/0d080f35378151398e4276bea807891855a0b96e))
* **observability:** split Solana logs into Agave and Exporter panels ([6f94a7b](https://github.com/5dlabs/cto/commit/6f94a7b926e83ef254ca3943bb03388419a6c6ca))
* **pitch-deck:** accurate traction — pilot phase, pipeline not closed revenue ([4cc7e10](https://github.com/5dlabs/cto/commit/4cc7e10f7c3f5b6a5c8c50821c8f3a5c09dc3007))
* **pitch-deck:** move Model mix from Problem to CTO slide ([#4558](https://github.com/5dlabs/cto/issues/4558)) ([61e780f](https://github.com/5dlabs/cto/commit/61e780fa18c5ad233daae5518d77f0b1ac4266d0))
* **pitch-deck:** replace "tax" metaphor with markup/overhead/fees ([#4560](https://github.com/5dlabs/cto/issues/4560)) ([b984429](https://github.com/5dlabs/cto/commit/b98442953a34ea98e301eb49d6bb0bafadd98ef2))
* **trading:** align secrets structure with remote openclaw chart ([0225cd4](https://github.com/5dlabs/cto/commit/0225cd4a44aeb47e0b4c1b9adca98ce5aa4b11ff))
* **trading:** bump memory limits to 1Gi and fix Mode A heartbeat instructions ([7aaee97](https://github.com/5dlabs/cto/commit/7aaee97555bbcbfad37131a4f19773623a70f8c9))
* **trading:** bump memory limits to 2Gi — 1Gi still OOMKilling Mode A agents ([a5bf166](https://github.com/5dlabs/cto/commit/a5bf1664e0c6d3899febf282549ade3888c61ba3))
* **trading:** correct repo URLs, OpenBao paths, disable hooks ([eb77c03](https://github.com/5dlabs/cto/commit/eb77c03169fdce5898c391bfc6d56bdbf68ad97f))
* **trading:** disable Yellowstone gRPC geyser plugin pending crash fix ([2f438a3](https://github.com/5dlabs/cto/commit/2f438a3f54f1011f6d0c6f26dbbeaaa0677aa1d7))
* **trading:** fix secrets structure blocking ArgoCD sync ([d2c5af3](https://github.com/5dlabs/cto/commit/d2c5af301cbe90da3f2358827d7d9b19a59210af))
* **trading:** mirror openclaw-api-keys pattern, add 1Password token ([c1a7250](https://github.com/5dlabs/cto/commit/c1a7250075252a7641ffe301f8f8138a3c6ea015))
* **trading:** override heartbeat model to Haiku (chart defaults to minimax) ([9bdaf61](https://github.com/5dlabs/cto/commit/9bdaf619572d26ab56cfdd6d54e7723b42dda320))
* **trading:** register Haiku 4.5 model in anthropic provider + fix fallback IDs ([fdda146](https://github.com/5dlabs/cto/commit/fdda146dbf8d4a5b3630cdc7affbaa1d56892391))
* **trading:** resolve PR [#4531](https://github.com/5dlabs/cto/issues/4531) conflicts — add simmer/helius/polymarket secrets and env vars ([#4532](https://github.com/5dlabs/cto/issues/4532)) ([57052a9](https://github.com/5dlabs/cto/commit/57052a93364abaa1205741ab38621b0f68b4add1))
* **trading:** set Helm releaseNames in ArgoCD apps to match existing releases ([78bca01](https://github.com/5dlabs/cto/commit/78bca010a9ae31fad3375e9d5fec07f8f32fba0c))
* **trading:** use pre-built asymmetric-research solana-exporter image ([a46fb8c](https://github.com/5dlabs/cto/commit/a46fb8c9a07608ffd49edfb97edae254ec0e7102))


### 🔧 Maintenance

* consolidate worktrees and merge trading-agents infra ([#4541](https://github.com/5dlabs/cto/issues/4541)) ([6e5c20a](https://github.com/5dlabs/cto/commit/6e5c20a187152672bb45e9a32ac578b6383e940c))
* **marketing:** remove Even G2 HUD images, keep product shot only ([dc8cb43](https://github.com/5dlabs/cto/commit/dc8cb4333b24f39fb35a83a14e3c25d598897a97))

## [0.2.51](https://github.com/5dlabs/cto/compare/v0.2.50...v0.2.51) (2026-03-08)


### ✨ Features

* add cloud parity callout to services page hero ([#4500](https://github.com/5dlabs/cto/issues/4500)) ([f8fb48a](https://github.com/5dlabs/cto/commit/f8fb48aa3c9ac8501aca5a66f8817cabed8acbf2))
* **avatar:** consolidate Morgan avatar PoC into CTO repo ([#4508](https://github.com/5dlabs/cto/issues/4508)) ([2668ea9](https://github.com/5dlabs/cto/commit/2668ea98d3cec9bac180556440d070f03eac6bbd))
* brand refresh, narrative realignment, and services catalog ([#4497](https://github.com/5dlabs/cto/issues/4497)) ([8f027f9](https://github.com/5dlabs/cto/commit/8f027f987998906e10a9da13cb87a9ad7f088a6a))
* **deliberation:** one meeting room architecture ([b9183b7](https://github.com/5dlabs/cto/commit/b9183b71312ff8b56db027a8d5facf0e8efe20e2))
* **intake:** generate AlertHub e2e test prompts ([c329f14](https://github.com/5dlabs/cto/commit/c329f143b92096e139a5cbb5152c371157358ca4))
* **intake:** research step, Discord reporting, PR creation ([#4462](https://github.com/5dlabs/cto/issues/4462)) ([bff007d](https://github.com/5dlabs/cto/commit/bff007d25d4bba3dcbf65fa924c5c63ee3943b51))
* **intake:** wire deliberation into real intake pipeline before task generation ([f6715c7](https://github.com/5dlabs/cto/commit/f6715c76fb74ed704318931a72c750fe4d8cbbe6))
* **linear-bridge:** Loki activity stream for ACP agent dialog ([#4491](https://github.com/5dlabs/cto/issues/4491)) ([904af87](https://github.com/5dlabs/cto/commit/904af87206654539e29e4235a8242a379348866c))
* Main Team (Lex, Hype, Tally, Chase) on splash and CTO ([#4487](https://github.com/5dlabs/cto/issues/4487)) ([9d990bd](https://github.com/5dlabs/cto/commit/9d990bdb0e4cc0f1dd67657f3e676bef2acec7fb))
* **marketing:** add 9 bare metal providers to infrastructure section ([#4446](https://github.com/5dlabs/cto/issues/4446)) ([109b55c](https://github.com/5dlabs/cto/commit/109b55c5ac7f8a13f7a357dff4fa1de1f1bf1c75))
* **marketing:** add Blockchain & AI Teams section ([d565ba9](https://github.com/5dlabs/cto/commit/d565ba902f8061ae4c480b14e061c9271d32ed92))
* **marketing:** add Glitch agent avatar ([#4486](https://github.com/5dlabs/cto/issues/4486)) ([3bd33a9](https://github.com/5dlabs/cto/commit/3bd33a9dac9c95a9cae4092f6fa53006f192edba))
* **marketing:** add Vex and Block agent avatars ([#4485](https://github.com/5dlabs/cto/issues/4485)) ([0135a4d](https://github.com/5dlabs/cto/commit/0135a4dd48443d985ea613a4b232cc671432a65a))
* **marketing:** agent card overhaul + content corrections ([#4482](https://github.com/5dlabs/cto/issues/4482)) ([bb62d9d](https://github.com/5dlabs/cto/commit/bb62d9d97d79b6973ddbf067b82ed7232a8c825a))
* **marketing:** clarify GPU compute optionality with 3-tier breakdown ([00eb520](https://github.com/5dlabs/cto/commit/00eb52025707ee461bb1b67eac5da934c171616e))
* **marketing:** expand Bolt to Infrastructure & SRE, add Healer agent ([#4445](https://github.com/5dlabs/cto/issues/4445)) ([46a6818](https://github.com/5dlabs/cto/commit/46a6818a2bc2ba5f21cc94f7bfd1b599a2a4be27))
* **marketing:** infra rewrite, new agents, blockchain refresh, security tooling ([485f3e9](https://github.com/5dlabs/cto/commit/485f3e948d668ab78acc92d3b005d5051addc3c2))
* **marketing:** integrations section, Block/Vex agents, Cipher pen testing ([6e51966](https://github.com/5dlabs/cto/commit/6e519666376bcd5e21772877f8a4a32556e50677))
* **marketing:** merge Specialists squad, refresh self-hosted model list to March 2026 ([e682eac](https://github.com/5dlabs/cto/commit/e682eacad1715e887a7b6a4fba74f63258dfb53e))
* **morgan:** add lean Morgan agent for avatar PoC ([#4504](https://github.com/5dlabs/cto/issues/4504)) ([a5ed834](https://github.com/5dlabs/cto/commit/a5ed8341a397c57e3f380cf9ef00ad7edc81c005))
* platform naming, logo, hiring philosophy, CLI additions, abstraction language ([#4498](https://github.com/5dlabs/cto/issues/4498)) ([e7cfffd](https://github.com/5dlabs/cto/commit/e7cfffdc888b55b61d7604ddc38d5bad67a79706))
* **splash,marketing:** boost grid visibility and restore perspective breathing ([#4473](https://github.com/5dlabs/cto/issues/4473)) ([391a535](https://github.com/5dlabs/cto/commit/391a53589c0262631be1d84561dc32bc075c6d4f))
* **splash,marketing:** Grid Pulse + Shift Dimensions, deploy via 1Password ([#4461](https://github.com/5dlabs/cto/issues/4461)) ([aa8dc6f](https://github.com/5dlabs/cto/commit/aa8dc6f08ea0eeef43fec6b3e01cbfe54d84e914))
* **splash,marketing:** match grid and shift values to bg-preview demo ([#4474](https://github.com/5dlabs/cto/issues/4474)) ([b0e6c7a](https://github.com/5dlabs/cto/commit/b0e6c7a71f1fede6767f92cfb9ec9b48c4eece78))
* **splash,marketing:** restore entrance animations with CSS-only approach ([#4472](https://github.com/5dlabs/cto/issues/4472)) ([dc1ff7c](https://github.com/5dlabs/cto/commit/dc1ff7cdcb3ffe86821e63e30862d65ec1225897))
* **splash:** add Chase (Sales) business agent, increase avatar size ([ca74c70](https://github.com/5dlabs/cto/commit/ca74c70d7425b20a61af0db5717f5228b92f5e4d))
* **splash:** add downloadable investor one-pager on investors page ([#4459](https://github.com/5dlabs/cto/issues/4459)) ([470ffab](https://github.com/5dlabs/cto/commit/470ffabf9419602dba215f4c7e002c071fb7b5f3))
* **splash:** expand Bolt from Deployment Specialist to Infrastructure & SRE ([#4444](https://github.com/5dlabs/cto/issues/4444)) ([0e04918](https://github.com/5dlabs/cto/commit/0e049186c5f48fdd3f3550cee9b88f9d6d846ced))
* **splash:** move Opportunities to footer, add Hivelocity provider ([#4441](https://github.com/5dlabs/cto/issues/4441)) ([86ab681](https://github.com/5dlabs/cto/commit/86ab681efcf545ca0d060cfbe8d27ebfff622248))
* **splash:** remove Sanctuary venture ([f1183fb](https://github.com/5dlabs/cto/commit/f1183fbfd94dbd5010e113caa98148f7e213ed42))
* **splash:** rename The Designers to The Product on team page ([#4442](https://github.com/5dlabs/cto/issues/4442)) ([075060a](https://github.com/5dlabs/cto/commit/075060a56a9becc022604daf10c5cd75e6722dab))
* **splash:** Rex, Grizz, Nova as peers — Rust/Go/Node Engineer ([#4443](https://github.com/5dlabs/cto/issues/4443)) ([3f17f38](https://github.com/5dlabs/cto/commit/3f17f381f086780a726a74905f12bbf7b81d92d9))
* **splash:** scroll-hide header ([#4480](https://github.com/5dlabs/cto/issues/4480)) ([de86689](https://github.com/5dlabs/cto/commit/de866891b181fa605939fd343b4da481ec6ccfa3))
* **splash:** Sui chain, Servers.com/PhoenixNAP/i3D.net providers, model optimization ([#4439](https://github.com/5dlabs/cto/issues/4439)) ([c4f5381](https://github.com/5dlabs/cto/commit/c4f538193509794205ebd9b59633ca419a98bb08))
* **telemetry:** enrich Loki labels with cli_name, session_type, activity_type ([d2393bd](https://github.com/5dlabs/cto/commit/d2393bdc0a96e8e7461947b05a1617255207b485))
* **tests:** alerthub e2e intake output — subtask prompts (146 subtasks) ([#4453](https://github.com/5dlabs/cto/issues/4453)) ([d793712](https://github.com/5dlabs/cto/commit/d793712cd7d4ec208d2ec5bcfb6a139f8e21703c))
* v0-inspired 'Build at the Speed of Thought' hero rewrite ([#4503](https://github.com/5dlabs/cto/issues/4503)) ([18c22ad](https://github.com/5dlabs/cto/commit/18c22aded104c252dea2eec80cc12621a94fe882))


### 🐛 Bug Fixes

* allow intake script to continue when deliberation fails ([3c894d1](https://github.com/5dlabs/cto/commit/3c894d1c0492e7f4f616f43514e91adf75317379))
* **bots:** add imagePullSecrets to discord-bridge and linear-bridge ([#4495](https://github.com/5dlabs/cto/issues/4495)) ([f9b5c85](https://github.com/5dlabs/cto/commit/f9b5c856bc9a4d9cb853cd96fbad9aed857b2622))
* **ci:** add --branch=main to wrangler deploy for production release ([9ef233e](https://github.com/5dlabs/cto/commit/9ef233e1ee58fa072980466c4890c5dc094db746))
* **controller-template:** fix REPOSITORY_URL set-u crash, workspace persistence, intake-files fallback ([6f04f47](https://github.com/5dlabs/cto/commit/6f04f478d2153c7bc1677fc5c5c9ae093bddfdeb))
* **controller:** add grok cliImage alias pointing to research image ([fdf902a](https://github.com/5dlabs/cto/commit/fdf902aa14cc6aebe25ae74ce572437210c3d2c7))
* **controller:** chown /workspace/runs so research user can clean workspace ([540f225](https://github.com/5dlabs/cto/commit/540f2256ef27b7d9c8b04e7c55ba803e80996fe1))
* **deliberation:** add dmPolicy=block to optimist/pessimist agents ([d2c1483](https://github.com/5dlabs/cto/commit/d2c148382dfa1423d0eafebb0fc6e753e5c11df0))
* **deliberation:** AgentMessage format, NATS reply inbox, bedrock→claude model configs ([#4456](https://github.com/5dlabs/cto/issues/4456)) ([d55f7f4](https://github.com/5dlabs/cto/commit/d55f7f4a81204cc3f3b43e770703ac6dbb995ef3))
* **deliberation:** close critical gaps in committee voting and prompts ([d663b21](https://github.com/5dlabs/cto/commit/d663b212283236dca12f0a16ad4d25374ff37e78))
* **deliberation:** fix dmPolicy from invalid 'block' to 'disabled' ([ac152dc](https://github.com/5dlabs/cto/commit/ac152dc3286fa388eb596933dd4953f2b3e84214))
* **deliberation:** set heartbeat=30s on all 7 debate agents ([43387e2](https://github.com/5dlabs/cto/commit/43387e2700b1e10aef057ba2b3053423f475af0a))
* disable acp plugin install for deliberation agents ([4eea796](https://github.com/5dlabs/cto/commit/4eea79669aea5c7be5d80ed063a3e7b77b362300))
* ignore normalized twingate access crd fields ([#4506](https://github.com/5dlabs/cto/issues/4506)) ([39395b6](https://github.com/5dlabs/cto/commit/39395b68b3c341b034b7dbefb37022c16c91ad8b))
* **intake-agent:** error_type, Operation union, dependencies, expand-task retry ([#4451](https://github.com/5dlabs/cto/issues/4451)) ([7362f6e](https://github.com/5dlabs/cto/commit/7362f6eaa84749c2dd07562b5067d8b44a8fe832))
* **intake-agent:** resolve all 53 TypeScript type errors ([db19d8f](https://github.com/5dlabs/cto/commit/db19d8f90e92e27aa8886f5cbd7fe8eb9acd4a1d))
* **intake:** make workspace cleanup fault-tolerant (chmod before rm) ([#4460](https://github.com/5dlabs/cto/issues/4460)) ([d8057b7](https://github.com/5dlabs/cto/commit/d8057b70a8ab9d77214e531e017f23eb0427cef8))
* **intake:** morgan agent → grok cli + claude-sonnet-4-20250514 model for e2e test ([88ff498](https://github.com/5dlabs/cto/commit/88ff4982fbbf7d7e570bc08234b17bfbc39ce689))
* **intake:** respect design_brief_path when deliberate=true; hard error on missing brief ([000e741](https://github.com/5dlabs/cto/commit/000e74154af03410c1cbc15b52e51157f767a214))
* **linear-bridge:** add LINEAR_TEAM_ID and fix missing env vars ([#4496](https://github.com/5dlabs/cto/issues/4496)) ([29b1624](https://github.com/5dlabs/cto/commit/29b16248c11196fa3217bb252d8800df8b150044))
* **marketing:** customer-facing provider copy, remove implementation details ([#4447](https://github.com/5dlabs/cto/issues/4447)) ([7f284b2](https://github.com/5dlabs/cto/commit/7f284b2edbf976c7d16e77454b6064fd7275d4a4))
* **marketing:** improve Telegram social preview rendering ([#4448](https://github.com/5dlabs/cto/issues/4448)) ([ff412a7](https://github.com/5dlabs/cto/commit/ff412a75b932a925072b7fdb0b8056a1863eb9b0))
* **marketing:** make social image routes static-export compatible ([#4449](https://github.com/5dlabs/cto/issues/4449)) ([306f097](https://github.com/5dlabs/cto/commit/306f097dfb97f1ea1fb684894b28b9924b50f4b7))
* **marketing:** remove on-premises provider listing ([#4450](https://github.com/5dlabs/cto/issues/4450)) ([bd8d3c0](https://github.com/5dlabs/cto/commit/bd8d3c04008d821bc1226419007cb24ea18015ee))
* **openclaw-agent:** add gateway.controlUi to configmap template ([a3b4b8a](https://github.com/5dlabs/cto/commit/a3b4b8ad254fe0cfc925f86893f8686f14406753))
* **openclaw-agent:** nil-safe controlUi template access ([1481b91](https://github.com/5dlabs/cto/commit/1481b91042e7787004beeafd051dfada2b01ab1e))
* **pm:** use claude as default intake CLI (was research→grok which has no configured image) ([e6649f1](https://github.com/5dlabs/cto/commit/e6649f19ce31b1cccac864c6259932009bd1b577))
* remove 'Proprietary Operating Stack' from splash hero badge ([#4499](https://github.com/5dlabs/cto/issues/4499)) ([1aedb28](https://github.com/5dlabs/cto/commit/1aedb286e8da73b875eb0e93931dee3fa5cbd69c))
* remove backslashes from single-quoted jq filter in deliberation ([de3d2c7](https://github.com/5dlabs/cto/commit/de3d2c79743daaffad3a797c998dda262fae2789))
* remove broken Twingate CRD manifests (TwingateRemoteNetwork doesn't exist) ([#4454](https://github.com/5dlabs/cto/issues/4454)) ([6e4c1a4](https://github.com/5dlabs/cto/commit/6e4c1a42c530a77507fe3cf24c526b902fb5f46e))
* remove redundant Team nav link — agents already on homepage ([#4505](https://github.com/5dlabs/cto/issues/4505)) ([9b34bd7](https://github.com/5dlabs/cto/commit/9b34bd70b4073e8471b7071a62e44be28caaa89d))
* rename Managed Foundation → 5D Runtime, rewrite cloud parity with confidence ([#4501](https://github.com/5dlabs/cto/issues/4501)) ([684a9d3](https://github.com/5dlabs/cto/commit/684a9d3127187f784fdd9e7324d7595ddb5e366b))
* replace 'proprietary' with 'in-house', remove negative framing ([#4502](https://github.com/5dlabs/cto/issues/4502)) ([11c53ae](https://github.com/5dlabs/cto/commit/11c53aec263a72d3721b86077e81b052267f4bf8))
* **research:** add gh CLI to image for intake workflow ([640f36c](https://github.com/5dlabs/cto/commit/640f36c28896ce455a6d8a0f92d2aeaee99c86e6))
* **research:** add intake CLI binary to research image ([0ad592c](https://github.com/5dlabs/cto/commit/0ad592cdf9d4a69fe13cef605cf008d6ac6c860c))
* **research:** add jq and curl to runtime image ([d283cb8](https://github.com/5dlabs/cto/commit/d283cb87f7c95c228f22600c9869f666ace92897))
* **research:** add python3 and nodejs to runtime-base image ([9a0cf4e](https://github.com/5dlabs/cto/commit/9a0cf4eea5af2a3f6d3182802bd504a351880c89))
* **research:** make intake verify non-fatal, add ldd diagnostic ([1f013e4](https://github.com/5dlabs/cto/commit/1f013e4f1463e5bded1aa71aa079a4d39833008e))
* resolve duplicate --prd flag and stderr corruption in intake script ([280e9e5](https://github.com/5dlabs/cto/commit/280e9e5738b7c012ffcc1ee4bba91a2d33a6a573))
* restore lifecycle-test dir and enable deliberation for alerthub e2e ([#4452](https://github.com/5dlabs/cto/issues/4452)) ([7d15e4c](https://github.com/5dlabs/cto/commit/7d15e4cce708c3f115ab104612a2ac4038d34427))
* **splash,marketing:** boost GridPulse visibility + match preview intensity ([30ca3a9](https://github.com/5dlabs/cto/commit/30ca3a9b772fc0dc6e701d88d63b6270e64dff77))
* **splash,marketing:** eliminate element flashing and fix LCP performance ([#4466](https://github.com/5dlabs/cto/issues/4466)) ([95b2488](https://github.com/5dlabs/cto/commit/95b24882899aba2dc88c86b0b160a9c12900b96d))
* **splash,marketing:** eliminate GridPulse flashing and 3D depth sorting ([9e90c89](https://github.com/5dlabs/cto/commit/9e90c89c4023a38e8f7295fdad56b66cb69c3f4e))
* **splash,marketing:** fix GridPulse visibility, remove Healer agent, SSR perf ([b401e9b](https://github.com/5dlabs/cto/commit/b401e9ba8d7740d2d9e131d74f7f8ed6df0141f4))
* **splash,marketing:** GridPulse z-index stacking + will-change GPU hints ([ccb860f](https://github.com/5dlabs/cto/commit/ccb860f9073ffe358eb9d35cc3b5fd464d3c39aa))
* **splash,marketing:** keep GridPulse lines in the background ([#4468](https://github.com/5dlabs/cto/issues/4468)) ([875eb72](https://github.com/5dlabs/cto/commit/875eb7216e1b6b328b3a552e8a94578562e92f6e))
* **splash,marketing:** move GridPulse inside ShiftDimensionsWrapper ([9107f6e](https://github.com/5dlabs/cto/commit/9107f6e668bd7879bc8c4f53b0738c8b7d38846a))
* **splash,marketing:** prevent grid foreground overlay from hiding content tiles ([#4470](https://github.com/5dlabs/cto/issues/4470)) ([b074809](https://github.com/5dlabs/cto/commit/b074809a3f266318f09400121f1f5c40dcace0f8))
* **splash,marketing:** remove foreground grid lines, restore pulse animation ([#4478](https://github.com/5dlabs/cto/issues/4478)) ([5e4ab65](https://github.com/5dlabs/cto/commit/5e4ab655819795c9ebf12e2c6a1b7f9a96b50d5b))
* **splash,marketing:** remove pulse, soften grid, fix investor page ([#4479](https://github.com/5dlabs/cto/issues/4479)) ([d7b6cad](https://github.com/5dlabs/cto/commit/d7b6cadcbb0a8e0931cadb160999db9d887a4a6d))
* **splash,marketing:** remove scroll animation and fix compositing ([#4476](https://github.com/5dlabs/cto/issues/4476)) ([a5c5cf0](https://github.com/5dlabs/cto/commit/a5c5cf0a19f11fef1ef0e46ea734ef109283fe81))
* **splash,marketing:** restore grid presence with rare foreground passes ([#4469](https://github.com/5dlabs/cto/issues/4469)) ([644fd23](https://github.com/5dlabs/cto/commit/644fd23f2afcc5e81dccb54509666564df371099))
* **splash,marketing:** stop post-render hiding and stale-cache flicker ([#4467](https://github.com/5dlabs/cto/issues/4467)) ([9970a80](https://github.com/5dlabs/cto/commit/9970a8065ffe33a232e06753798286ebc632cdf2))
* **splash,marketing:** strip all content animations to eliminate flashing ([#4477](https://github.com/5dlabs/cto/issues/4477)) ([0de36b5](https://github.com/5dlabs/cto/commit/0de36b5067038d4f32c4ea5520f4a640c0fe4d2e))
* **splash:** consistent investor CTAs + Motion hover expand ([#4481](https://github.com/5dlabs/cto/issues/4481)) ([5269dc5](https://github.com/5dlabs/cto/commit/5269dc548aacbbedefb20e112c2b32b87867883a))
* **splash:** strip Framer Motion from all sub-pages to eliminate flash ([#4475](https://github.com/5dlabs/cto/issues/4475)) ([02d8245](https://github.com/5dlabs/cto/commit/02d8245d26402696727590cb911e422cd52c39cf))
* **telemetry:** add acp-cli promtail jobs for all CLI log directories ([#4494](https://github.com/5dlabs/cto/issues/4494)) ([93595a0](https://github.com/5dlabs/cto/commit/93595a0199479da6d52b539a1c0c0c0485553642))
* **template:** make workspace cleanup non-fatal (permission denied fallback) ([dc8125f](https://github.com/5dlabs/cto/commit/dc8125f56884c34b17bc43d9ddc380496edbf9d5))
* **twingate:** allowEmpty=true, ignoreDifferences for stale v1alpha1 resources ([e8696d1](https://github.com/5dlabs/cto/commit/e8696d138a89cb310a089d25e6efd868b9579c91))
* **twingate:** empty kustomization - remove broken v1alpha1 CRD manifests ([2ed83c2](https://github.com/5dlabs/cto/commit/2ed83c204d5d695e01caa49748ca2f4f0516a01c))


### ⚡ Performance

* **splash,marketing:** add Service Worker for instant cached loads ([af17a9d](https://github.com/5dlabs/cto/commit/af17a9dd66a6bfe971334650e92ddf21e75c96fb))
* **splash,marketing:** compositor-only GridPulse animations ([3d944ca](https://github.com/5dlabs/cto/commit/3d944ca5cff56573f2caf8149c90ec8773b549ca))


### 📚 Documentation

* add Cloudflare tunnel setup guide for intake testing agent ([3235987](https://github.com/5dlabs/cto/commit/32359870b59dae249eb2cb3e85c9934c4ea6a7d1))
* **skill:** add bridge communication section and new intake-util commands to intake-pipeline skill ([cf678f5](https://github.com/5dlabs/cto/commit/cf678f525181b30803d7ced6490c84636a75694d))


### 🔧 Maintenance

* **discord-bridge:** trigger GHCR rebuild after package relink ([7ae8a26](https://github.com/5dlabs/cto/commit/7ae8a26504ceefa1c3e365bbd035df1c87fde33a))
* remove all hardcoded agent counts across both apps ([c849531](https://github.com/5dlabs/cto/commit/c849531763a999d5f64746c22b45616f32eb9646))
* untrack local build artifacts and add to gitignore ([6aabbac](https://github.com/5dlabs/cto/commit/6aabbac1cb4e0c8e58e0c2140f53bae4d416afdd))


### 👷 CI/CD

* **controller:** temp disable cancel-in-progress to allow build to complete ([1927dc3](https://github.com/5dlabs/cto/commit/1927dc325c5527b7b65b4b1c1657cb3893fd53a8))

## [0.2.50](https://github.com/5dlabs/cto/compare/v0.2.49...v0.2.50) (2026-02-27)


### ✨ Features

* add AI, GPU, and Nvidia operators; enable twingate-operator ([6ec364a](https://github.com/5dlabs/cto/commit/6ec364a1aa2be73f3c151c2e544f8e0e9ad53096))
* add kotal-operator ArgoCD app (5dlabs fork with Reth/NEAR support) ([78a5da3](https://github.com/5dlabs/cto/commit/78a5da3321d7e0bf3d8a9baf2aa17451119fe57d))
* configure optimist/pessimist Discord + fix gp3 storage on OVH ([aee795a](https://github.com/5dlabs/cto/commit/aee795a8cd4cb3e48d97edf6100008cbc23e5a64))
* **infra:** add switchable webhook exposure and re-enable Stitch Argo app ([#4419](https://github.com/5dlabs/cto/issues/4419)) ([db88b6d](https://github.com/5dlabs/cto/commit/db88b6dd7146ba51ed2760e9ed9a85d270d18466))
* **infra:** AWS cluster bootstrap – OpenClaw bots disabled, storage overlay app ([#4416](https://github.com/5dlabs/cto/issues/4416)) ([5dd793f](https://github.com/5dlabs/cto/commit/5dd793f1d69994ce66ed86ef1d6ef6e46587dd86))
* **infra:** AWS cluster bootstrap with skip-reconcile overlay strategy ([#4415](https://github.com/5dlabs/cto/issues/4415)) ([25afd22](https://github.com/5dlabs/cto/commit/25afd220e73400cbb2c07f3c056c5cf613a52b96))
* **intake:** deliberation system — debate agents + committee voting ([#4434](https://github.com/5dlabs/cto/issues/4434)) ([6c95735](https://github.com/5dlabs/cto/commit/6c9573584afbe24c70c08eabbe71ebd73230b1e0))
* **intake:** enable AI prompt generation (Session 2) by default ([d6578bd](https://github.com/5dlabs/cto/commit/d6578bdcb291cab0afb41362b63e39db5531d6fe))
* **openclaw-agent:** add nodeSelector support and remove CPU limits ([89f7360](https://github.com/5dlabs/cto/commit/89f7360a3bad32991ae5c4479c34b2b03db86ed3))
* **runtime:** add steipete CLI tools to baseline image ([#4423](https://github.com/5dlabs/cto/issues/4423)) ([4525b9e](https://github.com/5dlabs/cto/commit/4525b9e2a89d97886c3b7a3e62b2cf4067163066))
* **skills:** add 10 ClawHub community skills to templates ([#4424](https://github.com/5dlabs/cto/issues/4424)) ([2f0f66e](https://github.com/5dlabs/cto/commit/2f0f66e4510a84923fe77846ea1bc6de3244b839))
* **splash:** add 5D Labs homepage app for 5dlabs.io ([#4425](https://github.com/5dlabs/cto/issues/4425)) ([5aabffb](https://github.com/5dlabs/cto/commit/5aabffb356b65f1cfdcf282889ce7dde30df86e5))
* **splash:** add consulting services page ([#4435](https://github.com/5dlabs/cto/issues/4435)) ([9061ee2](https://github.com/5dlabs/cto/commit/9061ee29ffe6abca3042b8a1b3c61b595dc0d410))
* **task-controller:** add research CLI image + switch intake to research image ([2eac6de](https://github.com/5dlabs/cto/commit/2eac6de8734b354430686f68a7db11b8145dbf06))
* update cilium values for k3s/OVH - tunnel mode, k3s API, correct pod CIDR ([b323e1e](https://github.com/5dlabs/cto/commit/b323e1e1c245cf02890c3fb4a78f37d0ca3004af))


### 🐛 Bug Fixes

* add custom ArgoCD Lua health checks for ClusterTunnel, TunnelBinding, HTTPRoute ([93f341b](https://github.com/5dlabs/cto/commit/93f341bab89fc6ff4323bb8a33f30bf12741213b))
* add githubSecret to committee agent values (was causing empty secretKeyRef) ([fe521e0](https://github.com/5dlabs/cto/commit/fe521e0fc43c59c335c25b0d38e1b5e637a57024))
* add headers field to tools mcp-servers.json configmap template ([83f65df](https://github.com/5dlabs/cto/commit/83f65df8ef9cb4081e67530db2332973bb1abdd4))
* add kube-dns service alias for RKE2 compatibility ([9c0ecb0](https://github.com/5dlabs/cto/commit/9c0ecb0c174f7be76c7e9e84a713d0001e3b07e8))
* **cilium:** disable bpf masquerade and enable nodeport for k3s ([5f0c9e6](https://github.com/5dlabs/cto/commit/5f0c9e6fbd0830716924e364ef5043f39be53197))
* exclude twingate apps from platform-apps glob (managed by networking-apps) ([f2f1d4f](https://github.com/5dlabs/cto/commit/f2f1d4f4cffd6c1b77ceee51b95e0a79e50dd74d))
* headscale TunnelBinding uses noTlsVerify directly in spec (not under originRequest) ([9565f80](https://github.com/5dlabs/cto/commit/9565f8069e57214dfd84eccbfe2418c7abaa9e87))
* heartbeat "off" invalid duration — use 87600h across all deliberation agents ([7b9849c](https://github.com/5dlabs/cto/commit/7b9849c5597b031ae581fc2e9aa53a16cf0c21e0))
* heartbeat 87600h overflows 32-bit setTimeout — use 500h (~3 weeks) ([9ed6487](https://github.com/5dlabs/cto/commit/9ed6487da9757f532a3319e911f8a5b36167ff1c))
* ignore ClusterPolicy spec/status drift in nvidia-gpu-operator ([f13d989](https://github.com/5dlabs/cto/commit/f13d98913321e8cc8080637c4a8092db27733f92))
* ignore HTTPRoute status in cto app (public-gateway not yet deployed) ([01dd660](https://github.com/5dlabs/cto/commit/01dd6608beef860f6a112e59900a2a9419dbdf33))
* ignore image-updater helm.parameters drift in platform-apps and networking-apps ([3e52d81](https://github.com/5dlabs/cto/commit/3e52d813be4c4081118213266c264915a916afac))
* ignore namespace label/annotation drift in platform-namespaces ([d2ccfa1](https://github.com/5dlabs/cto/commit/d2ccfa11bc6fc9ce86f4e1b1f73a8b5afe0279c5))
* increase MCP server initialization timeout from 8-10s to 45-60s ([faa9787](https://github.com/5dlabs/cto/commit/faa978767712404e2777ebf5c81f261b4ffb8c50))
* **infra:** remove Claude Code from runtime, add 1Password to minimal, restore ARC runners for EKS ([#4431](https://github.com/5dlabs/cto/issues/4431)) ([4617ab5](https://github.com/5dlabs/cto/commit/4617ab5bc05636bacb155cf7931ebdf62d25acd1))
* **intake-agent:** add NATS_URL env fallback for CodeRun pod execution ([4be0680](https://github.com/5dlabs/cto/commit/4be0680e37dbc921dd463de09354332ce1ee4049))
* **marketing:** add OpenGraph and Twitter splash image assets ([009d87a](https://github.com/5dlabs/cto/commit/009d87a77bff556ec31dbb169edaa6b2a6b3a4f6))
* **marketing:** bust social card cache with versioned OG/Twitter images ([2e62dca](https://github.com/5dlabs/cto/commit/2e62dcab0bd6f514e5079834bb8e65546935f067))
* **marketing:** remove Sidero/Omni infra cards ([#4426](https://github.com/5dlabs/cto/issues/4426)) ([614eefd](https://github.com/5dlabs/cto/commit/614eefd0fe2c10ab67617bcd7c4d7eca5ad21c2a))
* move discord config into agent.messaging (not root messaging) ([020af4d](https://github.com/5dlabs/cto/commit/020af4d970752eb1ea80f1c962be12aa906d3efd))
* nfd master nodeSelector to use worker nodes; twingate retry with GHCR creds ([e3d20fb](https://github.com/5dlabs/cto/commit/e3d20fb3ef11199b842cab36f7dfcc06bdb12520))
* **ntfy:** update HTTP listen port to 8080 in configmap and deployment manifests ([bd735f5](https://github.com/5dlabs/cto/commit/bd735f5d12dcd88b69d8022382758ac86ce68c0b))
* openclaw-agent image tag v prefix (2026.2.12 → v2026.2.12) ([b3031f1](https://github.com/5dlabs/cto/commit/b3031f1144a4cdf54918bf35593fb80c91da2f29))
* **openclaw-agent:** reduce CPU requests to fit 14 agents on worker ([a1ff714](https://github.com/5dlabs/cto/commit/a1ff71460d3edef003ddef29d7533f229030441e))
* otel skipSchemaValidation, seaweedfs nodeSelector removed, twingate OCI repo in AppProject ([c564d79](https://github.com/5dlabs/cto/commit/c564d79468a1d576537cd5b506a58efb4bfaf986))
* patch TunnelBinding CRD to preserve unknown fields (headscale originRequest) ([f1059aa](https://github.com/5dlabs/cto/commit/f1059aaa90427ec85ee26bbdde857a2472e9aa23))
* per-agent messaging secrets so DISCORD_TOKEN env var carries correct token ([e5486b7](https://github.com/5dlabs/cto/commit/e5486b741bea10823c108ea612399d3831bfd1b8))
* persist ArgoCD ignoreDifferences for cert/CRD/namespace drift (OVH cluster) ([fae8dc6](https://github.com/5dlabs/cto/commit/fae8dc64749fe4693e28639162d66c41042ba0a4))
* **pm:** scale PM server back to one replica ([#4420](https://github.com/5dlabs/cto/issues/4420)) ([caad801](https://github.com/5dlabs/cto/commit/caad801ad74f158150c2524ff41d5275e44b7757))
* **research:** bake default NATS_URL into image for OVH CodeRun pods ([2b22832](https://github.com/5dlabs/cto/commit/2b2283293d9386751f79112d61d6659116605d3b))
* resolve CSS layer specificity, blur variants, and missing magenta theme color ([cd6cdb9](https://github.com/5dlabs/cto/commit/cd6cdb990ba0f4b7df23c62f70117066606e27b8))
* revert discord to top-level messaging (agent.messaging causes dm.policy panic) ([80520e7](https://github.com/5dlabs/cto/commit/80520e75c0b81acdaa22fbffd901671eb8962e8a))
* seaweedfs tolerations type, otel featureGates schema, twingate-operator OCI chart ref ([cec0b6d](https://github.com/5dlabs/cto/commit/cec0b6d9f9a08f90b467642527c902c96d180dc5))
* seaweedfs volume 100Gi-&gt;50Gi (mayastor pools max ~60GiB free per node) ([a7e44d8](https://github.com/5dlabs/cto/commit/a7e44d8f745518f2c46aab2fbe6093b3845217c3))
* skip-reconcile aws-load-balancer-controller on OVH bare metal ([138234f](https://github.com/5dlabs/cto/commit/138234f6f2f1b0994edc439e7d81051ce5b2404f))
* skip-reconcile Mayastor on OVH - using local-path storage overlay ([e89b2dc](https://github.com/5dlabs/cto/commit/e89b2dc2d9887fdb912365d14a3c92a8979ecafb))
* **splash:** add Telegram-friendly OG image ([#4428](https://github.com/5dlabs/cto/issues/4428)) ([917137c](https://github.com/5dlabs/cto/commit/917137c242ee66261e70fe26d5520ad4025ed9de))
* **splash:** button import from @radix-ui/react-slot not radix-ui ([6f90751](https://github.com/5dlabs/cto/commit/6f907511f79e2d0da2757f3538b54af7df9d1c14))
* **splash:** replace [@apply](https://github.com/apply) glass-bg with inline CSS for Tailwind compat ([95bf65d](https://github.com/5dlabs/cto/commit/95bf65d501e40d1549682213dfb637aff9d7688a))
* **splash:** restore dark theme so text is visible ([39694ed](https://github.com/5dlabs/cto/commit/39694edd8f23377870a0156c58e7cd3dd098a217))
* **splash:** use Slot not Slot.Root from @radix-ui/react-slot ([a57cf1a](https://github.com/5dlabs/cto/commit/a57cf1ae8c33dc698ff51844a75ddf0e03b82f2c))
* switch loki MCP from ghcr.io image (GHCR auth denied) to npx @elad12390/loki-mcp ([1a5ac57](https://github.com/5dlabs/cto/commit/1a5ac5750f137a873ed3b9c1913fd53156eaad1e))
* switch storage overlay from AWS EBS to local-path for OVH deployment ([c398ca4](https://github.com/5dlabs/cto/commit/c398ca4c82d3490f8c88d085794e8ff8bb0b31b4))
* treat pending WaitForFirstConsumer PVCs as Healthy in ArgoCD ([11f002f](https://github.com/5dlabs/cto/commit/11f002f05cc3679021a2587b13a768fe60e37f61))
* twingate-operator version v0.28.0 -&gt; 0.28.0 (OCI tags have no v prefix) ([ffda650](https://github.com/5dlabs/cto/commit/ffda65007b7dde4116fc577a28c49ec837987fe1))
* update ingress-nginx externalIPs to OVH node IPs ([92f0572](https://github.com/5dlabs/cto/commit/92f0572f656be778f962d85cf94be17755a0ffd4))


### 📚 Documentation

* add Solana K8s architecture and operator spec ([94d332a](https://github.com/5dlabs/cto/commit/94d332a3a9cd5c3c894af3b5853ba0f749b3fea1))
* add Talos + Cilium responsibilities & overlap section ([c524d7e](https://github.com/5dlabs/cto/commit/c524d7e7ef0ef34783e6d5fa87a8a3aba190e225))
* trading cluster — clarify dual-homed direct public IP model ([ceeaadf](https://github.com/5dlabs/cto/commit/ceeaadf0a98fd2e6581dec4a30757bab24107c46))
* trading cluster — Cloudflare tunnels out, Twingate throughout ([3dc5469](https://github.com/5dlabs/cto/commit/3dc54697d47167fa9d215091dd8495325a6a78e4))
* trading cluster — resolve open decisions, add Transit implementation ([c2426cc](https://github.com/5dlabs/cto/commit/c2426cc431095aa74dd60eed6a7a47eff93a7482))
* trading cluster architecture & security plan ([d6c91a0](https://github.com/5dlabs/cto/commit/d6c91a0905c32802df5d8145bc00b4d4bd6b7563))


### 🔧 Maintenance

* add development notice and clean up stale root markdown files ([91d84d6](https://github.com/5dlabs/cto/commit/91d84d67b4512a0822d380b59bb8e18384b95a9e))
* **cilium:** re-enable argocd management ([6ea452f](https://github.com/5dlabs/cto/commit/6ea452fbabd9d3c733ba4511756cbed9392252a5))
* **docker:** pin OpenClaw to 2026.2.12, fix Gemini CLI build ([#4422](https://github.com/5dlabs/cto/issues/4422)) ([64afa7b](https://github.com/5dlabs/cto/commit/64afa7b491d382f559299946474fc807470e2549))
* **docker:** update Dockerfiles for agents and Gemini CLI ([#4421](https://github.com/5dlabs/cto/issues/4421)) ([ad53801](https://github.com/5dlabs/cto/commit/ad53801ad578465ba2cd5de4a8e4902269b96aad))
* enable all operators - remove skip-reconcile, fix otel issuerRef, set aws-lb clusterName ([c30fe8a](https://github.com/5dlabs/cto/commit/c30fe8a8c521280307a25898fcf12c84c8ffd152))
* **gitops:** disable bots namespace apps by default ([#4417](https://github.com/5dlabs/cto/issues/4417)) ([a2d39dc](https://github.com/5dlabs/cto/commit/a2d39dc20883299c1fc2ba49d5477380338a405b))
* **prd:** expand Hivelocity provider implementation plan ([#4418](https://github.com/5dlabs/cto/issues/4418)) ([071b881](https://github.com/5dlabs/cto/commit/071b88156f7455ed3b5b31b7a7b3c37e7ac0b23e))
* re-add skip-reconcile to twingate-operator (GHCR OCI auth unresolved) ([38b624a](https://github.com/5dlabs/cto/commit/38b624aed4daffbf26d8c33d0c1b3d2f199cad7d))
* re-disable cilium - requires k3s --flannel-backend=none (CNI conflict with flannel) ([dd390d9](https://github.com/5dlabs/cto/commit/dd390d9f450070a4d3626f66e7d17589375f6a04))
* remove openclaw agent apps from CTO cluster gitops ([d4a7e70](https://github.com/5dlabs/cto/commit/d4a7e70a3377bb5ce068ce0dbae90dbd0d69a6c9))
* twingate-operator - re-disable, document OCI 3-level path ArgoCD limitation ([c040d30](https://github.com/5dlabs/cto/commit/c040d303f4c371e64c1c1642c8c5d4525ff4226e))


### 👷 CI/CD

* add deploy-splash workflow for Cloudflare Pages (5dlabs.ai) ([19396bb](https://github.com/5dlabs/cto/commit/19396bb53ba617bae42ec9156af4b4001912756c))

## [0.2.49](https://github.com/5dlabs/cto/compare/v0.2.48...v0.2.49) (2026-02-08)


### ✨ Features

* **mcp:** add toggle_app tool for enabling/disabling ArgoCD applications ([#4403](https://github.com/5dlabs/cto/issues/4403)) ([57b8384](https://github.com/5dlabs/cto/commit/57b83848b41220a3badb26d3690cc6bd37b286ed))

## [0.2.48](https://github.com/5dlabs/cto/compare/v0.2.47...v0.2.48) (2026-02-07)


### ✨ Features

* **helm:** add cloud provider support (Bedrock, Vertex AI, Foundry) ([#4396](https://github.com/5dlabs/cto/issues/4396)) ([252b168](https://github.com/5dlabs/cto/commit/252b168d9d669d6c9bb5a888e7b8d18063905c24))
* **nats:** add discovery protocol, agent roster, and ping-pong guard ([4542490](https://github.com/5dlabs/cto/commit/4542490e60ce0596291b6dd8812cb464b9bbc83c))


### 🐛 Bug Fixes

* add ipv4NativeRoutingCIDR for Cilium native routing ([#4397](https://github.com/5dlabs/cto/issues/4397)) ([dd51e3a](https://github.com/5dlabs/cto/commit/dd51e3afd5af7c2eeb17fbd1aa91b0fbbb0a166b))
* **ci:** stop redirecting stderr into scan-results.json ([1ed08aa](https://github.com/5dlabs/cto/commit/1ed08aa990b612b6dc549852231f152bf896ddf2))
* disable WireGuard to unblock Mayastor hostNetwork connectivity ([#4398](https://github.com/5dlabs/cto/issues/4398)) ([fca9a40](https://github.com/5dlabs/cto/commit/fca9a40b88136b842fbe7f141854852b0810611a))
* **dns:** switch CoreDNS to public DNS after Talos firewall blocked port 53 ([8907965](https://github.com/5dlabs/cto/commit/89079657e1d3268c7e02470dd0500338e98250ba))
* enable Cilium native routing for hostNetwork pod connectivity ([#4395](https://github.com/5dlabs/cto/issues/4395)) ([631a32a](https://github.com/5dlabs/cto/commit/631a32a80017c7d17d9f0a4d6510e4f386403e73))
* metrics-server containerPort and defaultArgs for port 4443 ([#4393](https://github.com/5dlabs/cto/issues/4393)) ([8f46b47](https://github.com/5dlabs/cto/commit/8f46b47f5d5819fa14bc907369c0529730307902))
* metrics-server port conflict on hostNetwork ([#4392](https://github.com/5dlabs/cto/issues/4392)) ([5783b43](https://github.com/5dlabs/cto/commit/5783b438f3051611d6b31d0656fa94c131af72f9))
* **nats:** add nats tool to agent allowlists and update skill guidance ([a1d35b8](https://github.com/5dlabs/cto/commit/a1d35b8dfd8c87a3641e1e553872067fb179bc9d))
* **nats:** use standalone createInbox() import from nats module ([6a18a38](https://github.com/5dlabs/cto/commit/6a18a38a7152794f3f486fc0e40198c073f7ce71))


### 🔧 Maintenance

* upgrade OpenClaw to 2026.2.6-3, set imagePullPolicy Always ([#4391](https://github.com/5dlabs/cto/issues/4391)) ([e2e3960](https://github.com/5dlabs/cto/commit/e2e396040d218e8eb9bad3ef22078541c5fac05e))


### 👷 CI/CD

* add skills security scanning via mcp-scan on every push ([146d1b8](https://github.com/5dlabs/cto/commit/146d1b88d445fa4683e8bb44b1c3dd39f937886d))

## [0.2.47](https://github.com/5dlabs/cto/compare/v0.2.46...v0.2.47) (2026-02-07)


### ✨ Features

* **keeper:** add keeper agent configuration and identity ([03a2973](https://github.com/5dlabs/cto/commit/03a29738a6b44a2dd3143f994a079f2ecb31690d))
* **nats:** NATS inter-agent messaging plugin and infrastructure ([#4387](https://github.com/5dlabs/cto/issues/4387)) ([7c13be1](https://github.com/5dlabs/cto/commit/7c13be1f1d8ae6d80e20ff32b3bc8b049b55e661))


### 🐛 Bug Fixes

* **nats:** align plugin with OpenClaw SDK conventions ([4754b84](https://github.com/5dlabs/cto/commit/4754b84748973451e99cfe443f1bad6861e2bce4))


### 🔧 Maintenance

* archive Argo Workflows and Argo Events infrastructure ([#4389](https://github.com/5dlabs/cto/issues/4389)) ([fc7d6ce](https://github.com/5dlabs/cto/commit/fc7d6ce4f13190783954139ed71425b5d1acae95))

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
