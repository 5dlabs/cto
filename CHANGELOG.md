# Changelog

## [0.0.2](https://github.com/5dlabs/cto/compare/v0.0.1...v0.0.2) (2025-12-09)


### ✨ Features

* add BYOK, infrastructure operators, Cloudflare Tunnels, Kilo VPN ([7f25e85](https://github.com/5dlabs/cto/commit/7f25e855b313c29e4416be3dc92c74603321d247))
* add External Secrets Operator for OpenBao secret sync ([#2797](https://github.com/5dlabs/cto/issues/2797)) ([f35ad4a](https://github.com/5dlabs/cto/commit/f35ad4a41bf595ca4d20498daead928d8eec241e))
* add Nova avatar, expand tools library documentation ([bc4ff4f](https://github.com/5dlabs/cto/commit/bc4ff4f8f5f4da8c64c188babf8c293a6412fac8))
* add Phase 5 Deployment with Bolt ([dcf4ede](https://github.com/5dlabs/cto/commit/dcf4ede5f25a6aef8bae5d5c5c12740ffd095882))
* emphasize full infrastructure platform with observability and cost savings ([bef8e34](https://github.com/5dlabs/cto/commit/bef8e344cffe050de896512beed6d2a288164775))
* GitOps cleanup and infrastructure consolidation ([#2805](https://github.com/5dlabs/cto/issues/2805)) ([520e3ce](https://github.com/5dlabs/cto/commit/520e3ce0418addb43dce81fa2fd6d01628d68c00))
* **gitops:** add QuestDB operator from 5dlabs fork ([#2808](https://github.com/5dlabs/cto/issues/2808)) ([d1b86ce](https://github.com/5dlabs/cto/commit/d1b86ce405476bbefcfa43e11be36013a0625775))
* Initial codebase ([8d34264](https://github.com/5dlabs/cto/commit/8d34264ced69ba691c11d9e6f967ef276469b8a9))
* **observability:** add Blackbox Exporter with Discord alerting ([93c7164](https://github.com/5dlabs/cto/commit/93c71646a1c0ce9c29dbcd35a84c43fc905db938))
* update README with new agent descriptions and avatars ([8624604](https://github.com/5dlabs/cto/commit/8624604e42ace725bfd4f6f8c7c5de3328ec3061))


### 🐛 Bug Fixes

* add ignoreDifferences for ESO server-side defaults ([#2800](https://github.com/5dlabs/cto/issues/2800)) ([cc48d53](https://github.com/5dlabs/cto/commit/cc48d53dbfd542ec1f7dfafef2dfe9b818c97382))
* add LINEAR_ENABLED env var to PM deployment ([#2807](https://github.com/5dlabs/cto/issues/2807)) ([af3760f](https://github.com/5dlabs/cto/commit/af3760f1aa75d7e33d10fdd312adc51bcedfb0eb))
* align workflow phases across documentation ([203ab75](https://github.com/5dlabs/cto/commit/203ab756364639626bbafc72c692efe480665507))
* **cto-app:** use jqPathExpressions for TunnelBinding ignoreDifferences ([c052b04](https://github.com/5dlabs/cto/commit/c052b042af5141eb3ed50df6c8bf00c96b3c2a0f))
* **cto-chart:** add activeDeadlineSeconds to job config ([ab924a6](https://github.com/5dlabs/cto/commit/ab924a6a3d74b32d8298ec86d187e92dd190586c))
* **cto-chart:** add batch/jobs RBAC permissions for controller ([f4a0bec](https://github.com/5dlabs/cto/commit/f4a0bece86d4dbb43907c778545a04ad4353c260))
* **cto-chart:** add complete task controller config with all required fields ([6f1e029](https://github.com/5dlabs/cto/commit/6f1e029955390c3d60a3ea2951e2c2c34923bd84))
* **cto-chart:** add controller RBAC and fix healer command ([0d8930b](https://github.com/5dlabs/cto/commit/0d8930b5528674de6695e4e17923b2224ab0fa9d))
* **cto-chart:** add job section to task controller config ([654825c](https://github.com/5dlabs/cto/commit/654825cf6584a99d8e66db9bb37a503aee47d3a1))
* **cto-chart:** add task controller config for agent settings ([de5368f](https://github.com/5dlabs/cto/commit/de5368f470021bd89c27715d27679447a232803a))
* **cto-chart:** add task-controller-config checksum to trigger pod restarts ([6cc997e](https://github.com/5dlabs/cto/commit/6cc997efa3704f99f57b59ea94068e04661acd8a))
* **cto-chart:** use controller-* naming for ConfigMaps ([bb898c2](https://github.com/5dlabs/cto/commit/bb898c266e283e5cf189836987abd056e4fe8603))
* **gitops:** remove orphaned monitoring-stack application ([d1b7b5e](https://github.com/5dlabs/cto/commit/d1b7b5e8bf117a55a3d479d14506b6b3e6bf44df))
* improve k8s-runner toolchain reliability ([#2794](https://github.com/5dlabs/cto/issues/2794)) ([17900f8](https://github.com/5dlabs/cto/commit/17900f8c32ea65842b144bbe5297d520f8bc973e))
* make intake config CLI-agnostic, update Grizz description ([74fe365](https://github.com/5dlabs/cto/commit/74fe365645a73aef00960ba51ad8ae7722ddae32))
* remove unclosed div tag causing code blocks to be centered ([200ae93](https://github.com/5dlabs/cto/commit/200ae936b9f4edd55ff35f4d67d325bce7540084))
* **tools:** allow println and uninlined format args in integration tests ([1169a07](https://github.com/5dlabs/cto/commit/1169a07db1fc85f805655d8755213e9c45e115ad))


### ♻️ Refactoring

* **cto-chart:** simplify chart to core services only ([84a9449](https://github.com/5dlabs/cto/commit/84a9449628620237b6aaa71785b7c7018b5d213d))
* **gitops:** consolidate CTO chart and Cloudflare tunnel ([ebc3f97](https://github.com/5dlabs/cto/commit/ebc3f9742f7e8799c414f0334b2313ee5a39cc45))
* move Stitch to Operations Squad, remove TaskMaster references ([34fe5cf](https://github.com/5dlabs/cto/commit/34fe5cfd23a0f5abd1f074c174c0ed86eefd4405))
* remove DocsRun CRD references, now only CodeRun ([4a5c5f5](https://github.com/5dlabs/cto/commit/4a5c5f5c9c49f97e07d55aa210f33de5451db2cc))
* remove HashiCorp Vault integration ([78208ec](https://github.com/5dlabs/cto/commit/78208ec4786011b957faf3efdb5fba9f1aa6ab81))


### 📚 Documentation

* add comprehensive tech stack section ([4fd3177](https://github.com/5dlabs/cto/commit/4fd3177697c2dd2334578fe6c3824731aaa53390))
* add realistic cost comparison with model usage estimates ([6680f65](https://github.com/5dlabs/cto/commit/6680f659a6302e06892ed6a26a189a1a2036fc01))
* add tool filtering feature, link to full tools list ([533fbb3](https://github.com/5dlabs/cto/commit/533fbb311812a0723cb036c03bf2bc3de902c67b))
* clarify user interaction methods, add deployment phase ([e27acc8](https://github.com/5dlabs/cto/commit/e27acc8ceda0b65f3e8337061a75c82062c5553c))
* emphasize cto-config.json as the only user configuration ([8b488d3](https://github.com/5dlabs/cto/commit/8b488d3185e8a27910ad5ad378b57b64ffbfdec6))
* fix cost comparison accuracy, add local model support ([c8bf37a](https://github.com/5dlabs/cto/commit/c8bf37a977d41789a6b9628a325cf42aaa3471cc))
* remove CLI/provider-specific emphasis, add shadcn/ui tools ([efd612d](https://github.com/5dlabs/cto/commit/efd612d1a9e4bb7edbdecd49aac5d7ef39a8098a))
* update cto-config with model rotation, parallel execution, auto-merge ([8eb2b69](https://github.com/5dlabs/cto/commit/8eb2b69d00868f1a96dd727397da17203ec733ec))
* update launch date to January 1st, 2025 ([7c6cdf6](https://github.com/5dlabs/cto/commit/7c6cdf6fe43f9ab9174abad4e3bb64e18b052d76))
* update PM integrations roadmap, Linear as MVP ([65c815b](https://github.com/5dlabs/cto/commit/65c815b9ec97474a1a6cb83cb663aa28456ac77a))


### 🔧 Maintenance

* remove duplicate Multi-CLI Support section ([d601328](https://github.com/5dlabs/cto/commit/d60132884e87efba32f9ed7b8cb488cbc9783baf))
* remove Task Master AI from Related Projects section ([edc9517](https://github.com/5dlabs/cto/commit/edc951733538489a5b823afe6026de7951fc8002))
* rename Vault references to OpenBao and version bump ([#2796](https://github.com/5dlabs/cto/issues/2796)) ([ffb7003](https://github.com/5dlabs/cto/commit/ffb7003a9049d66817490c21f0eb9fa0c723346e))
* reset version to v0.0.1 and consolidate codebase ([#2811](https://github.com/5dlabs/cto/issues/2811)) ([6176950](https://github.com/5dlabs/cto/commit/61769509ee7e3ca7ff436aba7641dd65aef83294))
* update comments for secret management in research.yaml and platform-project.yaml ([bd7663e](https://github.com/5dlabs/cto/commit/bd7663e1cea15a04ef39e5bb6de637887699e7b2))
* update secret management references in values.yaml ([763b3a6](https://github.com/5dlabs/cto/commit/763b3a62bf802f28f5d390147d1e38df9366a7d5))
* update secret store integration in values.yaml ([8654d4d](https://github.com/5dlabs/cto/commit/8654d4dd56d779c2dd0a6d6390b8ca5ad6187f17))
