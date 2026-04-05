## Decision Points

- LinkedIn data source: should the LinkedInClient use the official LinkedIn Marketing/Company API (requires partner-level access), a third-party enrichment service (e.g., Proxycurl, Apollo), or a scraping proxy? Each has different cost, reliability, and ToS implications.
- Commercial credit API provider selection: Creditsafe, Dun & Bradstreet, Experian Business, or another provider? Pricing models and data coverage differ significantly by geography.
- Circuit breaker implementation: use an existing Rust crate (e.g., `recloser`, `failsafe-rs`) or implement a custom lightweight circuit breaker? Existing crates vary in maintenance status.
- Should the vetting pipeline steps run sequentially or concurrently (tokio::join! on all external calls)? Concurrent is faster but complicates per-step error handling and partial result storage.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7