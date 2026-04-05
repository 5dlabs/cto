## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which PostgreSQL operator should be used for managing the main database cluster?
- Should the Finance and Customer Vetting services be implemented as separate Rust/Axum services or merged into a single multi-domain service?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service communication?
- Which external currency exchange rate API should be used for the scheduled rate sync job?
- How should monetary amounts be stored and computed to ensure financial precision?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum