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
- [ ] Initialize Rust project, Axum, and define vetting data models: Set up a new Rust project for the Customer Vetting Service, configure Axum, and define `VettingResult` and `LeadScore` data models with `sqlx` migrations.
- [ ] Implement customer vetting API endpoints: Develop API endpoints for triggering the vetting pipeline and retrieving vetting results and credit signals for an organization.
- [ ] Implement business verification and online presence vetting modules: Develop the Business Verification module (mock OpenCorporates API) and Online Presence module (mock LinkedIn/website checks) as part of the vetting pipeline.
- [ ] Implement reputation analysis and credit signal vetting modules: Develop the Reputation module (mock Google Reviews sentiment) and Credit Signals module (mock commercial credit API) as part of the vetting pipeline.
- [ ] Implement final lead scoring, persistence, and error handling: Implement the weighted algorithm to compute the GREEN/YELLOW/RED lead score, store vetting results in PostgreSQL, and ensure robust error handling and retry mechanisms for external API calls.