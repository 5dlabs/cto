## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images, event photos, and other media assets?
- Which PostgreSQL operator should be used for managing the main transactional database?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- Should the Signal integration for Morgan be self-hosted via Signal-CLI or use a third-party Signal gateway service?

## Coordination Notes

- Agent owner: Bolt
- Primary stack: Kubernetes/Helm