## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- What API paradigm should be used for inter-service communication between Morgan (OpenClaw) and backend services?
- Which object storage provider should be used for product images and social media photos?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- How should the public-facing API endpoints (e.g., Equipment Catalog, RMS REST, Finance) be versioned and exposed?
- What access control model should be used for admin endpoints (e.g., product add/update, finance, vetting)?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust/Axum