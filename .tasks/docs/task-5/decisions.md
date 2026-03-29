## Decision Points

- External API Mocking Strategy: Define the specific mocking framework or approach for OpenCorporates, LinkedIn, Google Reviews, and commercial credit APIs.
- Lead Scoring Algorithm Weights: Determine the exact weighting and thresholds for each vetting signal to compute the GREEN/YELLOW/RED score.
- Data Retention Policy: Define how long vetting results are stored and any data privacy considerations.
- Inter-service Authentication: How will the Vetting service authenticate calls from other services (e.g., RMS, AI Agent)? (e.g., JWT, API keys, mTLS).

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum