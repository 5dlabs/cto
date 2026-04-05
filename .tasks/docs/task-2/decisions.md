## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images, event photos, and other media assets?
- Which PostgreSQL operator should be used for managing the main transactional database?
- What API paradigm should be used for inter-service communication between backend services (e.g., Equipment Catalog, RMS, Finance, Vetting)?
- Should the Finance, Equipment Catalog, and Customer Vetting services be deployed as a single Rust/Axum monolith or as separate microservices?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- What is the access control model for admin endpoints (e.g., product add/update, payroll entry, vetting pipeline)?
- How should API versioning be handled for public and internal APIs?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust 1.75+/Axum 0.7