## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images and social media photos?
- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- What API paradigm should be used for the public-facing Equipment Catalog and RMS APIs?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Should the Finance, Customer Vetting, and Equipment Catalog services be deployed as separate microservices or merged into a single Rust/Axum monolith?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7