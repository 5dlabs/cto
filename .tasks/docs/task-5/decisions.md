## Decision Points

- Use anchor-client crate for instruction construction vs. manually building instructions from the IDL — anchor-client adds a large dependency but provides type safety; manual construction is lighter but more error-prone.
- Observability approach: structured tracing logs only vs. Prometheus metrics endpoint — Prometheus provides richer dashboarding but adds complexity; tracing logs are simpler and sufficient for hackathon scope.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/solana-sdk/solana-client