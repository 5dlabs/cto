Implement subtask 5005: Implement final lead scoring, persistence, and error handling

## Objective
Implement the weighted algorithm to compute the GREEN/YELLOW/RED lead score, store vetting results in PostgreSQL, and ensure robust error handling and retry mechanisms for external API calls.

## Steps
1. Implement the weighted algorithm to combine results from all vetting modules and compute the final `LeadScore`.2. Store the complete `VettingResult` (including `LeadScore`) in PostgreSQL using `sqlx`, referencing the `sigma1-infra-endpoints` ConfigMap.3. Implement error handling and retry logic for all external API calls (mocked or real).

## Validation
1. Call `POST /api/v1/vetting/run` with various mock inputs and verify the `final_score` is correctly computed and persisted in PostgreSQL.2. Simulate API failures (e.g., network errors, 500s) and verify the service handles them gracefully with retries or appropriate error responses.