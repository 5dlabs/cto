## Decision Points

- gRPC-Gateway REST Mapping: Decide on the specific REST endpoint mappings for gRPC services.
- Barcode Scanning Implementation: Define the initial mocking strategy for barcode scanning and future integration plan.
- Crew Scheduling Algorithm: Determine the initial complexity and approach for crew scheduling (e.g., simple availability check vs. complex optimization).
- Session Caching Strategy: Define what data to cache in Redis for sessions and its invalidation strategy.
- Inter-service Authentication: How will RMS authenticate calls to other services (e.g., Catalog, Finance) and vice-versa? (e.g., JWT, API keys, mTLS).

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go/gRPC