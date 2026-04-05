## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Should the Finance and Customer Vetting services be implemented as separate Rust/Axum services or merged into a single multi-domain service?
- What API paradigm should be used for inter-service communication between Morgan (OpenClaw) and backend services?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- How should the public-facing API endpoints (e.g., Equipment Catalog, RMS REST, Finance) be versioned and exposed?
- What access control model should be used for admin endpoints (e.g., product add/update, finance, vetting)?
- Should the Stripe integration for payments be implemented directly or via a third-party payment orchestration platform?
- What precision and storage strategy should be used for financial amounts to avoid floating-point errors?
- Which currency exchange rate data source should be used for the scheduled sync job?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust/Axum