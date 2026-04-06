## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- How should multi-tenancy be handled in the PostgreSQL schema for all backend services?
- What API paradigm should be used for inter-service communication between backend services and the Morgan agent?
- What authentication and authorization mechanism should be used for API access between services and for the frontend?
- Which object storage provider should be used for product images, event photos, and other media assets?
- How should the public API endpoints be versioned and documented for external and internal consumers?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust 1.75+/Axum 0.7