## Decision Points

- Credit signal provider for Phase 2: Equifax vs. Dun & Bradstreet — affects data model fields, API contract, and cost. Should be decided before the stub is replaced so the schema does not need a breaking migration.
- LinkedIn OAuth2 client_credentials flow availability: LinkedIn's API may require a partner program or specific product access for company lookup. Confirm API access tier and whether a fallback scraping approach is acceptable if partner access is unavailable.
- Valkey vs. Redis for caching layer: the details mention both 'redis 0.24' crate and 'Valkey'. Confirm which backing store is provisioned by the bolt infra task so the connection string and crate choice are consistent.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7