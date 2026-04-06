## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- What API paradigm should be used for the public-facing Equipment Catalog and RMS APIs?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Which Google Calendar API authentication method should be used for project scheduling — OAuth 2.0 user-delegated access or a service account with domain-wide delegation?
- How should barcode formats be handled for inventory check-in/check-out — should the system generate its own barcodes or support scanning of pre-existing manufacturer/rental-house barcodes?

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go 1.22+/gRPC