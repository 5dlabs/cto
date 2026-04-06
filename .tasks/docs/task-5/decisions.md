## Decision Points

- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Should the Finance, Customer Vetting, and Equipment Catalog services be deployed as separate microservices or merged into a single Rust/Axum monolith?
- Which specific credit scoring API provider should be integrated (e.g., Experian, Equifax, Dun & Bradstreet, CreditSafe)? Each has different data formats, pricing, and coverage.
- Should the vetting pipeline stages (business verification, online presence, reputation, credit) run sequentially or in parallel with a fan-out/fan-in pattern?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7