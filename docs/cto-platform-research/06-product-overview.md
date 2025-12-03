# CTO Platform Product Overview

> Cognitive Task Orchestrator — GitOps for Agents

## What CTO Is

CTO (Cognitive Task Orchestrator) is a **multi-agent AI engineering platform** that deploys autonomous agents to ship production code via GitHub PRs. It's not a coding assistant — it's an engineering team that runs 24/7 on Kubernetes.

---

## The Agent Team

Eight specialized agents with distinct personalities working together:

| Agent | Role | Personality | What They Do |
|-------|------|-------------|--------------|
| **Morgan** 📚 | Project Lead | Articulate & organized | Oversees architecture, generates documentation, manages GitHub projects |
| **Rex** 🦀 | Backend Engineer | Hardcore engineer | Builds APIs, services, and backend infrastructure |
| **Blaze** 🎨 | Frontend Engineer | Creative & UX obsessed | Creates frontends and user experiences |
| **Cleo** 🔍 | Code Reviewer | Meticulous & wise | Reviews code, refactors for quality, ensures standards |
| **Cipher** 🛡️ | Security Engineer | Vigilant & protective | Security reviews, vulnerability scanning |
| **Tess** 🕵️ | QA Engineer | Curious & thorough | Creates tests, validates functionality |
| **Atlas** 🔗 | Integration Lead | Systematic & reliable | Manages PR merges, resolves conflicts |
| **Bolt** ⚡ | DevOps/SRE | Fast & action-oriented | Infrastructure operations, deployment monitoring |

**The pitch:** "It's like having a senior development team that never sleeps, never argues, and always delivers."

---

## How It Works

### The Three-Phase Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  📚 Phase 1: INTAKE                                            │
│  via Morgan                                                     │
│  ───────────────────────────────────────────────────────────── │
│  • Parses PRD and generates TaskMaster task breakdown          │
│  • Enriches context via Firecrawl (auto-scrapes URLs)          │
│  • Creates docs (task.md, prompt.md, acceptance-criteria.md)   │
│  • Adds agent routing hints for frontend/backend tasks         │
│  • Submits PR with complete project structure                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ⚡ Phase 2: PLAY (Implementation)                              │
│  via Rex/Blaze → Cleo → Tess/Cipher                            │
│  ───────────────────────────────────────────────────────────── │
│  • Phase 2a: Rex/Blaze build the core functionality            │
│  • Phase 2b: Cleo reviews and refactors                        │
│  • Phase 2c: Tess validates, Cipher secures                    │
│  • Event-driven coordination with automatic handoffs           │
│  • Each phase submits detailed PRs                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  🛡️ Phase 3: SHIP                                               │
│  via Atlas → Bolt                                               │
│  ───────────────────────────────────────────────────────────── │
│  • Atlas manages PR merges, resolves conflicts                 │
│  • Bolt handles deployment, monitors production                │
│  • Automatic cleanup and resource management                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### MCP Tool Interface

Simple commands drive complex workflows:

```javascript
// Start a new project from PRD
intake({
  project_name: "my-awesome-app"
});

// Execute full development cycle on a task
play({
  task_id: 1
});

// Monitor running workflows
jobs();

// Stop a workflow
stop_job({
  job_type: "play",
  name: "play-workflow-abc123"
});
```

---

## Technical Architecture

### Core Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| **MCP Server** | Rust (`cto-mcp`) | Handles MCP protocol, configuration-driven defaults |
| **Controller Service** | Kubernetes | REST API managing CodeRun/DocsRun CRDs |
| **Workflow Engine** | Argo Workflows | Orchestrates agent deployment |
| **Agent Runtime** | Kubernetes Jobs | Isolated workspaces with persistent volumes |
| **Authentication** | GitHub Apps | Secure per-agent authentication |
| **CLI Support** | Multi-CLI | Claude Code, Cursor, Codex, Factory, OpenCode |

### Data Flow

```
Any CLI (Cursor, Claude Code, etc.)
        │
        │ MCP Protocol
        ▼
┌─────────────────┐
│   MCP Server    │◄── cto-config.json (defaults)
│   (cto-mcp)     │
└────────┬────────┘
         │
         │ REST API
         ▼
┌─────────────────┐
│   Controller    │
│   Service       │
└────────┬────────┘
         │
         │ CRD Creation
         ▼
┌─────────────────┐
│ Argo Workflows  │
└────────┬────────┘
         │
         │ Job Orchestration
         ▼
┌─────────────────┐
│  Agent Jobs     │──► GitHub PRs
│  (K8s)          │
└─────────────────┘
```

---

## Project Flexibility

| Project Type | Agents Involved | Output |
|--------------|-----------------|--------|
| **Backend** | Rex builds APIs, services, databases | Production-ready backend code |
| **Frontend** | Blaze creates UIs, dashboards, apps | Polished user interfaces |
| **Full-Stack** | Rex & Blaze work together seamlessly | Complete applications |
| **Quality** | Cleo reviews, Tess tests, Cipher secures | Enterprise-grade code |

**Result:** Fast • Elegant • Tested • Documented • Secure

---

## Deployment Options

### Kubernetes Deployment (Primary)

```bash
# Add Helm repository
helm repo add 5dlabs https://5dlabs.github.io/cto
helm repo update

# Install CRDs
kubectl apply -f https://raw.githubusercontent.com/5dlabs/cto/main/infra/charts/cto/crds/platform-crds.yaml

# Install CTO
helm install cto 5dlabs/cto --namespace cto --create-namespace
```

**Requirements:**
- Kubernetes 1.19+
- Helm 3.2.0+
- GitHub Personal Access Token
- Anthropic API Key (or other LLM provider keys)

### What You Get

- Complete CTO platform on Kubernetes
- REST API for task management
- Kubernetes controllers for CodeRun/DocsRun
- Agent workspace management with persistent volumes
- Automatic resource cleanup
- MCP tools for CLI integration

---

## Multi-CLI Support

CTO is **CLI-agnostic** — use whatever coding assistant fits your workflow:

| CLI | Provider | Configuration |
|-----|----------|---------------|
| **Claude Code** | Anthropic | `"cli": "claude"` |
| **Cursor** | Cursor | `"cli": "cursor"` |
| **Codex** | OpenAI | `"cli": "codex"` |
| **OpenCode** | Open Source | `"cli": "opencode"` |
| **Factory** | Factory AI | `"cli": "factory"` |

**Mix and match:** Each agent can use a different CLI optimized for its role:

```json
{
  "agents": {
    "morgan": { "cli": "claude", "model": "claude-opus-4-5-20250929" },
    "rex": { "cli": "codex", "model": "gpt-5-codex" },
    "blaze": { "cli": "cursor", "model": "claude-sonnet-4-20250514" },
    "cleo": { "cli": "claude", "model": "claude-sonnet-4-20250514" }
  }
}
```

---

## Configuration

### cto-config.json

Central configuration file in project root:

```json
{
  "version": "1.0",
  "defaults": {
    "intake": {
      "githubApp": "5DLabs-Morgan",
      "primary": { "model": "opus", "provider": "claude-code" }
    },
    "play": {
      "model": "claude-sonnet-4-20250514",
      "cli": "claude",
      "implementationAgent": "5DLabs-Rex",
      "qualityAgent": "5DLabs-Cleo",
      "testingAgent": "5DLabs-Tess",
      "repository": "your-org/your-repo"
    }
  },
  "agents": {
    "morgan": {
      "githubApp": "5DLabs-Morgan",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514",
      "tools": {
        "remote": ["memory_create_entities", "brave_search_brave_web_search"],
        "localServers": {
          "filesystem": { "enabled": true },
          "git": { "enabled": true }
        }
      }
    }
  }
}
```

### Per-Agent Tool Control

Fine-grained control over what each agent can access:

- **Remote tools:** Memory, search, external APIs
- **Local servers:** Filesystem, Git operations
- **Security:** Restrict capabilities per agent role

---

## Licensing

**AGPL-3.0** — Open source with network clause protection.

| What You Can Do | Requirement |
|-----------------|-------------|
| Use commercially | ✅ Allowed |
| Modify | ✅ Allowed |
| Distribute | ✅ Allowed |
| Deploy as service | ⚠️ Must provide source access |
| Create derivatives | ⚠️ Must also be AGPL-3.0 |

**Why AGPL:** Ensures improvements stay open source even when deployed as a service. Protects the community while allowing commercial use.

---

## The CTO Platform Vision

CTO (the orchestrator) is the **AI brain**. Combined with bare metal infrastructure automation, it becomes the **CTO Platform** — a complete AI-powered engineering department:

```
┌─────────────────────────────────────────────────────────────────┐
│                      CTO PLATFORM                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐    ┌─────────────────────┐           │
│  │   CTO (Open Core)   │    │  Platform-in-a-Box  │           │
│  │   ────────────────  │    │  ─────────────────  │           │
│  │   • Agent Orchestra │    │  • Bare Metal Auto  │           │
│  │   • MCP Server      │◄──►│  • Provider APIs    │           │
│  │   • K8s Controllers │    │  • Talos Linux      │           │
│  │   • Multi-CLI       │    │  • Cost Optimization│           │
│  │   • GitHub Apps     │    │  • Self-Healing     │           │
│  └─────────────────────┘    └─────────────────────┘           │
│         AGPL-3.0                  Proprietary                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**For startups:** Sign up, deploy, ship — without hiring an engineering team.

**For enterprises:** Cloud repatriation with AI-managed operations.

---

## Links

| Resource | URL |
|----------|-----|
| **GitHub** | https://github.com/5dlabs/cto |
| **Documentation** | https://docs.5dlabs.com |
| **Discord** | https://discord.gg/A6yydvjZKY |
| **Twitter/X** | https://x.com/5dlabs |
| **YouTube** | https://www.youtube.com/@5DLabs |
