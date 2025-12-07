# CTO Platform: Distribution & Monetization Strategy

## Executive Summary

This document outlines the comprehensive strategy for distributing and monetizing the CTO Platform, balancing community adoption with revenue protection through a **Source Available** licensing model with a **Kind-only free tier**.

**Core Strategy:**
- **Source Available License** (FSL/BSL) - Code is visible but competitors can't clone the business
- **Free Tier on Kind Only** - Developers can try locally, production requires payment
- **Tiered Subscription** - From $79/mo (Starter) to Enterprise custom pricing
- **Web Portal** - Configuration-driven deployment with cost analysis
- **BYOK Secrets** - Customers bring their own API keys, stored in their infrastructure
- **Optional Hosted** - Premium managed offering at 3-4x self-hosted pricing

---

## Licensing Strategy

### Why Source Available (Not Open Source)

After analyzing the HashiCorp BSL transition, community reactions, and competitive landscape, we recommend a **Source Available** approach using the **Functional Source License (FSL)** or similar:

| Aspect | Open Source (Apache 2.0) | Source Available (FSL) | Proprietary |
|--------|--------------------------|------------------------|-------------|
| Code Visibility | ✅ Public | ✅ Public | ❌ Private |
| Commercial Use | ✅ Unrestricted | ✅ With restrictions | ✅ Licensed only |
| Competitor Can Fork | ⚠️ Yes (risk) | ❌ Non-compete clause | ❌ No access |
| Community Trust | ✅ High | ✅ Moderate | ⚠️ Lower |
| Enterprise Auditability | ✅ Yes | ✅ Yes | ⚠️ Requires NDA |

### Recommended License Grant

```
Functional Source License, Version 1.1

Grant of Rights

You are granted the right to use, copy, modify, and distribute this software
for any purpose, subject to the following conditions:

1. COMPETING USE RESTRICTION
   You may not use this software to provide a product or service that competes
   with the CTO Platform, including but not limited to:
   - Offering the software as a hosted/managed service
   - Bundling the software in a competing infrastructure platform
   - Reselling the software or access to it

2. CHANGE DATE
   On [Date + 4 years], this software becomes available under Apache 2.0.

3. ATTRIBUTION
   You must retain all copyright notices and include attribution to 5D Labs.
```

### What This Protects Against

| Threat | Protection |
|--------|------------|
| AWS/GCP launching "CTO Platform Service" | ✅ Explicitly prohibited |
| Competitor forking and selling | ✅ Non-compete clause |
| Consulting firm reselling | ✅ Requires partner agreement |
| Developer using for personal projects | ✅ Allowed |
| Enterprise deploying internally | ✅ Allowed |
| Security researchers auditing code | ✅ Allowed |

---

## Product Tiers & Pricing

### Tier Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CTO PLATFORM TIERS                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  FREE (Kind Only)              STARTER              TEAM                    │
│  ────────────────              ───────              ────                    │
│  $0/forever                    $79/month            $249/month              │
│                                $790/year            $2,490/year             │
│                                                                              │
│  • Kind clusters only          • 1 production node  • 3 production nodes   │
│  • 2 agents (Rex, Blaze)       • 5 agents           • 15 agents            │
│  • Manual install only         • Setup wizard       • Setup wizard         │
│  • No Guardian agent           • Basic Guardian     • Full Guardian        │
│  • No OTA updates              • OTA updates        • OTA updates          │
│  • Community support           • Email support      • Priority email       │
│  • "Powered by 5D Labs"        • No badge required  • No badge required    │
│                                                                              │
│  ───────────────────────────────────────────────────────────────────────────│
│                                                                              │
│  BUSINESS                      ENTERPRISE                                   │
│  ────────                      ──────────                                   │
│  $699/month                    Custom ($20K+/year)                          │
│  $6,990/year                                                                │
│                                                                              │
│  • 10 production nodes         • Unlimited nodes                            │
│  • Unlimited agents            • Unlimited agents                           │
│  • Setup wizard                • Setup wizard + white-glove                │
│  • Full Guardian + analytics   • Full Guardian + custom rules              │
│  • OTA updates + staging       • OTA updates + staged rollout              │
│  • Priority support (8hr SLA)  • Dedicated support (4hr SLA)               │
│  • Advanced analytics          • SSO/LDAP integration                      │
│  • Custom agent builder        • Air-gap bundle                            │
│                                • Audit logging                              │
│                                • SLA guarantees                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Free Tier: Kind-Only Constraint

The free tier is **technically constrained** to run only on Kind clusters:

```rust
// License validation includes cluster detection
pub enum ClusterType {
    Kind,           // ✅ Free tier allowed
    K3s,            // ❌ Requires paid license
    Talos,          // ❌ Requires paid license
    EKS,            // ❌ Requires paid license
    GKE,            // ❌ Requires paid license
    AKS,            // ❌ Requires paid license
    Kubeadm,        // ❌ Requires paid license
    OpenShift,      // ❌ Requires paid license
    RKE2,           // ❌ Requires paid license
    Unknown,        // ❌ Requires paid license
}

impl LicenseValidator {
    pub fn validate_cluster_type(&self, license: &License) -> Result<(), LicenseError> {
        let cluster_type = self.detect_cluster_type().await?;
        
        match (&license.tier, cluster_type) {
            // Free tier only works on Kind
            (Tier::Free, ClusterType::Kind) => Ok(()),
            (Tier::Free, other) => Err(LicenseError::ClusterTypeNotAllowed {
                tier: Tier::Free,
                detected: other,
                message: "Free tier is limited to Kind clusters. Upgrade to Starter for production use.",
            }),
            
            // Paid tiers work everywhere
            (_, _) => Ok(()),
        }
    }
    
    async fn detect_cluster_type(&self) -> ClusterType {
        // Detection methods:
        // 1. Check node labels (kind sets specific labels)
        // 2. Check API server certificate (Kind has distinctive CN)
        // 3. Check node provider ID prefix
        // 4. Check for Kind-specific ConfigMaps
        
        if self.has_kind_node_labels().await {
            return ClusterType::Kind;
        }
        
        if self.has_talos_node_labels().await {
            return ClusterType::Talos;
        }
        
        // ... other detection logic
        
        ClusterType::Unknown
    }
}
```

### Why Kind-Only Works

| Benefit | Explanation |
|---------|-------------|
| **Low barrier to try** | `kind create cluster && helm install cto` - 5 minutes |
| **Production impossible** | Kind isn't production-grade, natural upgrade path |
| **Developer-friendly** | Standard local dev workflow |
| **No support burden** | Kind issues are well-documented |
| **Clear value prop** | "Loved it locally? $79/mo for production" |

### Free Tier Limitations Detail

```yaml
# Free tier constraints (enforced by license validator)
free_tier:
  # Cluster constraints
  cluster:
    allowed_types: ["kind"]
    max_nodes: 3  # Kind default
    
  # Agent constraints
  agents:
    max_count: 2
    allowed_agents:
      - rex       # Backend agent
      - blaze     # Frontend agent
    restricted_agents:
      - cypher    # Security (paid)
      - guardian  # Self-healing (paid)
      - atlas     # Infrastructure (paid)
      
  # Feature constraints
  features:
    setup_wizard: false           # Manual install only
    guardian_agent: false         # No self-healing
    ota_updates: false            # Manual updates only
    service_marketplace: false    # No add-on services
    analytics_dashboard: false    # Basic metrics only
    project_management: false     # No Linear/Jira sync
    
  # Branding
  branding:
    badge_required: true          # "Powered by 5D Labs"
    badge_location: "footer"
    
  # Support
  support:
    level: "community"
    channels: ["github_issues", "discord"]
```

---

## Premium Add-Ons

### Add-On Pricing Matrix

| Add-On | Monthly | Description | Available From |
|--------|---------|-------------|----------------|
| **Agent Builder** | $49 | Visual agent creation, custom prompts, export/import | Starter |
| **Design System** | $29 | shadcn/ui library, theme editor, page templates | Starter |
| **Performance Analytics** | $39 | Cost attribution, usage forecasting, custom dashboards | Starter |
| **Project Management** | $19 | Linear/Jira full sync, automated issue creation | Starter |
| **Local Model Hosting** | $99 | Ollama integration, GPU scheduling, model management | Team |
| **Pro Bundle** | $99 | All add-ons (save $37) | Starter |

### Add-On Inclusion by Tier

| Add-On | Free | Starter | Team | Business | Enterprise |
|--------|------|---------|------|----------|------------|
| Agent Builder | ❌ | +$49 | +$49 | ✅ Included | ✅ Included |
| Design System | ❌ | +$29 | +$29 | +$29 | ✅ Included |
| Analytics | ❌ | +$39 | +$39 | ✅ Included | ✅ Included |
| Project Mgmt | ❌ | +$19 | ✅ Included | ✅ Included | ✅ Included |
| Local Models | ❌ | ❌ | +$99 | +$99 | ✅ Included |
| **Pro Bundle** | ❌ | $99 | $69 | $29 | ✅ Included |

---

## Web Portal Architecture

### Portal Overview

The web portal serves as the primary distribution and configuration interface:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       CTO PLATFORM PORTAL                                    │
│                       portal.5dlabs.io                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 1: DEPLOYMENT TARGET                                             │  │
│  │                                                                         │  │
│  │  How would you like to deploy?                                         │  │
│  │                                                                         │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │   🖥️        │  │   ☁️        │  │   🏢        │  │   🧪        │   │  │
│  │  │ Bare Metal  │  │   Cloud     │  │  On-Prem    │  │ Local Dev   │   │  │
│  │  │             │  │             │  │  (Helm)     │  │  (Kind)     │   │  │
│  │  │ Bootable    │  │ Terraform   │  │             │  │             │   │  │
│  │  │ ISO         │  │ to AWS/GCP  │  │ Existing    │  │ FREE TIER   │   │  │
│  │  │             │  │ /Azure      │  │ Kubernetes  │  │             │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  │       ⬆️                                                               │  │
│  │   SELECTED                                                             │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 2: CLUSTER SIZE                                                  │  │
│  │                                                                         │  │
│  │  How many nodes?                                                       │  │
│  │                                                                         │  │
│  │  [1]  [3]  [5]  [10]  [Custom: ___]                                   │  │
│  │   ⬆️                                                                    │  │
│  │                                                                         │  │
│  │  Recommended tier: STARTER ($79/mo)                                    │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 3: BACKEND SERVICES                                              │  │
│  │                                                                         │  │
│  │  Databases                         Message Queues                      │  │
│  │  ┌─────────────────────────┐      ┌─────────────────────────┐        │  │
│  │  │ [x] PostgreSQL (CNPG)   │      │ [x] NATS                │        │  │
│  │  │ [ ] MySQL               │      │ [ ] RabbitMQ            │        │  │
│  │  │ [ ] MongoDB             │      │ [ ] Apache Kafka        │        │  │
│  │  │ [ ] CockroachDB         │      │ [ ] Redis Streams       │        │  │
│  │  └─────────────────────────┘      └─────────────────────────┘        │  │
│  │                                                                         │  │
│  │  Caching                          Search & Analytics                   │  │
│  │  ┌─────────────────────────┐      ┌─────────────────────────┐        │  │
│  │  │ [x] Redis               │      │ [ ] OpenSearch          │        │  │
│  │  │ [ ] Dragonfly           │      │ [x] Meilisearch         │        │  │
│  │  │ [ ] KeyDB               │      │ [ ] ClickHouse          │        │  │
│  │  └─────────────────────────┘      └─────────────────────────┘        │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 4: AI AGENTS                                                     │  │
│  │                                                                         │  │
│  │  Select agents to deploy:                                              │  │
│  │                                                                         │  │
│  │  ┌──────────────────────────────────────────────────────────────────┐ │  │
│  │  │  [x] Rex      Backend development      CLI: [Claude Code ▼]      │ │  │
│  │  │  [x] Blaze    Frontend development     CLI: [Claude Code ▼]      │ │  │
│  │  │  [x] Tess     Testing & QA             CLI: [Claude Code ▼]      │ │  │
│  │  │  [ ] Cypher   Security analysis        CLI: [Claude Code ▼]      │ │  │
│  │  │  [ ] Atlas    Infrastructure           CLI: [Claude Code ▼]      │ │  │
│  │  │  [ ] Morgan   Code review              CLI: [Claude Code ▼]      │ │  │
│  │  └──────────────────────────────────────────────────────────────────┘ │  │
│  │                                                                         │  │
│  │  Default Model: [Claude Sonnet 4 ▼]                                    │  │
│  │                                                                         │  │
│  │  [ ] Enable local model hosting (Ollama) [+$99/mo]                    │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 5: INTEGRATIONS                                                  │  │
│  │                                                                         │  │
│  │  Version Control              Project Management                       │  │
│  │  ┌─────────────────────────┐  ┌─────────────────────────┐            │  │
│  │  │ [x] GitHub              │  │ [x] Linear              │            │  │
│  │  │ [ ] GitLab              │  │ [ ] Jira                │            │  │
│  │  │ [ ] Bitbucket           │  │ [ ] Asana               │            │  │
│  │  └─────────────────────────┘  └─────────────────────────┘            │  │
│  │                                                                         │  │
│  │  Observability                Communication                            │  │
│  │  ┌─────────────────────────┐  ┌─────────────────────────┐            │  │
│  │  │ [x] Grafana (included)  │  │ [ ] Slack               │            │  │
│  │  │ [ ] Datadog export      │  │ [ ] Discord             │            │  │
│  │  │ [ ] PagerDuty           │  │ [ ] Microsoft Teams     │            │  │
│  │  └─────────────────────────┘  └─────────────────────────┘            │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 6: SECRETS (BYOK - Bring Your Own Keys)                         │  │
│  │                                                                         │  │
│  │  Your API keys are stored in YOUR cluster's Vault instance.           │  │
│  │  We never see or store your keys.                                      │  │
│  │                                                                         │  │
│  │  AI Provider Keys                                                      │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Anthropic API Key:  [sk-ant-••••••••••••••••••••••••]  [Test] │  │  │
│  │  │  OpenAI API Key:     [sk-••••••••••••••••••••••••••••]  [Test] │  │  │
│  │  │  Google AI Key:      [Optional - not set]               [Add]  │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                         │  │
│  │  Integration Keys                                                      │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  GitHub Token:       [ghp_••••••••••••••••••••••••••]   [Test] │  │  │
│  │  │  Linear API Key:     [lin_api_••••••••••••••••••••••]   [Test] │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                         │  │
│  │  🔒 Keys are encrypted and stored directly in your cluster's Vault    │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  STEP 7: FRONTEND COMPONENTS (Premium Add-on: $29/mo)                 │  │
│  │                                                                         │  │
│  │  [ ] Enable Design System Add-on                                       │  │
│  │                                                                         │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Theme                                                           │  │  │
│  │  │  [◉] Dark    [ ] Light    [ ] System                            │  │  │
│  │  │                                                                   │  │  │
│  │  │  Component Library                                               │  │  │
│  │  │  [x] shadcn/ui base components                                  │  │  │
│  │  │  [x] Form components                                            │  │  │
│  │  │  [ ] Data visualization                                         │  │  │
│  │  │  [ ] Advanced layouts                                           │  │  │
│  │  │                                                                   │  │  │
│  │  │  Page Templates                                                  │  │  │
│  │  │  [x] Dashboard                                                  │  │  │
│  │  │  [x] Landing page                                               │  │  │
│  │  │  [x] Admin panel                                                │  │  │
│  │  │  [ ] E-commerce                                                 │  │  │
│  │  │  [ ] Blog/CMS                                                   │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  COST ANALYSIS                                                         │  │
│  │                                                                         │  │
│  │  Your Configuration                    vs. Cloud Equivalent            │  │
│  │  ─────────────────────                 ────────────────────            │  │
│  │                                                                         │  │
│  │  Platform License:     $79/month       EKS + RDS + ElastiCache:       │  │
│  │  Hardware (amortized): $70/month         Compute:      $432/month     │  │
│  │  Power & Cooling:      $70/month         Database:     $195/month     │  │
│  │  Design System Add-on: $29/month         Cache:        $98/month      │  │
│  │  ────────────────────────────────         Storage:      $50/month      │  │
│  │  TOTAL:               $248/month         Monitoring:   $150/month     │  │
│  │                                           ─────────────────────────    │  │
│  │                                           TOTAL:       $925/month     │  │
│  │                                                                         │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐  │  │
│  │  │  ████████████████████████░░░░░░░░░░  73% SAVINGS                │  │  │
│  │  │                                                                   │  │  │
│  │  │  Annual Savings: $8,124                                         │  │  │
│  │  │  3-Year Savings: $24,372                                        │  │  │
│  │  └─────────────────────────────────────────────────────────────────┘  │  │
│  │                                                                         │  │
│  │  💡 Your data never leaves your infrastructure                        │  │
│  │  💡 No per-GB egress fees                                             │  │
│  │  💡 Self-healing reduces ops burden                                   │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                                                                         │  │
│  │  [📥 Download ISO]    [☁️ Deploy to Cloud]    [📦 Get Helm Charts]   │  │
│  │                                                                         │  │
│  │           [🖥️ Want us to manage it? Try Hosted →]                     │  │
│  │                                                                         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Portal Technical Implementation

The portal generates a configuration manifest that can be:
1. Embedded in a custom ISO
2. Applied via Helm values
3. Used to provision Terraform infrastructure

```yaml
# Generated configuration manifest
apiVersion: cto.5dlabs.io/v1
kind: PlatformConfig
metadata:
  name: acme-corp-deployment
  annotations:
    cto.5dlabs.io/generated-at: "2025-12-07T15:30:00Z"
    cto.5dlabs.io/portal-version: "1.0.0"
spec:
  # License information
  license:
    key: "lic_abc123def456..."
    tier: starter
    organization: "Acme Corp"
    
  # Deployment target
  deployment:
    type: bare-metal  # bare-metal | cloud | on-prem | kind
    provider: null    # aws | gcp | azure | null for bare-metal
    nodes: 1
    
  # Backend services
  services:
    databases:
      postgresql:
        enabled: true
        operator: cnpg
        instances: 1
        storage: 50Gi
      mysql:
        enabled: false
      mongodb:
        enabled: false
        
    cache:
      redis:
        enabled: true
        mode: standalone
        memory: 1Gi
        
    messaging:
      nats:
        enabled: true
        jetstream: true
        
    search:
      meilisearch:
        enabled: true
        storage: 10Gi
        
  # AI agents
  agents:
    enabled:
      - name: rex
        cli: claude
        model: claude-sonnet-4-20250514
      - name: blaze
        cli: claude
        model: claude-sonnet-4-20250514
      - name: tess
        cli: claude
        model: claude-sonnet-4-20250514
    localModels:
      enabled: false
      
  # Integrations
  integrations:
    github:
      enabled: true
    linear:
      enabled: true
    slack:
      enabled: false
      
  # Secrets (BYOK - stored in customer's Vault)
  secrets:
    provider: vault
    byok: true
    # Keys are injected at install time, never stored in portal
    
  # Add-ons
  addons:
    designSystem:
      enabled: true
      theme: dark
      components:
        - shadcn-base
        - forms
      templates:
        - dashboard
        - landing
        - admin
    agentBuilder:
      enabled: false
    analytics:
      enabled: false
    projectManagement:
      enabled: true  # Included with Linear integration
      
  # Guardian agent settings
  guardian:
    enabled: true  # Based on tier
    mode: full     # basic | full | monitor-only
    autoRemediation:
      enabled: true
      scope:
        - certificate-renewal
        - log-cleanup
        - service-restart
      rateLimit:
        maxActionsPerHour: 3
        
  # Observability
  observability:
    prometheus:
      enabled: true
      retention: 15d
    grafana:
      enabled: true
    loki:
      enabled: true
      retention: 7d
```

---

## Hosted Solution

### Hosted Tier Pricing

For customers who want fully managed infrastructure:

| Tier | Self-Hosted | Hosted | Markup |
|------|-------------|--------|--------|
| Starter | $79/mo | $299/mo | 3.8x |
| Team | $249/mo | $799/mo | 3.2x |
| Business | $699/mo | $1,999/mo | 2.9x |
| Enterprise | Custom | Custom | ~2.5x |

### Hosted Economics

| Component | Our Cost | Customer Pays | Margin |
|-----------|----------|---------------|--------|
| **Starter Hosted** | | | |
| Cloud compute (1 node) | $50/mo | - | - |
| Storage | $20/mo | - | - |
| Bandwidth | $10/mo | - | - |
| Support allocation | $20/mo | - | - |
| **Total Cost** | $100/mo | $299/mo | **$199 (67%)** |
| | | | |
| **Team Hosted** | | | |
| Cloud compute (3 nodes) | $150/mo | - | - |
| Storage | $50/mo | - | - |
| Bandwidth | $30/mo | - | - |
| Support allocation | $40/mo | - | - |
| **Total Cost** | $270/mo | $799/mo | **$529 (66%)** |
| | | | |
| **Business Hosted** | | | |
| Cloud compute (10 nodes) | $500/mo | - | - |
| Storage | $150/mo | - | - |
| Bandwidth | $100/mo | - | - |
| Support allocation | $100/mo | - | - |
| **Total Cost** | $850/mo | $1,999/mo | **$1,149 (57%)** |

### Hosted Feature Additions

Hosted customers get additional benefits:

```yaml
hosted_features:
  # Infrastructure management
  infrastructure:
    - automatic_scaling         # Scale nodes based on demand
    - multi_az_deployment       # High availability across zones
    - managed_backups           # Daily automated backups
    - disaster_recovery         # Cross-region DR (Business+)
    
  # Enhanced support
  support:
    - proactive_monitoring      # We watch before you notice
    - incident_response         # We fix infrastructure issues
    - upgrade_management        # We handle all updates
    
  # Compliance (Enterprise)
  compliance:
    - soc2_environment          # SOC 2 compliant infrastructure
    - hipaa_environment         # HIPAA compliant (additional cost)
    - pci_environment           # PCI DSS compliant (additional cost)
```

---

## Distribution Channels

### Channel Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       DISTRIBUTION CHANNELS                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  PRIMARY (Direct)                                                           │
│  ─────────────────                                                          │
│                                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Portal    │  │  CLI Tool   │  │   GitHub    │  │   Docker    │       │
│  │             │  │             │  │  Releases   │  │    Hub      │       │
│  │ portal.     │  │ cto-cli    │  │             │  │             │       │
│  │ 5dlabs.io   │  │ install    │  │ ISO + Helm  │  │  Container  │       │
│  │             │  │             │  │  artifacts  │  │   images    │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                              │
│  SECONDARY (Partners)                                                       │
│  ─────────────────────                                                      │
│                                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                        │
│  │  Hardware   │  │    MSP      │  │  Cloud      │                        │
│  │  Bundles    │  │  Partners   │  │ Marketplace │                        │
│  │             │  │             │  │             │                        │
│  │ Dell/HPE    │  │ Consulting  │  │ AWS/GCP     │                        │
│  │ pre-install │  │ firms       │  │ marketplace │                        │
│  └─────────────┘  └─────────────┘  └─────────────┘                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Distribution Artifacts

| Artifact | Target | License Required | Size |
|----------|--------|------------------|------|
| **ISO (Connected)** | Bare metal (internet) | Yes* | ~2 GB |
| **ISO (Air-gapped)** | Bare metal (isolated) | Enterprise | ~30 GB |
| **Helm Charts** | Existing Kubernetes | Yes* | ~50 MB |
| **Terraform Modules** | Cloud deployment | Yes* | ~10 MB |
| **Container Images** | Any container runtime | Yes* | ~5 GB total |
| **Kind Bootstrap** | Local development | Free tier OK | ~500 MB |

*Free tier works only on Kind clusters

### CLI Distribution

```bash
# Installation
curl -fsSL https://get.5dlabs.io/cto | sh

# Or via Homebrew
brew install 5dlabs/tap/cto-cli

# Quick start (Kind - Free tier)
cto init --target kind
cto deploy

# Production (requires license)
cto init --target bare-metal --license-key lic_xxx
cto deploy

# Cloud deployment
cto init --target aws --region us-west-2 --license-key lic_xxx
cto deploy
```

---

## Technical Monetization Implementation

### License Validation System

```rust
//! License validation module
//! 
//! Multiple layers of validation to ensure license compliance
//! while maintaining good UX for legitimate customers.

use chrono::{DateTime, Utc};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use serde::{Deserialize, Serialize};

/// License tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    Free,
    Starter,
    Team,
    Business,
    Enterprise,
}

/// License structure (cryptographically signed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub tier: LicenseTier,
    pub organization: String,
    pub max_nodes: u32,
    pub max_agents: u32,
    pub features: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(with = "hex")]
    pub signature: Vec<u8>,
}

/// Cluster type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterType {
    Kind,
    K3s,
    K3d,
    Minikube,
    Talos,
    EKS,
    GKE,
    AKS,
    Kubeadm,
    OpenShift,
    RKE2,
    Unknown,
}

/// Validation result
#[derive(Debug)]
pub enum ValidationResult {
    Valid {
        tier: LicenseTier,
        days_remaining: i64,
    },
    Expired {
        days_since: i64,
        degradation_level: DegradationLevel,
    },
    InvalidSignature,
    ClusterTypeNotAllowed {
        detected: ClusterType,
        allowed: Vec<ClusterType>,
    },
    NodeLimitExceeded {
        current: u32,
        max: u32,
    },
    Revoked,
}

/// Graceful degradation levels
#[derive(Debug, Clone, Copy)]
pub enum DegradationLevel {
    Warning,           // 0-7 days expired: warnings only
    UpdatesDisabled,   // 7-14 days: no OTA updates
    GuardianReduced,   // 14-30 days: guardian monitor-only
    NewAgentsBlocked,  // 30-90 days: can't deploy new agents
    ReadOnly,          // 90+ days: platform read-only
}

pub struct LicenseValidator {
    public_key: PublicKey,
    phone_home_client: Option<PhoneHomeClient>,
}

impl LicenseValidator {
    /// Validate a license
    pub async fn validate(&self, license: &License) -> ValidationResult {
        // Layer 1: Cryptographic signature verification
        if !self.verify_signature(license) {
            return ValidationResult::InvalidSignature;
        }
        
        // Layer 2: Cluster type check (Free tier = Kind only)
        let cluster_type = self.detect_cluster_type().await;
        if license.tier == LicenseTier::Free && cluster_type != ClusterType::Kind {
            return ValidationResult::ClusterTypeNotAllowed {
                detected: cluster_type,
                allowed: vec![ClusterType::Kind],
            };
        }
        
        // Layer 3: Expiration check with graceful degradation
        let now = Utc::now();
        if license.expires_at < now {
            let days_since = (now - license.expires_at).num_days();
            return ValidationResult::Expired {
                days_since,
                degradation_level: Self::calculate_degradation(days_since),
            };
        }
        
        // Layer 4: Node count verification
        let node_count = self.count_cluster_nodes().await;
        if node_count > license.max_nodes {
            return ValidationResult::NodeLimitExceeded {
                current: node_count,
                max: license.max_nodes,
            };
        }
        
        // Layer 5: Phone-home revocation check (if enabled)
        if let Some(client) = &self.phone_home_client {
            if let Ok(false) = client.is_license_valid(&license.id).await {
                return ValidationResult::Revoked;
            }
        }
        
        ValidationResult::Valid {
            tier: license.tier,
            days_remaining: (license.expires_at - now).num_days(),
        }
    }
    
    fn verify_signature(&self, license: &License) -> bool {
        // Create message from license fields (excluding signature)
        let message = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            license.id,
            license.tier as u8,
            license.organization,
            license.max_nodes,
            license.max_agents,
            license.issued_at.timestamp(),
            license.expires_at.timestamp()
        );
        
        let signature = match Signature::from_bytes(&license.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        self.public_key
            .verify(message.as_bytes(), &signature)
            .is_ok()
    }
    
    async fn detect_cluster_type(&self) -> ClusterType {
        // Check node labels for Kind
        if self.has_label("node.kubernetes.io/instance-type", "kind").await {
            return ClusterType::Kind;
        }
        
        // Check for Kind-specific node name pattern
        if self.nodes_match_pattern("kind-*").await {
            return ClusterType::Kind;
        }
        
        // Check for Talos
        if self.has_label("node.kubernetes.io/instance-type", "talos").await {
            return ClusterType::Talos;
        }
        
        // Check cloud providers
        if self.has_label("eks.amazonaws.com/nodegroup", "").await {
            return ClusterType::EKS;
        }
        
        if self.has_label("cloud.google.com/gke-nodepool", "").await {
            return ClusterType::GKE;
        }
        
        if self.has_label("kubernetes.azure.com/agentpool", "").await {
            return ClusterType::AKS;
        }
        
        // Check for k3s
        if self.has_annotation("k3s.io/node-args", "").await {
            return ClusterType::K3s;
        }
        
        ClusterType::Unknown
    }
    
    fn calculate_degradation(days_expired: i64) -> DegradationLevel {
        match days_expired {
            0..=7 => DegradationLevel::Warning,
            8..=14 => DegradationLevel::UpdatesDisabled,
            15..=30 => DegradationLevel::GuardianReduced,
            31..=90 => DegradationLevel::NewAgentsBlocked,
            _ => DegradationLevel::ReadOnly,
        }
    }
}
```

### Feature Gating

```rust
//! Feature gating based on license tier

use std::collections::HashSet;

/// Features available by tier
pub fn features_for_tier(tier: LicenseTier) -> HashSet<Feature> {
    let mut features = HashSet::new();
    
    match tier {
        LicenseTier::Free => {
            // Very limited - Kind only
            features.insert(Feature::BasicAgents);
            features.insert(Feature::ManualInstall);
            features.insert(Feature::CommunitySupport);
        }
        
        LicenseTier::Starter => {
            features.insert(Feature::BasicAgents);
            features.insert(Feature::SetupWizard);
            features.insert(Feature::BasicGuardian);
            features.insert(Feature::OtaUpdates);
            features.insert(Feature::EmailSupport);
            features.insert(Feature::SingleNode);
        }
        
        LicenseTier::Team => {
            // All Starter features plus:
            features.extend(features_for_tier(LicenseTier::Starter));
            features.insert(Feature::MultiNode);
            features.insert(Feature::FullGuardian);
            features.insert(Feature::AllAgents);
            features.insert(Feature::ServiceMarketplace);
            features.insert(Feature::PrioritySupport);
            features.insert(Feature::ProjectManagement);
        }
        
        LicenseTier::Business => {
            // All Team features plus:
            features.extend(features_for_tier(LicenseTier::Team));
            features.insert(Feature::AdvancedAnalytics);
            features.insert(Feature::CustomAgents);
            features.insert(Feature::AgentBuilder);
            features.insert(Feature::StagedRollouts);
        }
        
        LicenseTier::Enterprise => {
            // Everything
            features.extend(features_for_tier(LicenseTier::Business));
            features.insert(Feature::SsoIntegration);
            features.insert(Feature::AirGapBundle);
            features.insert(Feature::AuditLogging);
            features.insert(Feature::DedicatedSupport);
            features.insert(Feature::SlaGuarantees);
            features.insert(Feature::CustomGuardianRules);
            features.insert(Feature::WhiteLabeling);
        }
    }
    
    features
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    // Basic
    BasicAgents,
    ManualInstall,
    CommunitySupport,
    
    // Starter
    SetupWizard,
    BasicGuardian,
    OtaUpdates,
    EmailSupport,
    SingleNode,
    
    // Team
    MultiNode,
    FullGuardian,
    AllAgents,
    ServiceMarketplace,
    PrioritySupport,
    ProjectManagement,
    
    // Business
    AdvancedAnalytics,
    CustomAgents,
    AgentBuilder,
    StagedRollouts,
    
    // Enterprise
    SsoIntegration,
    AirGapBundle,
    AuditLogging,
    DedicatedSupport,
    SlaGuarantees,
    CustomGuardianRules,
    WhiteLabeling,
}
```

---

## BYOK (Bring Your Own Keys) Implementation

### Secrets Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       BYOK SECRETS FLOW                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐                                                       │
│  │   Web Portal     │                                                       │
│  │                  │                                                       │
│  │  User enters:    │                                                       │
│  │  - API keys      │                                                       │
│  │  - Tokens        │                                                       │
│  └────────┬─────────┘                                                       │
│           │                                                                  │
│           │ Encrypted in browser (WebCrypto)                                │
│           │ One-time use key derived from license                           │
│           ▼                                                                  │
│  ┌──────────────────┐                                                       │
│  │ Generated Config │                                                       │
│  │ (includes        │                                                       │
│  │ encrypted blob)  │                                                       │
│  └────────┬─────────┘                                                       │
│           │                                                                  │
│           │ Downloaded to customer                                          │
│           │ OR sent directly to cluster                                     │
│           ▼                                                                  │
│  ┌──────────────────┐     ┌──────────────────┐                             │
│  │ Customer Cluster │     │  5D Labs Portal  │                             │
│  │                  │     │                  │                             │
│  │ ┌──────────────┐ │     │  ⚠️ NEVER sees   │                             │
│  │ │   OpenBao    │ │     │  decrypted keys  │                             │
│  │ │   (Vault)    │ │     │                  │                             │
│  │ │              │ │     │  Keys go:        │                             │
│  │ │  Decryption  │ │     │  Browser →       │                             │
│  │ │  happens     │ │     │  Customer Vault  │                             │
│  │ │  HERE only   │ │     │                  │                             │
│  │ └──────────────┘ │     └──────────────────┘                             │
│  │                  │                                                       │
│  └──────────────────┘                                                       │
│                                                                              │
│  RESULT: Zero-knowledge architecture                                        │
│  - 5D Labs never sees customer API keys                                    │
│  - Keys encrypted end-to-end                                               │
│  - Only customer's Vault can decrypt                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Vault Path Structure

```hcl
# Customer secrets stored at:
# secret/cto/<cluster-id>/

# AI Provider Keys
secret/cto/cluster-abc123/ai/anthropic
secret/cto/cluster-abc123/ai/openai
secret/cto/cluster-abc123/ai/google

# Integration Keys
secret/cto/cluster-abc123/integrations/github
secret/cto/cluster-abc123/integrations/linear
secret/cto/cluster-abc123/integrations/slack

# Database Credentials (auto-generated)
secret/cto/cluster-abc123/databases/postgresql
secret/cto/cluster-abc123/databases/redis

# TLS Certificates
secret/cto/cluster-abc123/tls/ingress
secret/cto/cluster-abc123/tls/internal
```

---

## Revenue Projections

### Conservative Forecast

| Year | Free | Starter | Team | Business | Enterprise | ARR |
|------|------|---------|------|----------|------------|-----|
| Y1 | 500 | 60 | 25 | 8 | 2 | $260K |
| Y2 | 2,000 | 200 | 80 | 25 | 5 | $850K |
| Y3 | 5,000 | 500 | 200 | 60 | 12 | $2.1M |
| Y4 | 10,000 | 1,000 | 400 | 120 | 25 | $4.5M |
| Y5 | 20,000 | 2,000 | 800 | 250 | 50 | $9.5M |

### Conversion Assumptions

| Metric | Rate |
|--------|------|
| Free → Starter | 12% (over 12 months) |
| Starter → Team | 25% (over 12 months) |
| Team → Business | 20% (over 12 months) |
| Business → Enterprise | 15% (over 12 months) |
| Annual Churn (Starter) | 15% |
| Annual Churn (Team+) | 8% |

### Revenue Mix by Year 5

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       REVENUE MIX (YEAR 5)                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Self-Hosted Licenses                                                       │
│  ██████████████████████████████████████████████  $6.8M (72%)                │
│                                                                              │
│  Hosted Platform                                                            │
│  ████████████████  $1.9M (20%)                                              │
│                                                                              │
│  Add-Ons                                                                    │
│  ████  $0.5M (5%)                                                           │
│                                                                              │
│  Professional Services                                                      │
│  ██  $0.3M (3%)                                                             │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│  TOTAL: $9.5M ARR                                                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Phase 1: MVP Launch (Months 1-3)

- [ ] Implement license validation system
- [ ] Build Kind-only free tier constraint
- [ ] Create web portal v1 (basic configuration)
- [ ] Set up license generation and management
- [ ] Publish Helm charts with license checks
- [ ] Create ISO builder pipeline
- [ ] Launch Starter and Team tiers

### Phase 2: Expansion (Months 4-6)

- [ ] Add Business and Enterprise tiers
- [ ] Implement full portal with cost calculator
- [ ] Build BYOK secrets flow
- [ ] Add Design System add-on
- [ ] Add Agent Builder add-on
- [ ] Implement staged rollout updates

### Phase 3: Hosted Launch (Months 7-9)

- [ ] Build hosted infrastructure
- [ ] Implement multi-tenant isolation
- [ ] Create hosted onboarding flow
- [ ] Add hosted-specific monitoring
- [ ] Launch Hosted Starter tier

### Phase 4: Scale (Months 10-12)

- [ ] Partner program for MSPs
- [ ] Hardware bundle partnerships
- [ ] Cloud marketplace listings
- [ ] International expansion
- [ ] Advanced analytics dashboard

---

## Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **License Type** | Source Available (FSL) | Protects against AWS/clones while allowing inspection |
| **Free Tier Constraint** | Kind-only | Natural dev→prod upgrade path, prevents production freeloading |
| **Entry Price** | $79/month | Low enough for startup budget approval |
| **Hosted Markup** | 3-4x | Standard managed service margin, funds ops team |
| **BYOK** | Required | Security selling point, reduces our liability |
| **Feature Gating** | Guardian + OTA + Wizard | These are the "magic" - keep them paid |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-07 | 5D Labs | Initial distribution strategy document |

---

*This document should be reviewed quarterly and updated based on market feedback and competitive landscape changes.*


