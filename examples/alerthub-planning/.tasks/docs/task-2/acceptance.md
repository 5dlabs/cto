# Acceptance Criteria: Task 2

- [ ] Build the high-performance core notification routing service in Rust using Axum. This service receives notifications, applies rate limiting, handles deduplication, and routes messages to the integration service via Kafka.
- [ ] Service starts successfully, all endpoints return expected responses, notifications are persisted to PostgreSQL, rate limiting blocks excessive requests, messages are published to Kafka, WebSocket connections work, and health/metrics endpoints are accessible.
- [ ] All requirements implemented
- [ ] Tests passing (`cargo test --workspace` exits 0)
- [ ] Lints passing (`cargo clippy -- -D warnings` exits 0)
- [ ] Formatted (`cargo fmt --all --check` exits 0)
- [ ] Build succeeds (`cargo build --release` exits 0)
- [ ] PR created and ready for review
