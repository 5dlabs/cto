# CTO Platform SaaS Architecture

## Overview

The CTO Platform operates as a **fully managed service** where 5D Labs handles all complexity: AI agent orchestration, infrastructure, Kubernetes, and integrations. Customers interact entirely through the web portal and API—no cluster required.

**Key principle: Zero infrastructure for customers.** They connect their GitHub, describe what they want built, and we handle everything else.

---

## Deployment Models

| Model | Customer Deploys | Best For |
|-------|------------------|----------|
| **Fully Managed** (default) | Nothing | 99% of customers - zero friction |
| **Gateway Mode** | Lightweight gateway pod | Customers who want CRD-based workflows |
| **Enterprise On-Prem** | Full platform | Regulated industries, air-gapped |

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
│  │                             INFRASTRUCTURE PROVISIONING                                      │   │
│  │                                                                                              │   │
│  │   Bare Metal (via Latitude.sh)         │    Cloud Providers                                 │   │
│  │   ┌────────────────────────────────┐   │   ┌────────────────────────────────────────────┐  │   │
│  │   │  • GPU servers for training    │   │   │  • AWS: EKS, EC2, Lambda, S3              │  │   │
│  │   │  • High-memory nodes           │   │   │  • GCP: GKE, Compute, Cloud Run           │  │   │
│  │   │  • Dedicated agent execution   │   │   │  • Azure: AKS, VMs, Functions             │  │   │
│  │   │  • Customer-dedicated hardware │   │   │  • DigitalOcean, Vultr, Hetzner           │  │   │
│  │   └────────────────────────────────┘   │   └────────────────────────────────────────────┘  │   │
│  │                                         │                                                   │   │
│  │   5D Labs provisions on behalf of customer OR customer links their own accounts           │   │
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
| **Infrastructure** | Provisioning, scaling | Cloud account credentials (optional) |
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

## Infrastructure Provider Integration

### How It Works

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          INFRASTRUCTURE PROVIDER FLOW                                    │
│                                                                                          │
│  Customer Onboarding:                                                                   │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   Step 1: "Connect your infrastructure"                                           │  │
│  │                                                                                    │  │
│  │   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                   │  │
│  │   │ ○ Use 5D Labs   │  │ ○ Bring Your    │  │ ○ Hybrid        │                   │  │
│  │   │   Infrastructure │  │   Own Cloud     │  │   (Both)        │                   │  │
│  │   └─────────────────┘  └─────────────────┘  └─────────────────┘                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option A: 5D Labs Infrastructure (simplest)                                            │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   • Agents run on 5D Labs managed bare metal (Latitude.sh)                        │  │
│  │   • No cloud credentials needed from customer                                     │  │
│  │   • Pay per usage (compute time billed)                                           │  │
│  │   • Best for: Getting started quickly, predictable costs                          │  │
│  │                                                                                    │  │
│  │   Infrastructure is provisioned:                                                  │  │
│  │     5D Labs Latitude Account → Bare metal servers → Agent execution pool          │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option B: Customer's Cloud (BYOC - Bring Your Own Cloud)                               │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   Customer provides cloud credentials:                                            │  │
│  │                                                                                    │  │
│  │   AWS:                                                                            │  │
│  │     • IAM Role ARN (cross-account assume role)                                    │  │
│  │     • Permissions: EC2, EKS, S3, Secrets Manager                                  │  │
│  │                                                                                    │  │
│  │   GCP:                                                                            │  │
│  │     • Service Account JSON key                                                    │  │
│  │     • Or: Workload Identity Federation                                            │  │
│  │     • Permissions: Compute, GKE, Storage, Secret Manager                          │  │
│  │                                                                                    │  │
│  │   Azure:                                                                          │  │
│  │     • Service Principal (client ID + secret)                                      │  │
│  │     • Or: Managed Identity                                                        │  │
│  │     • Permissions: AKS, VMs, Storage, Key Vault                                   │  │
│  │                                                                                    │  │
│  │   5D Labs provisions infrastructure IN customer's account:                        │  │
│  │     Customer AWS Account → EKS cluster → Agent execution (customer pays AWS)      │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option C: Customer's Bare Metal (Latitude.sh linked account)                           │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   Customer has their own Latitude.sh account:                                     │  │
│  │                                                                                    │  │
│  │   • Provide Latitude API key                                                      │  │
│  │   • 5D Labs provisions servers in customer's account                              │  │
│  │   • Customer pays Latitude directly                                               │  │
│  │   • Full control over hardware specs                                              │  │
│  │                                                                                    │  │
│  │   Benefits:                                                                       │  │
│  │     • GPU access for ML workloads                                                 │  │
│  │     • Predictable pricing (no cloud markup)                                       │  │
│  │     • Data sovereignty (choose regions)                                           │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### Infrastructure MCP Tools Available

| Tool | What It Does | Credentials Needed |
|------|--------------|-------------------|
| `mcp_latitudesh_*` | Provision bare metal servers | Latitude API key |
| `mcp_kubernetes_*` | Manage K8s resources | Kubeconfig / ServiceAccount |
| `mcp_terraform_*` | Infrastructure as Code | Provider-specific |
| `mcp_argocd_*` | GitOps deployments | ArgoCD token |
| `mcp_docker_*` | Container management | Docker socket access |
| `mcp_cloudflare_*` | DNS, CDN, Workers | Cloudflare API token |
| `mcp_aws_*` (future) | EC2, EKS, S3, etc. | IAM credentials |
| `mcp_gcp_*` (future) | GCE, GKE, GCS, etc. | Service account |

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
│  │  • Customer's cloud (data never leaves their account)                             │  │
│  │  • On-premises (air-gapped option)                                                │  │
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

## Alternative: Gateway Mode (For K8s-Native Workflows)

For teams that prefer Kubernetes-native workflows with CRDs:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         GATEWAY MODE (Optional)                                          │
│                                                                                          │
│  For teams that want:                                                                   │
│  • CRD-based declarative workflows                                                      │
│  • GitOps integration (ArgoCD, Flux)                                                    │
│  • kubectl-based operations                                                             │
│                                                                                          │
│  Setup (5 minutes):                                                                     │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  helm install cto-gateway 5dlabs/cto-gateway \                                    │  │
│  │    --set tenantId=acme --set apiToken=ctop_xxx                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Usage:                                                                                 │
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
│  Gateway syncs CRD status with 5D Labs control plane - execution still happens          │
│  on 5D Labs infrastructure.                                                             │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```
