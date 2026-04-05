## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which PostgreSQL operator should be used for managing the main transactional database?
- What API paradigm should be used for inter-service communication between backend services (e.g., Equipment Catalog, RMS, Finance, Vetting)?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- How should API versioning be handled for public and internal APIs?
- Which Google Calendar API client library and authentication method should be used for project event integration?

## Coordination Notes

- Agent owner: Grizz
- Primary stack: Go 1.22+/gRPC