# OpenClaw Sub-Agent & Resource Orchestration Strategy
# Avatar / Autonomous Operation

**Status:** planning (2026-04)
**North-star goal:** Browser-controllable Morgan GLB/VRM-style runtime avatar delivered through
an OpenClaw-native conductor that coordinates generation, refinement, and hosting sub-agents
in parallel without human micro-management.
**Companion docs:**
- [`docs/2026-04/avatar/model-dag-plan.md`](avatar/model-dag-plan.md) — model pipeline
- [`docs/2026-04/avatar/provider-switch.md`](avatar/provider-switch.md) — EchoMimic ↔ LemonSlice toggle
- [`docs/avatar-architecture.md`](../avatar-architecture.md) — LemonSlice / OpenClaw plugin design
- [`docs/2026-03/cursor-openclaw-subagent-plan.md`](../2026-03/cursor-openclaw-subagent-plan.md) — Cursor ↔ OpenClaw monitoring mesh
- [`docs/agent-presence-hub.md`](../agent-presence-hub.md) — Agent Presence Hub generic design

---

## 1. Task Decomposition

Avatar work decomposes into three **vertical lanes** each containing discrete tasks.
Each task maps to one or more named agents and is a `parallelizable: true` CodeRun
sub-task at the same execution level unless ordered dependency is stated.

### Lane A — Asset Generation (GLB/VRM artifact chain)
| Step | Task | Agent | Depends on |
|------|------|-------|-----------|
| A1 | Design concept brief + style tokens from PRD | Morgan (orchestrator) | — |
| A2 | 2D reference sheet generation (Scenario `txt2img`) | Nova / Blaze | A1 |
| A3 | 3D mesh generation (Scenario `txt23d` or Hyper3D Rodin) | Blaze / Nova | A2 |
| A4 | VRM bone-map & expression-shape setup | Rex (Python tooling) | A3 |
| A5 | Texture & material bake review | Blaze | A4 |
| A6 | Asset contract validation (artifact schema check) | Tess | A4 |
| A7 | Publish to Scenario project + tag release | Blaze | A5, A6 |

### Lane B — Runtime Wiring (avatar-agent pipeline)
| Step | Task | Agent | Depends on |
|------|------|-------|-----------|
| B1 | `avatar/agent/` Python LiveKit agent — implement `AvatarProvider` interface | Rex | — |
| B2 | STT → LLM → TTS → avatar track integration | Rex | B1 |
| B3 | LemonSlice plugin adapter (primary) and audio-only fallback | Rex | B1 |
| B4 | EchoMimic async MP4 adapter behind feature flag | Rex | B1 |
| B5 | `avatar/web/` Next.js room harness | Blaze | B2 |
| B6 | Browser PiP integration (Document Picture-in-Picture API) | Blaze | B5 |
| B7 | GLB/VRM client-side renderer (Three.js / VRM) in room harness | Blaze | A7, B5 |
| B8 | E2E integration test (mock LiveKit room) | Tess | B3, B7 |

### Lane C — Platform / Cluster Enablement
| Step | Task | Agent | Depends on |
|------|------|-------|-----------|
| C1 | Agent Presence Hub Helm chart (generic) | Bolt | — |
| C2 | MuseTalk / LivePortrait V100 build validation (`TORCH_CUDA_ARCH_LIST=7.0`) | Rex + Bolt | — |
| C3 | GPU node labeling + tolerations for avatar workloads | Bolt | — |
| C4 | LiveKit SFU config for avatar room slots | Bolt | — |
| C5 | Healer alarm rules for avatar pod restarts | Healer | C1 |
| C6 | Observability — latency / VRAM dashboards | Keeper | C1, C2 |

**Conductor (Morgan / Orchestrator skill) owns cross-lane dependency gates.**
Lanes A and C may run fully in parallel from the start. Lane B starts B1–B4 immediately;
B7 gates on A7 (final GLB asset) but B5–B6 can proceed before the 3D mesh is ready.

---

## 2. Concurrency Limits

### Per-conductor spawn budget
| Resource tier | Max parallel sub-agents | Notes |
|---|---|---|
| Scenario generation (image/3D) | **3** simultaneous jobs | Scenario rate-limit guard; Rodin jobs are expensive |
| GPU-bound V100 workloads | **1** | Single-node; serialize builds / inference stages |
| CodeRun pods (non-GPU) | **6** | Cluster node budget × 0.6 safety factor |
| Cursor/ACP background shadows | **4** | Per session; more degrades context quality |

### CRD-level gating
```yaml
# Example sub-task marking parallel eligibility
subTasks:
  - id: asset-gen-2d
    parallelizable: true
  - id: asset-gen-3d
    parallelizable: true
  - id: asset-publish
    parallelizable: false   # serialized gate — waits for 2D + 3D
```

### Queue discipline
1. **GPU tasks:** FIFO with one active slot. Pending tasks hold in `Pending` phase; conductor
   NATS-publishes `avatar.gpu.slot.released` on completion so the queue unblocks.
2. **Generation tasks:** Scenario API handles remote queuing; local concurrency capped at 3
   in-flight requests. Use `wait: false` in Scenario `run_model` + poll via `manage_jobs`.
3. **Lint / test CodeRuns:** Uncapped (lightweight); do not count against the 6-slot budget.

---

## 3. Tool / Provider Routing

### Generation surface
| Capability | Primary route | Fallback |
|---|---|---|
| 2D concept art | Scenario `txt2img` (hosted) | Hugging Face Z-Image-Turbo |
| 3D mesh from text | Scenario `txt23d` (Hyper3D Rodin API) | Hugging Face Hyper3D MCP space |
| 3D mesh from image | Scenario `img23d` | Blender `generate_hyper3d_model_via_images` |
| Texture / style analysis | Scenario `analyze` (caption / describe_style) | — |
| VRM rig validation | `blender-execute_blender_code` via Blender MCP | Python VRM parser (Rex) |
| MuseTalk lip-sync (V100) | Self-hosted V100 GPU pod | EchoMimic async MP4 |
| LemonSlice realtime avatar | `@lemonsliceai/openclaw-avatar` plugin | Audio-only TTS + waveform |

### Cluster / infrastructure surface
| Capability | Tool |
|---|---|
| Pod inspection / exec | `kubectl` via `kubernetes_mcp_*` tools |
| Helm deploy / upgrade | `kubectl apply` or ArgoCD sync |
| GitHub PR / branch | `gh` + `github-mcp-server-*` |
| Secret lookup | `op read 'op://Automation/…'` |
| Build (no Docker daemon) | kaniko sidecar via `kubectl exec` |

### Browser surface (avatar UI validation)
| Capability | Tool |
|---|---|
| LiveKit room inspection | `playwright_browser_navigate` → room URL |
| Discord status post | `playwright_browser_*` on discord.com |
| Scenario asset review | `scenario-display_asset` inline |

### Routing decision tree (conductor prompt)
```
IF task.type == "generate-asset"
  IF task.output == "3d" → route Scenario txt23d, max 3 parallel
  IF task.output == "image" → route Scenario txt2img, max 3 parallel
ELSE IF task.type == "code"
  → spawn CodeRun with appropriate subagentType
ELSE IF task.type == "infra"
  → subagentType: "bolt", parallelizable depends on resource conflict
ELSE IF task.type == "test"
  → subagentType: "tess", always parallelizable
```

---

## 4. Model Rotation Policy

This maps directly onto the existing `provider-failover.md` skill ladder, extended for
avatar-specific workloads.

### Tier ladder (per-agent, in order)
| Tier | CLI / Provider | Trigger condition |
|---|---|---|
| 1 | `claude` via `CLAUDE_CODE_OAUTH_TOKEN` (Sub 1) | Default |
| 2 | `claude` via `CLAUDE_CODE_OAUTH_TOKEN_SUB2` (Sub 2) | 429 / 402 / `usage_limit_exceeded` on Sub 1 |
| 3 | `opencode` → DO Gradient `anthropic-claude-opus-4.7` | Both Claude subs exhausted |
| 4 | Fireworks gateway: `fireworks/kimi-k2p6` (auto-chain) | DO 401/403 |
| 4a | Fireworks: `fireworks/qwen3p6-plus` | 429 on kimi-k2p6 |
| 4b | Fireworks: `fireworks/minimax-m2p7` | 429 on qwen3p6-plus |
| 4c | Fireworks: `fireworks/glm-5p1` | Last resort non-Claude |

### Avatar-workload model preferences
| Task | Preferred model tier | Rationale |
|---|---|---|
| Conductor planning + PRD | Tier 1 (Sonnet / Opus) | Complex reasoning, tool use |
| Asset generation prompting | Tier 1–2 | Quality matters for prompt craft |
| Infra Helm templating | Tier 1–2 (Haiku acceptable) | Deterministic, lower complexity |
| Test harness generation | Tier 2–3 acceptable | High volume, lower stakes |
| Lint / format | Tier 4 Fireworks fine | Minimal reasoning required |

### Rotation signals
- **Memory flush before switch:** always call `memory_add` with task state before rotating
- **Resume signal after switch:** `memory_search({ query: "current task" })` at session start
- **Escalation block:** if tier 4c returns malformed JSON 3× in a row, pause that sub-task
  and NATS-publish `avatar.agent.blocked` to conductor; do not spin endlessly

---

## 5. Shared Context & Handoff Format

### Session-level context (HANDOFF.md + mem0)
Every agent pod writes `/workspace/HANDOFF.md` on compaction / before reboot:
```markdown
# Handoff State — <agent-name>
Updated: <ISO timestamp>

## Active Task
<what is in progress; CodeRun ID if applicable>

## Asset State
<list of Scenario asset_ids produced this session with names/tags>

## Progress (done / pending)
- [x] A2 reference sheet: asset_scXXX
- [ ] A3 3D mesh: not yet started

## Blockers
<none | description of what is blocking>

## Next Steps
<ordered list; first item is next action>
```

### Cross-agent handoff packet (NATS message schema)
Published on `avatar.task.handoff.<target-agent>`:
```json
{
  "from": "nova",
  "to": "blaze",
  "coderun_id": "avatar-web-pip-v2",
  "lane": "B",
  "step": "B7",
  "artifacts": {
    "glb_asset_id": "asset_xxx",
    "scenario_project_id": "project_yyy",
    "vrm_path": "avatar/design/morgan.vrm"
  },
  "context_summary": "<one paragraph; critical decisions made>",
  "open_questions": ["VRM expression map: joy expression not yet validated"],
  "handoff_at": "2026-04-25T14:00:00Z"
}
```

### Conductor state (OpenMemory tags)
The conductor stores cross-lane state in OpenMemory with tag `avatar-conductor`:
- Current phase (design / wiring / validation / published)
- Outstanding lane blockers
- Latest Scenario project/collection IDs
- GPU slot status

---

## 6. Artifact Contracts

All durable artifacts produced during the avatar pipeline must satisfy the following contracts
so any agent (or human) can pick up the chain at any step.

### GLB / VRM asset contract
```json
{
  "schema_version": "1",
  "asset_type": "avatar-glb" | "avatar-vrm",
  "scenario_asset_id": "asset_xxx",
  "tags": ["morgan", "vrm", "v1"],
  "source_prompt": "<generation prompt used>",
  "mesh_info": {
    "poly_count_approx": 12000,
    "bone_count": 55,
    "expression_shapes": ["joy", "sorrow", "angry", "surprised", "blink_l", "blink_r"]
  },
  "validation_status": "passed" | "pending" | "failed",
  "produced_by": "nova",
  "produced_at": "2026-04-25T10:00:00Z"
}
```
Stored as `avatar/design/morgan-artifact-v<N>.json`. Tess validates this on every publish.

### Audio / lip-sync artifact contract
```json
{
  "schema_version": "1",
  "artifact_type": "avatar-turn-video" | "lipsync-mp4",
  "provider": "lemonslice" | "echomimic" | "musetalk",
  "session_id": "...",
  "turn_id": "...",
  "duration_ms": 4200,
  "video_url": "...",
  "audio_asset_id": "asset_zzz",
  "latency_ms": 850
}
```

### Provider adapter contract (TypeScript interface — from handoff doc)
```typescript
type AvatarMode = "live-realtime" | "async-turn-video";

interface AvatarProvider {
  readonly mode: AvatarMode;
  capabilities(): { video: boolean; audio: boolean; streaming: boolean };
  startSession(input: AvatarSessionInput): Promise<AvatarSession>;
  renderTurn(input: AvatarTurnInput): Promise<AvatarTurnResult>;
  stopSession(sessionId: string): Promise<void>;
}
```
All adapters (LemonSlice, EchoMimic, audio-only fallback) implement this interface.
Agents must not call provider APIs directly — always go through the adapter boundary.

---

## 7. Failure Recovery

### Failure classification
| Class | Examples | Recovery action |
|---|---|---|
| **Transient / retriable** | 429 rate limit, network timeout, GPU OOM (single attempt) | Retry with backoff (3×); rotate model tier on 3rd failure |
| **Provider exhausted** | Claude sub exhausted, Scenario credits zero | Rotate to next tier; flush state to memory first |
| **Asset invalid** | Tess validation fails on GLB | Quarantine asset; re-run generation with refined prompt; max 2 re-runs before escalation |
| **Blocked (dependency)** | GPU slot taken, LiveKit room full | Wait and poll; NATS-publish `avatar.task.waiting`; conductor re-queues |
| **Hard failure** | Pod crash-loop, kaniko build failure, CRD schema error | NATS-publish `avatar.agent.blocked`; write to HANDOFF.md; Healer picks up; escalate if no resolution in 10 min |

### Recovery patterns (per existing skills)

**Provider switch:**
```bash
# On 429/402 for Sub 1
export CLAUDE_CODE_OAUTH_TOKEN="$CLAUDE_CODE_OAUTH_TOKEN_SUB2"
memory_add "switched to Sub2 due to rate limit; task: <current task>"
```

**GPU OOM fallback:**
```
MuseTalk V100 OOM → EchoMimic async MP4 → audio-only TTS
```
Gate: if V100 OOM occurs on first attempt, retry once with reduced batch size.
If OOM again, immediately fall through to EchoMimic without further V100 attempts in this session.

**CodeRun retry:** increment `contextVersion` in the CRD spec; controller re-spawns the pod.
The `retryCount` status field is the source of truth; conductor monitors it via `kubectl get coderun`.

**Healer integration:** Healer watches for pods stuck in `CrashLoopBackOff` or `OOMKilled`
with label `avatar=true`. Automated remediation: restart pod, bump resource requests if OOM.
Healer NATS-publishes `healer.action.taken` so conductor updates its blockers list.

### Escalation thresholds
| Condition | Escalation trigger |
|---|---|
| 3 consecutive asset validation failures | Post to `#avatar-dev` Discord; block that sub-task |
| All 4 model tiers exhausted | Post to `#agent-coordination`; block all CodeRun spawning |
| GPU slot unavailable > 30 min | Conductor considers async MP4 fallback for the whole session |
| Conductor HANDOFF.md > 24h old | Healer alerts; conductor is assumed stale |

---

## 8. CRD Mapping

### `CodeRunSpec` fields used by avatar sub-tasks

| CRD field | Avatar usage |
|---|---|
| `subagentType` | `"blaze"`, `"rex"`, `"tess"`, `"bolt"`, `"nova"` — routes to correct agent pod |
| `parallelizable` | `true` for independent generation tasks (A2+A3, B1+B5); `false` for gates |
| `modelRotation` | Array of model IDs tried in order; maps to tier ladder in §4 |
| `continueSession` | `true` when re-queuing after a provider switch (preserves HANDOFF.md context) |
| `contextVersion` | Incremented on each retry; used by pod to determine if context is fresh |
| `maxTokens` | Set to `65536` for Conductor planning; `8192` for leaf implementation tasks |
| `linearIntegration.enabled` | `true` — avatar tasks are tracked in Linear project |
| `runQualityReview` | `true` for all B-lane tasks (Cleo lint gate) |
| `runSecurityScan` | `true` for asset-publishing tasks (Cipher) |
| `runTestingPhase` | `true` for B-lane wiring tasks (Tess) |
| `runDeployment` | `true` only for C-lane Helm apply tasks (Bolt) |

### Status phase lifecycle for avatar tasks
```
Pending → Running → (phase: build | lint | test | deploy | review) → Succeeded
                                                                    ↘ Failed → retried via contextVersion bump
```

### Proposed `AvatarRun` CRD (future, tracked separately)

For richer native tracking of avatar-specific resources, a future `AvatarRun` CRD
would extend `CodeRun` with:
```yaml
spec:
  lane: A | B | C
  glbAssetId: ""           # Scenario asset_id of the GLB once produced
  vrmPath: ""              # repo-relative path to .vrm file
  providerMode: live-realtime | async-turn-video | audio-only
  gpuRequired: false
status:
  assetValidated: false
  liveKitRoomId: ""
  lastLatencyMs: 0
  fallbackActive: false
```
Until this CRD ships, avatar metadata is carried in CodeRun `annotations` and OpenMemory.

### NATS subject map (avatar namespace)
| Subject | Publisher | Subscriber | Purpose |
|---|---|---|---|
| `avatar.task.handoff.<agent>` | Any agent | Target agent | Cross-agent task handoff |
| `avatar.gpu.slot.released` | Rex / Bolt | Conductor | Unblock GPU queue |
| `avatar.agent.blocked` | Any agent | Conductor, Healer | Hard-failure escalation |
| `avatar.task.waiting` | Any agent | Conductor | Dependency wait signal |
| `avatar.asset.published` | Blaze / Nova | Conductor, Lane B | GLB/VRM ready signal |
| `avatar.session.started` | Rex (B-lane) | Blaze (web harness) | Avatar session live signal |

---

## Summary

The strategy is **three lanes (Asset / Runtime / Platform) with conductor-owned dependency
gates**, a **5-tier model rotation ladder** inherited from existing provider-failover skill,
**Scenario hosted generation as the primary asset path** (no GPU for 3D work in v1), and a
**LemonSlice → EchoMimic → audio-only fallback ladder** for the live avatar session.

Failure recovery leverages existing Healer + HANDOFF.md + mem0 patterns.
The only new infrastructure is the NATS `avatar.*` subject namespace and an eventual
`AvatarRun` CRD — both small, additive changes.

**Do not** bind product code directly to any single avatar provider.
**Do** implement the `AvatarProvider` TypeScript interface in `avatar/agent/` as the
mandatory abstraction layer before any B-lane work goes to production.
