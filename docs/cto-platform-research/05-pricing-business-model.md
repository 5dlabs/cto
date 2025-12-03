# Pricing & Business Model

> Pricing strategy, licensing models, and business model recommendations for CTO Platform.

## Executive Summary

CTO Platform operates at the intersection of four markets with distinct pricing expectations. The recommended approach is a **hybrid model** combining:

1. **Open source core** (drives adoption)
2. **Proprietary AI agents** (protects differentiation)
3. **Tiered subscription** (recurring revenue)
4. **Infrastructure pass-through** (aligns incentives)

---

## Market Pricing Benchmarks

### AI Coding Agent Platforms

| Product | Pricing Model | Entry Price | Business Price |
|---------|---------------|-------------|----------------|
| Cursor | Per-seat subscription | $20/month | $40/month |
| GitHub Copilot | Per-seat subscription | $10/month | $19-39/month |
| Devin (Cognition) | Usage + subscription | $20/month + $2.25/ACU | $500/month teams |
| Factory AI | Subscription + BYOK | $20/month | Custom |
| Augment | Credits-based | $30/month | Custom |

**Insight:** Per-seat subscription ($20-40/month) is dominant model, with usage-based emerging.

---

### Bare Metal + Kubernetes Platforms

| Product | Pricing Model | Entry Price | Enterprise |
|---------|---------------|-------------|------------|
| Sidero Labs (Omni) | Per-node tiers | $10/month (10 nodes) | $1,000/month (self-hosted) |
| Platform9 | Node-based tiers | Free (20 nodes) | Custom |
| Rancher (SUSE) | Per-node subscription | ~$100/node/year | Custom |

**Insight:** Per-node pricing ($10-100/month per 10 nodes) is standard.

---

### Platform Engineering / DevOps-as-a-Service

| Product | Pricing Model | Entry Price | Notes |
|---------|---------------|-------------|-------|
| Humanitec | Per-developer | $999/month (25 devs) | ~$40/dev |
| Cortex | Per-user | ~$65/user/month | Minimum seats |
| Spacelift | Resource-based | $0.005/resource/hour | Usage-based |
| Mission Cloud | Managed service | $3,000-$20,000/month | Human-powered |
| Cloud Posse | Consulting + OSS | Project-based | ~$25K+ projects |

**Insight:** Wide range from $40/developer to $20K/month managed services.

---

### Fractional CTO / Engineering Services

| Service | Pricing Model | Typical Range |
|---------|---------------|---------------|
| Fractional CTO (hourly) | Hourly | $225-$350/hour |
| Fractional CTO (retainer) | Monthly | $3,000-$10,000/month |
| Full-time CTO equivalent | Annual | $250K-$400K total comp |
| Dev agency | Hourly/project | $150-$250/hour |
| Offshore team | Monthly | $5,000-$20,000/month |

**Insight:** CTO Platform should price below fractional CTO but above simple tools.

---

## Recommended Pricing Structure

### Tiered Subscription Model

| Tier | Monthly Price | Annual Price | Target Customer |
|------|---------------|--------------|-----------------|
| **Hacker** | Free | - | Side projects, experiments, evaluation |
| **Starter** | $99 | $948 (20% off) | Indie hackers, early validation |
| **Team** | $499 | $4,788 | Seed-stage startups, small teams |
| **Growth** | $1,499 | $14,388 | Post-seed, scaling companies |
| **Enterprise** | Custom | Custom | Large organizations, compliance needs |

### What's Included by Tier

| Feature | Hacker | Starter | Team | Growth | Enterprise |
|---------|--------|---------|------|--------|------------|
| Projects | 1 | 3 | 10 | Unlimited | Unlimited |
| AI agent hours | 10/month | 50/month | 200/month | Unlimited | Unlimited |
| Developers | 1 | 2 | 5 | Unlimited | Unlimited |
| Nodes (bare metal) | 1 | 3 | 20 | 100 | Unlimited |
| Support | Community | Email | Priority | Dedicated | 24/7 SLA |
| Self-hosted | ❌ | ❌ | ❌ | ✅ | ✅ |
| Air-gapped | ❌ | ❌ | ❌ | ❌ | ✅ |
| SSO/SAML | ❌ | ❌ | ✅ | ✅ | ✅ |
| Audit logs | ❌ | ❌ | ✅ | ✅ | ✅ |
| Custom SLA | ❌ | ❌ | ❌ | ✅ | ✅ |

### Infrastructure Pass-Through

Bare metal costs are **passed through at cost** (or slight markup for margin):

| Provider | Typical Monthly | CTO Platform Pass-Through |
|----------|-----------------|---------------------------|
| Hetzner | €39-€150 | At cost |
| Latitude.sh | $85-$500 | At cost |
| Hivelocity | $62-$300 | At cost |
| Voltage Park (H100) | $1,433+ | At cost |

**Rationale:** 
- Aligns incentives (no motivation to over-provision)
- Transparent pricing builds trust
- Margin comes from platform value, not markup

---

## Pricing Psychology

### Why These Price Points

| Tier | Price | Psychology |
|------|-------|------------|
| Free | $0 | Removes friction, builds community, generates word of mouth |
| $99 | Below "need approval" threshold | Founders can expense personally, fast decision |
| $499 | 95% cheaper than one engineer | Easy ROI calculation, compelling value |
| $1,499 | "Team replacement" positioning | Justifies premium, serious commitment signal |
| Custom | Enterprise validation | Large deals need negotiation, legal review |

### Anchoring Strategy

**Anchor against alternatives, not competitors:**

| Alternative | Annual Cost | CTO Platform | Savings |
|-------------|-------------|--------------|---------|
| 1 junior engineer | $150K | $6K-$18K | 88-96% |
| Fractional CTO | $36K-$120K | $6K-$18K | 70-95% |
| Dev agency (6 months) | $150K+ | $6K-$18K | 88-96% |
| AWS + engineering team | $1M+ | $18K + infra | 95%+ |

---

## Business Model Components

### Revenue Streams

| Stream | Contribution | Notes |
|--------|--------------|-------|
| Subscription (primary) | 70-80% | Recurring, predictable |
| Infrastructure margin | 10-15% | Pass-through with small markup |
| Professional services | 5-10% | Migration, custom development |
| Training/certification | 5% | Future expansion |

### Unit Economics Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Customer Acquisition Cost (CAC) | <$500 | Low-touch, self-serve |
| Lifetime Value (LTV) | >$5,000 | 12+ month retention |
| LTV:CAC Ratio | >10:1 | Strong unit economics |
| Gross Margin | >70% | SaaS benchmark |
| Net Revenue Retention | >110% | Expansion from upgrades |
| Monthly Churn | <5% | B2B SaaS benchmark |

---

## Open Source / Commercial Split

### Recommended Licensing Model

Following the **Sidero Labs pattern**:

| Component | License | Rationale |
|-----------|---------|-----------|
| Core infrastructure automation | Apache 2.0 or AGPL | Maximum adoption, community contributions |
| AI agent capabilities | Proprietary or BSL | Protects differentiation |
| Management console | BSL → Apache 2.0 (4 years) | Source available, delayed open source |
| Enterprise features | Proprietary | SSO, audit, compliance |

### What's Open Source (Community Edition)

- Bare metal provisioning (Talos Linux integration)
- Kubernetes deployment automation
- Basic monitoring and logging
- Infrastructure as Code templates
- CLI tools
- Provider integrations (APIs)

### What's Proprietary (Commercial)

- AI coding agents
- Autonomous operations / self-healing
- Multi-cluster management
- Team collaboration features
- Advanced security (SSO, RBAC, audit)
- SLA guarantees
- Priority support

### License Conversion Timeline

For BSL-licensed components:
- **0-2 years:** Source available, no competitive use
- **2-4 years:** Usage restrictions loosen
- **4+ years:** Converts to Apache 2.0

**Rationale:** Balances community trust with commercial protection during growth phase.

---

## Competitive Pricing Analysis

### vs. AI Coding Assistants

| Comparison | AI Assistant | CTO Platform | Differentiation |
|------------|--------------|--------------|-----------------|
| Cursor Pro | $20/month | $99-$499/month | Full stack, not just code completion |
| Copilot Business | $19/user/month | $99-$499/month | Infrastructure included |
| Devin Teams | $500/month | $499/month | Bare metal, self-hosted option |

**Positioning:** Not competing on "AI coding" — competing on "engineering team replacement"

### vs. Infrastructure Platforms

| Comparison | Platform | CTO Platform | Differentiation |
|------------|----------|--------------|-----------------|
| Sidero Omni Build | $250/month (10 nodes) | $499/month (20 nodes) | AI agents included |
| Platform9 Growth | ~$500/month | $499/month | AI agents included |
| Humanitec | $999/month (25 devs) | $499/month | Bare metal, AI agents |

**Positioning:** More value at comparable price point

### vs. Hiring

| Comparison | Hiring | CTO Platform | Differentiation |
|------------|--------|--------------|-----------------|
| Junior engineer | $12,500/month | $499/month | 25x cheaper |
| Senior engineer | $20,000/month | $499/month | 40x cheaper |
| Fractional CTO | $5,000/month | $499/month | 10x cheaper |
| Dev agency | $15,000/month | $499/month | 30x cheaper |

**Positioning:** Obvious ROI, no risk

---

## Monetization Strategy by Stage

### Stage 1: Adoption (0-18 months)

**Goal:** Maximize users, build community

| Focus | Actions |
|-------|---------|
| Free tier | Generous limits, low friction |
| Open source | Release core automation, build contributors |
| Content | Technical blog, tutorials, case studies |
| Community | Discord, GitHub, events |

**Revenue focus:** Minimal — prioritize growth

---

### Stage 2: Conversion (12-24 months)

**Goal:** Convert free users to paid

| Focus | Actions |
|-------|---------|
| Paid features | Gate advanced features behind subscription |
| Usage limits | Enforce limits on free tier |
| Success stories | Publish ROI case studies |
| Sales motion | Light-touch sales for Team tier |

**Revenue focus:** $100K-$500K MRR

---

### Stage 3: Expansion (18-36 months)

**Goal:** Grow revenue from existing customers

| Focus | Actions |
|-------|---------|
| Upsells | Push Growth tier adoption |
| Add-ons | Professional services, training |
| Enterprise | Dedicated sales team |
| Partners | Reseller/MSP program |

**Revenue focus:** $500K-$2M MRR

---

### Stage 4: Scale (36+ months)

**Goal:** Market leadership, profitability

| Focus | Actions |
|-------|---------|
| Enterprise | Large contract focus |
| International | Global expansion |
| Platform | Ecosystem/marketplace |
| M&A | Acquire complementary products |

**Revenue focus:** $2M+ MRR, path to profitability

---

## Anti-Patterns to Avoid

### Pricing Anti-Patterns

| Anti-Pattern | Problem | Alternative |
|--------------|---------|-------------|
| Per-seat only | Limits adoption, encourages seat sharing | Hybrid model |
| Complex usage-based | Unpredictable bills, customer anxiety | Tiered with clear limits |
| No free tier | High friction, slow growth | Generous free tier |
| Enterprise-only | Misses startup market | Bottom-up motion |
| Aggressive markup on infra | Trust erosion, comparison shopping | Pass-through pricing |

### Business Model Anti-Patterns

| Anti-Pattern | Problem | Alternative |
|--------------|---------|-------------|
| SSPL licensing | Triggers forks, community backlash | BSL with conversion |
| Fully proprietary | No community flywheel | Open core model |
| No self-hosted option | Blocks enterprise adoption | Self-hosted at Growth+ tier |
| Hard feature gates | Customer frustration | Soft limits, upsell prompts |
| Long contracts required | Friction, churn at renewal | Month-to-month with annual discount |

---

## Key Metrics Dashboard

### Monthly Tracking

| Category | Metrics |
|----------|---------|
| **Acquisition** | New signups, free-to-paid conversion, CAC |
| **Revenue** | MRR, ARR, ARPU, expansion revenue |
| **Retention** | Churn rate, NRR, logo retention |
| **Usage** | Active projects, deployments, AI agent hours |
| **Satisfaction** | NPS, support tickets, feature requests |

### Quarterly Review

| Category | Metrics |
|----------|---------|
| **Growth** | MRR growth rate, customer count |
| **Economics** | LTV:CAC, gross margin, burn rate |
| **Product** | Feature adoption, platform reliability |
| **Market** | Competitive position, market share |

---

## Summary Recommendations

1. **Primary model:** Tiered subscription (Hacker → Starter → Team → Growth → Enterprise)

2. **Price anchoring:** Against hiring costs, not competitor tools

3. **Infrastructure:** Pass-through pricing at cost (margin from platform, not markup)

4. **Licensing:** Open core — Apache 2.0 for automation, proprietary for AI agents

5. **Free tier:** Generous but limited — drives adoption and word of mouth

6. **Enterprise:** Custom pricing, self-hosted option, compliance features

7. **Avoid:** Per-seat only, complex usage billing, aggressive infra markup, SSPL licensing
