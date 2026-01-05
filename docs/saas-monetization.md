# CTO Platform Monetization Strategy

## Overview

This document outlines monetization options for the CTO Platform SaaS offering, from self-serve starter tiers to enterprise contracts.

---

## Pricing Models

### Model 1: Usage-Based (Pay-As-You-Go)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              USAGE-BASED PRICING                                         │
│                                                                                          │
│  Concept: Pay only for what you use, no commitments                                     │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                           Billing Dimensions                                       │  │
│  │                                                                                    │  │
│  │  1. CodeRun Minutes                                                               │  │
│  │     └─ Time from pod start to completion                                          │  │
│  │     └─ Example: $0.10/minute                                                      │  │
│  │                                                                                    │  │
│  │  2. AI Tokens (if using 5D Labs keys)                                             │  │
│  │     └─ Pass-through + margin on API costs                                         │  │
│  │     └─ Example: Anthropic cost + 20% margin                                       │  │
│  │                                                                                    │  │
│  │  3. Infrastructure (if using 5D Labs infra)                                       │  │
│  │     └─ Compute time on bare metal / cloud                                         │  │
│  │     └─ Example: $0.05/minute for standard, $0.50/minute for GPU                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Example Invoice:                                                                       │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  Acme Corp - January 2025                                                         │  │
│  │  ─────────────────────────────────────────────────────────────                    │  │
│  │  CodeRun execution          142 runs × 8 min avg = 1,136 min    $113.60           │  │
│  │  AI tokens (Claude)         2.4M input, 800K output             $42.00            │  │
│  │  Infrastructure (standard)  1,136 min                           $56.80            │  │
│  │  ─────────────────────────────────────────────────────────────                    │  │
│  │  Total                                                          $212.40           │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Pros:                                                                                  │
│    ✓ Low barrier to entry                                                              │
│    ✓ Fair - pay for actual usage                                                       │
│    ✓ Scales naturally with customer growth                                             │
│                                                                                          │
│  Cons:                                                                                  │
│    ✗ Unpredictable revenue for 5D Labs                                                 │
│    ✗ Unpredictable costs for customers                                                 │
│    ✗ Complex billing infrastructure needed                                             │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### Model 2: Tiered Subscription (SaaS Standard)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            TIERED SUBSCRIPTION                                           │
│                                                                                          │
│  ┌─────────────────────┬─────────────────────┬─────────────────────┬─────────────────┐  │
│  │       STARTER       │        TEAM         │     ENTERPRISE      │   ENTERPRISE+   │  │
│  │      $49/month      │     $299/month      │    $999+/month      │     Custom      │  │
│  ├─────────────────────┼─────────────────────┼─────────────────────┼─────────────────┤  │
│  │                     │                     │                     │                 │  │
│  │  Included:          │  Included:          │  Included:          │  Included:      │  │
│  │  • 50 CodeRuns/mo   │  • 500 CodeRuns/mo  │  • 2000 CodeRuns/mo │  • Unlimited    │  │
│  │  • 1 user           │  • 10 users         │  • Unlimited users  │  • Dedicated    │  │
│  │  • 1 repository     │  • 10 repositories  │  • Unlimited repos  │    infra        │  │
│  │  • Community        │  • Email support    │  • Slack support    │  • 24/7 support │  │
│  │    support          │  • Basic SSO        │  • Full SSO/SAML    │  • SLA          │  │
│  │                     │                     │  • Audit logs       │  • Custom       │  │
│  │                     │                     │  • SCIM             │    contracts    │  │
│  │                     │                     │                     │                 │  │
│  │  Overage:           │  Overage:           │  Overage:           │  Negotiated     │  │
│  │  $2/CodeRun         │  $1.50/CodeRun      │  $1/CodeRun         │                 │  │
│  │                     │                     │                     │                 │  │
│  │  AI Keys:           │  AI Keys:           │  AI Keys:           │  AI Keys:       │  │
│  │  BYOK only          │  BYOK or managed    │  BYOK or managed    │  Flexible       │  │
│  │                     │  (+20% markup)      │  (+15% markup)      │                 │  │
│  │                     │                     │                     │                 │  │
│  └─────────────────────┴─────────────────────┴─────────────────────┴─────────────────┘  │
│                                                                                          │
│  Pros:                                                                                  │
│    ✓ Predictable revenue (MRR)                                                         │
│    ✓ Predictable costs for customers                                                   │
│    ✓ Clear upgrade path                                                                │
│    ✓ Industry standard model                                                           │
│                                                                                          │
│  Cons:                                                                                  │
│    ✗ May over-provision (customer pays for unused capacity)                            │
│    ✗ May under-provision (customer hits limits, bad UX)                                │
│    ✗ Complex tier management                                                           │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### Model 3: Seat-Based (Per Developer)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              SEAT-BASED PRICING                                          │
│                                                                                          │
│  Concept: Pay per developer who uses the platform                                       │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Base pricing:                                                                    │  │
│  │    • $99/seat/month (annual commitment)                                           │  │
│  │    • $129/seat/month (monthly)                                                    │  │
│  │                                                                                    │  │
│  │  What a "seat" includes:                                                          │  │
│  │    • Unlimited CodeRuns                                                           │  │
│  │    • Unlimited repositories                                                       │  │
│  │    • Access to all agents (Rex, Blaze, Tess, etc.)                               │  │
│  │    • Full MCP tools access                                                        │  │
│  │                                                                                    │  │
│  │  Additional costs:                                                                │  │
│  │    • AI API tokens (pass-through or BYOK)                                         │  │
│  │    • Infrastructure for custom deployments                                        │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Volume discounts:                                                                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │   1-10 seats:     $99/seat/month                                                  │  │
│  │   11-50 seats:    $79/seat/month                                                  │  │
│  │   51-100 seats:   $59/seat/month                                                  │  │
│  │   100+ seats:     Custom pricing                                                  │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Pros:                                                                                  │
│    ✓ Simple to understand                                                              │
│    ✓ Aligns with team growth                                                           │
│    ✓ Encourages adoption (unlimited usage per seat)                                    │
│    ✓ Sales loves it (easy to quote)                                                    │
│                                                                                          │
│  Cons:                                                                                  │
│    ✗ Doesn't capture heavy usage value                                                 │
│    ✗ One power user = same price as occasional user                                    │
│    ✗ Hard to define "seat" (what about CI/CD usage?)                                   │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### Model 4: Hybrid (Recommended)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         HYBRID PRICING (RECOMMENDED)                                     │
│                                                                                          │
│  Concept: Platform fee + usage-based components                                         │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  STARTER: $0/month + usage                                                        │  │
│  │  ─────────────────────────────────                                                │  │
│  │  Platform fee:        $0 (free tier)                                              │  │
│  │  Included:            20 CodeRuns/month free                                      │  │
│  │  CodeRun overage:     $3.00/run                                                   │  │
│  │  AI tokens:           BYOK only (bring your own keys)                             │  │
│  │  Infrastructure:      5D Labs shared pool                                         │  │
│  │  Support:             Community (Discord/GitHub)                                  │  │
│  │                                                                                    │  │
│  │  ─────────────────────────────────────────────────────────────────────────────    │  │
│  │                                                                                    │  │
│  │  TEAM: $199/month + usage                                                         │  │
│  │  ─────────────────────────────────                                                │  │
│  │  Platform fee:        $199/month                                                  │  │
│  │  Included:            200 CodeRuns/month                                          │  │
│  │  CodeRun overage:     $1.50/run                                                   │  │
│  │  AI tokens:           BYOK or managed (+15% margin)                               │  │
│  │  Infrastructure:      5D Labs shared pool                                         │  │
│  │  Support:             Email (48h response)                                        │  │
│  │  Features:            Team management, SSO (Google/Microsoft)                     │  │
│  │                                                                                    │  │
│  │  ─────────────────────────────────────────────────────────────────────────────    │  │
│  │                                                                                    │  │
│  │  GROWTH: $499/month + usage                                                       │  │
│  │  ─────────────────────────────────                                                │  │
│  │  Platform fee:        $499/month                                                  │  │
│  │  Included:            1000 CodeRuns/month                                         │  │
│  │  CodeRun overage:     $0.75/run                                                   │  │
│  │  AI tokens:           BYOK or managed (+10% margin)                               │  │
│  │  Infrastructure:      Dedicated namespace                                         │  │
│  │  Support:             Slack (24h response)                                        │  │
│  │  Features:            Full SSO/SAML, audit logs, SCIM                             │  │
│  │                                                                                    │  │
│  │  ─────────────────────────────────────────────────────────────────────────────    │  │
│  │                                                                                    │  │
│  │  ENTERPRISE: $2,000+/month (custom)                                               │  │
│  │  ─────────────────────────────────                                                │  │
│  │  Platform fee:        Custom (based on scale)                                     │  │
│  │  Included:            Custom CodeRun allotment                                    │  │
│  │  CodeRun overage:     $0.50/run (negotiable)                                      │  │
│  │  AI tokens:           BYOK, managed, or volume discount                           │  │
│  │  Infrastructure:      Dedicated infra, BYOC, or on-prem                           │  │
│  │  Support:             Dedicated CSM, 4h response SLA                              │  │
│  │  Features:            Everything + custom integrations                            │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Revenue Streams

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              REVENUE STREAMS                                             │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  1. PLATFORM SUBSCRIPTION (Recurring)                                             │  │
│  │                                                                                    │  │
│  │     Monthly/annual subscription fees                                              │  │
│  │     └─ Predictable, foundation of MRR/ARR                                         │  │
│  │     └─ Target: 60% of revenue                                                     │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  2. CODERUN OVERAGE (Usage)                                                       │  │
│  │                                                                                    │  │
│  │     Per-run charges beyond included allotment                                     │  │
│  │     └─ Captures value from heavy users                                            │  │
│  │     └─ Target: 15% of revenue                                                     │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  3. AI API MARGIN (Pass-through + margin)                                         │  │
│  │                                                                                    │  │
│  │     When customers use 5D Labs managed API keys:                                  │  │
│  │     └─ Anthropic, OpenAI, Google costs + 10-20% margin                            │  │
│  │     └─ Convenience fee for not managing own keys                                  │  │
│  │     └─ Target: 10% of revenue                                                     │  │
│  │                                                                                    │  │
│  │     Example margin structure:                                                     │  │
│  │     ┌────────────────┬────────────────┬────────────────┬─────────────────┐        │  │
│  │     │   Provider     │   Our Cost     │   Customer     │   Margin        │        │  │
│  │     ├────────────────┼────────────────┼────────────────┼─────────────────┤        │  │
│  │     │ Claude Opus    │ $15/M input    │ $18/M input    │ 20%             │        │  │
│  │     │ Claude Sonnet  │ $3/M input     │ $3.45/M input  │ 15%             │        │  │
│  │     │ GPT-4          │ $10/M input    │ $12/M input    │ 20%             │        │  │
│  │     │ Gemini         │ $2.50/M input  │ $3/M input     │ 20%             │        │  │
│  │     └────────────────┴────────────────┴────────────────┴─────────────────┘        │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  4. INFRASTRUCTURE (Compute time)                                                 │  │
│  │                                                                                    │  │
│  │     When customers use 5D Labs infrastructure:                                    │  │
│  │     └─ Bare metal (Latitude.sh) - cost + margin                                   │  │
│  │     └─ GPU instances - premium pricing                                            │  │
│  │     └─ Target: 10% of revenue                                                     │  │
│  │                                                                                    │  │
│  │     Example:                                                                      │  │
│  │     ┌────────────────────┬────────────────┬────────────────┬─────────────────┐    │  │
│  │     │   Instance Type    │   Our Cost     │   Customer     │   Margin        │    │  │
│  │     ├────────────────────┼────────────────┼────────────────┼─────────────────┤    │  │
│  │     │ Standard (4 CPU)   │ $0.03/min      │ $0.05/min      │ 67%             │    │  │
│  │     │ High-mem (32GB)    │ $0.08/min      │ $0.12/min      │ 50%             │    │  │
│  │     │ GPU (A100)         │ $0.80/min      │ $1.20/min      │ 50%             │    │  │
│  │     └────────────────────┴────────────────┴────────────────┴─────────────────┘    │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  5. PROFESSIONAL SERVICES (One-time + ongoing)                                    │  │
│  │                                                                                    │  │
│  │     Enterprise add-ons:                                                           │  │
│  │     └─ Custom integrations: $10K-50K one-time                                     │  │
│  │     └─ On-premises deployment: $25K setup + $5K/month support                     │  │
│  │     └─ Training/onboarding: $2K-10K                                               │  │
│  │     └─ Custom agent development: $15K-100K                                        │  │
│  │     └─ Target: 5% of revenue                                                      │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Pricing Calculator Example

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           PRICING CALCULATOR                                             │
│                                                                                          │
│  Company: Mid-size Startup (50 developers, 20 active on platform)                       │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │  Monthly Usage Estimate:                                                          │  │
│  │                                                                                    │  │
│  │  • CodeRuns: 800/month (40 per active dev)                                        │  │
│  │  • Avg run duration: 10 minutes                                                   │  │
│  │  • AI tokens: 50M input, 15M output (Claude Sonnet)                               │  │
│  │  • Infrastructure: Using 5D Labs shared                                           │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option A: GROWTH Plan ($499/month base)                                                │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Platform fee (Growth):              $499.00                                      │  │
│  │  Included CodeRuns:                  1000 (covers 800 usage)                      │  │
│  │  CodeRun overage:                    $0.00 (under limit)                          │  │
│  │  AI tokens (managed, 10% margin):    $195.00                                      │  │
│  │    └─ Input: 50M × $3.30/M = $165                                                 │  │
│  │    └─ Output: 15M × $2/M = $30 (estimated)                                        │  │
│  │  ─────────────────────────────────────────────────                                │  │
│  │  Total monthly:                      $694.00                                      │  │
│  │  Per active developer:               $34.70/dev                                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option B: GROWTH Plan + BYOK (bring own API keys)                                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Platform fee (Growth):              $499.00                                      │  │
│  │  Included CodeRuns:                  1000                                         │  │
│  │  CodeRun overage:                    $0.00                                        │  │
│  │  AI tokens:                          $0.00 (customer pays Anthropic directly)     │  │
│  │  ─────────────────────────────────────────────────                                │  │
│  │  Total to 5D Labs:                   $499.00                                      │  │
│  │  Customer's Anthropic bill:          ~$177.00                                     │  │
│  │  Total cost to customer:             $676.00                                      │  │
│  │  Per active developer:               $33.80/dev                                   │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  Option C: ENTERPRISE (negotiated)                                                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                    │  │
│  │  Platform fee:                       $2,000.00 (annual commit)                    │  │
│  │  Included CodeRuns:                  5000/month                                   │  │
│  │  CodeRun overage:                    $0.50/run                                    │  │
│  │  AI tokens:                          Volume discount (5% margin)                  │  │
│  │  Dedicated support:                  Included                                     │  │
│  │  ─────────────────────────────────────────────────                                │  │
│  │  Total monthly:                      ~$2,180.00                                   │  │
│  │  Per active developer:               $109/dev                                     │  │
│  │                                                                                    │  │
│  │  Value prop: SLA, dedicated CSM, priority support, custom features               │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Competitive Positioning

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         COMPETITIVE LANDSCAPE                                            │
│                                                                                          │
│  ┌──────────────────┬─────────────────┬─────────────────┬────────────────────────────┐  │
│  │    Competitor    │    Model        │    Pricing      │    Our Differentiation     │  │
│  ├──────────────────┼─────────────────┼─────────────────┼────────────────────────────┤  │
│  │                  │                 │                 │                            │  │
│  │  GitHub Copilot  │  Per-seat       │  $19/seat/mo    │  We do full tasks, not     │  │
│  │                  │                 │  (individual)   │  just autocomplete         │  │
│  │                  │                 │  $39/seat/mo    │                            │  │
│  │                  │                 │  (enterprise)   │                            │  │
│  │                  │                 │                 │                            │  │
│  │  Cursor          │  Per-seat       │  $20/seat/mo    │  We're infrastructure,     │  │
│  │                  │                 │  (pro)          │  not an IDE                │  │
│  │                  │                 │                 │                            │  │
│  │  Devin (Cognition)│ Waitlist/      │  Unknown        │  Open architecture,        │  │
│  │                  │  Enterprise     │  (reportedly    │  multi-agent, customer     │  │
│  │                  │                 │  high)          │  owns their repos          │  │
│  │                  │                 │                 │                            │  │
│  │  Replit Agent    │  Per-seat +     │  $25/seat/mo    │  Enterprise-grade,         │  │
│  │                  │  compute        │  + usage        │  works with existing       │  │
│  │                  │                 │                 │  codebase                  │  │
│  │                  │                 │                 │                            │  │
│  │  Custom internal │  Build cost     │  $500K-2M/yr    │  10x faster to deploy,     │  │
│  │  platform        │                 │  (eng time)     │  continuously improved     │  │
│  │                  │                 │                 │                            │  │
│  └──────────────────┴─────────────────┴─────────────────┴────────────────────────────┘  │
│                                                                                          │
│  Key differentiators:                                                                   │
│    1. Full task completion (not just code suggestions)                                  │
│    2. Multi-agent orchestration (specialized agents)                                    │
│    3. Infrastructure management (bare metal + cloud)                                    │
│    4. Enterprise-grade (SSO, audit, compliance)                                         │
│    5. Self-healing (Healer auto-fixes CI failures)                                      │
│    6. Kubernetes-native (works with existing DevOps)                                    │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Go-to-Market Tiers

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              GO-TO-MARKET STRATEGY                                       │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                           PHASE 1: LAUNCH (Q1-Q2)                                 │  │
│  │                                                                                    │  │
│  │  Target: Early adopters, indie devs, small teams                                  │  │
│  │                                                                                    │  │
│  │  Offer:                                                                           │  │
│  │    • Free tier (generous - 50 CodeRuns/month)                                     │  │
│  │    • Simple pricing (Team at $199/month)                                          │  │
│  │    • Self-serve signup                                                            │  │
│  │    • Product-led growth                                                           │  │
│  │                                                                                    │  │
│  │  Goal: 100 paying customers, validate pricing                                     │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                           PHASE 2: GROWTH (Q3-Q4)                                 │  │
│  │                                                                                    │  │
│  │  Target: Startups, mid-market companies                                           │  │
│  │                                                                                    │  │
│  │  Offer:                                                                           │  │
│  │    • Growth tier ($499/month)                                                     │  │
│  │    • Sales-assisted for Growth+                                                   │  │
│  │    • Case studies from Phase 1                                                    │  │
│  │    • Partner program (agencies, consultants)                                      │  │
│  │                                                                                    │  │
│  │  Goal: $50K MRR, 10 Growth tier customers                                         │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐  │
│  │                           PHASE 3: ENTERPRISE (Year 2)                            │  │
│  │                                                                                    │  │
│  │  Target: Enterprise, Fortune 500                                                  │  │
│  │                                                                                    │  │
│  │  Offer:                                                                           │  │
│  │    • Enterprise tier (custom pricing)                                             │  │
│  │    • SOC 2 certification                                                          │  │
│  │    • On-premises option                                                           │  │
│  │    • Dedicated success team                                                       │  │
│  │    • Custom SLAs                                                                  │  │
│  │                                                                                    │  │
│  │  Goal: 5 enterprise contracts ($50K+ ACV each)                                    │  │
│  │                                                                                    │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary: Recommended Pricing

| Tier | Monthly | Annual | CodeRuns | Key Features |
|------|---------|--------|----------|--------------|
| **Starter** | Free | Free | 20 included | BYOK, 1 user, community support |
| **Team** | $199 | $1,990 | 200 included | 10 users, basic SSO, email support |
| **Growth** | $499 | $4,990 | 1000 included | Unlimited users, SAML, audit logs |
| **Enterprise** | Custom | Custom | Custom | Dedicated infra, SLA, compliance |

**Overage rates:**
- Starter: $3.00/CodeRun
- Team: $1.50/CodeRun
- Growth: $0.75/CodeRun
- Enterprise: $0.50/CodeRun (negotiable)

**AI API options:**
- BYOK: No charge (customer manages)
- Managed: Pass-through + 10-20% margin
