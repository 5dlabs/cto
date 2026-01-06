---
name: stripe-llm-docs
description: Fetch LLM-optimized documentation for Stripe, the payments infrastructure platform.
agents: [blaze, nova, rex, grizz]
triggers: [stripe, stripe docs, payments, billing, subscriptions, checkout]
llm_docs:
  - stripe
---

# Stripe Documentation (llms.txt)

Stripe is a technology company that builds economic infrastructure for the internet, providing payment processing and financial services. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for Stripe, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://stripe.com/llms.txt", formats: ["markdown"] })
```

## Key Features

- **Payments** - Accept payments globally
- **Checkout** - Pre-built payment pages
- **Subscriptions** - Recurring billing
- **Connect** - Marketplace payments
- **Billing** - Invoicing and revenue management
- **Identity** - Fraud prevention and verification

## Related Skills

- `effect-patterns` - Effect TypeScript for type-safe Stripe integration
- `better-auth` - Authentication (often paired with Stripe for SaaS)
