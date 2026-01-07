# CTO Platform Open Source Strategy

## Overview

The CTO Platform follows an **open-core model**: a free, open-source version for local development that showcases the agent orchestration capabilities, with a natural upgrade path to the fully-managed SaaS for production use.

**Goal**: Let developers experience the "magic" of multi-agent orchestration locally, then convert to SaaS when they hit the intentional limitations.

---

## Open Source vs SaaS Feature Matrix

| Feature | Open Source | SaaS |
|---------|-------------|------|
| **Agent Orchestration** | ✓ | ✓ |
| Multi-agent task execution | ✓ | ✓ |
| Task decomposition | ✓ | ✓ |
| Agent templates (Rex, Blaze, Nova, etc.) | ✓ | ✓ |
| **CLI Tools** | ✓ | ✓ |
| `cto intake` (PRD → tasks) | ✓ | ✓ |
| `cto play` (run workflows) | ✓ | ✓ |
| **Installation** | Kind-only | Fully managed |
| Local Kind cluster | ✓ Auto-provisioned | N/A (cloud) |
| Production clusters | ✗ | ✓ |
| **Integrations** | Manual | One-click |
| GitHub | PAT (manual) | OAuth App |
| Linear | ✗ | ✓ |
| Slack | ✗ | ✓ |
| **Infrastructure** | ✗ | ✓ |
| Bolt (project setup) | ✗ | ✓ |
| Cloud provisioning | ✗ | ✓ |
| Bare metal (Latitude.sh) | ✗ | ✓ |
| **Auto-Healing** | ✗ | ✓ |
| Healer (CI fix) | ✗ | ✓ |
| **Team Features** | ✗ | ✓ |
| Multi-user | ✗ | ✓ |
| SSO/SAML | ✗ | ✓ |
| Audit logs | ✗ | ✓ |
| **AI Keys** | BYOK only | BYOK or managed |
| **Execution** | Local machine | 5D Labs cloud |

---

## Architecture: Open Source Edition

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           OPEN SOURCE ARCHITECTURE                                       │
│                              (Single Machine, Kind)                                      │
│                                                                                          │
│   User's Machine                                                                        │
│   ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│   │                                                                                    │ │
│   │   ┌─────────────────────────────────────────────────────────────────────────────┐ │ │
│   │   │                              CTO CLI                                         │ │ │
│   │   │                                                                              │ │ │
│   │   │   $ cto intake ./prd.md                                                     │ │ │
│   │   │   $ cto play --task 1                                                       │ │ │
│   │   │   $ cto status                                                              │ │ │
│   │   │                                                                              │ │ │
│   │   └─────────────────────────────────────────────────────────────────────────────┘ │ │
│   │                                        │                                          │ │
│   │                                        ▼                                          │ │
│   │   ┌─────────────────────────────────────────────────────────────────────────────┐ │ │
│   │   │                         Kind Cluster (auto-provisioned)                      │ │ │
│   │   │                                                                              │ │ │
│   │   │   ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                   │ │ │
│   │   │   │  Controller   │  │  MCP Tools    │  │  Agent Pods   │                   │ │ │
│   │   │   │  (lite)       │  │  (basic)      │  │  (ephemeral)  │                   │ │ │
│   │   │   └───────────────┘  └───────────────┘  └───────────────┘                   │ │ │
│   │   │                                                                              │ │ │
│   │   │   Included MCP tools:                                                       │ │ │
│   │   │   • GitHub (via PAT)                                                        │ │ │
│   │   │   • Filesystem                                                              │ │ │
│   │   │   • Basic search                                                            │ │ │
│   │   │                                                                              │ │ │
│   │   │   NOT included:                                                             │ │ │
│   │   │   • Kubernetes MCP                                                          │ │ │
│   │   │   • Terraform MCP                                                           │ │ │
│   │   │   • Cloud provider MCPs                                                     │ │ │
│   │   │   • Linear MCP                                                              │ │ │
│   │   │                                                                              │ │ │
│   │   └─────────────────────────────────────────────────────────────────────────────┘ │ │
│   │                                                                                    │ │
│   └───────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
│   External (user-provided):                                                             │
│   ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│   │                                                                                    │ │
│   │   • GitHub PAT (manual setup)                                                     │ │
│   │   • AI API keys (Anthropic, OpenAI, etc.)                                         │ │
│   │   • Local compute resources                                                       │ │
│   │                                                                                    │ │
│   └───────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Installation Experience

### Open Source (Local)

```bash
# One-line install
curl -fsSL https://5dlabs.io/install | sh

# What happens:
# 1. Checks prerequisites (Docker)
# 2. Downloads CTO CLI binary
# 3. Creates Kind cluster with CTO components
# 4. Prompts for GitHub PAT and AI API key
# 5. Ready to use in ~2 minutes
```

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                          │
│   $ curl -fsSL https://5dlabs.io/install | sh                                           │
│                                                                                          │
│   ╭──────────────────────────────────────────────────────────────────────────────────╮  │
│   │                                                                                   │  │
│   │   ██████╗████████╗ ██████╗                                                       │  │
│   │  ██╔════╝╚══██╔══╝██╔═══██╗                                                      │  │
│   │  ██║        ██║   ██║   ██║   AI-Powered Development Platform                    │  │
│   │  ██║        ██║   ██║   ██║   Open Source Edition                                │  │
│   │  ╚██████╗   ██║   ╚██████╔╝                                                      │  │
│   │   ╚═════╝   ╚═╝    ╚═════╝                                                       │  │
│   │                                                                                   │  │
│   ╰──────────────────────────────────────────────────────────────────────────────────╯  │
│                                                                                          │
│   [✓] Docker detected                                                                   │
│   [✓] Downloaded CTO CLI v0.1.0                                                         │
│   [•] Creating Kind cluster...                                                          │
│       └─ Pulling images (this may take a minute on first run)                           │
│   [✓] Kind cluster 'cto' created                                                        │
│   [✓] Controller deployed                                                               │
│   [✓] MCP tools deployed                                                                │
│                                                                                          │
│   ─────────────────────────────────────────────────────────────────────────────────     │
│                                                                                          │
│   Setup credentials:                                                                    │
│                                                                                          │
│   GitHub PAT (for repo access):                                                         │
│   → Create at: https://github.com/settings/tokens                                       │
│   → Scopes needed: repo, workflow                                                       │
│   Enter PAT: ghp_xxxxxxxxxxxxxxxxxxxx                                                   │
│                                                                                          │
│   AI Provider:                                                                          │
│   [1] Anthropic (Claude) - recommended                                                  │
│   [2] OpenAI (GPT-4)                                                                    │
│   [3] Google (Gemini)                                                                   │
│   Select [1-3]: 1                                                                       │
│                                                                                          │
│   Anthropic API Key:                                                                    │
│   → Get at: https://console.anthropic.com/                                              │
│   Enter key: sk-ant-xxxxxxxxxxxxxxxxxxxx                                                │
│                                                                                          │
│   [✓] Credentials saved                                                                 │
│                                                                                          │
│   ─────────────────────────────────────────────────────────────────────────────────     │
│                                                                                          │
│   🎉 Ready! Try your first task:                                                        │
│                                                                                          │
│   $ cto intake ./prd.md           # Convert PRD to tasks                                │
│   $ cto play --task 1             # Run first task                                      │
│   $ cto status                    # Watch progress                                      │
│                                                                                          │
│   Documentation: https://docs.5dlabs.io                                                 │
│   Discord: https://discord.gg/5dlabs                                                    │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### SaaS (Zero Install)

```
Sign up → Connect GitHub → Create task → Done

No CLI required (though available). No local setup. Works from any device.
```

---

## Crate Distribution

### What's Included in Open Source

| Crate | Included | Notes |
|-------|----------|-------|
| `crates/intake` | ✓ Full | PRD parsing, task generation |
| `crates/cli` | ✓ Lite | No infrastructure commands |
| `crates/controller` | ✓ Lite | Single-tenant, no CRD operator |
| `crates/mcp` | ✓ Basic | GitHub, filesystem, search only |
| `crates/config` | ✓ Full | Configuration management |
| `crates/utils` | ✓ Full | Shared utilities |

### What's Excluded from Open Source

| Crate | Reason |
|-------|--------|
| `crates/metal` | Infrastructure provisioning (Latitude.sh) |
| `crates/healer` | Auto-healing CI failures |
| `crates/pm` | Linear/GitHub project management |
| `crates/cloud` | Cloud provider integrations |
| `crates/gpu` | GPU cluster management |
| `crates/cost` | Cost tracking and billing |

### Build Flags

```toml
# Cargo.toml feature flags

[features]
default = ["oss"]
oss = ["basic-mcp", "kind-only"]
saas = ["full-mcp", "multi-tenant", "infrastructure", "healer", "pm"]

basic-mcp = []
full-mcp = ["basic-mcp", "kubernetes-mcp", "terraform-mcp", "linear-mcp"]
kind-only = []
multi-tenant = []
infrastructure = ["metal", "cloud", "gpu"]
healer = []
pm = []
```

---

## Conversion Funnel

### Natural Pain Points (Drive Upgrade)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                        PAIN POINTS → SAAS VALUE PROP                                     │
│                                                                                          │
│   Open Source Pain                      │    SaaS Solution                              │
│   ─────────────────────────────────────────────────────────────────────────────────     │
│                                         │                                               │
│   "My laptop fan is always running"     │    Agents run on our infrastructure          │
│   (local compute is expensive)          │    Your machine stays cool                   │
│                                         │                                               │
│   "CI keeps failing, manual fixes"      │    Healer auto-detects and fixes             │
│   (no auto-healing)                     │    CI failures automatically                 │
│                                         │                                               │
│   "Managing PATs is annoying"           │    One-click GitHub App install              │
│   (manual credential setup)             │    OAuth flow, done in 30 seconds            │
│                                         │                                               │
│   "Can't use from my iPad"              │    Web portal works everywhere               │
│   (requires local Docker)               │    No local requirements                     │
│                                         │                                               │
│   "Linear integration would help"       │    Full Linear integration                   │
│   (no PM tools)                         │    Auto-sync tasks, status updates           │
│                                         │                                               │
│   "Team can't collaborate"              │    Multi-user, shared workspace              │
│   (single user only)                    │    Team dashboards, audit logs               │
│                                         │                                               │
│   "Need to deploy infra for project"    │    Bolt agent handles it                     │
│   (no infrastructure management)        │    One task: "Set up AWS infra"              │
│                                         │                                               │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### In-CLI Upgrade Prompts

Tasteful, non-intrusive prompts after task completion:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                          │
│   ✓ Task completed! PR created: https://github.com/acme/api/pull/42                     │
│                                                                                          │
│   ───────────────────────────────────────────────────────────────────────────────────   │
│                                                                                          │
│   💡 Running agents locally? Try 5D Labs SaaS:                                          │
│      • No local resources needed                                                        │
│      • Auto-healing CI failures                                                         │
│      • One-click GitHub & Linear                                                        │
│                                                                                          │
│   20 free tasks/month: https://app.5dlabs.io/signup?ref=cli                             │
│                                                                                          │
│   (Disable with: cto config set show_upgrade_prompts false)                             │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### Contextual Prompts

Show relevant upgrade messages based on what the user is experiencing:

| Trigger | Message |
|---------|---------|
| Agent runs > 10 min | "Agents running slow? SaaS runs on dedicated GPU servers." |
| GitHub PAT expires | "Tired of managing PATs? SaaS uses OAuth—set once, forget." |
| User runs `cto healer` (not available) | "Healer is a SaaS feature. Auto-fix CI failures for $X/mo." |
| User runs `cto linear` (not available) | "Linear integration is available on SaaS. Try free!" |
| 50+ tasks completed locally | "You're a power user! SaaS would save you X hours/month." |

---

## Licensing

### Open Source Components

```
Apache 2.0 License

- Agent orchestration core
- CLI (lite version)  
- Basic MCP tools
- Templates and prompts
- Documentation
```

### SaaS-Only Components (Not Open Source)

```
Proprietary / Source Available

- Healer
- PM Server (Linear integration)
- Infrastructure management (Metal, Cloud)
- Multi-tenant operators
- Advanced MCP tools
```

### Contributor License Agreement

Contributors to open source components sign CLA allowing 5D Labs to:
- Include contributions in both OSS and SaaS
- Relicense if needed for SaaS distribution

---

## Competitive Positioning

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              MARKET POSITIONING                                          │
│                                                                                          │
│                        Open Source                          SaaS                        │
│                        ───────────                          ────                        │
│                                                                                          │
│   Devin (Cognition)    ✗ Closed                            ✓ (waitlist, expensive)     │
│   Cursor               ✗ Closed                            ✓ $20/seat                  │
│   GitHub Copilot       ✗ Closed                            ✓ $19/seat                  │
│                                                                                          │
│   CTO Platform         ✓ Full agent orchestration          ✓ Managed + infra          │
│                          (Kind, local)                        (zero-ops)               │
│                                                                                          │
│   ─────────────────────────────────────────────────────────────────────────────────     │
│                                                                                          │
│   Our advantage:                                                                        │
│   • Only platform with real open source agent orchestration                             │
│   • Try before you buy (not just a demo)                                                │
│   • Community contributions improve agents                                              │
│   • Transparent about what's OSS vs paid                                                │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

| Aspect | Open Source | SaaS |
|--------|-------------|------|
| **Target** | Individual devs, evaluation | Teams, production |
| **Install** | `curl \| sh` (Kind) | Sign up (nothing to install) |
| **Compute** | Your machine | Our cloud |
| **Integrations** | Manual (PAT) | One-click OAuth |
| **Features** | Agent orchestration core | Full platform |
| **Price** | Free | $0-499+/month |
| **Goal** | Experience the magic | Scale with zero ops |

The open source version is intentionally limited to showcase capabilities while naturally driving conversion to SaaS for production use.
