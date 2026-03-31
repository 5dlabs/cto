## Decision Points

- What is the primary inter-service communication paradigm for backend microservices (synchronous REST/gRPC vs asynchronous event-driven messaging)?
- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Which PostgreSQL operator should be used for the main database cluster?
- What API paradigm should be used for Morgan's tool-server interface to backend services (REST, gRPC, or GraphQL)?
- How should multi-tenancy and data isolation be handled in the PostgreSQL schema (single database with multiple schemas vs separate databases per service)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- How should the REST API versioning strategy be handled for all public endpoints?
- Which Google Calendar API client library and authentication method should be used for crew scheduling integration?
- How should barcode scanning be modeled — should the service accept raw barcode strings and resolve them internally, or expect pre-decoded asset identifiers?

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go 1.22+, gRPC