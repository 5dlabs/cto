## Build Customer Vetting Service (Rex - Rust/Axum)

### Objective
Develop the Customer Vetting backend to automate business verification, online presence checks, reputation analysis, and credit scoring.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust Axum service with PostgreSQL and external API keys from 'sigma1-infra-endpoints'", "Define VettingResult and LeadScore models as per PRD", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id", "Integrate OpenCorporates, LinkedIn, Google Reviews, and credit APIs for data aggregation", "Implement pipeline: business verification, online presence, reputation, credit signals, scoring", "Store results in PostgreSQL and expose via API", "Write unit and integration tests for all pipeline stages"]}

### Subtasks
- [ ] Initialize Rust/Axum project with PostgreSQL connectivity and configuration: Scaffold the Rust Axum 0.7 service, configure connection to PostgreSQL and load external API keys from the 'sigma1-infra-endpoints' ConfigMap via envFrom. Set up project structure with modules for models, handlers, services, and integrations.
- [ ] Define VettingResult and LeadScore data models and database migrations: Create the VettingResult and LeadScore domain models as Rust structs with serde and sqlx derives, and write SQL migrations to create the corresponding PostgreSQL tables.
- [ ] Implement OpenCorporates business verification integration: Build the HTTP client integration for the OpenCorporates API to perform business entity verification, including company search, registration status, and officer lookups.
- [ ] Implement LinkedIn online presence check integration: Build the HTTP client integration for LinkedIn API to assess the online presence and legitimacy of a business entity.
- [ ] Implement Google Reviews reputation analysis integration: Build the HTTP client integration for Google Reviews/Places API to analyze business reputation based on review data.
- [ ] Implement credit API integration for credit signal retrieval: Build the HTTP client integration for the selected credit scoring API to retrieve business credit signals and financial health indicators.
- [ ] Implement vetting pipeline orchestration and scoring algorithm: Build the vetting pipeline that orchestrates all four verification stages (business verification, online presence, reputation, credit) and combines their results into a final GREEN/YELLOW/RED lead score.
- [ ] Implement API endpoints for vetting operations: Build the three Axum HTTP endpoints: POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, GET /api/v1/vetting/credit/:org_id with request validation and proper error responses.
- [ ] Write comprehensive integration tests with mocked external APIs: Create end-to-end integration tests that exercise the full vetting pipeline using mocked external API responses, verifying correct data flow from request through pipeline to database storage.