# Morgan Spotlight Pitch Flow

This document is the source-of-truth brief for turning the CTO architecture map into a Morgan-led spotlight presentation. It is both pitch material and implementation guidance for the hidden `/cto/harness-map` experience.

The goal is not to make people read a dense architecture diagram. Morgan should walk the viewer through the system, spotlight one region at a time, and make the platform feel obvious: the user speaks to Morgan, Morgan turns ambiguity into a plan, CTO runs the deterministic workflow, and specialized agents ship the work.

## Audience and positioning

The primary audience is technical buyers, founders, engineering leads, and power users who can appreciate the architecture but do not want to decode it from a wall of labels.

Use this framing:

- CTO is an agent-managed private infrastructure gateway, not just a cloud IDE and not a neocloud.
- Morgan is the command layer: intake lead, project manager, context keeper, dispatcher, and presenter.
- The platform bundles named agents, prompts, skills, tools, identities, avatars, and runtime guardrails.
- The orchestration layer blends deterministic workflow control with LLM calls, so work is parallel but not chaotic.
- The infrastructure story matters: local-first bootstrap, secure tunnels, closed external interfaces, GitHub/GitLab source-control paths, provider redundancy, operators, and mesh networking.

Visible text should be sparse. Use logos, agent portraits, paths, light, motion, and Morgan's narration to carry the explanation. Technical depth should be available through transcript/audio buttons or side sheets, not always-visible paragraphs.

## Visual direction

The presentation should feel like a guided tour through the 5D Labs universe.

- Dark 5D base, not pure black.
- Trippy texture, subtle field motion, and spotlight beams.
- One active region at a time; inactive regions remain visible but dimmed.
- Morgan front-and-center near the top, animated/pulsing while narrating.
- Low visible text: labels, logos, short verbs, step title, and a concise caption.
- Rich detail lives in narration, transcript, audio brief, or side sheet.
- The viewer should understand the start and end of the pipeline without reading a static legend.
- The full page can be a scrollable/stepped presentation, but each active step should compose as a clean pitch slide.

Preferred motion vocabulary:

- Spotlight sweeps to active subsystem.
- Agent avatars pulse when selected.
- Data/context lines light up as Morgan dispatches work.
- Tool/skill bundles unfold briefly, then collapse back into the orbit.
- Play/runtime execution can animate as a pipeline, circuit, or orbital route as long as the beginning and end are clear.

### Slide/movie direction

Treat the page less like a static HTML diagram and more like a narrated scene deck. The implementation can still be HTML/CSS/React, but the mental model should be a movie timeline:

- One slide/scene owns the screen at a time.
- Elements enter, fade, connect, and leave; they do not all remain visible.
- The same objects can reappear in later scenes at different scale or position.
- Morgan's narration advances the sequence, but manual controls should allow scrubbing.
- Technical details open as transcript/audio side panels, not as text blocks on the main stage.

The website already has `framer-motion`, which is enough for this phase: staged entrance/exit, shared layout transitions, opacity/transform motion, pulsing agents, and a timeline driven by React state. Do not add a new animation library for the first implementation. If later scenes need cinematic scroll-timelines, complex timeline scrubbing, or canvas/WebGL backgrounds, add GSAP or Three.js in an isolated component; do not mix them into the same animation tree as Framer Motion.

The page should be implemented as a data-driven storyboard:

- `scenes[]` contains title, narration, transcript, audio source, active objects, entering objects, exiting objects, and camera/framing hints.
- `objects[]` contains stable IDs for Morgan, Desktop, ACP Agent, Harness, CLI list, providers, models, agents, operators, and delivery artifacts.
- `connections[]` defines which lines light up during the scene.
- The UI renders a single scene at a time and animates object presence through `AnimatePresence` and `layoutId`.

### Animated Morgan and mock-first run

Use animated Morgan in the pitch now, not only a static image. The hidden route should become the testbed for:

- longer Morgan narration MP3s
- streamed audio playback timing
- transcript/caption fallback
- future video-avatar clips
- agent-by-agent video introductions

The first product-choice experience should be mocked rather than running real intake. The user picks one of the three scenario tiles, the selected tile visually locks/drops into the run, and the presentation flips into the mocked pipeline. Later, the same structure can trigger a real intake run.

Do not pretend the mock is doing real work. Narration should say the demo is compressing the actual pipeline order so investors and technical users can understand the complete product before the full end-to-end path is exercised live.

### Opening Morgan narration

The first long-form narration asset lives at `apps/website/public/harness/morgan-spotlight-intro.txt`, with generated audio at `apps/website/public/harness/morgan-spotlight-intro.mp3`. This replaces the short lip-sync test phrase with the real pitch intro Morgan should say while the spotlight sequence starts.

The script is intentionally product-language first and phoneme-varied enough for a useful mouth-motion pass. It should be treated as the default opening script until a later full scene-by-scene narration pass replaces it.

The current in-page avatar clip uses `morgan-spotlight-preview.txt`, `morgan-spotlight-preview.mp3`, and `morgan-spotlight-preview.mp4` because the deployed EchoMimic V100 endpoint is returning a roughly 2.6-second video window. Keep the preview audio short and duration-matched for lip-sync validation until a longer-render path is available.

## Core story in one sentence

The user asks Morgan for a product, Morgan turns the conversation into a reviewed PRD, intake turns that PRD into decisions, tasks, prompts, tools, skills, workflows, and issues, and then Morgan dispatches the right CTO agents through CodeRun/OpenClaw/Hermes to ship in parallel.

## A-to-Z narrated user journey

### 1. Install CTO Desktop

The user downloads and opens the CTO desktop app. The app should present this as a polished onboarding flow, not as "install Kubernetes."

Repo-grounded current flow:

- CTO Lite setup wizard checks a container runtime.
- User chooses stack/preferences.
- User enters model/API keys.
- User connects GitHub.
- User optionally configures Cloudflare Tunnel.
- The app creates a local Kind cluster.
- It installs local-path storage, ingress-nginx, NATS, CTO support services, Morgan/OpenClaw, and CTO Lite services through Helm and Kubernetes manifests.
- It uses Kind context `kind-cto-lite`.
- It uses the Argo CLI/workflow machinery for play workflows.

Pitch wording:

> "CTO sets up a private local runtime for you. Docker, Kind, ingress, support services, Morgan, and the workflow engine come online behind a friendly setup screen."

Implementation note:

- The user mentioned Kine. In the current repo, Kine appears in research notes as a future/lightweight cluster-state idea, while the CTO Lite installer path verified here uses Kind and normal Kubernetes state. Do not pitch Kine as shipped unless the implementation is added or a separate source is confirmed.
- The user mentioned Argo CD. The repo has ArgoCD tooling and ArgoCD application discovery in intake infrastructure context, but the CTO Lite play path verified here is Argo Workflows/Argo CLI. Phrase carefully: "Argo workflows/GitOps control" unless a specific ArgoCD install path is confirmed for this setup.

### 2. Morgan appears

Once the local runtime is ready, the app opens to Morgan.

Morgan is not just a chatbot. Morgan is the command layer and project operator.

Morgan owns:

- conversation context
- PRD shaping
- memory and handoffs
- system artifacts
- dispatch decisions
- Linear/GitHub/GitLab coordination
- intake start
- play start
- agent assignment
- status narration

Pitch wording:

> "Morgan is the front door. Everything important flows through Morgan: the conversation, the PRD, the decisions, the handoffs, and the agent dispatch."

### 3. User starts a project

The user says something like:

> "Let's start a new project."

Morgan begins a product conversation, asks only the questions needed to shape the work, and keeps the user in a guided flow. The app should show the product becoming clearer rather than exposing the internal mechanics too early.

Possible visible UI:

- Project name input.
- Short conversation with Morgan.
- A "Draft PRD" state.
- A "Review and confirm" state.
- A "Start intake" action.

### 4. PRD is drafted and reviewed

The PRD becomes a first-class document. The user can review it in the desktop app, markdown editor, or code-server sidecar depending on the runtime.

Repo-grounded current surfaces:

- CTO Lite has a PRD view that accepts paste, Linear issue, file, or URL input.
- CTO Lite has a markdown editor/preview component.
- The CodeRun path can enable code-server for richer workspace editing.
- Morgan's intake bootstrap stages PRD and architecture docs into `.tasks/docs/`.

Pitch wording:

> "Morgan does not run intake on vague vibes. The PRD is made explicit, reviewable, and editable before the machine starts decomposing work."

### 5. User confirms intake

Once the PRD is confirmed, intake begins.

This is the point where the spotlight should leave the chat UI and move into the pipeline. Morgan stays visible as narrator, but the diagram should show Morgan initiating the workflow rather than the desktop directly controlling everything.

## Actual intake pipeline

The production chain of command is:

1. Morgan receives or prepares intake inputs.
2. Morgan's intake bootstrap handles repository setup, PRD/document staging, Linear metadata, optional webhook setup, and workspace preparation.
3. Morgan delegates to `intake/workflows/pipeline.lobster.yaml`.
4. Lobster/OpenClaw runs deterministic steps and calls LLM/tools where appropriate.
5. The pipeline invokes deliberation and intake sub-workflows.
6. The output becomes task artifacts, docs, prompts, workflows, Linear issues, and PRs.

### Morgan bootstrap

`templates/agents/morgan/intake.sh.hbs` is the production bridge between Morgan and the Lobster/OpenClaw intake graph.

It is responsible for:

- loading intake files from `/intake-files` or `/task-files`
- reading PRD and architecture inputs
- creating or validating the target repo
- setting up a GitHub webhook when a callback URL is available
- cloning the repo
- staging `.tasks/docs/prd.txt` and architecture docs
- preparing Linear metadata
- invoking `openclaw.invoke --workflow pipeline.lobster.yaml`

Pitch wording:

> "Morgan does the operator work first: repo, docs, metadata, webhook, and run context. Then Morgan hands the job to the workflow graph so the rest is deterministic and inspectable."

### Pipeline graph

`intake/workflows/pipeline.lobster.yaml` coordinates the broader intake run.

Key stages to narrate:

- load config from `cto-config.json`
- preflight local/bridge/cluster assumptions
- materialize the PRD into `.intake/run-prd.txt`
- materialize design context and optional design assets
- persist design metadata to Linear/Discord bridges
- generate/review design variants when enabled
- set up or create the repository
- create Linear project/session metadata
- verify agent identity resolution
- build infrastructure context from cluster services, operators, ArgoCD apps, CRDs, and services
- discover tool context from `cto-config.json` and MCP services
- optionally analyze the codebase
- initially parse the PRD for deliberate intake
- invoke deliberation when requested
- invoke the core intake workflow

Pitch wording:

> "The pipeline starts by making the world explicit: config, repo, PRD, design context, infrastructure context, tools, agents, and project state."

### Research, parsing, and decisions

The user described the early intake order as:

1. Research
2. Parse the initial PRD
3. Identify decision points
4. Initiate deliberation
5. Vote

That maps well to the repo:

- `deliberation.lobster.yaml` runs PRD research, records artifacts, starts a debate, and creates activity updates.
- `decision-voting.lobster.yaml` runs a committee vote for individual decision points.
- `intake.lobster.yaml` parses the PRD, analyzes complexity, refines tasks, and validates outputs.
- `task-refinement.lobster.yaml` expands tasks and supports vote-driven revisions.

Pitch wording:

> "Morgan does not pretend every decision is obvious. The system extracts decision points, runs a small committee, records the vote, and carries those decisions into task docs and prompts."

### Deliberation

Deliberation is the part of the story where CTO should feel different from a single-shot agent.

Narrate it as:

- research notes are collected
- options are identified
- optimistic and pessimistic positions are argued
- committee voters evaluate the decision
- unresolved or weak decisions can be revised
- final decisions become artifacts the implementation agents can use

The visual can show three voters around a decision point, not a dense debate transcript. The transcript/audio brief can hold the detail.

### Intake artifacts

`intake/workflows/intake.lobster.yaml` turns decisions and tasks into implementation material.

Important artifact stages:

- parse PRD
- verify parsed tasks
- analyze complexity
- refine tasks through committee review
- generate scaffolds
- fan out docs
- validate docs
- discover skills
- generate tool manifest
- generate per-task prompts
- generate workflows
- generate scale/hardening tasks
- generate security report
- generate remediation tasks
- verify artifacts
- generate agent package/provisioning metadata
- commit outputs
- create PR
- sync Linear issues after push
- verify delivery gates
- write handoff summary

Pitch wording:

> "The output of intake is not a chat response. It is a project operating package: tasks, docs, prompts, workflows, skills, tool manifests, security analysis, Linear issues, and a PR."

## Play and execution

After intake, "play" is the execution run. Morgan behaves like a strong project manager: deciding who does what, where they need to work, what context they need, and how to ship fastest without creating merge chaos.

For the pitch route, "play" should be simulated first:

- user picks a project tile
- Morgan narrates the selected PRD
- intake phases animate in the actual order
- the right agents give short role intros
- quality/security/review/deploy stages appear
- output lands as mocked docs, issues, handoffs, PRs, and deployable artifacts

The simulated run is valuable because it can show the whole CTO product to investors and users without waiting on a live workflow run.

### CTO Lite play path

The CTO Lite play workflow is `play-workflow-lite`.

Current simplified flow:

- Morgan intake/router analyzes the request.
- Backend implementation runs through the selected stack:
  - `nova` for TypeScript backend
  - `grizz` for Rust/Go backend in the current lite template
- Blaze handles frontend when needed.
- Tess runs testing.
- Cleo runs quality review.
- Cipher runs security scan.
- Bolt handles deploy preview when enabled.
- The individual user reviews and merges PRs; no Atlas auto-merge in Lite.

Pitch wording:

> "Morgan routes the work. Implementation, frontend, testing, quality, security, and preview deployment happen as separate responsibilities, not as one overloaded agent."

### Full CTO / CodeRun path

For paid/private and platform-scale flows, the CodeRun CRD and controller are the more important visual object.

CodeRun should be explained as the execution envelope that carries:

- CLI configuration
- remote tools
- local MCP servers
- skills URL and project overlay
- Linear integration metadata
- ACP settings
- OpenClaw settings
- harness agent selection
- optional code-server sidecar
- workspace identity/persona files

The controller then:

- resolves provider/model/env
- fetches skills/personas
- writes CLI config and MCP config
- chooses OpenClaw or Hermes harness
- creates ConfigMap/PVC/Job resources
- launches Kubernetes work

Pitch wording:

> "CodeRun is the contract. It says what agent is running, what it can access, what skills it has, which CLI it uses, and which harness owns the runtime."

### Harnesses

Show OpenClaw and Hermes as two modes of one "ACP agent runtime" element for the pitch, not as two unrelated planets. Technically they are different harnesses; visually they should be one configurable runtime block with a segmented switch or split-face treatment.

OpenClaw:

- gateway-backed harness
- stronger fit for Discord/presence/plugin flows
- ACPX/CLI delegation
- gateway-backed status and interaction

Hermes:

- standalone Lobster/ACPX path
- file sentinel and MCP sidecar style
- no default Discord/NATS gateway presence
- useful as a more direct runtime harness

Pitch wording:

> "OpenClaw gives us a gateway and presence surface. Hermes gives us a lean standalone execution path. CTO can choose the harness instead of forcing every job through one shape."

### ACP agent org-chart slide

Add a dedicated slide that expands like an org chart. This replaces the old always-visible triangle for the harness story.

The hierarchy should read:

1. Users
2. Desktop users / web users / collaboration surfaces
3. Morgan
4. Morgan's ACP agent runtime
5. Harness mode: OpenClaw or Hermes
6. CLI layer
7. Provider layer
8. Model layer

Suggested visual behavior:

- Start with only Users and Morgan.
- Desktop/Web/Discord/Linear/GitHub/GitLab/Meet appear as input surfaces around the top.
- Morgan expands downward into "ACP agent runtime."
- The runtime block flips between OpenClaw and Hermes modes rather than drawing two separate systems.
- CLI nodes fan out underneath.
- Provider families fan out beneath CLIs.
- Individual model chips appear only as the final drill-down, then collapse.

Pitch wording:

> "Morgan does not talk to one model. Morgan runs an ACP agent runtime. That runtime can use OpenClaw or Hermes, then route through the right CLI, provider, and model for the job."

Current repo-grounded config:

- `model-providers.json` currently enables ACP entries for `Claude Code`, `Factory`, and `Cursor`.
- `Codex` is present but disabled because this environment does not have an OpenAI key.
- `cto-config.json` still contains CLI-model IDs for `claude`, `codex`, `cursor`, `gemini`, `factory`, `opencode`, and `dexter`.
- Before the public slide claims "eight CLIs," align `model-providers.json`, `cto-config.json`, and the actual runtime adapters.

Target eight-slot CLI slide:

| Slot | CLI label | Current status for pitch |
| --- | --- | --- |
| 1 | Claude Code | Enabled in `model-providers.json` |
| 2 | Factory | Enabled in `model-providers.json` |
| 3 | Cursor | Enabled in `model-providers.json` |
| 4 | Codex | Present but disabled until OpenAI is configured |
| 5 | Gemini CLI | Present as `gemini` model mapping in `cto-config.json`; needs provider/config alignment |
| 6 | OpenCode | Present as `opencode` model mapping in `cto-config.json`; needs provider/config alignment |
| 7 | Dexter | Present as `dexter` model mapping in `cto-config.json`; needs provider/config alignment |
| 8 | Additional ACP-compatible CLI | Reserved slot; validate exact adapter before naming publicly |

Provider/model fan-out:

- Currently configured provider families include Anthropic, Cursor, Fireworks, ZhipuAI, and disabled OpenAI.
- Current model catalog also references Gemini/Google-style models, Kimi/Moonshot-style models, MiniMax, GLM, Qwen, Llama, and Cursor auto-routing.
- Additional provider families can be shown as catalog/adapter candidates, not guaranteed deployed infra, if OpenClaw/Hermes adapters support them: Google Vertex/Gemini, Azure OpenAI, AWS Bedrock, Mistral, DeepSeek, Groq, Together, Cerebras, OpenRouter, local/Ollama, and other OpenAI-compatible endpoints.
- Visually distinguish "configured now" from "adapter/catalog expansion" so the pitch communicates broad support without implying every provider is live in our cluster today.

## CTO agent bundle

CTO ships with a named roster of agents. Each agent is more than a prompt: it can have identity, avatar, skills, tools, default CLI/model, persona files, and runtime behavior.

The current runtime config includes:

| Agent | Primary role in the pitch | Current config notes |
| --- | --- | --- |
| Morgan | command layer, intake, PM, dispatcher | default intake agent, broad research/context/tool skills |
| Rex | Rust/backend systems | Rust patterns, MCP development, GitHub/context tooling |
| Nova | TypeScript/backend | TypeScript/backend lane |
| Grizz | Go/backend and backend systems | backend lane in Lite template with Rust/Go wording |
| Blaze | web/React/Next frontend | frontend/design skills |
| Tap | mobile/integration | mobile lane candidate |
| Spark | prototyping/desktop experiments | desktop/prototype lane candidate |
| Pixel | CTO Lite/Tauri desktop | desktop-app lane candidate |
| Tess | testing | testing gate |
| Cleo | code quality | quality review/evaluation |
| Cipher | security | security scan |
| Stitch | code review | automated review / PR quality |
| Atlas | merge gate | merge/branch management in full CTO |
| Bolt | DevOps/infrastructure | deployment, hardening, operators |
| Angie | agent architecture | OpenClaw/MCP/agent orchestration |
| Vex | debugging | troubleshooting and root cause |
| Block | blockchain/Solana | Solana, EVM, trading, node operations |

Narration should emphasize that users can later add custom agents, but CTO should shape those agents into the 5D Labs motif with strict boundaries. The custom-agent future should not look like a generic "paste any prompt" feature; it should feel like commissioning a new member of the CTO universe.

Pitch wording:

> "Every CTO agent is a package: name, avatar, voice, motion, prompt files, tools, skills, permissions, and guardrails. Users can extend the roster, but Morgan keeps the system coherent."

## Tools, skills, and agent-package discovery

The pitch should include ACP, MCP, CLI, and Skills together as the capability rail.

Runtime facts:

- `cto-config.json` defines default tools and skills per agent.
- Remote skills/personas can be distributed from `5dlabs/cto-agent-personas` as release tarballs.
- The controller caches and injects persona/skill files into CodeRun pods.
- Intake can inventory effective tools and skills.
- Intake can analyze required capabilities, compute gaps, query a catalog, resolve missing tools/skills, install or generate skills, and produce an agent package manifest.
- Intake can open a PR to `5dlabs/cto-agents` with generated package files when available.

Pitch wording:

> "Morgan does not just pick a model. CTO checks the task against the available tools and skills, finds gaps, and can generate or install the missing package pieces for the run."

## Three demo PRDs

The pitch should include three project examples so viewers see CTO adapt to different markets.

### Demo 1: Solana operator and copy-trading app

Target market:

- blockchain infrastructure teams
- Solana operators
- trading infrastructure founders

PRD concept:

> "Build a private Solana node and trading operations console that watches selected wallets, simulates copy-trading rules, exposes risk controls, and deploys with secure operator-managed infrastructure."

Spotlight path:

- Morgan parses blockchain and infra requirements.
- Block becomes the domain specialist.
- Bolt handles node/operator/deployment needs.
- Rex/Grizz handle systems components where needed.
- Blaze builds the control UI.
- Cipher flags wallet/security/key-management risks.
- Tess validates behavior.

Important constraints:

- Do not expose secrets or imply live trading without controls.
- Frame this as a subset/pattern, not a clone of internal/private trading systems.
- Highlight Solana operators as part of the operator catalog story.

### Demo 2: Voice/agent product with ElevenLabs and avatar media

Target market:

- AI-agent products
- customer support and sales-assistant products
- voice/video UX teams

PRD concept:

> "Build a TypeScript-heavy agent product with voice responses, ElevenLabs speech, Morgan-style avatar explainers, transcript capture, and a web console for reviewing conversations and generated clips."

Spotlight path:

- Morgan identifies voice/media decisions.
- Nova handles TypeScript backend.
- Blaze handles web app and player UI.
- Angie handles agent architecture and runtime boundaries.
- Vex helps debug media/latency failures.
- Cipher reviews secret and consent handling.
- Tess validates transcript and playback flows.

Current product cutline to preserve:

- EchoMimic is async MP4/artifact oriented.
- LemonSlice/OpenClaw-style realtime avatar work is a separate provider mode.
- TalkingHead/3D/VRM style enhanced agents are a future asset-bundle direction unless revived explicitly.

Future asset-bundle idea:

- avatar
- voice
- motion pack
- prompt/persona files
- GLB/VRM or equivalent rig contract
- optional content-addressed or NFT-style ownership metadata
- richer engagement states for animated agents

### Demo 3: Conventional Web2 company app

Target market:

- SaaS teams
- internal tooling teams
- agencies
- product teams that want delivery velocity without adopting crypto/agent-native positioning

PRD concept:

> "Build a B2B operations portal with a React web app, TypeScript or Rust backend, auth, audit logs, admin workflows, deploy preview, test plan, and a secure path to production."

Spotlight path:

- Morgan parses normal product requirements.
- The user chooses backend lane: Rust, Node.js/TypeScript, or Go.
- The user chooses frontend lane: web, mobile, desktop, or multiple.
- Blaze handles web.
- Nova/Rex/Grizz handle backend depending on choice.
- Cleo, Tess, Cipher, Stitch, Atlas, and Bolt handle maturity gates.
- Intake discovers relevant skills/tools from defaults and task needs.

Pitch value:

- CTO is not only for exotic AI/blockchain work.
- The same workflow improves ordinary software delivery.
- The platform can pitch practical engineering outcomes before every feature is fully productized.

## Infrastructure, operators, and security story

The diagram currently under-represents operators. The pitch should make infrastructure feel like a first-class product capability, especially for a technical audience.

### Operators and service catalog

Represent operators as grouped capability domains rather than a long list.

Suggested visible groups:

- compute and runtime
- storage and databases
- messaging and eventing
- auth and secrets
- search and retrieval
- observability
- workflow/GitOps
- edge/networking
- GPU/media
- blockchain/Solana
- backup and recovery

Narration should explain that Morgan can use infra context from the actual cluster. The pipeline can inspect deployed operators, services, CRDs, MCP services, databases, caches, and ArgoCD apps where configured.

### Cloudflare tunnels and closed interfaces

The pitch should cover Cloudflare Tunnels clearly.

Current repo story:

- CTO Lite setup includes a Cloudflare Tunnel step.
- Tauri tunnel commands create/start `cloudflared` tunnels and store tokens in the system keychain.
- Cluster-side docs describe Cloudflare Tunnel routes such as `agents.5dlabs.ai` to the Linear bridge.
- Discord bridge can remain cluster-internal while selected webhook/callback surfaces are exposed through controlled routes.

Pitch wording:

> "We do not have to open every internal service to the internet. CTO can expose the few callback surfaces that need to be reachable, keep internal bridges private, and route access through tunnels or mesh networks."

### Provider and mesh redundancy

Show redundancy as organizational resilience, not only cost savings.

Include:

- GitHub for free/local/open workflow
- self-hosted GitLab for paid/private secure source control and registry
- hyperscalers
- neocloud/GPU providers
- VMs and bare metal
- on-prem and private cluster paths
- serverless/Lambda-style services
- bastion/access paths
- Twingate or equivalent mesh-network posture where relevant

Pitch wording:

> "The point is not chasing cheap compute. The point is optionality: private control, provider fallback, network segmentation, and the ability to move work where it belongs."

## Source control story

The presentation should make source-control tiers clear without turning into pricing cards.

Free/open path:

- GitHub
- local Kind
- BYOK
- individual review/merge

Paid/private path:

- self-hosted GitLab
- private registry
- controlled runners
- organization-owned secrets
- stronger audit and network boundaries
- managed agent identities

Pitch wording:

> "GitHub is the open, fast path. Self-hosted GitLab is the private enterprise path. Morgan can operate in either model."

## Suggested spotlight sequence

Use this as the first full presentation script outline. Each step should map to a visual target and an audio/transcript panel.

| Step | Spotlight target | Morgan narration intent |
| --- | --- | --- |
| 1 | Desktop app | CTO installs a local private runtime behind a friendly setup flow. |
| 2 | Local cluster | Kind, ingress, NATS, support services, Morgan, and workflow pieces come online. |
| 3 | Morgan | Morgan is the command layer, not just chat. |
| 4 | Project conversation | User asks to start a project; Morgan shapes the PRD. |
| 5 | PRD editor | The PRD becomes explicit and reviewable. |
| 6 | Intake trigger | Confirmation moves the work from conversation into the pipeline. |
| 7 | Preflight/context | Pipeline gathers config, repo, tools, infra, design, and codebase context. |
| 8 | Research/parse | Research and parsing turn the PRD into structured requirements. |
| 9 | Decision points | Deliberation identifies choices that need judgment. |
| 10 | Committee vote | Agents/models vote and revise weak decisions. |
| 11 | Task/refinement | Tasks, subtasks, dependencies, and acceptance criteria are generated. |
| 12 | Design/storybook | Design artifacts and Storybook handoff appear when relevant. |
| 13 | Tool/skill discovery | The pipeline finds tools/skills and builds capability packages. |
| 14 | Artifact generation | Docs, prompts, workflows, scale tasks, security report, and remediation tasks are written. |
| 15 | Linear/Git/PR | Issues and PRs are created with handoff context. |
| 16 | Stack choice | User picks Rust/Node/Go and Web/Mobile/Desktop lanes. |
| 17 | Agent dispatch | Morgan dispatches specialized CTO agents in parallel. |
| 18 | CodeRun | CodeRun carries the execution contract into Kubernetes. |
| 19 | ACP runtime org chart | Users -> Desktop/Web -> Morgan -> ACP agent -> OpenClaw/Hermes -> CLIs -> providers -> models. |
| 20 | Harness mode | The runtime block flips between OpenClaw gateway mode and Hermes standalone mode. |
| 21 | Quality gates | Tess, Cleo, Cipher, Stitch, Atlas, and Bolt harden the result. |
| 22 | Operators/security | Infrastructure, tunnels, operators, and provider redundancy are shown. |
| 23 | Play complete | The work lands as code, issues, docs, PRs, and deployable artifacts. |

## Script style guide

Morgan should sound strong and capable.

Avoid weak terms:

- coordinator
- assistant
- helper
- memory steward

Prefer:

- command layer
- project operator
- dispatch lead
- execution captain
- intake lead
- runtime commander
- context authority

Morgan should not over-explain every internal step. Use simple phrases, then let the technical audio brief expand.

Example short narration:

> "I turn the conversation into a contract. Then I route it through intake, deliberate the hard choices, and dispatch the right agents with the right tools."

Example technical brief:

> "This step invokes the Lobster pipeline from Morgan's intake bootstrap. The run loads config from `cto-config.json`, materializes the PRD, checks bridges and cluster access, builds design and infrastructure context, and then invokes deliberation or intake sub-workflows depending on run flags."

## Implementation notes for `/cto/harness-map`

The current hidden map already has a first guided pitch layer. Evolve it toward this document.

Needed changes:

- Add the full A-to-Z step list above.
- Convert the page from a static map into a scene deck / movie-like storyboard with animated object presence.
- Use animated Morgan as the presenter and audio/streaming testbed instead of a static avatar image.
- Start with a mocked run for the three project scenarios before wiring real intake execution.
- Reduce visible text further; move detail into audio/transcript/side sheet.
- Keep Morgan and context directly connected.
- Keep external surfaces outside the CTO core.
- Replace the old harness triangle with the ACP runtime org-chart slide.
- Render OpenClaw and Hermes as two modes of one runtime element unless the slide is explicitly teaching their technical differences.
- Add the CLI -> provider -> model fan-out sequence and distinguish configured-now from catalog/adapter expansion.
- Explain Morgan StatefulSet versus short-lived CodeRun/CRD agents, including context refresh, memory handoff, and concurrent agent visibility through Discord.
- Keep Google Meet visible as Morgan's media/presence connector.
- Add operators/infrastructure as a first-class spotlight.
- Add Cloudflare/security/mesh spotlight.
- Add demo-PRD selector or three guided scenario runs.
- Add per-agent click states that play/preview a short MP4 or transcript about the agent's role.
- Replace temporary Pixel monogram when a proper asset exists.
- Add real Morgan narration audio when the voice path is stable.

Audio model:

- Every spotlight step should have `title`, `shortCaption`, `transcript`, optional `audioSrc`, and `targetIds`.
- If audio is missing, the UI should still expose the transcript.
- Later, generated Morgan audio can be dropped in without changing the page structure.

Agent media model:

- Every CTO agent should eventually have:
  - avatar image
  - optional short MP4/animation
  - voice/persona description
  - language/domain tags
  - default tools
  - default skills
  - runtime guardrails

## Repo functionality the user did not explicitly mention but should appear

These are important enough to include in the pitch or side sheets:

- Remote persona and skills distribution from `5dlabs/cto-agent-personas`.
- CodeRun skill/persona injection into pod ConfigMaps and CLI workspaces.
- Tool and skill inventory across all agents.
- Capability gap analysis and generated/custom skills during intake.
- Agent package manifest and optional PR to `5dlabs/cto-agents`.
- Storybook/component-library handoff from design intake for frontend agents.
- Linear AgentSession and activity updates as the observable project timeline.
- Discord bridge notifications as the human-visible intake channel.
- Infrastructure context discovery from operators, CRDs, services, and ArgoCD apps.
- Security report and remediation task generation before delivery.
- Delivery gates for PR and Linear issue assignment.
- Cloudflare Tunnel token storage in local keychain for CTO Lite.
- GitHub free/open path and GitLab private/paid path are already represented in config.
- Hermes/OpenClaw should remain technically visible, but the first-glance pitch can represent them as a single configurable ACP runtime element.

## Open questions

- Confirm whether Kine should be added to the actual install/runtime story or kept as future infrastructure research.
- Confirm whether the CTO Lite install pitch should say Argo Workflows, ArgoCD, or both for the specific desktop setup.
- Inspect the Sigma 1/Test Repo 2 examples if they live outside this repo and should influence the final 3D/spotlight visual style.
- Choose final names and exact PRDs for the three demo runs.
- Confirm the exact eighth CLI name before the public slide labels "eight CLIs."
- Decide whether to update `model-providers.json` now so the eight-slot CLI story matches actual configured runtime support.
- Decide whether the first pitch implementation should generate Morgan narration audio now or stay transcript-first until the avatar/audio path is fully stable.
- Decide how much of the future enhanced-agent asset bundle belongs in this pitch versus a separate CTO Play / avatar product pitch.
- Decide whether custom user agents should be exposed as an early UI concept or kept as a later roadmap reveal.
