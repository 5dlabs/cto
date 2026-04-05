Implement task 5: Build Customer Vetting Service (Rex - Rust/Axum)

## Goal
Develop the Customer Vetting service to automate background checks, business verification, and risk scoring using OpenCorporates, LinkedIn, Google Reviews, and credit APIs. Supports Morgan's lead qualification.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust 1.75+ project with Axum 0.7 and sqlx for PostgreSQL.", "Define VettingResult and LeadScore models as per PRD.", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.", "Integrate with OpenCorporates, LinkedIn, Google Reviews, and credit APIs using API keys from secrets.", "Implement the full vetting pipeline: business verification, online presence, reputation, credit signals, and scoring.", "Reference connection strings from 'sigma1-infra-endpoints' ConfigMap via envFrom.", "Write unit and integration tests for all endpoints and external API integrations."]}

## Acceptance Criteria
Vetting pipeline runs end-to-end and returns correct scores; all external API integrations are functional; tests cover at least 80% of code paths.

## Subtasks
- Scaffold Rust/Axum project with dependencies and infrastructure wiring: Initialize the Rust 1.75+ project with Axum 0.7, sqlx, reqwest (for external HTTP calls), serde, and tokio. Configure Cargo.toml with workspace structure. Set up the Axum router skeleton, health check endpoint, and environment configuration to read connection strings and API keys from the 'sigma1-infra-endpoints' ConfigMap via envFrom and Kubernetes secrets.
- Define data models and create PostgreSQL migrations for VettingResult and LeadScore: Design and implement the VettingResult and LeadScore domain models as Rust structs with serde and sqlx derives. Create sqlx migrations for the vetting-related tables in the appropriate schema (e.g., crm schema). Include fields for business verification status, reputation scores, credit signals, composite risk score, and timestamps.
- Implement OpenCorporates business verification integration module: Build a standalone Rust module that integrates with the OpenCorporates API to verify business registration, retrieve incorporation details, officer information, and filing status. Implement proper error handling, response parsing, and a trait-based interface for testability.
- Implement LinkedIn online presence integration module: Build a standalone Rust module that integrates with the LinkedIn API to assess a company's online presence, including company page existence, follower count, employee count, and recent activity signals.
- Implement Google Reviews reputation integration module: Build a standalone Rust module that integrates with a Google Reviews data source (Google Places API or commercial alternative) to assess a company's reputation based on review count, average rating, and sentiment signals.
- Implement credit signals integration module: Build a standalone Rust module that integrates with a credit/financial data API to retrieve credit signals for a business, including credit score, payment history indicators, and financial risk level.
- Implement vetting pipeline orchestration and composite risk scoring: Build the core vetting pipeline that orchestrates all four integration modules (OpenCorporates, LinkedIn, Google Reviews, Credit) in parallel, aggregates their results, computes a composite risk score, derives a lead qualification tier, and persists VettingResult and LeadScore to the database.
- Implement API endpoints for vetting service: Implement the three Axum HTTP endpoints: POST /api/v1/vetting/run (trigger a vetting pipeline run), GET /api/v1/vetting/:org_id (retrieve vetting results for an organization), and GET /api/v1/vetting/credit/:org_id (retrieve credit-specific data for an organization). Wire up the pipeline and database queries to the router.
- Write comprehensive integration and end-to-end tests for vetting service: Create a full test suite covering the vetting service end-to-end, including integration tests against a test PostgreSQL database with mocked external APIs, and ensure at least 80% code path coverage.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.