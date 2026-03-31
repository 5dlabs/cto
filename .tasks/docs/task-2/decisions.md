## Decision Points

- What is the primary inter-service communication paradigm for backend microservices (synchronous REST/gRPC vs asynchronous event-driven messaging)?
- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Which object storage provider should be used for product images and social media photos (Cloudflare R2 vs AWS S3)?
- Which PostgreSQL operator should be used for the main database cluster?
- What API paradigm should be used for Morgan's tool-server interface to backend services (REST, gRPC, or GraphQL)?
- How should multi-tenancy and data isolation be handled in the PostgreSQL schema (single database with multiple schemas vs separate databases per service)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- How should the REST API versioning strategy be handled for all public endpoints?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+, Axum 0.7