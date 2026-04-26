# CodeRun CRD Field Processing Reference

How each `CodeRunSpec` field is consumed by the controller after a CodeRun is applied to the cluster.

All file paths are relative to `crates/controller/src/tasks/code/`.

---

## Core Identity

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `runType` | `"implementation"` | **Deprecated for task work.** Still routes special workflows: `intake` → Morgan templates + fresh workspace + intake ConfigMap; `documentation` → intake templates; `review`/`remediate` → PR-specific naming and templates. For standard tasks, Lobster manages phases via the boolean fields below. Used in: `naming.rs:80–108` (job prefix selection), `templates.rs:170` (template dispatch), `resources.rs:684–689` (fresh workspace default for intake), `resources.rs:1187–1189` (intake ConfigMap mounting), `controller.rs:563–572` (implementation stage detection). |
| `taskId` | `null` | Embedded in job names (`t{taskId}-…`), passed as `TASK_ID` env var, used in task ConfigMap naming and Datadog tags. Required for implementation runs. `naming.rs:38,114,201`, `templates.rs:667,677`, `resources.rs:853–857`. |
| `service` | *(required)* | Fallback for `workingDirectory`. Drives PVC naming (`workspace-{service}`), Kubernetes labels, Datadog tags, and is passed into every template context. `naming.rs:137,181`, `resources.rs:601–610`, `templates.rs:678,4087`. |
| `implementationAgent` | `null` | **Primary agent identifier.** Lowercase slug (e.g. `"rex"`, `"blaze"`). Used for: job naming, system prompt template selection (`agents/{agent}/{job}.md.hbs`), Datadog `agent` tag, and auto-deriving `githubApp` when absent (`"rex"` → `"5DLabs-Rex"`). Takes precedence over `githubApp`. `naming.rs:60–66`, `templates.rs:4113–4120,4131–4136`, `resources.rs:461,657,817`. |

## Repository Configuration

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `repositoryUrl` | *(required)* | Cloned into the pod workspace. Passed into every template context. Also used to extract `repo_slug` for Stitch review and Codex templates. `templates.rs:679,2588,2801,4727–4732`. |
| `docsRepositoryUrl` | *(required)* | Source for task definitions (PRD, acceptance criteria). Cloned separately. Passed into every template context. `templates.rs:680,773,1085,1919,3414`. |
| `docsProjectDirectory` | `null` (`""` fallback) | Subdirectory within docs repo containing project task definitions. Passed as template context variable. `templates.rs:686–688,1090–1092,1372,3419`. |
| `workingDirectory` | `null` → falls back to `service` | Resolved via `get_working_directory()`: explicit value if non-empty, otherwise `service`. Passed into every template context. `templates.rs:4084–4088,683,1087,1922,3416`. |
| `docsBranch` | `"develop"` | Git branch to checkout in the docs repo. Passed into template context, sometimes aliased as `source_branch`. `templates.rs:681–682,774,1086,1200,1644,1920–1921,3415`. |

## Model & CLI Selection

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `model` | `""` (empty) | Model identifier for the AI CLI. When empty, resolved from the first ACP entry's first model. Passed as template context variable and used in container script CLI flags. Also used in job naming (shortened), Kubernetes labels, Datadog tags. Overridden by `cliConfig.model` when present. `naming.rs:77`, `resources.rs:96–98,2322,2570,2634`, `templates.rs:211,1374,2349`. |
| `cliConfig` | `null` | Central config block: determines CLI type, model, provider, provider URL, temperature, max tokens, model rotation, and settings. Auto-populated from agent config if absent (`populate_cli_config_if_needed`). The `cli` field determines which container template is rendered. Timeout sets pod `activeDeadlineSeconds`. `resources.rs:90–98,108,174,387,3106–3148`, `templates.rs:214,289–318,328`, `naming.rs:110,130,175,349–375`. |
| `promptStyle` | `null` | Appends a variant suffix to the system prompt template path (e.g., `"minimal"` → `agents/rex/coder-minimal.md.hbs`). `templates.rs:4519–4527`. |

## Authentication

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `githubApp` | `null` | **Deprecated — prefer `implementationAgent`.** GitHub App name (e.g. `"5DLabs-Rex"`). When absent, auto-derived from `implementationAgent` as `"5DLabs-{Capitalized}"`. The secret `{githubApp}-credentials` is mounted for git auth. Used in cto-config.json agent lookup for skills/tools. `templates.rs:4105–4123`, `resources.rs:414–418,1405–1476`, `naming.rs:60–73`. |
| `githubUser` | `null` | **Deprecated.** Legacy GitHub username for commit attribution. Fallback for Git author identity after `implementationAgent` and `githubApp`. `resources.rs:2596–2598,2769,2807`. |

## Execution Control

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `contextVersion` | `1` | Appended as `-v{version}` to job names and ConfigMap names for uniqueness. Included in Datadog tags. Incremented by controller on retries. `naming.rs:39,91,114,201`, `resources.rs:855–857`, `controller.rs:1445`. |
| `continueSession` | `false` | Resolved via `get_continue_session()`: true if retry count > 0 OR explicitly set. Passed into template context (controls session resumption). Also in Datadog tags. `templates.rs:4092–4098,684,1088,1923,3417,3739`, `resources.rs:2352–2353`. |
| `overwriteMemory` | `false` | Passed into template context — templates use it to decide whether to reset CLAUDE.md / memory files before starting. `templates.rs:685,1089,1924,3418,3740`. |
| `freshWorkspace` | `null` (defaults to `true` for intake, `false` otherwise) | Controls whether the existing PVC is deleted before creating a new one. Explicit `true` always deletes; `null` uses the `runType=="intake"` heuristic. `resources.rs:683–694,448–451`. |
| `enableDocker` | `true` | When `true`, adds DinD sidecar container, Docker socket volume mount, Docker group/security context, and DinD init container. Also in Datadog tags. `resources.rs:1329–1330,1586,1651,1708,2355`. |

## Environment & Secrets

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `env` | `{}` | All key-value pairs injected as container env vars. Also mined for special keys: `PR_NUMBER`, `PR_URL`, `INTAKE_CONFIGMAP`, `LINEAR_PROJECT_ID`, `TASK_LANGUAGE`, `MAX_RETRIES`, `TELEMETRY_ENABLED`. Propagated to watcher. `resources.rs:2437–2438,1190,1213,2620`, `templates.rs:1549,2560,4158,4185,4206`. |
| `envFromSecrets` | `[]` | Each entry creates a `valueFrom.secretKeyRef` env var. Only processed when `taskRequirements` is absent (prevents conflicts). `resources.rs:2529–2533`. |
| `taskRequirements` | `null` | Base64-encoded YAML decoded by `process_task_requirements()` — extracts secrets and env vars into container env. When present, suppresses `envFromSecrets`. `resources.rs:2426–2454,1495`, `templates.rs:1555–1558,3806`. |
| `serviceAccountName` | `null` | Set as `pod_spec.serviceAccountName` on the Job. Falls back to controller-level default. `resources.rs:2282–2297`. |

## Integrations

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `linearIntegration` | `null` | When enabled, adds a Linear sidecar container with shared `/status` volume for status sync. Injects session ID, issue ID, team ID as env vars. `resources.rs:1785–1830,2680`. |
| `escalationPolicy` | `null` | Serialized as JSON and injected into the MCP tools server config, enabling controlled mid-session tool escalation with allow/deny globs. `templates.rs:1827`. |

## Content Overrides

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `promptModification` | `null` | Raw markdown written directly to `prompt.md` in the task ConfigMap, overriding normal PRD pipeline. Used by healer CI runs. `templates.rs:180–189`. |
| `acceptanceCriteria` | `null` | Raw markdown written to `acceptance-criteria.md` in the task ConfigMap. The acceptance probe checks these checkboxes after task completion. `templates.rs:193–204`. |
| `remoteTools` | `null` | Comma-separated glob patterns parsed and merged into `client-config.json` under `remoteTools`. Overrides/extends tools from agent config. `templates.rs:3187–3217,709,788,1127,1661`. |
| `localTools` | `null` | Comma-separated local MCP server names enabled in `client-config.json` under `localServers`. `templates.rs:3206–3212`. |
| `subtasks` | `null` | Serialized into every template context as `subtasks` array for multi-step task execution. `templates.rs:704,783,1113,1212,1656,1951,2035,3438,3522,3750,3892`. |

## Phase Booleans (Lobster-controlled)

These booleans are **not directly read by the controller** — they are stored in the CRD as metadata for the external Lobster workflow engine to decide which phases to run.

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `quality` | `true` | Lobster: run quality review phase (Cleo agent). |
| `security` | `true` | Lobster: run security scan phase (Cipher agent). |
| `testing` | `true` | Lobster: run testing phase (Tess agent). |
| `deployment` | `false` | Lobster: run deployment phase (Bolt agent). Opt-in only. |

## AI Provider Configuration

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `acp` | `null` | Array of AI-CLI-Provider entries. Each has `cli`, `providers[]` with `name`, `credits` (mandatory, dynamic), `baseUrl`, `apiKeyEnvVar`, and `models[]` with `name`, `score`, `thinkingLevel`. Serialized as JSON → `CTO_ACP_CONFIG` env var. Also used in naming to extract CLI/provider names. `resources.rs:1600–1603`, `naming.rs:352–353,367–368`. |
| `openclaw` | `null` (defaults to Fireworks+Google) | OpenClaw gateway provider config. Contains `providers[]` with `name` (slug), `baseUrl`, `apiKeyEnvVar`, `api` (adapter type), and `models[]`. Serialized as `CTO_OPENCLAW_CONFIG` env var. Also drives `openclaw.json` ConfigMap rendering — when absent, `default_providers()` supplies fireworks+google defaults. `resources.rs:1607–1611`, `templates.rs:1419–1423`. |

## Deprecated / Obsolete

| CRD Field | Default | Controller Processing |
|-----------|---------|----------------------|
| `watcherConfig` | `null` | **Obsolete.** Was used for dual-model watcher pattern. `watcher.rs:74–85,113–156,227–289`. |
| `watcherFor` | `null` | **Obsolete.** Marked this CodeRun as a watcher for the named executor. `watcher.rs:69`. |
