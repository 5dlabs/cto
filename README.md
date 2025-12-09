<div align="center">

<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/5dlabs-logo-dark.svg" alt="5D Labs Logo" width="400px">

# **Cognitive Task Orchestrator**
## **Your AI Engineering Team in a Box** 🚀

[![GitHub Stars](https://img.shields.io/github/stars/5dlabs/cto?style=for-the-badge&logo=github&logoColor=white&labelColor=24292e&color=0969da)](https://github.com/5dlabs/cto)
[![Discord](https://img.shields.io/badge/Discord-5dlabs.ai-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/A6yydvjZKY)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue?style=for-the-badge&logo=gnu&logoColor=white)](LICENSE)
[![Kubernetes](https://img.shields.io/badge/Kubernetes-1.19+-326CE5?style=for-the-badge&logo=kubernetes&logoColor=white)](https://kubernetes.io/)

### **💎 Self-Hosted AI Development Platform • Bare-Metal Ready • MCP Native 💎**
*Deploy an autonomous engineering team on your infrastructure—ship production code while slashing cloud & staffing costs*

</div>

---

<div align="center">

## **💰 Why CTO?**

### **Not Just a Team—An Entire Engineering Infrastructure**

<table>
<tr>
<td align="center" width="25%">

### **🏗️ Full Engineering Team**
13 specialized AI agents: backend, frontend, QA, security, DevOps—working 24/7

</td>
<td align="center" width="25%">

### **🔧 Self-Hosted & Bare-Metal**
Deploy anywhere: bare-metal, on-prem, or cloud—complete data sovereignty

</td>
<td align="center" width="25%">

### **📊 Production Observability**
Prometheus, Grafana, Loki, AlertManager—full metrics, logs, and alerting out of the box

</td>
<td align="center" width="25%">

### **🔄 Self-Healing Platform**
Auto-detects failures, stuck workflows, and CI issues—spawns healing agents automatically

</td>
</tr>
</table>

### **💸 Slash Your Engineering Costs**

| Traditional Approach | With CTO |
|---------------------|----------|
| $150k-250k/yr per engineer × 5-10 engineers | **$0** — AI agents included |
| $5k-50k/mo cloud infrastructure | **60-80% savings** on bare-metal |
| 24/7 on-call rotation costs | **Automated** self-healing |
| Weeks to onboard new team members | **Instant** agent deployment |

### **🔐 Bring Your Own Keys (BYOK)**

- **Your API keys** — Anthropic, OpenAI, Google, etc. stored securely in your infrastructure
- **Your cloud credentials** — AWS, GCP, Azure keys never leave your cluster
- **Secret management with OpenBao** — Open-source HashiCorp Vault fork for enterprise-grade secrets
- **Zero vendor lock-in** — Switch providers anytime, no data hostage situations

### **🌐 Zero-Trust Networking**

| Feature | Technology | What It Does |
|---------|------------|--------------|
| **Cloudflare Tunnels** | `cloudflared` | Expose services publicly without opening firewall ports — no public IPs needed, automatic TLS, global edge CDN |
| **Kilo VPN** | WireGuard | Secure mesh VPN for remote cluster access — connect from anywhere with encrypted tunnels |
| **OpenBao** | Vault fork | Centralized secrets management with dynamic credentials and audit logging |

**Cloudflare Tunnels** is a game-changer: your entire platform can run on air-gapped infrastructure while still being accessible from anywhere. No ingress controllers, no load balancers, no exposed ports—just secure outbound tunnels through Cloudflare's network.

### **🏭 Infrastructure Operators (Managed by Bolt)**

Replace expensive managed cloud services with open-source Kubernetes operators:

| Operator | Replaces | Savings |
|----------|----------|---------|
| **CloudNative-PG** | AWS RDS, Cloud SQL, Azure PostgreSQL | ~70-80% |
| **Strimzi Kafka** | AWS MSK, Confluent Cloud | ~60-70% |
| **MinIO** | AWS S3, GCS, Azure Blob | ~80-90% |
| **Redis Operator** | ElastiCache, Memorystore | ~70-80% |
| **OpenSearch** | AWS OpenSearch, Elastic Cloud | ~60-70% |
| **ClickHouse** | BigQuery, Redshift, Snowflake | ~70-80% |
| **QuestDB** | TimescaleDB Cloud, InfluxDB Cloud | ~70-80% |

**Bolt** automatically deploys, monitors, and maintains these operators—giving you managed-service reliability at self-hosted prices.

</div>

---

<div align="center">

## **🚧 Development Status**

**Public launch: January 1st, 2025** 🚀

The platform is in beta and being refined based on production usage.

**Current Status:**
✅ Core platform architecture implemented  
✅ MCP server with dynamic tool registration  
✅ Kubernetes controllers with self-healing  
✅ GitHub Apps + Linear integration  
✅ Bare-metal deployment (Latitude, Hetzner, OVH, Vultr, Scaleway, Cherry, DigitalOcean)  
✅ Cloudflare Tunnels for public access without exposed interfaces  
✅ Infrastructure operators (PostgreSQL, Kafka, Redis, MinIO, OpenSearch, ClickHouse, QuestDB)  
🔄 Documentation and onboarding improvements  

</div>

---

<div align="center">

## **Meet Your AI Engineering Team**

*Thirteen specialized agents with distinct personalities working together 24/7—your full-stack engineering department in a box*

<div align="center">

### **🎯 Project Management & Architecture**

<table>
<tr>
<td align="center" width="100%">

### **Morgan**
#### *The Technical Program Manager*

<div align="center">
<img src="assets/morgan-avatar-512.png" width="180" height="180" alt="Morgan Avatar">
</div>

🐕 **Personality:** Articulate & organized  
📋 **Superpower:** Turns chaos into actionable roadmaps  
💬 **Motto:** *"A plan without tasks is just a wish."*

**Morgan orchestrates project lifecycles—syncing GitHub Issues with Linear roadmaps, decomposing PRDs into sprint-ready tasks, and keeping stakeholders aligned through `intake()` MCP calls.**

</td>
</tr>
</table>

### **🦀 Backend Engineering Squad**

<table>
<tr>
<td align="center" valign="top" width="33%">

### **Rex**
#### *The Rust Architect*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/rex-avatar.png" width="180" height="180" alt="Rex Avatar">
</div>

🦀 **Stack:** Rust, Tokio, Axum  
⚡ **Superpower:** Zero-cost abstractions at scale  
💬 **Motto:** *"If it compiles, it ships."*

**Rex builds high-performance APIs, real-time services, and systems-level infrastructure. When microseconds matter, Rex delivers.**

</td>
<td align="center" valign="top" width="33%">

### **Grizz**
#### *The Go Specialist*

<div align="center">
<img src="assets/grizz-avatar-512.png" width="180" height="180" alt="Grizz Avatar">
</div>

🐻 **Stack:** Go, gRPC, PostgreSQL  
🛠️ **Superpower:** Ships bulletproof services under pressure  
💬 **Motto:** *"Simple scales."*

**Grizz builds backend services, REST/gRPC APIs, CLI tools, and Kubernetes operators. From simple CRUD to distributed systems—battle-tested reliability is his signature.**

</td>
<td align="center" valign="top" width="33%">

### **Nova**
#### *The Node.js Engineer*

<div align="center">
<img src="assets/nova-avatar-512.png" width="180" height="180" alt="Nova Avatar">
</div>

✨ **Stack:** Node.js, TypeScript, Fastify  
🌌 **Superpower:** Rapid API development & integrations  
💬 **Motto:** *"Move fast, type safe."*

**Nova builds REST/GraphQL APIs, serverless functions, and third-party integrations. Speed-to-market is her specialty.**

</td>
</tr>
</table>

### **🎨 Frontend Engineering Squad**

<table>
<tr>
<td align="center" valign="top" width="33%">

### **Blaze**
#### *The Web App Developer*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/blaze-avatar.png" width="180" height="180" alt="Blaze Avatar">
</div>

🎨 **Stack:** React, Next.js, shadcn/ui  
✨ **Superpower:** Pixel-perfect responsive interfaces  
💬 **Motto:** *"Great UX is invisible."*

**Blaze creates stunning web applications with modern component libraries. From dashboards to marketing sites, she delivers polished experiences.**

</td>
<td align="center" valign="top" width="33%">

### **Tap**
#### *The Mobile Developer*

<div align="center">
<img src="assets/tap-avatar-512.png" width="180" height="180" alt="Tap Avatar">
</div>

📱 **Stack:** Expo, React Native, NativeWind  
🎯 **Superpower:** Cross-platform mobile excellence  
💬 **Motto:** *"One codebase, every pocket."*

**Tap builds native-quality iOS and Android apps from a single TypeScript codebase. App Store ready, always.**

</td>
<td align="center" valign="top" width="33%">

### **Spark**
#### *The Desktop Developer*

<div align="center">
<img src="assets/spark-avatar-512.png" width="180" height="180" alt="Spark Avatar">
</div>

⚡ **Stack:** Electron, Tauri, React  
🖥️ **Superpower:** Native desktop apps that feel right  
💬 **Motto:** *"Desktop isn't dead—it's evolved."*

**Spark crafts cross-platform desktop applications with native integrations, system tray support, and offline-first architectures.**

</td>
</tr>
</table>

### **🛡️ Quality & Security Squad**

<table>
<tr>
<td align="center" valign="top" width="33%">

### **Cleo**
#### *The Quality Guardian*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/cleo-avatar.png" width="180" height="180" alt="Cleo Avatar">
</div>

🔍 **Personality:** Meticulous & wise  
✨ **Superpower:** Spots code smells instantly  
💬 **Motto:** *"Excellence isn't negotiable."*

**Cleo refactors for maintainability, enforces patterns, and ensures enterprise-grade code quality across every PR.**

</td>
<td align="center" valign="top" width="33%">

### **Cipher**
#### *The Security Sentinel*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/cipher-avatar.png" width="180" height="180" alt="Cipher Avatar">
</div>

🛡️ **Personality:** Vigilant & protective  
🔒 **Superpower:** Finds vulnerabilities before attackers  
💬 **Motto:** *"Trust nothing, verify everything."*

**Cipher runs security audits, dependency scans, and ensures OWASP compliance across all workflows.**

</td>
<td align="center" valign="top" width="33%">

### **Tess**
#### *The Testing Genius*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/tess-avatar.png" width="180" height="180" alt="Tess Avatar">
</div>

🕵️ **Personality:** Curious & thorough  
🎪 **Superpower:** Finds edge cases others miss  
💬 **Motto:** *"If it can break, I'll find it first!"*

**Tess creates comprehensive test suites—unit, integration, and e2e—ensuring reliability before every merge.**

</td>
</tr>
</table>

### **🚀 Operations Squad**

<table>
<tr>
<td align="center" valign="top" width="33%">

### **Atlas**
#### *The Integration Master*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png" width="180" height="180" alt="Atlas Avatar">
</div>

🔗 **Personality:** Systematic & reliable  
🌉 **Superpower:** Resolves merge conflicts automatically  
💬 **Motto:** *"Every branch finds its way home."*

**Atlas manages PR merges, rebases stale branches, and ensures clean integration with trunk-based development.**

</td>
<td align="center" valign="top" width="33%">

### **Bolt**
#### *The Deployment Specialist*

<div align="center">
<img src="https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/bolt-avatar.png" width="180" height="180" alt="Bolt Avatar">
</div>

⚡ **Personality:** Fast & action-oriented  
🚀 **Superpower:** Zero-downtime deployments  
💬 **Motto:** *"Ship it fast, ship it right!"*

**Bolt handles GitOps deployments, monitors rollouts, and ensures production health with automated rollbacks.**

</td>
<td align="center" valign="top" width="33%">

### **Stitch**
#### *The Automated Code Reviewer*

<div align="center">
<img src="assets/stitch-avatar-512.png" width="180" height="180" alt="Stitch Avatar">
</div>

🧵 **Personality:** Meticulous & tireless  
🔎 **Superpower:** Reviews every PR with surgical precision  
💬 **Motto:** *"No loose threads."*

**Stitch provides automated code review on every pull request—like Cursor's Bugbot, but integrated into your workflow. Catches bugs, suggests improvements, and ensures consistency.**

</td>
</tr>
</table>

</div>

---

### 🌟 **The Magic: How Your AI Team Collaborates**

<div align="center">

**Watch the magic happen when they work together:**

<table>
<tr>
<td align="center" width="33%">

**📚 Phase 1**  
**Morgan** documents  
requirements & architecture

*via `intake()` MCP call*

</td>
<td align="center" width="33%">

**⚡ Phase 2**  
**Rex & Blaze** build in parallel  
*(backend, frontend, or both)*

*via `play()` multi-agent workflows*

</td>
<td align="center" width="33%">

**🛡️ Phase 3**  
**Cleo, Tess & Cipher** ensure  
quality, testing & security

*via `play()` multi-agent orchestration*

</td>
</tr>
</table>

**💡 Project Flexibility:**

<table>
<tr>
<td align="center" width="50%">
**🦀 Backend Projects**<br/>
Rex (Rust) • Grizz (Go) • Nova (Node.js)
</td>
<td align="center" width="50%">
**🎨 Frontend Projects**<br/>
Blaze (Web/shadcn) • Tap (Mobile/Expo) • Spark (Desktop/Electron)
</td>
</tr>
<tr>
<td align="center" width="50%">
**🚀 Full-Stack Projects**<br/>
Mix backend + frontend agents seamlessly
</td>
<td align="center" width="50%">
**🛡️ Quality Always**<br/>
Cleo reviews • Tess tests • Cipher secures • Stitch code-reviews
</td>
</tr>
</table>

### **🎯 Result: Production-Ready Code**
*Fast • Elegant • Tested • Documented • Secure*

**It's like having a senior development team that never sleeps, never argues, and always delivers!** 🎭

</div>

---

## **⚡ What CTO Does**

The Cognitive Task Orchestrator provides a complete AI engineering platform:

### **🚀 Unified Project Intake (`intake()`)**
**Morgan** processes PRDs, generates tasks, and syncs with your project management tools.

- Parses PRD and generates task breakdown via built-in task engine
- **Linear Integration** (MVP): Two-way sync with Linear roadmaps and sprints
- **GitHub Projects**: Auto-creates issues and project boards
- **More PM tools coming**: Jira, Asana, Notion, Monday.com
- Enriches context via Firecrawl (auto-scrapes referenced URLs)
- Creates comprehensive documentation (task.md, prompt.md, acceptance-criteria.md)
- Agent routing: automatically assigns frontend/backend/mobile tasks
- **Configurable AI models** — use any supported provider and model

### **🎮 Multi-Agent Play Workflows (`play()`)**
**The entire team** orchestrates complex multi-agent workflows with event-driven coordination.

- **Phase 1 - Implementation**: Backend (Rex/Grizz/Nova) or Frontend (Blaze/Tap/Spark)
- **Phase 2 - Quality Assurance**: Cleo reviews and refactors
- **Phase 3 - Testing & Security**: Tess validates, Cipher secures
- **Phase 4 - Code Review**: Stitch provides automated PR review (like Bugbot)
- **Event-Driven Coordination**: Automatic handoffs between phases
- **GitHub Integration**: Each phase submits detailed PRs

### **🔧 Three Core MCP Tools**

| Tool | Purpose |
|------|---------|
| **`addTool()`** | Dynamically add any MCP server by GitHub URL — agents instantly gain access to new capabilities |
| **`intake()`** | Project onboarding — initializes new projects with proper structure and configuration |
| **`play()`** | Full orchestration — coordinates the entire team through build/test/deploy phases |

### **🔄 Self-Healing Infrastructure (Healer)**
The platform includes comprehensive self-healing for both CTO and your deployed applications:

- **Platform Self-Healing**: Monitors CTO's own health—detects stuck workflows, pod failures, step timeouts
- **Application Self-Healing**: Extends to your deployed apps—CI failures, silent errors, stale progress
- **Intelligent Alert Routing**: 9 alert types with context-aware remediation strategies
- **Automated Remediation**: Spawns healing agents to diagnose and fix issues without human intervention
- **Continuous Learning**: Tracks remediation success and adapts strategies

### **📊 Production Observability Stack**
Full observability out of the box—no additional setup required:

| Component | Purpose |
|-----------|---------|
| **Prometheus** | Metrics collection, alerting rules, service discovery |
| **Grafana** | Dashboards, visualization, alert management |
| **Loki** | Log aggregation and querying |
| **AlertManager** | Alert routing, deduplication, notification channels |
| **Blackbox Exporter** | External endpoint monitoring and probing |

**Included Dashboards:**
- Agent performance and task completion rates
- Workflow execution times and success rates
- Resource utilization across all agents
- GitHub PR metrics and merge times
- Self-healing remediation statistics

All operations run as **Kubernetes jobs** with enhanced reliability through TTL-safe reconciliation, preventing infinite loops and ensuring proper resource cleanup.

---

## **🚀 Getting Started**

### Prerequisites
- Access to any supported AI CLI (see [Supported CLIs](#supported-ai-clis))
- GitHub repository for your project

---

## **🏗️ Platform Architecture**

This is an integrated platform with crystal-clear data flow:

### **🖥️ Supported AI CLIs**

CTO works with your favorite AI coding assistant:

| CLI | Description | Status |
|-----|-------------|--------|
| **Claude Code** | Anthropic CLI | ✅ Full support |
| **Cursor** | AI-first code editor | ✅ Full support |
| **Codex** | OpenAI's coding assistant | ✅ Full support |
| **Factory** | Code Factory CLI | ✅ Full support |
| **Gemini** | Google's AI assistant | ✅ Full support |
| **OpenCode** | Open-source alternative | ✅ Full support |
| **Dexter** | Lightweight AI CLI | ✅ Full support |

### **🔧 Integrated Tools Library**

Dynamic MCP tool registration with **57+ pre-configured tools** across GitHub, Kubernetes, ArgoCD, OpenMemory, Context7, and more.

👉 **[View full tools list](https://github.com/5dlabs/cto/blob/main/tools-config.json)**

**Tool Filtering** — Only expose the tools each agent needs:
```json
{
  "agents": {
    "rex": {
      "tools": {
        "remote": ["github_create_pull_request", "github_push_files"],
        "localServers": { "filesystem": { "enabled": true } }
      }
    }
  }
}
```

**Why tool filtering matters:**
- 🎯 **Smaller context** — agents don't waste tokens on irrelevant tool descriptions
- 🔒 **Security** — restrict sensitive operations to specific agents
- ⚡ **Faster responses** — fewer tools = faster tool selection
- 🧠 **Better focus** — agents stay on-task with curated capabilities

**Categories:** GitHub (28) • Kubernetes (18) • shadcn/ui (8) • OpenMemory (5) • ArgoCD (4) • Context7 (2) • Filesystem • Brave Search • Firecrawl

**Frontend Stack**: shadcn/ui components, Tailwind CSS, React patterns built-in

**Component Architecture:**
- **MCP Server (`cto-mcp`)**: Handles MCP protocol calls from any CLI with dynamic tool registration
- **Controller Service**: Kubernetes REST API that manages CodeRun CRDs via Argo Workflows
- **Healer Service**: Self-healing daemon monitoring platform and application health
- **Argo Workflows**: Orchestrates agent deployment through workflow templates
- **Kubernetes Controllers**: CodeRun controller with TTL-safe reconciliation
- **Agent Workspaces**: Isolated persistent volumes for each service with session continuity
- **GitHub Apps + PM Integration**: Linear (MVP), with Jira, Asana, Notion, Monday.com planned
- **Cloudflare Tunnels**: Expose services publicly without opening firewall ports
- **Kilo VPN**: WireGuard-based secure remote cluster access

### **🌐 Cloudflare Tunnels**

Access your services from anywhere without exposing your infrastructure:

- **Zero External Interface**: No public IPs, no open firewall ports, no ingress controllers
- **Automatic TLS**: End-to-end encryption via Cloudflare's edge network
- **Global Edge CDN**: Low-latency access from anywhere in the world
- **Air-Gapped Ready**: Run on isolated networks while remaining publicly accessible
- **Secure by Default**: Only outbound connections — nothing to attack

**Data Flow:**
1. Any CLI calls MCP tools (`intake()`, `play()`, etc.) via MCP protocol
2. MCP server loads configuration from `cto-config.json` and applies defaults
3. MCP server submits workflow to Argo with all required parameters
4. Argo Workflows creates CodeRun custom resources
5. Dedicated Kubernetes controllers reconcile CRDs with idempotent job management
6. Controllers deploy configured CLI agents as Jobs with workspace isolation
7. Agents authenticate via GitHub Apps and complete work
8. Agents submit GitHub PRs with automatic cleanup
9. Healer monitors for issues and auto-remediates failures

---

## **📦 Installation**

### **🔧 Deployment Options**

CTO runs anywhere you have Kubernetes—from bare-metal servers to managed cloud:

| Deployment Type | Providers | Best For |
|-----------------|-----------|----------|
| **Bare-Metal** | Latitude, Hetzner, OVH, Vultr, Scaleway, Cherry, DigitalOcean | Maximum cost savings, data sovereignty |
| **On-Premises** | Any server with Talos Linux | Air-gapped environments, full control |
| **Cloud** | AWS, Azure, GCP | Existing cloud infrastructure |

### Deploy on Bare-Metal (Recommended)

Save 60-80% vs cloud by running on dedicated servers:

```bash
# Bootstrap a Talos cluster on bare-metal (Latitude example)
cto-metal init --provider latitude --region MIA --plan c3-large-x86 --nodes 3

# Or use your own hardware
cto-metal init --provider onprem --config ./my-servers.yaml

# Deploy CTO platform
helm repo add 5dlabs https://5dlabs.github.io/cto
helm install cto 5dlabs/cto --namespace cto --create-namespace
```

**Supported Bare-Metal Providers:**
- **Latitude.sh** - Global bare-metal cloud
- **Hetzner** - European dedicated servers
- **OVH** - European cloud & bare-metal
- **Vultr** - Global bare-metal & cloud
- **Scaleway** - European cloud provider
- **Cherry Servers** - European bare-metal
- **DigitalOcean** - Droplets & bare-metal

### Deploy on Existing Kubernetes

```bash
# Add the 5dlabs Helm repository
helm repo add 5dlabs https://5dlabs.github.io/cto
helm repo update

# Install Custom Resource Definitions (CRDs) first
kubectl apply -f https://raw.githubusercontent.com/5dlabs/cto/main/infra/charts/cto/crds/platform-crds.yaml

# Install the cto
helm install cto 5dlabs/cto --namespace cto --create-namespace

# Setup agent secrets (interactive)
wget https://raw.githubusercontent.com/5dlabs/cto/main/infra/scripts/setup-agent-secrets.sh
chmod +x setup-agent-secrets.sh
./setup-agent-secrets.sh --help
```

**Requirements:**
- Kubernetes 1.19+
- Helm 3.2.0+
- GitHub Personal Access Token (or GitHub App)
- API key for your chosen provider (Anthropic, OpenAI, Google, etc.)

**What you get:**
- Complete cto platform deployed to Kubernetes
- Self-healing infrastructure monitoring
- REST API for task management
- Kubernetes controller for CodeRun resources with TTL-safe reconciliation
- Agent workspace management and isolation with persistent volumes
- Automatic resource cleanup and job lifecycle management
- MCP tools with dynamic registration
- Cloudflare Tunnels for secure public access

### Remote Cluster Access with Kilo VPN

Kilo is an open-source WireGuard-based VPN that provides secure access to cluster services. It's deployed automatically via ArgoCD.

**Client Setup:**

1. Install WireGuard and kgctl:
```bash
# macOS
brew install wireguard-tools
go install github.com/squat/kilo/cmd/kgctl@latest

# Linux
sudo apt install wireguard-tools
go install github.com/squat/kilo/cmd/kgctl@latest
```

2. Generate your WireGuard keys and create a Peer resource (see `docs/vpn/kilo-client-setup.md`)

3. Connect to access cluster services:
```bash
sudo wg-quick up ~/.wireguard/kilo.conf
```

This enables direct access to:
- ClusterIPs (e.g., `curl http://10.x.x.x:port`)
- Service DNS (e.g., `curl http://service.namespace.svc.cluster.local`)

See `docs/vpn/kilo-client-setup.md` for full setup instructions.

### Install MCP Server

For CLI integration, install the MCP server:

```bash
# One-liner installer (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/5dlabs/cto/releases/download/v0.2.0/tools-installer.sh | sh

# Verify installation
cto-mcp --help   # MCP server for any CLI
```

**What you get:**
- `cto-mcp` - MCP server that integrates with any CLI
- Multi-platform support (Linux x64/ARM64, macOS Intel/Apple Silicon)
- Automatic installation to system PATH

---

## **⚙️ Configuration**

### Configure Project Settings

Create a `cto-config.json` file in your project root. See the [full example config](https://github.com/5dlabs/cto/blob/main/cto-config.template.json) for all options.

```json
{
  "version": "1.0",
  "defaults": {
    "docs": {
      "model": "your-preferred-model",
      "githubApp": "5DLabs-Morgan",
      "includeCodebase": false,
      "sourceBranch": "main"
    },
    "play": {
      "model": "your-preferred-model",
      "cli": "factory",
      "implementationAgent": "5DLabs-Rex",
      "frontendAgent": "5DLabs-Blaze",
      "qualityAgent": "5DLabs-Cleo",
      "securityAgent": "5DLabs-Cipher",
      "testingAgent": "5DLabs-Tess",
      "repository": "your-org/your-repo",
      "maxRetries": 10,
      "autoMerge": true,
      "parallelExecution": true
    },
    "intake": {
      "githubApp": "5DLabs-Morgan",
      "primary": { "model": "opus", "cli": "claude" },
      "research": { "model": "gpt-4o", "cli": "codex" },
      "fallback": { "model": "gemini-pro", "cli": "gemini" }
    }
  },
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "factory",
      "model": "your-preferred-model",
      "maxTokens": 64000,
      "temperature": 0.7,
      "reasoningEffort": "high",
      "modelRotation": {
        "enabled": true,
        "models": ["model-a", "model-b", "model-c"]
      },
      "tools": {
        "remote": ["github_create_pull_request", "github_push_files", "context7_get_library_docs"]
      }
    }
  }
}
```

### Key Configuration Features

| Feature | Description |
|---------|-------------|
| **Model Rotation** | Automatically cycle through models on retries — improves success rate |
| **Parallel Execution** | Run implementation agents concurrently for faster builds |
| **Auto-Merge** | Automatically merge PRs that pass all checks |
| **Max Retries** | Per-phase retry limits with exponential backoff |
| **Reasoning Effort** | Control thinking depth (`low`, `medium`, `high`) |
| **Temperature** | Adjust creativity vs. determinism per agent |
| **Tool Filtering** | Restrict tools per agent to reduce context and improve focus |

**Agent Configuration Fields:**
- **`githubApp`**: GitHub App name for authentication
- **`cli`**: Which CLI to use (`claude`, `cursor`, `codex`, `opencode`, `factory`, `gemini`, `dexter`)
- **`model`**: Model identifier for the CLI
- **`maxTokens`**: Maximum tokens for responses
- **`temperature`**: Creativity level (0.0-1.0)
- **`reasoningEffort`**: Thinking depth for supported models
- **`modelRotation`**: Automatic model cycling on failures
- **`tools`**: Fine-grained tool access control

**Benefits:**
- **CLI Flexibility**: Different agents can use different CLIs
- **Model Selection**: Each agent can use its optimal model
- **Tool Profiles**: Customize tool access per agent
- **Security**: Restrict agent capabilities as needed

### Configure MCP Integration

After creating your configuration file, configure your CLI to use the MCP server. Add to your CLI's MCP configuration:

```json
{
  "mcpServers": {
    "cto-mcp": {
      "command": "cto-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

**Configuration Locations by CLI:**
| CLI | Config Path |
|-----|-------------|
| Cursor | `.cursor/mcp.json` in project directory |
| Claude | `~/.config/claude/mcp.json` |
| Other CLIs | Refer to CLI documentation |

**Usage:**
1. Create the `cto-config.json` file in your project root with your specific settings
2. Configure your CLI's MCP integration as shown above
3. Restart your CLI to load the MCP server
4. All MCP tools will be available with your configured defaults

**Benefits of Configuration-Driven Approach:**
- **Simplified MCP Calls**: Most parameters have sensible defaults from your config
- **Dynamic Agent Lists**: Tool descriptions show available agents from your config
- **Consistent Settings**: All team members use the same model/agent assignments
- **Easy Customization**: Change defaults without modifying MCP server setup

---

## **🔧 MCP Tools Reference**

The platform exposes three powerful MCP tools:

### 1. **`addTool()` - Dynamic Tool Registration**
Instantly extend agent capabilities by adding any MCP server from GitHub.

```javascript
// Add a new MCP server by GitHub URL
addTool({
  github_url: "https://github.com/anthropics/mcp-server-memory",
  name: "memory"
});

// Add with custom configuration
addTool({
  github_url: "https://github.com/org/custom-mcp-server",
  name: "custom-tool",
  config: { api_key: "..." }
});
```

**What addTool does:**
✅ Clones and installs MCP server from GitHub  
✅ Registers tools with all active agents  
✅ Agents immediately gain access to new capabilities  
✅ No restart required — hot-reload of tools

### 2. **`intake()` - Project Onboarding**
Initialize new projects with proper structure, tasks, and configuration.

```javascript
// Minimal call - handles everything
intake({
  project_name: "my-awesome-app"
});

// Customize with options
intake({
  project_name: "my-awesome-app",
  enrich_context: true,        // Auto-scrape URLs via Firecrawl
  include_codebase: false      // Include existing code context
});
```

**What intake does:**
✅ Parses PRD and generates task breakdown  
✅ Enriches context by scraping URLs found in PRD  
✅ Creates documentation (task.md, prompt.md, acceptance-criteria.md)  
✅ Adds agent routing hints for frontend/backend tasks  
✅ Syncs with Linear (MVP) for project management  
✅ Submits PR with complete project structure

### 3. **`play()` - Full Team Orchestration**
Coordinates the entire AI engineering team through build/test/deploy phases.

```javascript
// Minimal call - the whole team collaborates!
play({
  task_id: 1
});

// Customize agent assignments
play({
  task_id: 1,
  implementation_agent: "rex",
  quality_agent: "cleo",
  testing_agent: "tess",
  repository: "myorg/my-project"
});
```

**What the team does:**
✅ **Phase 1 - Implementation**: Backend (Rex/Grizz/Nova) or Frontend (Blaze/Tap/Spark)  
✅ **Phase 2 - Quality**: Cleo reviews and refactors  
✅ **Phase 3 - Testing**: Tess validates, Cipher secures  
✅ **Phase 4 - Review**: Stitch provides automated PR review  
✅ **Event-Driven**: Automatic phase transitions  
✅ **GitHub Integration**: PRs from each phase

---

## **📋 MCP Tool Parameters**

### `docs` Tool Parameters

**Required:**
- `working_directory` - Working directory containing `.tasks/` folder (e.g., `"projects/simple-api"`)

**Optional (with config defaults):**
- `agent` - Agent name to use (defaults to `defaults.docs.githubApp` mapping)
- `model` - Model to use for the docs agent (defaults to `defaults.docs.model`)
- `source_branch` - Source branch to work from (defaults to `defaults.docs.sourceBranch`)
- `include_codebase` - Include existing codebase as context (defaults to `defaults.docs.includeCodebase`)

### `play` Tool Parameters

**Required:**
- `task_id` - Task ID to implement from task files (integer, minimum 1)

**Optional (with config defaults):**
- `repository` - Target repository URL (e.g., `"5dlabs/cto"`) (defaults to `defaults.play.repository`)
- `service` - Service identifier for persistent workspace (defaults to `defaults.play.service`)
- `docs_repository` - Documentation repository URL (defaults to `defaults.play.docsRepository`)
- `docs_project_directory` - Project directory within docs repository (defaults to `defaults.play.docsProjectDirectory`)
- `implementation_agent` - Agent for implementation work (defaults to `defaults.play.implementationAgent`)
- `quality_agent` - Agent for quality assurance (defaults to `defaults.play.qualityAgent`)
- `testing_agent` - Agent for testing and validation (defaults to `defaults.play.testingAgent`)
- `model` - Model to use for play-phase agents (defaults to `defaults.play.model`)

---

## **🎨 Template Customization**

The platform uses a template system to customize agent behavior, settings, and prompts. Templates are Handlebars (`.hbs`) files rendered with task-specific data at runtime. All supported CLIs follow the same template structure.

**Model Defaults**: Models are configured through `cto-config.json` defaults (and can be overridden via MCP parameters). Any model supported by your chosen CLI can be specified via configuration.

### Template Architecture

All templates now live under `infra/charts/controller/agent-templates/` with CLI-specific subdirectories:

**Docs Tasks (Multi-CLI Support)**

- **Prompts**: Rendered from `docs/{cli}/prompt.md.hbs` into the ConfigMap
- **Settings**: `docs/{cli}/settings.json.hbs` controls model, permissions, tools
- **Container Script**: `docs/{cli}/container.sh.hbs` handles Git workflow and CLI execution

**Code Tasks (multi-CLI)**

- **Claude**: `code/claude/**`
  - Settings: `code/claude/settings.json.hbs`
  - Container: `code/claude/container.sh.hbs`
- **Codex**: `code/codex/**`
  - Agents memory: `code/codex/agents.md.hbs`
  - Config: `code/codex/config.toml.hbs`
  - Container scripts: `code/codex/container*.sh.hbs`
- **Factory**: `code/factory/**`
  - Agents memory: `code/factory/agents*.md.hbs`
  - Config: `code/factory/factory-cli-config.json.hbs`
  - Container scripts: `code/factory/container*.sh.hbs`
- **Shared assets**: `code/mcp.json.hbs`, `code/coding-guidelines.md.hbs`, and `code/github-guidelines.md.hbs`

**Play Workflows**: Multi-agent orchestration with event-driven coordination

- **Workflow Template**: `play-workflow-template.yaml` defines the multi-phase workflow
- **Phase Coordination**: Each phase triggers the next phase automatically
- **Agent Handoffs**: Seamless transitions between implementation → QA → testing phases

### How to Customize

#### 1. Changing Agent Settings

Edit the settings template files for your chosen CLI:

```bash
# Edit settings for any CLI (claude, codex, cursor, factory, opencode, gemini, dexter)
vim infra/charts/controller/agent-templates/code/{cli}/settings.json.hbs
vim infra/charts/controller/agent-templates/docs/{cli}/settings.json.hbs
```

Settings control:
- Model selection (CLI-specific model identifiers)
- Tool permissions and access
- MCP tool configuration

Refer to your CLI's documentation for complete configuration options.

#### 2. Updating Prompts

**For docs tasks** (affects all documentation generation):

```bash
# Edit the docs prompt template for your CLI
vim infra/charts/controller/agent-templates/docs/{cli}/prompt.md.hbs

# Examples:
vim infra/charts/controller/agent-templates/docs/claude/prompt.md.hbs
vim infra/charts/controller/agent-templates/docs/cursor/prompt.md.hbs
```

**For code tasks** (affects specific task implementation):

```bash
# Edit task-specific files in your docs repository
vim {docs_project_directory}/.tasks/docs/task-{id}/prompt.md
vim {docs_project_directory}/.tasks/docs/task-{id}/task.md
vim {docs_project_directory}/.tasks/docs/task-{id}/acceptance-criteria.md
```

#### 3. Customizing Play Workflows

**For play workflows** (affects multi-agent orchestration):

```bash
# Edit the play workflow template
vim infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml
```

The play workflow template controls:
- Phase sequencing and dependencies
- Agent assignments for each phase
- Event triggers between phases
- Parameter passing between phases

#### 4. Deploying Template Changes

After editing any template files, redeploy the cto:

```bash
# Deploy template changes
helm upgrade cto infra/charts/controller -n cto

# Verify ConfigMap was updated (fullname = <release>-controller)
kubectl get configmap cto-controller-agent-templates -n cto -o yaml
```

**Important**: Template changes only affect new agent jobs. Running jobs continue with their original templates.

### Template Variables

Common variables available in templates:
- `{{task_id}}` - Task ID for code tasks
- `{{service_name}}` - Target service name
- `{{github_user}}` - GitHub username
- `{{repository_url}}` - Target repository URL
- `{{working_directory}}` - Working directory path
- `{{model}}` - Model name for the configured CLI
- `{{docs_repository_url}}` - Documentation repository URL

---

## **💡 Best Practices**

1. **Configure `cto-config.json` first** to set up your agents, models, tool profiles, and repository defaults
2. **Use `intake()` for new projects** to parse PRD, generate tasks, and create documentation in one operation
3. **Choose the right tool for the job**:
   - Use `addTool()` to extend agent capabilities with new MCP servers
   - Use `intake()` for new project setup from PRDs
   - Use `play()` for full-cycle development (implementation → QA → testing → review)
4. **Mix and match CLIs** - assign the best CLI to each agent based on task requirements
5. **Customize tool access** - use the `tools` configuration to control agent capabilities
6. **Use minimal MCP calls** - let configuration defaults handle most parameters
7. **Review GitHub PRs promptly** - agents provide detailed logs and explanations
8. **Update config file** when adding new agents, tools, or changing project structure

---

## **🛠️ Building from Source (Development)**

```bash
# Build from source
git clone https://github.com/5dlabs/cto.git
cd cto/controller

# Build MCP server
cargo build --release --bin cto-mcp

# Verify the build
./target/release/cto-mcp --help   # MCP server

# Install to your system (optional)
cp target/release/cto-mcp /usr/local/bin/
```

---

## **🆘 Support**

- Check GitHub PRs for detailed agent logs and explanations
- Review task structure in `.tasks/` directory
- Verify `cto-config.json` configuration and GitHub Apps authentication setup
- Ensure Argo Workflows are properly deployed and accessible

---

## **📄 License**

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). This means:

- You can use, modify, and distribute this software freely
- You can use it for commercial purposes
- ⚠️ If you deploy a modified version on a network server, you must provide source code access to users
- ⚠️ Any derivative works must also be licensed under AGPL-3.0

The AGPL license is specifically designed for server-side software to ensure that improvements to the codebase remain open source, even when deployed as a service. This protects the open source nature of the project while allowing commercial use.

**Source Code Access**: Since this platform operates as a network service, users interacting with it have the right to access the source code under AGPL-3.0. The complete source code is available at this repository, ensuring full compliance with AGPL-3.0's network clause.

For more details, see the [LICENSE](LICENSE) file.

---

## **🚀 Roadmap**

### **Project Management Integrations**
| Tool | Status |
|------|--------|
| **Linear** | ✅ Available (MVP) |
| **Jira** | 🔜 Planned |
| **Asana** | 🔜 Planned |
| **Notion** | 🔜 Planned |
| **Monday.com** | 🔜 Planned |

### **Coming Soon**
- Additional bare-metal provider integrations
- Enhanced self-healing remediation strategies
- Multi-cluster deployment support
- Advanced workflow templates

---

<div align="center">

### **🌟 Join the AI Development Revolution**

| | | | |
|:---:|:---:|:---:|:---:|
| [**⭐ Star**](https://github.com/5dlabs/cto)<br/>Support project | [**🍴 Fork**](https://github.com/5dlabs/cto/fork)<br/>Build with us | [**💬 Discord**](https://discord.gg/A6yydvjZKY)<br/>Join community | [**🐦 X**](https://x.com/5dlabs)<br/>Get updates |
| [**📺 YouTube**](https://www.youtube.com/@5DLabs)<br/>Watch tutorials | [**📖 Docs**](https://docs.5dlabs.com)<br/>Learn more | [**🐛 Issues**](https://github.com/5dlabs/cto/issues)<br/>Report bugs | [**💡 Discuss**](https://github.com/orgs/5dlabs/discussions)<br/>Share ideas |

**Built with ❤️ and 🤖 by the 5D Labs Team**

---

*The platform runs on Kubernetes and automatically manages multi-CLI agent deployments, workspace isolation, and GitHub integration. All you need to do is call the MCP tools and review the resulting PRs.*

</div>

