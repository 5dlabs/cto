## Develop Customer Vetting Service (Rex - Rust/Axum)

### Objective
Create the Customer Vetting Service to automate background research on prospects, providing a GREEN/YELLOW/RED lead score. This service supports Morgan's lead qualification workflow.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize a new Rust project targeting Rust 1.77.2.2. Set up Axum 0.7.5.3. Define `VettingResult` and `LeadScore` data models. Implement database migrations for these schemas.4. Implement endpoints:    - `POST /api/v1/vetting/run` (triggers the full pipeline)    - `GET /api/v1/vetting/:org_id`    - `GET /api/v1/vetting/credit/:org_id`5. Implement the vetting pipeline logic:    - **Business Verification**: Integrate with OpenCorporates API (mock API calls initially).    - **Online Presence**: Mock LinkedIn and website checks.    - **Reputation**: Mock Google Reviews sentiment analysis.    - **Credit Signals**: Mock commercial credit API integration.    - **Final Score**: Implement weighted algorithm to compute GREEN/YELLOW/RED score.6. Store vetting results in PostgreSQL using `sqlx`, referencing the `sigma1-infra-endpoints` ConfigMap.7. Ensure robust error handling and retry mechanisms for external API calls.

### Subtasks
