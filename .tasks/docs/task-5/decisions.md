## Decision Points

- What is the primary inter-service communication paradigm for backend microservices (synchronous REST/gRPC vs asynchronous event-driven messaging)?
- Which PostgreSQL operator should be used for the main database cluster?
- What API paradigm should be used for Morgan's tool-server interface to backend services (REST, gRPC, or GraphQL)?
- How should multi-tenancy and data isolation be handled in the PostgreSQL schema (single database with multiple schemas vs separate databases per service)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- How should the REST API versioning strategy be handled for all public endpoints?
- Which specific external APIs and providers should be used for each vetting pipeline stage (business verification, online presence, reputation, credit scoring)? Are OpenCorporates, LinkedIn, Google Reviews, and a credit API confirmed, or should alternatives/fallbacks be evaluated?
- What are the exact scoring weights and thresholds for the composite GREEN/YELLOW/RED lead score? Are these business rules defined, or do they need to be configurable?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+, Axum 0.7