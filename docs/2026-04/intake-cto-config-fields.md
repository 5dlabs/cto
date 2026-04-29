# Intake-only `cto-config.json` field map

Scope: every `cto-config.json` field consumed by the **intake pipeline**
(`intake/workflows/*.lobster.yaml`, `intake/scripts/*`,
`apps/intake-util/`, plus the harness/template partials rendered by the
intake-driven Morgan/play flow). Controller-only fields (CRD reconciliation,
Helm rendering, runtime watchers, model rotation, etc.) are intentionally
excluded.

Worktree: `/Users/edge_kase/5dlabs/cto-intake-video` on branch
`intake-video`. Source files inspected at HEAD on that branch.

---

## TL;DR — obsolete / orphaned fields in `cto-config.json`

These fields exist in the live `cto-config.json` but are either:
(a) never read by any intake code path, or
(b) superseded by the recent **agents-repo + Context7 catalog** refactor
    where the canonical MCP server registry / tool catalog moved out of
    `cto-config.json`.

| Field | Why obsolete |
|---|---|
| `agents.<name>.tools.localServers.<server>.enabled` | All values in `cto-config.json` today are `null` or `false`; consumed only by `inventory-effective-tools` which produces a noisy, unused inventory. Local server enablement now lives in the **cto-agents** repo provisioning, not here. |
| `agents.<name>.tools.localServers.<server>.tools[]` | Same as above. Every entry today is `[]`. Local-server tool selection now flows from the agents-repo `client-config.json` template. |
| `agents.<name>.tools.localServers.<server>` (key names) | The literal server names committed today are: `remotion-documentation` (blaze), `livekit-mcp`, `elevenlabs-mcp`, `deepgram-mcp` (angie). Per the **agents-repo + Context7 catalog refactor** these names are no longer the source of truth — the canonical local-server registry is the `localServers` block in **cto-agents** `client-config.json`, generated from the Context7 / MCP catalog. The `cto-config.json` copies are leftover and drift silently. |
| `agents.<name>.tools.remote[]` (per-agent literal tool names) | Still read by `discover-tools` and `inventory-effective-tools`, but the tool-prefix → MCP-server mapping (e.g. `context7_*`, `octocode_*`, `kubernetes-mcp_*`, `argocd_*`, `github_*`) is now sourced from the cto-agents repo + Context7 catalog. Keeping per-agent allowlists in `cto-config.json` duplicates that registry; only the agent ↔ tool affinity is meaningful here. |
| `defaults.play.modelCatalog` (`$ref` to `model-providers.json`) | Not read by any intake script, workflow step, or `intake-util` command. The catalog is consumed by the controller (CodeRun rendering), not by intake. |
| `defaults.play.acp` (`$ref` to `model-providers.json`) | Not read by intake. Superseded by `defaults.play.agentCommunication` (which itself is missing from `cto-config.json` — see "Should-exist-but-doesn't" below). The string `"acp"` is now an explicit deprecated alias inside `intake-util/src/invoke-agent.ts`. |
| `defaults.play.openclaw` (`$ref` to `model-providers.json`) | Not read by intake. Controller-side. |
| `defaults.play.healerEndpoint` | Not read by intake. Healer routing happens at controller dispatch time. |
| `defaults.play.autoMerge` | Not read by intake. Merge-gate (Atlas) is downstream of intake. |
| `defaults.play.securityMaxRetries` | Not read by intake. Only `implementationMaxRetries` and `qualityMaxRetries` are consumed by `intake-util generate-workflows`. |
| `defaults.play.testingMaxRetries` | Same — not read by intake. |
| `defaults.play.watcher`, `defaults.play.quality`, `defaults.play.security`, `defaults.play.testing`, `defaults.play.deployment`, `defaults.play.repository`, `defaults.play.service`, `defaults.play.docsRepository`, `defaults.play.docsProjectDirectory`, `defaults.play.workingDirectory` | None of these are read by any intake-side jq, Python, or `intake-util` code path. They are controller/play-runtime concerns. |
| `defaults.remediation`, `defaults.monitor`, `defaults.stitch`, `defaults.scm` | Not referenced by anything under `intake/` or `apps/intake-util/`. Controller-only. |
| `agents.<name>.cli`, `agents.<name>.model` | Not read by intake jq/Python directly. The harness/template partials (`templates/_shared/partials/tools-config.sh.hbs`, `templates/agents/morgan/play.md.hbs`) read the **whole file** but only `cat` it for visibility or extract `agents.<key>.tools` — they do not consume `cli` or `model`. These fields are a controller concern (CodeRun spec). |
| `agents.<name>.githubApp` | Not read by intake. Controller-only (used to assemble GitHub App auth in CRD reconciler). |
| `agents.<name>.modelRotation` (`primary`, `fallback`, `tertiary`) | Not read by intake. Rotation is enforced at runtime by the controller. |
| `agents.bolt.maxRetries` | Not read by intake. Controller-only. |

> **Note on `.agents.list` and `.agents.svc`** — these strings appear in
> `intake/scripts/verify-lobster-openclaw.sh` and
> `apps/intake-util/src/invoke-agent.ts` respectively, but they are **not**
> `cto-config.json` paths. The first targets a different file
> (`intake/config/openclaw-llm-task.json`) that has an `agents.list[]`
> array; the second is the Kubernetes service hostname
> `<agent>.agents.svc:18789`. They are red herrings for this map.

---

## Intake-active fields

Columns:
- **Path**: dotted JSON path inside `cto-config.json`.
- **Consumer**: the file:line in the intake codebase that reads the field.
- **Purpose**: what the value actually drives.
- **Status**: `active` | `active-but-redundant` | `obsolete`.

### `defaults.intake.models.*`

| Path | Consumer (file:line) | Purpose | Status |
|---|---|---|---|
| `defaults.intake.models.tiers.primary.provider` | `intake/workflows/pipeline.lobster.yaml:74-92` (`load-config`); `intake/scripts/resume-task-refinement.sh:32-` | Default LLM provider for the "primary" tier (refine-tasks, capability-analysis, etc.). Exposed as `${model_primary_provider}`. | active |
| `defaults.intake.models.tiers.primary.model` | `intake/workflows/pipeline.lobster.yaml:74-92`; `intake/scripts/resume-task-refinement.sh:32-` | Model id for the primary tier (`${model_primary}`). | active |
| `defaults.intake.models.tiers.fast.provider` | `intake/workflows/pipeline.lobster.yaml:74-92`; `intake/scripts/quality-gate.sh:37`; `intake/scripts/resume-task-refinement.sh:32-` | Provider for low-cost fast-path scoring (quality-gate, lightweight checks). | active |
| `defaults.intake.models.tiers.fast.model` | `intake/workflows/pipeline.lobster.yaml:74-92`; `intake/scripts/quality-gate.sh:38`; `intake/scripts/resume-task-refinement.sh:32-` | Model id for the fast tier. | active |
| `defaults.intake.models.tiers.frontier.provider` | `intake/workflows/pipeline.lobster.yaml:74-92`; `intake/workflows/intake.lobster.yaml:1727` (`analyze-required-capabilities`); `intake/scripts/resume-task-refinement.sh:32-` | Provider for the strongest tier (capability analysis, deep reasoning). | active |
| `defaults.intake.models.tiers.frontier.model` | same as above | Model id for the frontier tier. | active |
| `defaults.intake.models.committee[].provider` | `intake/workflows/pipeline.lobster.yaml:74-92`; `intake/scripts/resume-task-refinement.sh:32-` | Per-voter provider for committee-of-N evaluation (exposed as `${voter_<N>_provider}`). | active |
| `defaults.intake.models.committee[].model` | same | Per-voter model id (`${voter_<N>_model}`). | active |
| `defaults.intake` (other keys) | none | The shape `defaults.intake.models.{tiers,committee}` is the only part read; any sibling keys under `defaults.intake` are unused by intake today. | obsolete (if any present) |

### `defaults.linear.*`

| Path | Consumer (file:line) | Purpose | Status |
|---|---|---|---|
| `defaults.linear.teamId` | `intake/workflows/pipeline.lobster.yaml:74-92` (`load-config`); `intake/scripts/linear-resolve-team-id.sh:23`; `intake/scripts/run-local-bridges.sh:8` | Default Linear team UUID for issue ingestion + bridge wiring. Exposed as `${linear_team_id}`. | active |

### `defaults.play.*`

| Path | Consumer (file:line) | Purpose | Status |
|---|---|---|---|
| `defaults.play.agentHarness` (`$ref` → `model-providers.json#/agentHarness`) | `intake/scripts/generate-pr-body.sh:233` | The `$ref` is **resolved indirectly** — `generate-pr-body.sh` reads the value as a string for PR-body annotation. The actual harness map is consumed by `apps/intake-util/src/generate-workflow.ts:167` (`config.agentHarness`) when intake-util loads `model-providers.json` standalone, not via cto-config. | active-but-redundant (only the `$ref` literal is read here; intake-util loads the catalog directly) |
| `defaults.play.implementationMaxRetries` | `apps/intake-util/src/index.ts:642-647`; `apps/intake-util/src/generate-workflow.ts:197` | Max attempts for implementation phase in the generated per-task Lobster workflow. | active |
| `defaults.play.qualityMaxRetries` | `apps/intake-util/src/index.ts:642-647`; `apps/intake-util/src/generate-workflow.ts:198, 394` | Max attempts for quality-gate phase in the generated per-task Lobster workflow. | active |
| `defaults.play.agentCommunication` | `apps/intake-util/src/index.ts:642` (typed as field on `playConfig`) — **field is missing from `cto-config.json`**, so the read silently falls through to the default. See "Should-exist-but-doesn't". | Selects subagent vs. A2A transport for cross-agent calls in generated workflows. | **active in code, missing from data** |

### `agents.<name>.tools.*`

| Path | Consumer (file:line) | Purpose | Status |
|---|---|---|---|
| `agents.<name>.tools.remote[]` | `intake/workflows/pipeline.lobster.yaml:971-983` (`discover-tools`); `intake/workflows/intake.lobster.yaml:1669` (`inventory-effective-tools`); `templates/_shared/partials/tools-config.sh.hbs:37,44` | Per-agent allowlist of remote MCP tool ids (e.g. `context7_resolve_library_id`, `kubernetes-mcp_pods_list`). `discover-tools` derives the MCP server set by splitting on the first `_`. `tools-config.sh.hbs` merges the union into the agent's runtime `client-config.json`. | active-but-redundant (canonical registry now in cto-agents + Context7) |
| `agents.<name>.tools.localServers` (object) | `intake/workflows/intake.lobster.yaml:1670-1675` (`inventory-effective-tools`) | Merged into the inventory blob produced for `analyze-required-capabilities`. | active-but-redundant (always-empty values in current data; see TL;DR) |
| `agents.<name>.tools.localServers.<server>.enabled` | same | Boolean toggle for the local server. | obsolete |
| `agents.<name>.tools.localServers.<server>.tools[]` | same | Allowlist of tool ids exposed by the local server. | obsolete |

### `agents.<name>.skills.*`

| Path | Consumer (file:line) | Purpose | Status |
|---|---|---|---|
| `agents.<name>.skills.default[]` | `intake/workflows/intake.lobster.yaml:1688-1690` (`inventory-effective-skills`) | Always-on Claude Skills assigned to the agent regardless of job type. Merged into the global skills inventory used by capability analysis. | active |
| `agents.<name>.skills.coder[]` | same | Skills enabled when the agent is dispatched as a coder (Rex/Blaze/Grizz/Bolt/Block/Angie). | active |
| `agents.<name>.skills.healer[]` | same | Skills enabled when the agent is dispatched as a healer (Morgan/Rex/Blaze/Angie). | active |
| `agents.<name>.skills.intake[]` | same | Skills enabled during the intake phase (only Morgan has this today). | active |
| `agents.<name>.skills.quality[]` | same | Skills enabled when the agent runs as a quality reviewer (Cleo). | active |

> Note: `inventory-effective-skills` flattens `to_entries[].value | .[]`, so
> **any** skill bucket (`default`, `coder`, `healer`, `intake`, `quality`,
> or any future job-type) is picked up automatically — the schema here is
> open-ended.

### Whole-file consumers (no specific path)

| Consumer | Purpose | Notes |
|---|---|---|
| `templates/agents/morgan/play.md.hbs:17-21,105,122` | `cat`s `/config/project/cto-config.json` into Morgan's prompt context so Morgan can see project-specific overrides during intake decomposition. | No specific field is jq'd — the file's existence is what matters. |
| `templates/_shared/partials/tools-config.sh.hbs:17-58` | Reads `agents.<AGENT_KEY>.tools` for runtime tool merging in the harness; copies the full file to `${CLAUDE_WORK_DIR}/project-cto-config.json` for in-pod inspection. | Only `agents.<key>.tools[.remote]` is path-derefed; everything else is just propagated whole. |
| `apps/intake-util/src/index.ts:643-647` (`generate-workflows`) | Loads the file, extracts `defaults.play`, ignores all other top-level keys. | Confirmed by reading the case branch end-to-end. |

---

## Should-exist-but-doesn't

Fields the intake pipeline (or its template/integration-test contract)
expects but which are currently **absent** from `cto-config.json` at HEAD.
These default silently to safe-but-suboptimal values.

| Path | Where it's expected | Default fallback | Recommended action |
|---|---|---|---|
| `defaults.play.agentCommunication` | (a) `apps/intake-util/src/index.ts:642` types `playConfig.agentCommunication?: string`; intake-util's generated workflows propagate the value into per-task Lobster files. (b) `cto-config.template.json:119` has `"agentCommunication": "subagent"`. (c) `intake/tests/pipeline-integration-test.sh:1348-1351` asserts the field exists in the template. | `undefined` → falls back to whatever the generated workflow's downstream `invoke-agent --mode` flag is set to (today implicitly `subagent` per `apps/intake-util/src/invoke-agent.ts:14-25`). | Add `"agentCommunication": "subagent"` (or `"a2a"`) under `defaults.play` in `cto-config.json` to make the runtime transport explicit and to match the template + integration-test contract. |
| `defaults.play.freshStartThreshold` | `cto-config.template.json:_keys` (template lists it under `defaults.play`). | Not consumed by intake-util today, but its presence in the template implies the controller/play side expects it. | Either prune from template (if truly unused) or land the field in `cto-config.json`. Pending controller audit. |
| `defaults.play.frontendMaxRetries` | Template-only (`cto-config.template.json` keys). | Not consumed. | Same as above — reconcile with `implementationMaxRetries` / `qualityMaxRetries`. |
| `defaults.play.maxRetries` | Template-only. | Not consumed. | Decide whether this is the umbrella default that supersedes the per-phase `*MaxRetries`; if not, drop from template. |
| `defaults.play.parallelExecution` | Template-only. | Not consumed by intake-util. | Reconcile or drop. |

---

## Cross-references

- Intake → controller field handoff: see `docs/2026-04/coderun-crd-field-processing.md` for the controller-side fields excluded from this doc.
- Intake observer / feedback loop touchpoints: `docs/2026-03/intake-coordinator.md`, `docs/2026-03/intake-discord-feedback-loop.md`.
- Per-agent tool/skill ground truth: `docs/2026-03/agent-inventory.md`.
- Catalog refactor context (why `tools.localServers.*` and per-agent `tools.remote[]` are now redundant): cto-agents repo `client-config.json` template + Context7 MCP catalog.

---

## Appendix — methodology

Total grep'd intake-side jq/Python/TS accesses against `cto-config.json`
(at HEAD on branch `intake-video`):

1. `intake/scripts/generate-pr-body.sh:233` → `.defaults.play.agentHarness`
2. `intake/scripts/linear-resolve-team-id.sh:23` → `.defaults.linear.teamId`
3. `intake/scripts/quality-gate.sh:37-38` → `.defaults.intake.models.tiers.fast.{provider,model}`
4. `intake/scripts/resume-task-refinement.sh:32-` (Python) → `defaults.intake.models.{tiers.{primary,fast,frontier},committee[]}`
5. `intake/scripts/run-local-bridges.sh:8` → `.defaults.linear.teamId`
6. `intake/workflows/pipeline.lobster.yaml:74-92` (load-config, Python) → all `defaults.intake.models.tiers.*`, `defaults.intake.models.committee[]`, `defaults.linear.teamId`
7. `intake/workflows/pipeline.lobster.yaml:971-983` (discover-tools, Python) → `agents.<name>.tools.remote[]`
8. `intake/workflows/intake.lobster.yaml:1669` → `agents[].tools.remote[]`
9. `intake/workflows/intake.lobster.yaml:1671-1673` → `agents[].tools.localServers`
10. `intake/workflows/intake.lobster.yaml:1689` → `agents[].skills` (every job-type bucket)
11. `apps/intake-util/src/index.ts:643-647` → `defaults.play` (only `implementationMaxRetries`, `qualityMaxRetries`, `agentCommunication` actually consumed downstream)
12. `templates/_shared/partials/tools-config.sh.hbs:37,44` → `agents.<key>.tools[.remote]`

Everything else in `cto-config.json` is **not** consumed by the intake
pipeline at HEAD.
