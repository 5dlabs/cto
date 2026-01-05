# CTO Platform SaaS Architecture

## Overview

The CTO Platform operates as a managed service where 5D Labs handles all the complexity of AI agent orchestration, infrastructure management, and integrations. Customers deploy only a lightweight gateway to their cluster, expressing intent via CRDs while all execution happens in the 5D Labs control plane.

---

## Architecture Diagram: Managed vs Client-Side

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
│  │                              INTEGRATIONS (managed)                                          │   │
│  │                                                                                              │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │   │
│  │   │   GitHub    │  │   Linear    │  │   Slack     │  │  Datadog    │  │  PagerDuty  │      │   │
│  │   │   (Apps)    │  │   (PM)      │  │  (Notifs)   │  │  (Metrics)  │  │  (Alerts)   │      │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                                     │
└──────────────────────────────────────────────────────────────────────────────────────────────┬──────┘
                                                                                               │
                                            Secure Tunnel (Cloudflare / WireGuard / gRPC)      │
                                                                                               │
┌──────────────────────────────────────────────────────────────────────────────────────────────┴──────┐
│                                                                                                     │
│                                    CUSTOMER SIDE (Thin Client)                                      │
│                                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                              Customer Kubernetes Cluster                                     │   │
│  │                                                                                              │   │
│  │   What customer deploys (ONE TIME):                                                         │   │
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
| **GitHub Access** | Token generation, API calls | GitHub App installation |
| **MCP Tools** | All tool servers | Tool-specific credentials |
| **Secrets Vault** | OpenBao infrastructure | Secret values |
| **Observability** | Logs, metrics, dashboards | Optional SIEM integration |
| **Infrastructure** | Provisioning, scaling | Cloud account credentials |
| **CRD Definitions** | Schema, validation | Nothing (auto-synced) |
| **Updates** | Automatic (SaaS) | Gateway pulls templates |

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

## Customer Journey

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              CUSTOMER JOURNEY                                            │
│                                                                                          │
│  Day 0: Sign Up                                                                         │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  1. Create account (email or GitHub OAuth)                                        │  │
│  │  2. Choose plan (Starter, Team, Enterprise)                                       │  │
│  │  3. Get tenant_id and api_token                                                   │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Day 0: Connect GitHub                                                                  │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  1. Install GitHub App on your org                                                │  │
│  │  2. Select repositories to grant access                                           │  │
│  │  3. Credentials automatically stored in your vault                                │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Day 0: Connect AI Provider                                                             │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  Option A: Use 5D Labs API keys (billed through us)                               │  │
│  │  Option B: Enter your own API keys (BYOK)                                         │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Day 0: Deploy Gateway (5 minutes)                                                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  helm install cto-gateway 5dlabs/cto-gateway \                                    │  │
│  │    --set tenantId=acme --set apiToken=ctop_xxx                                    │  │
│  │                                                                                    │  │
│  │  Gateway connects, CRD is auto-installed, ready to use!                           │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                             │                                            │
│                                             ▼                                            │
│  Day 1+: Use the Platform                                                               │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  # Create a CodeRun                                                               │  │
│  │  kubectl apply -f - <<EOF                                                         │  │
│  │  apiVersion: agents.platform/v1                                                   │  │
│  │  kind: CodeRun                                                                    │  │
│  │  metadata:                                                                        │  │
│  │    name: add-auth                                                                 │  │
│  │  spec:                                                                            │  │
│  │    service: my-api                                                                │  │
│  │    repositoryUrl: https://github.com/acme/my-api                                  │  │
│  │    taskId: 1                                                                      │  │
│  │    model: claude-sonnet-4-20250514                                                │  │
│  │  EOF                                                                              │  │
│  │                                                                                    │  │
│  │  # Watch progress                                                                 │  │
│  │  kubectl get coderuns -w                                                          │  │
│  │                                                                                    │  │
│  │  # Check status                                                                   │  │
│  │  kubectl get coderun add-auth -o yaml                                             │  │
│  │  # status:                                                                        │  │
│  │  #   phase: Succeeded                                                             │  │
│  │  #   pullRequestUrl: https://github.com/acme/my-api/pull/42                       │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```
