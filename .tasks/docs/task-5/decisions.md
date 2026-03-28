## Decision Points

- Weighted algorithm and thresholds for computing the GREEN/YELLOW/RED lead score.
- Specific external APIs to mock for Business Verification, Online Presence, Reputation, and Credit Signals.
- Strategy for mocking external API responses (e.g., static JSON, dynamic based on input).
- Error handling and retry policies for external API calls (e.g., exponential backoff, circuit breakers).

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum