## Build Customer Vetting Service (Rex - Rust/Axum)

### Objective
Develop the Customer Vetting Service to automate business verification, online presence checks, reputation analysis, and credit scoring, integrating with OpenCorporates, LinkedIn, Google Reviews, and credit APIs.

### Ownership
- Agent: Rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust 1.75+ Axum 0.7 project.", "Define VettingResult and LeadScore models as per PRD.", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.", "Integrate with PostgreSQL for vetting results.", "Implement pipeline: OpenCorporates API, LinkedIn API, Google Reviews (scraping/API), credit data APIs.", "Aggregate results and compute final GREEN/YELLOW/RED score.", "Add Prometheus metrics and health endpoints.", "Document OpenAPI spec for endpoints."]}

### Subtasks
- [ ] Initialize Rust/Axum project with data models and PostgreSQL schema: Set up the Rust 1.75+ Axum 0.7 project structure for the Customer Vetting Service, define VettingResult and LeadScore domain models, and create the PostgreSQL migration for storing vetting results.
- [ ] Implement OpenCorporates API integration for business verification: Build the OpenCorporates API client module to look up company registration data, verify incorporation status, and return structured business verification results.
- [ ] Implement LinkedIn API integration for online presence analysis: Build the LinkedIn API client module to retrieve company profile data including follower count, employee count, and company description for online presence scoring.
- [ ] Implement Google Reviews integration for reputation analysis: Build the Google Reviews client module to fetch review data, average ratings, and review counts for an organization, and compute a reputation summary.
- [ ] Implement credit scoring API integration: Build the credit data API client module to retrieve business credit scores and risk assessments for organizations.
- [ ] Build aggregation pipeline and scoring engine (GREEN/YELLOW/RED): Implement the vetting pipeline that orchestrates all data source integrations concurrently, aggregates results, and computes the final GREEN/YELLOW/RED lead score.
- [ ] Add Prometheus metrics, health endpoints, and OpenAPI documentation: Instrument the vetting service with Prometheus metrics for observability, add health/readiness endpoints, and generate OpenAPI documentation for all endpoints.