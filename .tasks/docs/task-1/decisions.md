## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Which object storage provider should be used for product images and social media photos (Cloudflare R2 vs AWS S3)?
- Which PostgreSQL operator should be used for the main database cluster?
- How should multi-tenancy and data isolation be handled in the PostgreSQL schema (single database with multiple schemas vs separate databases per service)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- Should the Signal integration for Morgan be self-hosted (Signal-CLI) or use a third-party SaaS relay?
- What is the approach for secret management and rotation across all services?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm