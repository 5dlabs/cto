## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which PostgreSQL operator should be used for managing the main database cluster?
- What API paradigm should be used for the Equipment Catalog and RMS services' public interfaces?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service communication?
- Which Google Calendar API authentication method should be used for project/crew scheduling integration?

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go/gRPC