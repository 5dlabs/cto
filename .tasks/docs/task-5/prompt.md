Implement task 5: Develop Customer Vetting Service (Rex - Rust/Axum)

## Goal
Create the Customer Vetting Service to automate background research on prospects, providing a GREEN/YELLOW/RED lead score. This service supports Morgan's lead qualification workflow.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize a new Rust project targeting Rust 1.77.2.2. Set up Axum 0.7.5.3. Define `VettingResult` and `LeadScore` data models. Implement database migrations for these schemas.4. Implement endpoints:    - `POST /api/v1/vetting/run` (triggers the full pipeline)    - `GET /api/v1/vetting/:org_id`    - `GET /api/v1/vetting/credit/:org_id`5. Implement the vetting pipeline logic:    - **Business Verification**: Integrate with OpenCorporates API (mock API calls initially).    - **Online Presence**: Mock LinkedIn and website checks.    - **Reputation**: Mock Google Reviews sentiment analysis.    - **Credit Signals**: Mock commercial credit API integration.    - **Final Score**: Implement weighted algorithm to compute GREEN/YELLOW/RED score.6. Store vetting results in PostgreSQL using `sqlx`, referencing the `sigma1-infra-endpoints` ConfigMap.7. Ensure robust error handling and retry mechanisms for external API calls.

## Acceptance Criteria
1. Deploy the service to Kubernetes and verify it starts successfully.2. Call `POST /api/v1/vetting/run` with sample organization data and verify a `VettingResult` is stored in PostgreSQL.3. Retrieve vetting results using `GET /api/v1/vetting/:org_id` and confirm the `final_score` is correctly computed based on mock inputs.4. Verify `GET /api/v1/vetting/credit/:org_id` returns expected credit signals.5. Test error handling for failed external API calls (e.g., by simulating an API timeout or error).

## Subtasks


## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.