## Decision Points

- ScoreLead weighting formula: the relative weights for event size, venue history, customer vetting score, and lead age are not specified — a product owner or business analyst must define the numeric weights before implementation.
- Vetting service transport: the ScoreLead implementation calls the vetting service via HTTP GET. If the vetting service is not yet deployed or uses a different protocol, the RMS team needs a confirmed base URL and fallback behavior (treat as unvetted vs. fail open/closed).
- Google Calendar OAuth2 flow: the details spec uses GOOGLE_CALENDAR_CLIENT_ID/SECRET which implies a service-account or OAuth2 client credential flow — the exact credential type (service account JSON vs. OAuth2 client ID + refresh token) must be confirmed before implementation.
- DeliveryService OptimizeRoute algorithm: no routing algorithm or third-party API (e.g., Google Maps Routes API, OSRM, custom) is specified — this must be decided before the DeliveryService handler is implemented.

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go 1.22+/gRPC/grpc-gateway