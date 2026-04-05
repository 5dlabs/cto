Implement task 5: Build Customer Vetting Service (Rex - Rust/Axum)

## Goal
Develop the Customer Vetting Service to automate background checks, business verification, online presence, reputation, and credit signals.

## Task Context
- Agent owner: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust project with Axum 0.7, connect to PostgreSQL using ConfigMap.", "Define VettingResult and LeadScore models as per PRD.", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.", "Integrate OpenCorporates, LinkedIn, Google Reviews, and credit APIs using stored secrets.", "Implement vetting pipeline: business verification, online presence, reputation, credit signals, scoring.", "Store results in PostgreSQL.", "Add Prometheus metrics and health endpoints.", "Document OpenAPI spec for all endpoints."]}

## Acceptance Criteria
Vetting pipeline runs end-to-end and stores results; endpoints return correct vetting data; integrations with all external APIs succeed; >80% code coverage on vetting logic.

## Subtasks
- Initialize Rust/Axum project with PostgreSQL connectivity and database migrations: Scaffold the Rust project with Axum 0.7, configure PostgreSQL connection using the infra ConfigMap (envFrom), set up SQLx or Diesel for database access, and create migrations for VettingResult and LeadScore tables.
- Implement OpenCorporates business verification integration: Build the HTTP client module for the OpenCorporates API to perform business entity verification, including company lookup, officer search, and filing status checks.
- Implement LinkedIn online presence integration: Build the integration module for LinkedIn API to assess a company's online presence including company page data, follower count, and activity signals.
- Implement Google Reviews reputation scoring integration: Build the integration module for Google Places/Reviews API to assess a company's reputation based on review ratings, volume, and recency.
- Implement credit signal API integration: Build the integration module for credit signal checking, implementing the provider trait against the selected credit API to retrieve commercial credit scores and payment history.
- Implement vetting pipeline orchestration and composite scoring algorithm: Build the vetting pipeline that orchestrates all four integration modules (business verification, online presence, reputation, credit signals) and computes the composite LeadScore.
- Implement REST endpoints for vetting operations: Build the Axum route handlers for POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id with request validation and error handling.
- Add Prometheus metrics and OpenAPI documentation: Instrument the vetting service with Prometheus metrics for observability and generate an OpenAPI specification documenting all endpoints.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.