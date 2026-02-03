# CTO App - Desktop Application

## Executive Summary

The CTO App is a desktop application built with Tauri that runs the CTO platform on a local Kind cluster. Users install via native installer, configure via GUI, and trigger workflows via MCP or GitHub events. The workflow runs the full lifecycle from implementation through PR merge.

**Target Users:** Individual developers who want AI-assisted development without enterprise infrastructure complexity.

**Business Model:** Tiered subscription model with feature flags controlling access at each tier (Free/Pro/Enterprise).

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Current State Analysis](#current-state-analysis)
3. [Delta Analysis - What Changes](#delta-analysis---what-changes)
4. [File Structure](#file-structure)
5. [Scope Definition](#scope-definition)
6. [Workflow Comparison](#workflow-comparison)
7. [Technical Implementation](#technical-implementation)
8. [Platform Packaging](#platform-packaging)
9. [User Experience](#user-experience)
10. [Design Guidance](#design-guidance)
11. [Potential Issues and Rework](#potential-issues-and-rework)
12. [Implementation Phases](#implementation-phases)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Workstation                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    CTO App (Tauri)                        │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐   │  │
│  │  │ Setup       │  │ Dashboard    │  │ Settings        │   │  │
│  │  │ Wizard      │  │ (Logs/Status)│  │ (API Keys)      │   │  │
│  │  └─────────────┘  └──────────────┘  └─────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │            Container Runtime (Colima/Docker/Podman)       │  │
│  │  ┌────────────────────────┼────────────────────────────┐  │  │
│  │  │              Kind Cluster (cto-app)                │  │  │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │  │  │
│  │  │  │ Argo         │  │ Controller   │  │ PM Server │  │  │  │
│  │  │  │ Workflows    │  │              │  │ (GitHub)  │  │  │  │
│  │  │  └──────────────┘  └──────────────┘  └───────────┘  │  │  │
│  │  │                                                      │  │  │
│  │  │  Agent Pods (on-demand):                            │  │  │
│  │  │  [Morgan] [Grizz/Nova] [Blaze] [Cleo] [Cipher]      │  │  │
│  │  │  [Tess] [Bolt]                                      │  │  │
│  │  └──────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │  Cloudflared ──────────── │ ──────────────────────────────│  │
│  │  (Tunnel to abc123.cto.dev)                               │  │
│  └───────────────────────────┼───────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴───────────────────────────────┐  │
│  │  MCP Background Service (for IDE integration)             │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                               │
                ┌──────────────┴──────────────┐
                │           GitHub            │
                │  (Webhooks → User's Tunnel) │
                └─────────────────────────────┘
```

---

## Current State Analysis

### Existing Crates (Full CTO)

| Crate | Purpose | Lite Status |
|-------|---------|-------------|
| `controller/` | CodeRun CRD orchestration | **REUSE** - minimal changes |
| `pm/` | Linear + GitHub webhooks, Play orchestration | **FORK** - create `pm-lite` |
| `intake/` | PRD processing, task generation | **REUSE** - single-agent mode |
| `healer/` | Self-healing, CI remediation | **EXCLUDE** - enterprise only |
| `mcp/` | MCP tool server | **FORK** - create `mcp-lite` |
| `tools/` | Tool configuration server | **FORK** - hardcode tools |
| `cli/` | CLI adapters (Claude, Codex, Factory) | **REUSE** - as-is |
| `config/` | Configuration types | **REUSE** - partial |
| `installer/` | Bare metal cluster setup | **EXCLUDE** - enterprise only |
| `metal/` | Multi-provider provisioning | **EXCLUDE** - enterprise only |
| `linear-sync/` | Linear activity sync | **EXCLUDE** - no Linear |
| `research/` | Twitter/content pipeline | **EXCLUDE** - enterprise only |
| `notify/` | Notifications (Discord, Slack) | **EXCLUDE** - enterprise only |
| `tenant-operator/` | Multi-tenant CRDs | **EXCLUDE** - enterprise only |

### Existing Charts

| Chart | Purpose | Lite Status |
|-------|---------|-------------|
| `cto/` | Full platform deployment | **FORK** - create `cto-app/` |
| `buildkit/` | Image building | **EXCLUDE** |
| `tenant-agents/` | Multi-tenant agent config | **EXCLUDE** |

### Existing Agent Templates

| Agent | Templates | Lite Status |
|-------|-----------|-------------|
| `morgan/` | intake.md.hbs, play.md.hbs | **REUSE** - single-agent mode |
| `grizz/` | coder.md.hbs, coder-minimal.md.hbs | **REUSE** |
| `nova/` | coder.md.hbs, coder-minimal.md.hbs | **REUSE** |
| `blaze/` | coder.md.hbs | **REUSE** |
| `bolt/` | infra.md.hbs, deploy.md.hbs | **MODIFY** - remove K8s operators |
| `cleo/` | quality.md.hbs | **REUSE** |
| `cipher/` | security.md.hbs | **REUSE** |
| `tess/` | test.md.hbs | **REUSE** |
| `atlas/` | integration.md.hbs | **EXCLUDE** - no Atlas in Lite |
| `rex/` | coder.md.hbs | **EXCLUDE** - paid tier |
| `stitch/` | review.md.hbs | **EXCLUDE** - Cleo handles quality |
| `tap/` | coder.md.hbs | **EXCLUDE** - paid tier |
| `spark/` | coder.md.hbs | **EXCLUDE** - paid tier |
| `vex/` | coder.md.hbs | **EXCLUDE** - paid tier |

### Existing Workflow Templates

Location: `templates/workflows/`

| Template | Lite Status |
|----------|-------------|
| `play-workflow.yaml` | **FORK** - create `play-workflow-lite.yaml` |
| Event sources/sensors | **EXCLUDE** - no Argo Events |

---

## Delta Analysis - What Changes

### NEW Code to Write

```
crates/cto-app/                    # NEW - All Lite-specific code
├── tauri/                          # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs                 # Tauri entry point
│   │   ├── commands/               # Tauri commands (IPC)
│   │   │   ├── cluster.rs          # Kind cluster management
│   │   │   ├── github.rs           # OAuth + webhook creation
│   │   │   ├── tunnel.rs           # Cloudflare tunnel mgmt
│   │   │   └── config.rs           # Settings CRUD
│   │   ├── keychain.rs             # OS keychain integration
│   │   └── runtime.rs              # Container runtime detection
│   └── Cargo.toml
├── ui/                             # React frontend
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── SetupWizard/
│   │   │   ├── Dashboard/
│   │   │   ├── LogViewer/
│   │   │   └── Settings/
│   │   └── hooks/
│   ├── package.json
│   └── tailwind.config.js
├── mcp/                            # Lite MCP server
│   ├── src/
│   │   ├── main.rs
│   │   └── tools.rs                # Curated tool set only
│   └── Cargo.toml
└── pm-lite/                        # Lite PM server (optional fork)
    ├── src/
    │   ├── main.rs
    │   └── handlers/               # GitHub-only handlers
    └── Cargo.toml
```

### FORKED Code (Modified Copies)

| Original | Fork To | Changes |
|----------|---------|---------|
| `crates/pm/` | `crates/cto-app/pm-lite/` | Remove Linear, direct Argo API |
| `crates/mcp/` | `crates/cto-app/mcp/` | Curated tools only, no customization |
| `crates/tools/` | Built into `mcp-lite` | Hardcode tool sets per agent |
| `infra/charts/cto/` | `infra/charts/cto-app/` | Single chart, simplified |

### MODIFIED Code (In-Place Changes)

| File | Change |
|------|--------|
| `templates/agents/bolt/infra.md.hbs` | Add conditionals for Lite (no K8s operators) |
| `templates/agents/*/coder.md.hbs` | Add clean PR handling instructions |
| `templates/workflows/play-workflow.yaml` | Fork to Lite version without Atlas |
| `crates/config/src/types.rs` | Add Lite-specific config types |

### EXCLUDED Code (Enterprise Only)

These are NOT included in CTO App images:

- `crates/healer/` - Self-healing
- `crates/installer/` - Bare metal provisioning
- `crates/metal/` - Multi-provider provisioning
- `crates/linear-sync/` - Linear activity sync
- `crates/research/` - Content pipeline
- `crates/notify/` - Notifications
- `crates/tenant-operator/` - Multi-tenant

---

## File Structure

### Complete CTO App Directory Structure

```
cto/                                        # Main repo (PRIVATE)
├── crates/
│   ├── cli/                                # SHARED - CLI adapters
│   ├── config/                             # SHARED - Config types
│   ├── controller/                         # SHARED - CodeRun orchestrator
│   ├── intake/                             # SHARED - PRD processing
│   │
│   ├── cto-app/                           # NEW - Lite-specific
│   │   ├── tauri/                          # Tauri desktop app
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── icons/                      # App icons
│   │   │   └── src/
│   │   │       ├── main.rs                 # Entry point
│   │   │       ├── commands/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── cluster.rs          # Kind cluster mgmt
│   │   │       │   ├── github.rs           # OAuth, webhooks
│   │   │       │   ├── tunnel.rs           # Cloudflare tunnel
│   │   │       │   ├── config.rs           # Settings
│   │   │       │   ├── workflow.rs         # Trigger workflows
│   │   │       │   └── uninstall.rs        # Cleanup
│   │   │       ├── keychain.rs             # OS keychain
│   │   │       ├── runtime.rs              # Container detection
│   │   │       └── resources.rs            # Resource tracking
│   │   │
│   │   ├── ui/                             # React frontend
│   │   │   ├── package.json
│   │   │   ├── tsconfig.json
│   │   │   ├── tailwind.config.js
│   │   │   ├── vite.config.ts
│   │   │   ├── index.html
│   │   │   └── src/
│   │   │       ├── App.tsx
│   │   │       ├── main.tsx
│   │   │       ├── components/
│   │   │       │   ├── SetupWizard/
│   │   │       │   │   ├── index.tsx
│   │   │       │   │   ├── RuntimeStep.tsx
│   │   │       │   │   ├── StackStep.tsx
│   │   │       │   │   ├── ApiKeyStep.tsx
│   │   │       │   │   └── GitHubStep.tsx
│   │   │       │   ├── Dashboard/
│   │   │       │   │   ├── index.tsx
│   │   │       │   │   ├── ActiveWorkflow.tsx
│   │   │       │   │   ├── LogViewer.tsx
│   │   │       │   │   └── WorkflowHistory.tsx
│   │   │       │   ├── Settings/
│   │   │       │   │   ├── index.tsx
│   │   │       │   │   ├── StackSettings.tsx
│   │   │       │   │   ├── ApiKeySettings.tsx
│   │   │       │   │   └── GitHubSettings.tsx
│   │   │       │   ├── SkillsDisplay/
│   │   │       │   │   └── index.tsx       # FOMO UI
│   │   │       │   ├── ToolsDisplay/
│   │   │       │   │   └── index.tsx       # FOMO UI
│   │   │       │   └── Uninstall/
│   │   │       │       └── index.tsx
│   │   │       ├── hooks/
│   │   │       │   ├── useCluster.ts
│   │   │       │   ├── useWorkflow.ts
│   │   │       │   └── useTauri.ts
│   │   │       ├── lib/
│   │   │       │   └── tauri.ts            # Tauri IPC wrapper
│   │   │       └── styles/
│   │   │           └── globals.css
│   │   │
│   │   ├── mcp/                            # Lite MCP server
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       └── tools/
│   │   │           ├── mod.rs
│   │   │           ├── intake.rs           # intake tool
│   │   │           ├── play.rs             # play tool
│   │   │           ├── status.rs           # play_status tool
│   │   │           └── jobs.rs             # jobs tool
│   │   │
│   │   └── pm-lite/                        # Lite PM server
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── main.rs
│   │           └── handlers/
│   │               ├── mod.rs
│   │               ├── github.rs           # GitHub webhooks only
│   │               └── workflow.rs         # Direct Argo API
│   │
│   ├── pm/                                 # FULL - Keep for paid
│   ├── mcp/                                # FULL - Keep for paid
│   ├── healer/                             # FULL - Enterprise only
│   └── ...                                 # Other enterprise crates
│
├── infra/
│   └── charts/
│       ├── cto/                            # FULL - Enterprise chart
│       │
│       └── cto-app/                       # NEW - Lite chart
│           ├── Chart.yaml
│           ├── values.yaml
│           ├── crds/
│           │   └── coderun-crd.yaml        # CodeRun only
│           └── templates/
│               ├── _helpers.tpl
│               ├── namespace.yaml
│               ├── controller/
│               │   ├── deployment.yaml
│               │   ├── service.yaml
│               │   └── rbac.yaml
│               ├── pm-lite/
│               │   ├── deployment.yaml
│               │   └── service.yaml
│               ├── cloudflared/
│               │   └── deployment.yaml
│               ├── workflow-templates/
│               │   └── play-lite.yaml
│               └── secrets.yaml
│
├── templates/
│   ├── agents/                             # Shared agent templates
│   │   └── (existing structure)
│   │
│   └── workflows/
│       ├── play-workflow.yaml              # FULL workflow
│       └── play-workflow-lite.yaml         # NEW - Lite workflow
│
├── apps/
│   └── cto-app-web/                       # NEW - Download page
│       ├── package.json
│       └── src/
│           └── pages/
│               └── download.tsx
│
└── .github/
    └── workflows/
        ├── release.yaml                    # FULL releases
        └── release-cto-app.yaml           # NEW - Lite releases
```

### What Goes in Each Docker Image

**Agent Images (Lite-specific builds):**

```dockerfile
# ghcr.io/5dlabs/cto-app-grizz:v1.0
FROM ghcr.io/5dlabs/runtime:lite

# Bundled skills (no runtime fetch)
COPY skills/go-patterns /skills/go-patterns
COPY skills/go-concurrency /skills/go-concurrency
COPY skills/git-integration /skills/git-integration
COPY skills/testing-strategies /skills/testing-strategies

ENV SKILLS_PATH=/skills
ENV CTO_APP=true
```

**Controller Image (Lite):**

```dockerfile
# ghcr.io/5dlabs/cto-app-controller:v1.0
FROM debian:bookworm-slim

COPY target/release/agent-controller /usr/local/bin/
COPY templates/agents/ /templates/agents/
COPY templates/workflows/play-workflow-lite.yaml /templates/workflows/

ENV CTO_APP=true
```

---

## Scope Definition

### IN SCOPE

| Component | Purpose | Notes |
|-----------|---------|-------|
| **Tauri Desktop App** | Native GUI | Cross-platform |
| **Native Installers** | .dmg, .msi, .deb/.rpm | Bundle dependencies |
| **Kind** | Local Kubernetes | In Docker |
| **Container Runtime** | Docker/Colima/Podman | Auto-detect |
| **Argo Workflows** | Orchestration | Direct API, no Events |
| **Controller** | CodeRun execution | Shared crate |
| **PM Lite** | GitHub webhooks | Forked, simplified |
| **MCP Lite** | IDE integration | Curated tools |
| **Cloudflare Tunnel** | Webhook ingress | Managed by 5dlabs |

### Implementation Agents

| Agent | Stack | Notes |
|-------|-------|-------|
| **Grizz** | Go (chi, grpc, pgx) | User chooses ONE backend |
| **Nova** | Node.js/Bun (Elysia, Effect) | User chooses ONE backend |
| **Blaze** | React/Next.js (shadcn) | Always included |

### Support Agents

| Agent | Purpose |
|-------|---------|
| **Morgan** | PRD intake (single-agent mode) |
| **Cleo** | Code quality review |
| **Cipher** | Security analysis |
| **Tess** | Test generation |
| **Bolt** | Local/Docker deployment |

### Supported CLIs

| CLI | Provider | Notes |
|-----|----------|-------|
| **Claude** | Anthropic | Default |
| **Factory** | CodeFactory | |
| **Codex** | OpenAI | |

### OUT OF SCOPE (Enterprise Only)

| Component | Reason |
|-----------|--------|
| **Linear Integration** | Paid tier |
| **Atlas** | No auto-merge |
| **Healer** | No self-healing |
| **ArgoCD** | No GitOps |
| **Argo Events** | PM handles webhooks |
| **Stitch** | Cleo handles quality |
| **K8s Operators** | Mayastor, SeaweedFS, etc. |
| **Bare Metal** | Latitude provisioning |
| **Observability** | Prometheus, Loki, Grafana |
| **External Secrets** | OpenBao |
| **Multi-repo** | Single repo only |
| **Rex/Tap/Spark/Vex/Forge** | Additional agents |

---

## Workflow Comparison

### Full CTO (Paid)

```
PRD → Intake (Morgan) → [Multiple Tasks Generated]
    → Infrastructure (Bolt) → Implementation (Multiple agents in parallel)
    → Quality (Cleo) → Security (Cipher) → Testing (Tess) 
    → Merge (Atlas) → Done
```

### CTO App (Freemium)

```
PRD → Intake (Morgan, single-agent) → [One Task at a Time]
    → Implementation (Grizz/Nova + Blaze) → Quality (Cleo) 
    → Security (Cipher) → Testing (Tess) → Deployment (Bolt) 
    → PR Merged → Done
```

### Key Differences

1. **Single-agent intake** - One task at a time, no parallel orchestration
2. **No Atlas** - Agents prompted to create clean, merge-ready PRs
3. **Bolt limitations** - Local/Docker only, no K8s operators
4. **No Linear** - App dashboard replaces Linear as visibility interface

---

## Technical Implementation

### Agent Tool Sets (Curated, Not Modifiable)

```rust
// crates/cto-app/mcp/src/tools.rs
pub fn get_tools_for_agent(agent: &str) -> Vec<Tool> {
    match agent {
        "morgan" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB, WEB_SEARCH],
        "grizz" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB, WEB_SEARCH],
        "nova" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB, WEB_SEARCH],
        "blaze" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB, WEB_SEARCH, BROWSER],
        "cleo" => vec![READ, GIT_DIFF, GITHUB_COMMENT, SHELL],
        "cipher" => vec![READ, GIT_DIFF, GITHUB_COMMENT, SHELL],
        "tess" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB],
        "bolt" => vec![READ, WRITE, EDIT, SHELL, GIT, GITHUB, WEB_SEARCH, 
                       CLOUDFLARE_TUNNEL, REPORT_URLS],
        _ => vec![READ]
    }
}
```

### MCP Tools (Lite)

| Tool | Purpose |
|------|---------|
| `intake` | Process PRD (single-agent mode) |
| `play` | Submit workflow |
| `play_status` | Query progress |
| `jobs` | List workflows |

**Not Available:**
- `add_skill` - No customization
- `add_mcp_server` - Fixed config
- `prometheus_query` - No observability
- `notify_healer` - No Healer

### GitHub App Permissions

| Permission | Access | Purpose |
|------------|--------|---------|
| `contents` | write | Push commits |
| `pull_requests` | write | Create PRs |
| `issues` | read/write | Read issues, post comments |
| `repository_hooks` | write | Create webhooks |
| `metadata` | read | Basic repo info |

### Webhook Flow

1. User connects GitHub via OAuth
2. App creates webhook → `https://abc123.cto.dev/webhooks/github`
3. GitHub events go to user's tunnel
4. PM Lite receives and triggers workflows

---

## Platform Packaging

### Pre-bundled Dependencies

| Binary | Version | Purpose |
|--------|---------|---------|
| `kind` | v0.20+ | K8s cluster |
| `kubectl` | v1.28+ | K8s management |
| `helm` | v3.13+ | Chart deployment |
| `cloudflared` | latest | Tunnel client |

### macOS (.dmg)

```
CTO-Lite.dmg
├── CTO App.app/
│   ├── Contents/
│   │   ├── MacOS/cto-app           # Tauri binary
│   │   ├── Resources/
│   │   │   ├── kind                 # Pre-bundled
│   │   │   ├── kubectl              # Pre-bundled
│   │   │   ├── helm                 # Pre-bundled
│   │   │   ├── cloudflared          # Pre-bundled
│   │   │   └── colima/              # Optional
│   │   └── Info.plist
│   └── _CodeSignature/
└── Applications symlink
```

**Code Signing:** Apple Developer ID, notarization via `notarytool`

### Windows (.msi)

```
CTO-Lite-Setup.msi
├── cto-app.exe                     # Tauri binary
├── resources/
│   ├── kind.exe
│   ├── kubectl.exe
│   ├── helm.exe
│   └── cloudflared.exe
└── scripts/
    └── install-docker.ps1           # Docker Desktop prompt
```

**Code Signing:** EV certificate

### Linux (.AppImage)

```
CTO-Lite.AppImage
├── AppRun
├── cto-app.desktop
├── usr/
│   ├── bin/cto-app
│   └── share/cto-app/
│       ├── kind
│       ├── kubectl
│       ├── helm
│       └── cloudflared
└── AppDir structure
```

### Container Runtime Detection

| OS | Preference Order |
|----|------------------|
| macOS | Colima → Docker Desktop → Podman |
| Linux | Docker → Podman → Colima |
| Windows | Docker Desktop → Podman (WSL2) |

### First Launch Automation

```
1. Check container runtime → Install/prompt if missing
2. Create Kind cluster → kind create cluster --name cto-app
3. Deploy Helm chart → helm install cto-app oci://ghcr.io/5dlabs/charts/cto-app
4. Wait for pods ready
5. Configure tunnel → Allocate subdomain, start cloudflared
6. Show setup wizard → API key + GitHub OAuth
7. Done → Show dashboard
```

---

## User Experience

### Credentials Required

**Just 2:**
1. AI provider API key (Anthropic/OpenAI)
2. GitHub OAuth (click to connect)

**NOT Required:**
- PAT (OAuth handles it)
- Cloudflare credentials (we manage)
- Linear API key (not in free tier)

### Setup Wizard UI

```
┌─────────────────────────────────────────────────────────┐
│                    CTO App Setup                       │
├─────────────────────────────────────────────────────────┤
│  [1/4] Container Runtime                                │
│  ✓ Detected: Docker Desktop                             │
│                                                         │
│  [2/4] Choose Your Stack                                │
│  Backend:  ○ Go (Grizz)  ● Node.js (Nova)              │
│  CLI:      ● Claude  ○ Factory  ○ Codex                │
│  Model:    ● Sonnet 4  ○ GPT-4  ○ Opus                 │
│                                                         │
│  [3/4] API Keys                                         │
│  Anthropic: sk-ant-••••••••••••••••  [Saved ✓]         │
│                                                         │
│  [4/4] GitHub Connection                                │
│  [Connect GitHub]  → Opens OAuth flow                   │
│  ✓ Connected: user/my-project                          │
│                                                         │
│                              [Finish Setup]             │
└─────────────────────────────────────────────────────────┘
```

### FOMO Strategy (Skills/Tools Display)

Show users their curated skills/tools AND locked premium options:

```
┌─────────────────────────────────────────────────────────┐
│  Your Skills                                    [?]     │
├─────────────────────────────────────────────────────────┤
│  ✓ go-patterns          ✓ effect-patterns              │
│  ✓ shadcn-stack         ✓ react-best-practices         │
│  ✓ security-analysis    ✓ testing-strategies           │
│  ... and 12 more                                        │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 🔒 Premium Skills                               │   │
│  │    kubernetes-operators, bare-metal-provisioning│   │
│  │    multi-agent-patterns, healer-monitoring...   │   │
│  │    [Upgrade to Pro →]                          │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Uninstall / Cleanup

Track resources in `~/.cto-app/resources.json`:

```json
{
  "kind_cluster": "cto-app",
  "tunnel_subdomain": "abc123",
  "github_webhooks": [
    {"repo": "user/my-project", "hook_id": 12345}
  ],
  "colima_installed_by_us": true,
  "docker_images": ["ghcr.io/5dlabs/cto-app-controller:v1.0"]
}
```

Clean all on uninstall.

---

## Design Guidance

### Technology Stack

| Component | Choice |
|-----------|--------|
| Framework | React 18+ |
| Styling | Tailwind CSS |
| Components | shadcn/ui |
| Icons | Lucide React |
| State | Zustand or Jotai |
| Forms | React Hook Form + Zod |

### Design Principles

- **Dark mode first**
- **Minimal chrome**
- **Clear hierarchy**
- **Responsive feedback**
- **Keyboard accessible**

### Style Inspiration

- Linear's clean UI
- Vercel's dashboard
- Raycast's native feel
- Arc browser's aesthetic

---

## Potential Issues and Rework

### High Risk

| Area | Issue | Mitigation |
|------|-------|------------|
| PM Server | Linear coupling | Fork to `pm-lite` |
| Play Workflow | Multi-agent design | Create single-agent template |
| Intake | Linear task storage | Local storage (SQLite/JSON) |
| Atlas Removal | Workflow expects Atlas | Modify to end at PR merged |
| Tool Server | Reads cto-config.json | Hardcode tool sets |

### Medium Risk

| Area | Issue | Mitigation |
|------|-------|------------|
| Agent Images | Runtime skill fetch | Pre-bake skills |
| Tunnel Allocation | Unique subdomains | Allocation service |
| MCP Server | Many tools | Create `mcp-lite` |
| Controller | Enterprise deps | Audit and stub |

### Open Questions

1. **Local Task Storage** - SQLite vs JSON files?
2. **Tunnel Naming** - Hash-based vs sequential?
3. **Offline Support** - How much works offline?
4. **Telemetry** - Anonymous tracking with consent?
5. **Rate Limiting** - Workflow limits for free tier?

---

## Implementation Phases

### Phase 1: Tauri App Foundation

- [ ] Set up Tauri project with React UI
- [ ] Implement setup wizard (stack selection, API keys, OAuth)
- [ ] Implement container runtime detection
- [ ] Build Kind cluster management

### Phase 2: Core Infrastructure

- [ ] Create `cto-app` Helm chart
- [ ] Fork PM server to `pm-lite`
- [ ] Update agent prompts (no Atlas, clean PRs)
- [ ] Build tunnel allocation system
- [ ] Bundle skills into agent images
- [ ] Configure Bolt for local/Docker

### Phase 3: Dashboard and MCP

- [ ] Build workflow status/logs view
- [ ] Create MCP background service
- [ ] Create GitHub App
- [ ] Integrate log streaming
- [ ] Create tool server lite

### Phase 4: Distribution

- [ ] Build download page at `cto.dev`
- [ ] Set up CI for Tauri builds
- [ ] Configure code signing
- [ ] Set up CDN for installers
- [ ] Push images to GHCR

### Phase 5: Polish

- [ ] User documentation
- [ ] Troubleshooting guide
- [ ] Quick start tutorial
- [ ] Beta testing

---

## Resource Footprint

| Component | RAM |
|-----------|-----|
| Tauri App | ~100MB |
| Container Runtime | ~200-400MB |
| Kind Cluster | ~300MB |
| Argo Workflows | ~150MB |
| Controller | ~50MB |
| PM Server | ~30MB |
| MCP Service | ~20MB |
| **Total baseline** | **~850MB-1GB** |
| Agent pod (each) | ~200-500MB |

---

## Distribution

| Artifact | Location | Visibility |
|----------|----------|------------|
| macOS Installer | cto.dev/download | Website |
| Windows Installer | cto.dev/download | Website |
| Linux Packages | cto.dev/download | Website |
| Helm chart | ghcr.io/5dlabs/charts/cto-app | Public |
| Agent images | ghcr.io/5dlabs/cto-app-* | Public |
| Source code | github.com/5dlabs/cto | Private |

---

## Freemium vs Paid

| Feature | Lite | Paid |
|---------|------|------|
| Desktop app | Yes | Web-based |
| Agents | Grizz, Nova, Blaze | All 8+ |
| Quality/Security/Test | Yes | Yes |
| Bolt | Local/Docker | Full K8s |
| PRD Intake | Single-agent | Multi-agent |
| Linear | No | Yes |
| Atlas | No | Yes |
| Healer | No | Yes |
| Multi-repo | No | Yes |
| Observability | App logs | Full stack |
| Customization | No | Yes |
