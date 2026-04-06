## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- How should multi-tenancy be handled in the PostgreSQL schema for all backend services?
- What authentication and authorization mechanism should be used for API access between services and for the frontend?
- Which object storage provider should be used for product images, event photos, and other media assets?
- How should the Signal messenger integration for Morgan be implemented?
- Which CDN and TLS termination solution should be used for public-facing endpoints and static assets?
- What approach should be used for secret management and rotation across all services?

## Coordination Notes

- Agent owner: Bolt
- Primary stack: Kubernetes/Helm