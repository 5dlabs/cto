Implement task 5: Build Customer Vetting Service (Rex - Rust/Axum)

## Goal
Create the Customer Vetting service to automate background checks using OpenCorporates, LinkedIn, Google Reviews, and credit APIs. Store results in PostgreSQL.

## Task Context
- Agent owner: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust project with Axum 0.7, sqlx for PostgreSQL.", "Define VettingResult and LeadScore models as per PRD.", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.", "Integrate with OpenCorporates, LinkedIn, Google Reviews, and credit APIs using stored secrets.", "Implement scoring algorithm for GREEN/YELLOW/RED.", "Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.", "Implement Prometheus metrics and health endpoints.", "Validate all input and output schemas."]}

## Acceptance Criteria
Vetting pipeline completes and stores results; integrations with all external APIs succeed; scoring is correct; endpoints return expected data; vetting completes within 10 seconds for simple cases.

## Subtasks
- Scaffold Rust/Axum project with sqlx, models, and database migrations: Initialize the Customer Vetting Rust project with Axum 0.7 and sqlx. Define VettingResult and LeadScore domain models as per PRD. Create PostgreSQL migrations for vetting_results and lead_scores tables. Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.
- Implement OpenCorporates API integration client: Build an async HTTP client module for querying the OpenCorporates API to retrieve company registration data, officer information, and filings for a given organization.
- Implement LinkedIn API integration client: Build an async HTTP client module for querying LinkedIn data to retrieve company profile information and employee signals for vetting purposes.
- Implement Google Reviews API integration client: Build an async HTTP client module for querying Google Places/Reviews API to retrieve business reviews, ratings, and reputation signals.
- Implement credit check API integration client: Build an async HTTP client module for querying a credit/financial data API to retrieve credit scores and financial health indicators for organizations.
- Implement GREEN/YELLOW/RED scoring algorithm: Build the scoring algorithm that combines signals from all four external API sources (OpenCorporates, LinkedIn, Google Reviews, credit) into a composite LeadScore with a GREEN/YELLOW/RED rating.
- Wire up API endpoints and orchestrate vetting pipeline: Implement the three REST endpoints (/api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id) that orchestrate the full vetting pipeline, persist results, and serve stored vetting data.
- Add Prometheus metrics and request/response schema validation: Instrument the service with Prometheus metrics for vetting pipeline observability and ensure all input/output schemas are validated.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.