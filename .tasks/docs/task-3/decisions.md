## Decision Points

- Lead scoring algorithm: What are the exact thresholds and weighting factors for GREEN/YELLOW/RED scoring? The PRD mentions customer vetting data, event size, and payment history but doesn't specify numeric boundaries or relative weights. This needs product input before implementation.
- Google Calendar integration scope: Should calendar sync be bidirectional (changes in Google Calendar reflected back into RMS) or unidirectional (RMS pushes events to Google Calendar only)? Bidirectional adds significant webhook/polling complexity.
- Delivery route optimization: The OptimizeRoute RPC implies a routing algorithm — should this use an external API (Google Directions, OSRM, Mapbox) or a simple heuristic? External APIs have cost and latency implications.
- Multi-tenancy enforcement: org_id column is specified for row-level filtering, but should this be enforced via PostgreSQL Row-Level Security policies or purely in application code? RLS is more secure but adds migration complexity.

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go 1.22+/gRPC