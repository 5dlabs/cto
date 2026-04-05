## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Which object storage provider should be used for product images and social media photos?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- Should the Signal integration for Morgan be self-hosted (Signal-CLI) or use a third-party Signal gateway service?

## Coordination Notes

- Agent owner: Bolt
- Primary stack: Kubernetes/Helm