# Morgan Avatar — Persistent OpenClaw Fleet Topology

> **Status**: design (2026-04). Companion to `model-dag-plan.md` (what to run)
> and `asset-feasibility.md` (quality gates). This doc covers **who runs it**
> (fleet roles), **how they coordinate** (state exchange + execution matrix),
> and **the CRD shape** the fleet will eventually become.

---

## 1. Core principle

The conductor is **always running**; workers are **ephemeral**. The conductor
owns the DAG state machine; workers own nothing except their current job.
All state lives in durable shared stores — not in process memory.

```
                       ┌──────────────────────────────────────────┐
                       │          avatar-conductor                │
                       │  (persistent OpenClaw agent deployment)  │
                       │                                          │
                       │  ┌────────────┐   ┌──────────────────┐  │
                       │  │ DAG state  │   │  manifest store  │  │
                       │  │  machine   │◄──│  (Scenario tags/ │  │
                       │  │            │   │  pipeline-state  │  │
                       │  └─────┬──────┘   │  .json on PVC)   │  │
                       │        │          └──────────────────┘  │
                       └────────┼─────────────────────────────────┘
                                │  spawn / queue via NATS + ACP sessions
            ┌───────────────────┼────────────────────────────────────────┐
            │                   │                                        │
            ▼                   ▼                                        ▼
   ┌────────────────┐  ┌─────────────────┐  ┌──────────────────────────────────┐
   │ scenario-gen   │  │ avatar-validator │  │ ops-watcher (persistent sidecar) │
   │ workers (×N)   │  │ (K8s Job, CPU)  │  │ + disco-observer (persistent)    │
   └────────────────┘  └─────────────────┘  └──────────────────────────────────┘
            │                   │
            ▼                   ▼
   Scenario hosted       GLB validation
   model runs            report + label
   → asset IDs           → conductor queue
```

---

## 2. Fleet roles

### 2.1 `avatar-conductor` — Persistent coordinator

| Attribute | Value |
|---|---|
| Persistence | Always-running `Deployment` (1 replica, rolling restart safe) |
| Base chart | `infra/charts/openclaw-agent` (inherits NATS plugin, skills, MCP, promtail) |
| OpenClaw skills | `orchestrator.md`, `reboot-continuity.md`, avatar-specific DAG skill |
| Identity | OpenClaw agent ID `avatar-conductor` |
| Compute | 500m CPU / 2Gi mem — no GPU, no Scenario secrets (workers carry those) |

**Responsibilities:**

- Owns the DAG state machine: `idle → source-prep → gen → validate → branch →
  repair → runtime-proof → done`.
- Reads/writes `avatar/manifests/pipeline-state.json` (the single source of
  truth for all Scenario asset IDs, job IDs, node statuses, and validation
  scores).
- Dispatches workers: `sessions_spawn` ACP calls with hard-edged task
  descriptions and explicit asset IDs / acceptance criteria.
- Applies serialization gates (§5) — never spawns a downstream node until the
  upstream gate score is written to the manifest.
- Receives completion events from workers via NATS `avatar.conductor.in` and
  forwards human-escalation notifications via `avatar.discord.notify`.

### 2.2 `scenario-gen-worker` — Ephemeral generation workers

| Attribute | Value |
|---|---|
| Persistence | Ephemeral — spawned per DAG node, terminates after result |
| Spawn mechanism | `sessions_spawn agentId=claude runtime=acp mode=run` from conductor |
| Tools | Scenario MCP (`run_model`, `upload_asset`, `manage_assets`, `display_asset`), `gh`, git |
| Parallelism | Up to N concurrent, one per DAG track (A/B/C/D in `model-dag-plan.md`) |
| Output | Scenario asset IDs + job metadata → written to `pipeline-state.json`, NATS event to conductor |

**Lifecycle:** conductor spawns with exact task string like:
> "Run Scenario Hunyuan 3D 3.1 Pro on asset_id=`<source_id>` (track B).
> Write result asset IDs and Scenario job_id to
> `avatar/manifests/pipeline-state.json` under `nodes.track-b-hunyuan`.
> Post completion to NATS `avatar.conductor.in`. Fail loudly if model or
> asset unavailable."

### 2.3 `avatar-validator` — K8s Job (CPU)

| Attribute | Value |
|---|---|
| Persistence | K8s `Job` — terminates after scoring |
| Trigger | Conductor creates Job after gen worker posts asset ID |
| Image | `ghcr.io/5dlabs/avatar-validator` (Python + Blender headless) |
| Script | `scripts/2026-04/validate-avatar-glb.py` |
| Profile | `talkinghead`, `morgan-canine`, or `vrm` (set per DAG branch) |
| Output | Validation report JSON → PVC + manifest; `acceptance_label` → conductor queue |

**Acceptance labels** (from `asset-feasibility.md`):
`runtime-ready` | `needs-face-authoring` | `needs-rigging` | `mesh-only` | `archive`

The validator never promotes a candidate on its own — it only writes a score
and label. Promotion decisions (branching to repair or to runtime proof) belong
to the conductor.

### 2.4 `artifact-publisher` — Ephemeral sub-agent

Spawned by conductor when a candidate reaches `runtime-ready` or is approved
for staging:
- Downloads GLB/VRM via `manage_assets download`.
- Pushes to GitHub release or object store via `gh release upload`.
- Tags Scenario asset with `avatar-validated` or `avatar-archived`.
- Updates manifest `artifact_urls` and `published_at` fields.
- Opens a PR (or updates existing) with the manifest delta.

### 2.5 `pr-shepherd` — Background sub-agent

Standard shepherd pattern (see AGENTS.md): spawned whenever a PR is opened.
- Monitors CI, rebases/resolves conflicts (prefer `main` for unrelated files).
- Addresses Stitch review comments.
- Merges when green.
- Escalates only for semantic conflicts or missing required approvals.

### 2.6 `disco-observer` — Persistent sidecar

| Attribute | Value |
|---|---|
| Persistence | Sidecar container in `avatar-conductor` pod, or standalone low-resource deployment |
| Tools | Discord bridge REST/WebSocket, Linear API via NATS bridge |
| Role | One-way relay: human feedback → NATS `avatar.conductor.in` events |

Surfaces:
- Approval signals from `#avatar-pipeline` Discord channel.
- Linear ticket state changes (e.g., "accepted" label on avatar task).
- Never writes code; never makes generation decisions.

### 2.7 `research-worker` — Ephemeral sub-agent

Spawned on demand for one-off research tasks (new Scenario models, provider
credit checks, upstream repo analysis):
- Uses `web_search`, `perplexity`, GitHub search tools.
- Writes structured output to `docs/2026-04/avatar/` or session memory.
- Terminates after delivering the research blob to conductor.

### 2.8 `ops-watcher` — Persistent sidecar

Low-resource (~100m CPU / 256Mi) persistent process alongside the conductor:
- Polls Scenario job status via `manage_jobs list` every 60s.
- Monitors `pipeline-state.json` for stale `in_progress` nodes (>30 min with
  no update → re-queue or escalate).
- Checks cluster health (`kubectl get pods -n avatar`).
- Triggers self-healing: re-queues failed generation jobs (up to 2 retries),
  escalates to Discord on repeated failure.

---

## 3. State exchange

### 3.1 Manifest file — primary ledger

**Path:** `avatar/manifests/pipeline-state.json` (committed to `main` after
each gate transition; also kept on shared PVC for in-flight reads)

```jsonc
{
  "schema_version": "1",
  "run_id": "2026-04-<hash>",
  "source_asset_id": "asset_xxx",
  "scenario_project_id": "project_xxx",
  "dag_state": "gen",          // idle|source-prep|gen|validate|repair|runtime-proof|done|error
  "nodes": {
    "source-upload": { "status": "done", "asset_ids": ["asset_xxx"] },
    "track-b-hunyuan": {
      "status": "in_progress",
      "scenario_job_id": "job_xxx",
      "spawned_at": "2026-04-26T00:00:00Z"
    },
    "track-c-tripo": { "status": "pending" }
  },
  "candidates": {
    "asset_yyy": {
      "track": "track-b-hunyuan",
      "acceptance_label": null,  // set by validator
      "validation_report_url": null,
      "artifact_url": null
    }
  },
  "promoted": null,   // asset_id of the final accepted candidate
  "escalations": [],
  "updated_at": "2026-04-26T00:00:00Z"
}
```

Rules:
- Workers write their output field, then issue a NATS event. **Never** rewrite
  unrelated fields.
- The conductor is the only process that updates `dag_state` and `promoted`.
- `updated_at` is always a UTC ISO-8601 string.

### 3.2 Scenario asset tags/collections

Scenario itself is the secondary ledger for all generated blobs:

| Tag | Meaning |
|---|---|
| `avatar-source` | Canonical Morgan source/reference uploads |
| `avatar-candidate` | Raw gen output, not yet validated |
| `avatar-validated` | Passed mechanical checks (`runtime-ready`) |
| `avatar-repair-needed` | Needs rigging / face-authoring pass |
| `avatar-archived` | Failed or superseded |

Collections mirror pipeline runs:
`morgan-avatar-run-<run_id>` contains all source + candidate + output assets
for that run, for cost and provenance tracking.

### 3.3 NATS topics

| Topic | Direction | Payload |
|---|---|---|
| `avatar.conductor.in` | Worker → Conductor | `{ node_id, status, asset_ids?, error? }` |
| `avatar.conductor.out` | Conductor → Worker | Spawn instructions (duplicated in `sessions_spawn` call) |
| `avatar.discord.notify` | Conductor → Discord bridge | Human-readable status string, escalation flag |
| `avatar.ops.heartbeat` | ops-watcher → conductor | `{ checks: [...], stuck_nodes: [...] }` |

### 3.4 Session SQL todos (per-session)

Standard todo tracking (see AGENTS.md SQL patterns) for intra-session
task decomposition. Not durable cross-restart — manifest file is the durable
source.

---

## 4. Parallel execution matrix

### Can run concurrently

| Parallel group | Workers | Gated by |
|---|---|---|
| Source conditioning tracks A/B/C | 3 × `scenario-gen-worker` | Source uploaded (`source-upload: done`) |
| Primary 3D candidates (Hunyuan, Tripo P1, Tripo 3.1) | 3 × `scenario-gen-worker` | Source cleanup done (`source-prep: done`) |
| Validation of multiple candidates | N × `avatar-validator` Jobs | Each Gen result available |
| Repair passes on different candidates | N × `scenario-gen-worker` (retopo, rig, face) | Per-candidate validation label set |
| Research workers | Any N | Always async; no gate dependency |

### Serialization gates

```
source-upload ─► source-prep ─► [gen tracks in parallel] ─► [validation in parallel]
                                                                      │
                                                 ┌────────────────────┼────────────────────┐
                                                 │                    │                    │
                                           runtime-ready         repair needed         archive
                                                 │                    │
                                        runtime-proof          [repair pass]
                                                 │                    │
                                          browser lab ◄──────────────┘
                                                 │
                                     human approval gate
                                                 │
                                        artifact-publisher → PR
```

**Gate conditions (conductor checks before advancing):**

| Gate | Condition |
|---|---|
| `source-prep → gen` | At least one `source-prep` track writes clean multiview refs |
| `gen → validate` | Scenario job terminates with asset ID available |
| `validate → branch` | `acceptance_label` written to manifest |
| `branch=runtime-proof → publish` | Browser lab render passes (human-inspected or automated) |
| `publish → done` | PR merged, artifact URL in manifest |

---

## 5. Human escalation points

The conductor should **never block silently**. It only escalates for:

| Trigger | Action |
|---|---|
| Scenario secret not available in cluster Secret | Post to `#avatar-pipeline` + pause DAG; ops-watcher detects |
| Final `runtime-ready` candidate exists | Discord notification requesting visual sign-off before `artifact-publisher` runs |
| All 3D tracks return `archive` label | Escalate to `#avatar-pipeline`; no automated fallback |
| Scenario credits exhausted / cost ceiling breach | Pause DAG, notify Discord with current candidate state |
| 3+ repeated generation failures on a node | ops-watcher escalates; conductor does not auto-retry beyond 2 |
| Semantic merge conflict in pipeline manifest / avatar code | pr-shepherd escalates |

All other decisions (retry logic, branching on label, track selection,
provider sequence) are conductor-autonomous.

---

## 6. Helm deployment shape (near-term)

### Phase 1 — Minimal (ship now)

No new infra; use existing `openclaw-agent` chart:

```yaml
# infra/charts/morgan-avatar-agent/values.yaml additions
conductor:
  enabled: true
  agentId: "avatar-conductor"
  skill: "avatar-dag"
  natsSubjects:
    - "avatar.conductor.in"
    - "avatar.ops.heartbeat"
  manifest:
    pvcName: "avatar-conductor-state"
    path: "/data/pipeline-state.json"
  discord:
    bridgeUrl: "http://discord-bridge.bots.svc.cluster.local"
    channel: "avatar-pipeline"

opsWatcher:
  enabled: true
  pollIntervalSeconds: 60
  maxNodeStalenessMinutes: 30
```

Workers are spawned as ACP sessions; no additional Helm chart needed.
`morgan-avatar-agent` deployment becomes the conductor pod.

### Phase 2 — Hardened

- `avatar-validator` gets its own `Job` template in the chart.
- `disco-observer` becomes a sidecar container spec.
- NATS persistent subjects with replay for ops-watcher.
- `artifact-publisher` gets a dedicated `ServiceAccount` with `gh` token mounted.

### Phase 3 — CRD

```yaml
apiVersion: 5dlabs.ai/v1alpha1
kind: AvatarPipelineRun
metadata:
  name: morgan-2026-04-run-01
spec:
  sourceImage: "s3://avatar-assets/morgan.jpg"    # or Scenario asset_id
  scenarioProjectId: "project_xxx"
  dagConfig: "avatar/manifests/dag-config.yaml"
  qualityThresholds:
    minAcceptanceLabel: "runtime-ready"
    maxCreditsPerRun: 50
  conductorRef:
    agentId: "avatar-conductor"
  escalation:
    discordBridgeUrl: "http://discord-bridge.bots.svc.cluster.local"
    channel: "avatar-pipeline"
status:
  dagState: "gen"
  promotedAssetId: null
  candidateCount: 3
  lastTransitionTime: "2026-04-26T00:00:00Z"
  conditions:
    - type: "SourceReady"
      status: "True"
    - type: "CandidateValidated"
      status: "False"
    - type: "RuntimeReady"
      status: "False"
```

The CRD reconciler pattern follows `crates/controller/src/crds/coderun.rs` (existing
`CodeRun` implementation): watch → reconcile → update status → requeue.

---

## 7. Fleet topology diagram (full)

```
                     Human (Discord #avatar-pipeline)
                              ▲  ▼  (escalation only)
                              │  │
                    ┌─────────┴──┴──────────────┐
                    │   disco-observer           │
                    │   (persistent sidecar)     │
                    └─────────────┬──────────────┘
                                  │ NATS avatar.conductor.in
                    ┌─────────────▼──────────────┐
                    │      avatar-conductor       │
                    │  (persistent OpenClaw pod)  │
                    │                             │
                    │  ┌──────────────────────┐   │
                    │  │  DAG state machine   │   │
                    │  │  pipeline-state.json │   │
                    │  └──────┬───────────────┘   │
                    │         │ sessions_spawn     │
                    └─────────┼─────────────────── ┘
                              │
          ┌───────────────────┼────────────────────────────┐
          │                   │                            │
          ▼                   ▼                            ▼
  scenario-gen-worker  avatar-validator          research-worker
  (×N ephemeral ACP)   (K8s Job, CPU)           (ephemeral ACP)
          │                   │
          │ asset_ids          │ validation_report
          │                   │ + acceptance_label
          ▼                   ▼
   Scenario API ◄──── pipeline-state.json ◄──── ops-watcher
   (asset store)        (manifest on PVC)        (persistent sidecar)
          │
          ▼
   artifact-publisher
   (ephemeral ACP)
          │
          ▼
   GitHub Release
   + PR (pr-shepherd)
```

---

## 8. Implementation checklist

Near-term (Phase 1) tasks, in dependency order:

- [ ] Add `pipeline-state.json` schema + initial skeleton to `avatar/manifests/`
- [ ] Write `avatar-dag` OpenClaw skill (conductor's decision rules + DAG state machine)
- [ ] Add `conductor` stanza to `infra/charts/morgan-avatar-agent/values.yaml`
- [ ] Add `pvc` template to `morgan-avatar-agent` chart for manifest storage
- [ ] Wire NATS `avatar.conductor.in` subscription in conductor skill
- [ ] Implement `ops-watcher` as conductor sidecar (simple shell/Python loop)
- [ ] Write first conductor spawn template strings for tracks A–D (from `model-dag-plan.md`)
- [ ] Validate: conductor spawns one scenario-gen-worker, worker writes manifest, conductor advances state

Phase 2/3 tasks are gated on Phase 1 working end-to-end.
