## Decision Points

- Stripe Integration Depth: Define the exact scope of Stripe integration (e.g., recording payments vs. full payment processing, webhook handling).
- RMS Integration Protocol: How will the Finance service communicate with the RMS service to fetch project details (e.g., gRPC, REST, message queue)?
- Financial Report Data Sources: Clarify which data sources are used for each financial report and how data aggregation occurs.
- Inter-service Authentication: How will Finance authenticate calls to RMS and vice-versa? (e.g., JWT, API keys, mTLS).

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum