## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Should the Finance, Customer Vetting, and Equipment Catalog services be deployed as separate microservices or merged into a single Rust/Axum monolith?
- Which currency exchange rate API provider should be used for the multi-currency rate sync job?
- How should automated payment reminders be delivered — directly via email from the Finance service, or by emitting events for a separate notification service to handle?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7