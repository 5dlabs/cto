## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images and social media photos?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- How should the Signal integration for Morgan be implemented?
- Which CDN and TLS termination solution should be used for serving the website and static assets?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm