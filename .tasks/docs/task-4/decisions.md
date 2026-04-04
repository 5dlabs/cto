## Decision Points

- Stripe integration approach: use the `stripe-rust` crate (higher-level, opinionated) vs raw `reqwest` with typed Stripe API models (more control, more boilerplate). Affects error handling patterns and webhook signature verification.
- Currency rate provider: exchangerate.host (free, no key) vs Open Exchange Rates (free tier with API key, more reliable). Affects secret management and rate limits.
- Invoice number generation strategy: database sequence (simple, gaps on rollback) vs application-level sequential generation with org-scoped prefixes (e.g., ORG-2024-0001). Affects migration design and concurrency handling.
- Stripe webhook signature verification: should the service verify webhook signatures using Stripe's signing secret (recommended for production) or skip verification in v1? Affects security posture and secret provisioning.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7