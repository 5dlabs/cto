## Build Customer Vetting Service (Rex - Rust/Axum)

### Objective
Develop the customer vetting pipeline for business verification, online presence, reputation, and credit signals, integrating with OpenCorporates, LinkedIn, Google Reviews, and credit APIs.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps":["Initialize Rust 1.75+ project with Axum 0.7, using POSTGRES_URL and API keys from ConfigMap/secrets.","Define VettingResult and LeadScore models as per PRD.","Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.","Integrate OpenCorporates, LinkedIn, Google Reviews, and credit APIs for data aggregation.","Implement scoring algorithm for GREEN/YELLOW/RED.","Persist vetting results in PostgreSQL.","Add Prometheus metrics and health endpoints."]}

### Subtasks
- [ ] Initialize Rust/Axum project and database schema for vetting models: Scaffold the Rust 1.75+ project with Axum 0.7, configure database connectivity using POSTGRES_URL from ConfigMap, and create SQLx migrations for VettingResult and LeadScore tables in the vetting schema.
- [ ] Implement OpenCorporates API integration module: Build a dedicated Rust module for querying the OpenCorporates API to retrieve business registration, incorporation status, officers, and filing data for a given organization.
- [ ] Implement LinkedIn data integration module: Build a Rust module for retrieving company online presence and profile data from LinkedIn (via selected API or enrichment provider) for organization vetting.
- [ ] Implement Google Reviews API integration module: Build a Rust module for fetching Google Reviews data (ratings, review count, sentiment) for a business using the Google Places API.
- [ ] Implement credit API integration module: Build a Rust module for fetching business credit signals (credit score, payment history, risk indicators) from the selected credit data provider.
- [ ] Implement GREEN/YELLOW/RED scoring algorithm with aggregation logic: Build the scoring engine that aggregates data from all four external sources (OpenCorporates, LinkedIn, Google Reviews, credit) and computes a composite vetting score classified as GREEN, YELLOW, or RED.
- [ ] Implement vetting REST endpoints and orchestration pipeline: Build the Axum REST endpoints (/api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id) and the orchestration logic that calls all integration modules, runs scoring, and persists results.
- [ ] Add Prometheus metrics and health endpoints: Implement Prometheus metrics exposition and health/readiness probe endpoints for the vetting service.
- [ ] End-to-end vetting pipeline integration tests: Write comprehensive integration tests that validate the full vetting pipeline from API request through external integrations (mocked), scoring, persistence, and retrieval.