## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images and social media photos?
- Which PostgreSQL operator should be used for managing the main database cluster?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service communication?
- Should the Signal integration for Morgan be self-hosted via Signal-CLI or use a third-party Signal gateway service?
- What approach should be used for secret management and rotation across all services?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm