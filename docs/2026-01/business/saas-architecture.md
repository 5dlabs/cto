# CTO Platform SaaS Architecture

## Overview

The CTO Platform operates as a **fully managed service** where 5D Labs handles all complexity: AI agent orchestration, infrastructure, Kubernetes, and integrations. Customers interact entirely through the web portal and API—no cluster required.

**Key principle: Zero infrastructure for customers.** They connect their GitHub, describe what they want built, and we handle everything else.

---

## Core Value Propositions

### 1. Fully Managed Infrastructure on Bare Metal

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                     VALUE PROP #1: FULLY MANAGED INFRASTRUCTURE                          │
│                                                                                          │
│  What customers get:                                                                    │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  • Bare metal servers (Latitude.sh) - no cloud markup                             │  │
│  │  • Full engineering team maintaining infrastructure 24/7                          │  │
│  │  • Zero Kubernetes knowledge required                                             │  │
│  │  • No DevOps hiring needed                                                        │  │
│  │  • Auto-scaling, auto-healing, monitoring included                                │  │
│  │                                                                                    │  │
│  │  Customer effort: Connect GitHub → Describe task → Review PR                      │  │
│  │  Our effort: Everything else                                                      │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Why bare metal matters:                                                                │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Cloud VMs          vs      Bare Metal                                            │  │
│  │  ──────────────────────────────────────────────────────                          │  │
│  │  Noisy neighbors            Dedicated resources                                   │  │
│  │  Variable performance       Consistent performance                                │  │
│  │  High cost (cloud markup)   Lower cost (direct hardware)                         │  │
│  │  Shared infrastructure      Isolated execution                                    │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 2. Always Current with AI Tooling Evolution

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                     VALUE PROP #2: STAYING AHEAD OF THE CURVE                            │
│                                                                                          │
│  The AI tooling landscape moves fast:                                                   │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  • New models every month (Claude, GPT, Gemini, open-source)                      │  │
│  │  • New CLIs and agents constantly emerging                                        │  │
│  │  • Best practices evolving weekly                                                 │  │
│  │  • MCP ecosystem expanding rapidly                                                │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Our research engine:                                                                   │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     │  │
│  │  │  Research   │ ──▶ │  Evaluate   │ ──▶ │  Integrate  │ ──▶ │   Deploy    │     │  │
│  │  │  (Twitter,  │     │  (benchmark │     │  (add to    │     │  (all cust- │     │  │
│  │  │   papers,   │     │   new tools │     │   platform) │     │   omers get │     │  │
│  │  │   releases) │     │   & models) │     │             │     │   updates)  │     │  │
│  │  └─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘     │  │
│  │                                                                                    │  │
│  │  Continuous cycle - we track so you don't have to                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  What this means for customers:                                                         │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  ✓ New models added within days of release                                        │  │
│  │  ✓ Agent prompts continuously optimized                                           │  │
│  │  ✓ Best-in-class MCP tools always available                                       │  │
│  │  ✓ No internal AI/ML team needed to stay current                                  │  │
│  │  ✓ Benefit from learnings across all customers                                    │  │
│  │                                                                                    │  │
│  │  "Subscribe to stay ahead" - not "build it yourself and fall behind"             │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Business Model: Freemium (Proprietary)

The CTO Platform is **proprietary software** with a generous free tier. We do not open-source the platform.

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            WHY NOT OPEN SOURCE?                                          │
│                                                                                          │
│  Our value is staying current. Open-sourcing would:                                     │
│                                                                                          │
│  ✗ Allow competitors to fork and compete on our R&D investment                          │
│  ✗ Dilute "always current" as our differentiator                                        │
│  ✗ Create support burden for community contributions                                    │
│  ✗ Slow velocity (external PR reviews, community coordination)                          │
│                                                                                          │
│  Instead, we offer:                                                                     │
│                                                                                          │
│  ✓ Generous free tier (experience the platform, no cost)                                │
│  ✓ Source code access for Enterprise customers (security review under NDA)              │
│  ✓ Full control = fastest iteration on AI tooling changes                               │
│  ✓ All customers benefit from continuous improvements                                   │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Deployment Models by Tier

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              DEPLOYMENT BY TIER                                          │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   FREE / TEAM / GROWTH TIERS                                                      │  │
│  │   ══════════════════════════                                                      │  │
│  │                                                                                    │  │
│  │   Deployment:     FULLY MANAGED (100%)                                            │  │
│  │   Customer does:  Nothing with infrastructure                                     │  │
│  │   We handle:      Everything - Kubernetes, bare metal, scaling, monitoring        │  │
│  │                                                                                    │  │
│  │   Customer experience:                                                            │  │
│  │   ┌────────────────────────────────────────────────────────────────────────────┐  │  │
│  │   │  Sign up → Connect GitHub → Describe task → Review PR → Done              │  │  │
│  │   │                                                                            │  │  │
│  │   │  No servers. No Kubernetes. No DevOps. No infrastructure decisions.       │  │  │
│  │   └────────────────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   ENTERPRISE TIER (Custom pricing)                                                │  │
│  │   ═════════════════════════════════                                               │  │
│  │                                                                                    │  │
│  │   Options available:                                                              │  │
│  │                                                                                    │  │
│  │   Option A: Fully Managed (same as other tiers)                                   │  │
│  │   └─ Best for: Enterprises who want zero ops                                      │  │
│  │                                                                                    │  │
│  │   Option B: Customer-Managed Infrastructure                                       │  │
│  │   └─ Platform deployed in customer's cloud/data center                            │  │
│  │   └─ Customer controls infrastructure, security, compliance                       │  │
│  │   └─ Best for: Regulated industries, air-gapped, data sovereignty                 │  │
│  │                                                                                    │  │
│  │   Option C: Hybrid                                                                │  │
│  │   └─ Control plane managed by 5D Labs                                             │  │
│  │   └─ Agent execution in customer's infrastructure                                 │  │
│  │   └─ Best for: "Our code never leaves our network" requirements                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

| Tier | Infrastructure | Customer Kubernetes Knowledge | Best For |
|------|---------------|------------------------------|----------|
| **Free** | Fully managed | None required | Individual devs, evaluation |
| **Team** | Fully managed | None required | Small teams getting started |
| **Growth** | Fully managed | None required | Scaling teams, production use |
| **Enterprise** | Choice (managed OR customer) | Optional (if customer-managed) | Regulated, compliance, control |

---

## Architecture Diagram: Fully Managed (Default)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                     │
│                              5D LABS MANAGED (SaaS Control Plane)                                   │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                                   CONTROL PLANE                                              │   │
│  │                                                                                              │   │
│  │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │   │
│  │   │    API       │  │  Controller  │  │   Healer     │  │  PM Server   │  │   Portal    │  │   │
│  │   │   Gateway    │  │  (per-tenant │  │  (CI fixes,  │  │  (Linear,    │  │   (Web UI)  │  │   │
│  │   │              │  │   or shared) │  │  auto-heal)  │  │   GitHub)    │  │             │  │   │
│  │   └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘  │   │
│  │                                                                                              │   │
│  │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐   │   │
│  │   │  MCP Tools   │  │  OpenMemory  │  │   Template   │  │      Secrets Vault           │   │   │
│  │   │   Server     │  │  (agent mem) │  │   Registry   │  │   (OpenBao per-tenant)       │   │   │
│  │   └──────────────┘  └──────────────┘  └──────────────┘  └──────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                                 AGENT EXECUTION POOL                                         │   │
│  │                                                                                              │   │
│  │   ┌────────────────────────────────────────────────────────────────────────────────────┐    │   │
│  │   │                          Agent Pods (ephemeral)                                     │    │   │
│  │   │                                                                                     │    │   │
│  │   │   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │    │   │
│  │   │   │ Claude  │  │ Codex   │  │ Cursor  │  │ Gemini  │  │ Factory │  │OpenCode │    │    │   │
│  │   │   │  Pod    │  │  Pod    │  │  Pod    │  │  Pod    │  │  Pod    │  │  Pod    │    │    │   │
│  │   │   └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │    │   │
│  │   │                                                                                     │    │   │
│  │   │   Each pod has: CLI + Workspace (git clone) + MCP sidecar + tenant credentials    │    │   │
│  │   └────────────────────────────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                        INFRASTRUCTURE (Bare Metal Only)                                      │   │
│  │                                                                                              │   │
│  │   5D Labs Managed Infrastructure (via Latitude.sh)                                          │   │
│  │   ┌───────────────────────────────────────────────────────────────────────────────────┐    │   │
│  │   │  • GPU servers for ML workloads                                                    │    │   │
│  │   │  • High-memory nodes for large codebases                                          │    │   │
│  │   │  • Dedicated agent execution pool                                                 │    │   │
│  │   │  • Enterprise option: Customer-dedicated hardware                                 │    │   │
│  │   └───────────────────────────────────────────────────────────────────────────────────┘    │   │
│  │                                                                                              │   │
│  │   Note: 5D Labs does NOT provision cloud resources (AWS/GCP/Azure).                        │   │
│  │   Enterprise customers who need cloud deployment manage their own infrastructure.           │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                              INTEGRATIONS (shared public apps)                               │   │
│  │                                                                                              │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │   │
│  │   │   GitHub    │  │   Linear    │  │   Slack     │  │  Datadog    │  │  PagerDuty  │      │   │
│  │   │  (Public    │  │  (Public    │  │  (Public    │  │  (Public    │  │  (Public    │      │   │
│  │   │   App)      │  │   OAuth)    │  │   OAuth)    │  │   OAuth)    │  │   OAuth)    │      │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │   │
│  │                                                                                              │   │
│  │   Single 5D Labs app installed by all tenants → per-tenant tokens in isolated vaults        │   │
│  │   Enterprise option: Bring-your-own-app for compliance requirements                         │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
└──────────────────────────────────────────────────────────────────────────────────────────────┬──────┘
                                                                                               │
                                            Secure Tunnel (Cloudflare / WireGuard / gRPC)      │
                                                                                               │
┌──────────────────────────────────────────────────────────────────────────────────────────────┴──────┐
│                                                                                                     │
│                          CUSTOMER SIDE: NOTHING REQUIRED (Fully Managed)                            │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                         DEFAULT: No infrastructure required!                                 │   │
│  │                                                                                              │   │
│  │   • Customer interacts via Web Portal, REST API, or CLI                                    │   │
│  │   • All execution happens on 5D Labs managed infrastructure                                │   │
│  │   • Zero Kubernetes knowledge required                                                     │   │
│  │                                                                                              │   │
│  │   ─────────────────────────────────────────────────────────────────────────────────────    │   │
│  │                                                                                              │   │
│  │   OPTIONAL: Gateway Mode (for K8s-native teams who want CRD workflows)                                                         │   │
│  │   ┌────────────────────────────────────────────────────────────────────────────────────┐    │   │
│  │   │                                                                                     │    │   │
│  │   │   helm install cto-gateway 5dlabs/cto-gateway \                                    │    │   │
│  │   │     --set tenantId=acme \                                                          │    │   │
│  │   │     --set apiToken=ctop_xxxxxxxxxxxxx                                              │    │   │
│  │   │                                                                                     │    │   │
│  │   └────────────────────────────────────────────────────────────────────────────────────┘    │   │
│  │                                                                                              │   │
│  │   What runs in customer cluster:                                                            │   │
│  │   ┌────────────────────────────────────────────────────────────────────────────────────┐    │   │
│  │   │                                                                                     │    │   │
│  │   │   ┌─────────────────────────────────────────────────────────────────────────────┐  │    │   │
│  │   │   │                        CTO Gateway (single pod)                              │  │    │   │
│  │   │   │                                                                              │  │    │   │
│  │   │   │  • Watches CodeRun CRDs                                                     │  │    │   │
│  │   │   │  • Forwards to SaaS control plane                                           │  │    │   │
│  │   │   │  • Syncs status back to local CRDs                                          │  │    │   │
│  │   │   │  • Pulls templates (CRD definitions, configs)                               │  │    │   │
│  │   │   │  • Health checks / heartbeat                                                │  │    │   │
│  │   │   │                                                                              │  │    │   │
│  │   │   │  Resources: ~50MB memory, ~10m CPU                                          │  │    │   │
│  │   │   └─────────────────────────────────────────────────────────────────────────────┘  │    │   │
│  │   │                                                                                     │    │   │
│  │   │   ┌─────────────────────────────────────────────────────────────────────────────┐  │    │   │
│  │   │   │                        CodeRun CRD (declarative intent)                      │  │    │   │
│  │   │   │                                                                              │  │    │   │
│  │   │   │  apiVersion: agents.platform/v1                                             │  │    │   │
│  │   │   │  kind: CodeRun                                                              │  │    │   │
│  │   │   │  spec:                                                                       │  │    │   │
│  │   │   │    service: my-api                                                          │  │    │   │
│  │   │   │    repositoryUrl: https://github.com/acme/my-api                           │  │    │   │
│  │   │   │    taskId: 3                                                                │  │    │   │
│  │   │   │    model: claude-sonnet-4-20250514                                          │  │    │   │
│  │   │   │  status:                       ← Updated by gateway                         │  │    │   │
│  │   │   │    phase: Succeeded                                                         │  │    │   │
│  │   │   │    pullRequestUrl: https://github.com/acme/my-api/pull/42                  │  │    │   │
│  │   │   └─────────────────────────────────────────────────────────────────────────────┘  │    │   │
│  │   └────────────────────────────────────────────────────────────────────────────────────┘    │   │
│  │                                                                                              │   │
│  │   Customer's existing workloads (untouched):                                                │   │
│  │   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐                               │   │
│  │   │  App A    │  │  App B    │  │ Database  │  │  Cache    │                               │   │
│  │   └───────────┘  └───────────┘  └───────────┘  └───────────┘                               │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Responsibility Matrix

| Component | 5D Labs Manages | Customer Provides |
|-----------|-----------------|-------------------|
| **Gateway** | Binary updates, template sync | Cluster to run it, API token |
| **Controller** | All orchestration logic | Nothing |
| **Agent Execution** | Pods, images, scaling | Nothing |
| **AI API Keys** | Optional (managed keys) | Own keys (BYOK option) |
| **GitHub Access** | Public GitHub App, token management | Click "Install" button |
| **Linear Access** | Public Linear OAuth App | Click "Connect" button |
| **MCP Tools** | All tool servers | Nothing (uses shared apps) |
| **Secrets Vault** | OpenBao infrastructure, per-tenant isolation | Nothing |
| **Observability** | Logs, metrics, dashboards | Optional SIEM integration |
| **Infrastructure** | Bare metal provisioning, scaling | Nothing (or their own K8s for Enterprise) |
| **CRD Definitions** | Schema, validation | Nothing (auto-synced) |
| **Updates** | Automatic (SaaS) | Gateway pulls templates |

---

## Shared Integration Model

The platform uses **public OAuth apps** for all third-party integrations. This is the industry-standard approach (used by Vercel, Netlify, CircleCI, etc.) and dramatically reduces onboarding friction.

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         SHARED PUBLIC APP ARCHITECTURE                                   │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                        5D Labs GitHub App (Public)                                 │  │
│  │                                                                                    │  │
│  │   App ID: 123456                                                                  │  │
│  │   Permissions: Contents (read/write), Pull Requests, Issues, Actions              │  │
│  │   Webhook URL: https://api.5dlabs.io/webhooks/github                              │  │
│  │                                                                                    │  │
│  │   ┌─────────────────────────────────────────────────────────────────────────────┐ │  │
│  │   │                        Per-Tenant Installations                              │ │  │
│  │   │                                                                              │ │  │
│  │   │   Acme Corp         │  Installation ID: 45678901                            │ │  │
│  │   │   (tenant: acme)    │  Repos: acme/api, acme/frontend, acme/docs            │ │  │
│  │   │                     │  Token: encrypted in acme's vault                     │ │  │
│  │   │                     │                                                        │ │  │
│  │   │   ─────────────────────────────────────────────────────────────────────────  │ │  │
│  │   │                     │                                                        │ │  │
│  │   │   BigCorp Inc       │  Installation ID: 45678902                            │ │  │
│  │   │   (tenant: bigcorp) │  Repos: bigcorp/monorepo                              │ │  │
│  │   │                     │  Token: encrypted in bigcorp's vault                  │ │  │
│  │   │                     │                                                        │ │  │
│  │   │   ─────────────────────────────────────────────────────────────────────────  │ │  │
│  │   │                     │                                                        │ │  │
│  │   │   Startup XYZ       │  Installation ID: 45678903                            │ │  │
│  │   │   (tenant: xyz)     │  Repos: startupxyz/* (all repos)                      │ │  │
│  │   │                     │  Token: encrypted in xyz's vault                      │ │  │
│  │   │                                                                              │ │  │
│  │   └─────────────────────────────────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  How it works:                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  1. Customer clicks "Connect GitHub" in 5D Labs portal                            │  │
│  │  2. Redirected to GitHub OAuth flow                                               │  │
│  │  3. Customer authorizes 5D Labs app for their org/repos                           │  │
│  │  4. GitHub returns installation_id                                                │  │
│  │  5. 5D Labs stores installation_id in customer's tenant vault                     │  │
│  │  6. When agents need GitHub access:                                               │  │
│  │     → Fetch installation token using installation_id + app private key            │  │
│  │     → Token is scoped ONLY to that customer's authorized repos                    │  │
│  │     → Token expires after 1 hour (auto-refreshed)                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Security properties:                                                                   │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  ✓ Tenant isolation: Installation tokens cannot access other tenants' repos      │  │
│  │  ✓ Least privilege: Customers choose exactly which repos to authorize            │  │
│  │  ✓ Revocable: Customer can uninstall app anytime from GitHub settings            │  │
│  │  ✓ Auditable: All API calls logged with installation_id                          │  │
│  │  ✓ Rate limits: Per-installation, not shared across tenants                      │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Same pattern for Linear, Slack, etc.:                                                  │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Linear:    Public OAuth app → workspace access token → stored per-tenant        │  │
│  │  Slack:     Public OAuth app → workspace bot token → stored per-tenant           │  │
│  │  Datadog:   API key provided by customer → stored per-tenant                     │  │
│  │  PagerDuty: OAuth or API key → stored per-tenant                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### Enterprise: Bring Your Own App (BYOA)

For customers with strict compliance requirements (SOC 2, FedRAMP, regulated industries), we offer dedicated app installations:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                     ENTERPRISE: BRING YOUR OWN APP (BYOA)                                │
│                                                                                          │
│  When required:                                                                         │
│  • Regulated industries (finance, healthcare, government)                               │
│  • Compliance frameworks that require dedicated integrations                            │
│  • Customers who need full audit trail in their own systems                             │
│  • Air-gapped or highly restricted environments                                         │
│                                                                                          │
│  How it works:                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  1. Customer creates their own GitHub App in their GitHub org                     │  │
│  │  2. Customer configures permissions per 5D Labs spec                              │  │
│  │  3. Customer provides:                                                            │  │
│  │     • App ID                                                                      │  │
│  │     • Private key (stored in their vault)                                         │  │
│  │     • Installation ID                                                             │  │
│  │  4. 5D Labs agents use customer's app credentials                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Benefits for enterprise:                                                               │
│  • Full audit trail in customer's GitHub                                               │
│  • Customer controls all permissions                                                   │
│  • Can revoke without affecting other tenants                                          │
│  • Meets compliance requirements for dedicated integrations                            │
│                                                                                          │
│  Pricing: Included in Enterprise tier, or +$500/month add-on for Growth tier           │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Infrastructure Model

### 5D Labs Managed (Default for Free/Team/Growth)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                    5D LABS MANAGED INFRASTRUCTURE (Bare Metal)                           │
│                                                                                          │
│  All Free, Team, and Growth customers run on 5D Labs managed bare metal:               │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   Customer experience:                                                            │  │
│  │   ─────────────────────                                                           │  │
│  │   • Sign up → Connect GitHub → Create task → Done                                 │  │
│  │   • Zero infrastructure decisions                                                 │  │
│  │   • Zero Kubernetes knowledge                                                     │  │
│  │   • Zero DevOps required                                                          │  │
│  │                                                                                    │  │
│  │   Behind the scenes (5D Labs manages):                                            │  │
│  │   ─────────────────────────────────────                                           │  │
│  │   • Bare metal servers via Latitude.sh                                            │  │
│  │   • Kubernetes cluster orchestration                                              │  │
│  │   • Agent pod scheduling and scaling                                              │  │
│  │   • Monitoring, logging, alerting                                                 │  │
│  │   • Security patching and updates                                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Why bare metal (not cloud VMs)?                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   ✓ Better performance (dedicated hardware, no noisy neighbors)                   │  │
│  │   ✓ Lower cost (no cloud markup)                                                  │  │
│  │   ✓ Predictable pricing                                                           │  │
│  │   ✓ Full control over hardware allocation                                         │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### Enterprise: Customer-Managed Infrastructure (Optional)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                    ENTERPRISE: CUSTOMER-MANAGED INFRASTRUCTURE                           │
│                                                                                          │
│  Enterprise customers who REQUIRE running in their own infrastructure can do so.        │
│  This is optional - Enterprise customers can also use 5D Labs managed infra.            │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   What we provide:                                                                │  │
│  │   ─────────────────                                                               │  │
│  │   • Helm chart for deployment                                                     │  │
│  │   • Container images                                                              │  │
│  │   • Documentation and runbooks                                                    │  │
│  │   • Support and updates                                                           │  │
│  │                                                                                    │  │
│  │   What customer provides:                                                         │  │
│  │   ───────────────────────                                                         │  │
│  │   • Kubernetes cluster (any provider: AWS EKS, GCP GKE, Azure AKS, on-prem)       │  │
│  │   • Networking and security                                                       │  │
│  │   • Compliance controls                                                           │  │
│  │   • Infrastructure operations                                                     │  │
│  │                                                                                    │  │
│  │   Important: 5D Labs does NOT provision cloud resources.                          │  │
│  │   We provide software; customer manages infrastructure.                           │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  When to use customer-managed:                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   • Regulated industries (finance, healthcare, government)                        │  │
│  │   • "Code never leaves our network" requirements                                  │  │
│  │   • Air-gapped environments                                                       │  │
│  │   • Specific compliance frameworks (FedRAMP, HIPAA, SOC 2)                        │  │
│  │   • Data sovereignty requirements                                                 │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### MCP Tools Available

| Tool | What It Does | Available To |
|------|--------------|--------------|
| `mcp_latitudesh_*` | Bare metal provisioning | 5D Labs internal |
| `mcp_kubernetes_*` | K8s resource management | All tiers |
| `mcp_argocd_*` | GitOps deployments | All tiers |
| `mcp_docker_*` | Container management | All tiers |
| `mcp_cloudflare_*` | DNS, CDN, tunnels | All tiers |
| `mcp_github_*` | Repository operations | All tiers |
| `mcp_linear_*` | Project management | All tiers |

---

## Data Flow: What Goes Where

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                   DATA FLOW                                              │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                        What stays in Customer's environment                        │  │
│  │                                                                                    │  │
│  │  • Source code repositories (GitHub/GitLab - accessed via API)                    │  │
│  │  • Production databases (agents can connect if credentials provided)              │  │
│  │  • Running applications                                                           │  │
│  │  • Customer's Kubernetes workloads                                                │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                        What flows through 5D Labs                                  │  │
│  │                                                                                    │  │
│  │  • CodeRun specs (task definitions, repo URLs, model selection)                   │  │
│  │  • Git clones during agent execution (ephemeral, deleted after run)               │  │
│  │  • AI API calls (prompts, responses)                                              │  │
│  │  • MCP tool invocations (GitHub API calls, K8s commands)                          │  │
│  │  • Logs and metrics (configurable retention)                                      │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                        What 5D Labs stores                                         │  │
│  │                                                                                    │  │
│  │  • Tenant configuration (settings, preferences)                                   │  │
│  │  • Credentials (encrypted in vault, per-tenant isolated)                          │  │
│  │  • CodeRun history (metadata, not full code)                                      │  │
│  │  • Agent memory (OpenMemory - optional, can be disabled)                          │  │
│  │  • Audit logs                                                                     │  │
│  │  • Billing/usage data                                                             │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                        Data residency options (Enterprise)                         │  │
│  │                                                                                    │  │
│  │  • US region (default)                                                            │  │
│  │  • EU region (GDPR compliance)                                                    │  │
│  │  • Customer-managed infrastructure (data never leaves their network)              │  │
│  │  • On-premises / air-gapped (for highly regulated environments)                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Customer Journey (Fully Managed - Default)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                    CUSTOMER JOURNEY - FULLY MANAGED (2 minutes to first run)            │
│                                                                                          │
│  Step 1: Sign Up (30 seconds)                                                           │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  1. Go to app.5dlabs.io                                                           │  │
│  │  2. Click "Sign up with GitHub"                                                   │  │
│  │  3. Done - account created, GitHub already connected!                             │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Step 2: Select Repositories (30 seconds)                                               │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  1. See list of your GitHub repos                                                 │  │
│  │  2. Click to enable repos for agent access                                        │  │
│  │  3. (Optional) Connect Linear workspace for project management                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Step 3: Create Your First Task (1 minute)                                              │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  In the portal:                                                                   │  │
│  │  ┌─────────────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  New Task                                                                    │  │  │
│  │  │  ─────────────────────────────────────────────────────────────────────────  │  │  │
│  │  │  Repository:  [acme/my-api           ▼]                                     │  │  │
│  │  │  Task:        [Add JWT authentication to the /api/users endpoint    ]       │  │  │
│  │  │  Agent:       [● Auto  ○ Rex (Rust)  ○ Blaze (React)  ○ Nova (Node)]       │  │  │
│  │  │                                                                              │  │  │
│  │  │                                            [Create Task & Start Agent]      │  │  │
│  │  └─────────────────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                                    │  │
│  │  Or via API:                                                                      │  │
│  │  curl -X POST https://api.5dlabs.io/v1/tasks \                                   │  │
│  │    -H "Authorization: Bearer $API_TOKEN" \                                       │  │
│  │    -d '{"repo": "acme/my-api", "prompt": "Add JWT auth..."}'                     │  │
│  │                                                                                    │  │
│  │  Or via CLI:                                                                      │  │
│  │  5d task create --repo acme/my-api "Add JWT authentication..."                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Step 4: Watch Progress, Review PR                                                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Live progress in portal:                                                         │  │
│  │  ┌─────────────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Task: Add JWT authentication                                                │  │  │
│  │  │  Status: ● Running (2m 34s)                                                  │  │  │
│  │  │                                                                              │  │  │
│  │  │  [====================                    ] 50%                              │  │  │
│  │  │                                                                              │  │  │
│  │  │  ✓ Cloned repository                                                        │  │  │
│  │  │  ✓ Analyzed codebase                                                        │  │  │
│  │  │  ● Implementing changes...                                                  │  │  │
│  │  │  ○ Running tests                                                            │  │  │
│  │  │  ○ Creating PR                                                              │  │  │
│  │  └─────────────────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                                    │  │
│  │  When complete:                                                                   │  │
│  │  → PR created: https://github.com/acme/my-api/pull/42                            │  │
│  │  → Review, merge, done!                                                          │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  No Kubernetes. No infrastructure. No CLI required. Just results.                       │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Enterprise Option: Gateway Mode (Customer Infrastructure)

**Enterprise tier only.** For organizations that require code execution within their own infrastructure:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                   GATEWAY MODE (Enterprise Only)                                         │
│                                                                                          │
│  When to use:                                                                           │
│  • Regulated industries requiring data sovereignty                                      │
│  • "Code never leaves our network" policies                                             │
│  • Air-gapped environments                                                              │
│  • Compliance requirements (FedRAMP, HIPAA, etc.)                                       │
│                                                                                          │
│  Setup (by customer's DevOps team):                                                     │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  helm install cto-gateway 5dlabs/cto-gateway \                                    │  │
│  │    --set tenantId=acme --set apiToken=ctop_xxx                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Usage (for K8s-native teams):                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  kubectl apply -f - <<EOF                                                         │  │
│  │  apiVersion: agents.platform/v1                                                   │  │
│  │  kind: CodeRun                                                                    │  │
│  │  metadata:                                                                        │  │
│  │    name: add-auth                                                                 │  │
│  │  spec:                                                                            │  │
│  │    repositoryUrl: https://github.com/acme/my-api                                  │  │
│  │    prompt: "Add JWT authentication to /api/users"                                 │  │
│  │  EOF                                                                              │  │
│  │                                                                                    │  │
│  │  kubectl get coderuns -w                                                          │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Important:                                                                             │
│  • This is NOT available on Free, Team, or Growth tiers                                 │
│  • Customer is responsible for infrastructure management                                │
│  • 5D Labs provides software updates, customer handles deployment                       │
│  • Premium pricing reflects additional support requirements                             │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```
