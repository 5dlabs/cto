## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- What API paradigm should be used for inter-service communication between Morgan (OpenClaw) and backend services?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- How should the public-facing API endpoints (e.g., Equipment Catalog, RMS REST, Finance) be versioned and exposed?
- Which Google Calendar API client library or approach should be used for project scheduling integration?
- How should lead scoring weights and thresholds (GREEN/YELLOW/RED) be configured?

## Coordination Notes

- Agent owner: Grizz
- Primary stack: Go/gRPC