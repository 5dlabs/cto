Implement task 5: Build Customer Vetting Service (Rex - Rust/Axum)

## Goal
Develop the Customer Vetting microservice to automate business verification, online presence checks, reputation analysis, and credit scoring using external APIs.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+, Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust Axum project with PostgreSQL and required API clients (OpenCorporates, LinkedIn, Google Reviews, credit APIs).", "Define VettingResult and LeadScore models.", "Implement endpoints: /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.", "Build vetting pipeline: business verification, online presence, reputation, credit signals, scoring.", "Store results in PostgreSQL.", "Add Prometheus metrics and health checks.", "Document OpenAPI spec."]]}

## Acceptance Criteria
POST /api/v1/vetting/run completes full vetting pipeline and stores result. GET endpoints return correct vetting data. External API integrations return expected data. Final score is GREEN/YELLOW/RED as per logic.

## Subtasks
- Scaffold Rust Axum project with shared application state and configuration: Initialize the Customer Vetting Service as a Rust Axum project with configuration loading (environment variables, config files), shared application state (database pool, HTTP clients), error handling patterns, and basic Axum router setup.
- Define VettingResult and LeadScore data models with SQLx migrations: Create the PostgreSQL schema and Rust data models for vetting results, individual pipeline stage outputs, and the composite lead score.
- Implement OpenCorporates business verification pipeline stage: Build the HTTP client integration with the OpenCorporates API to verify business registration, legal status, and company details as the first stage of the vetting pipeline.
- Implement LinkedIn online presence pipeline stage: Build the HTTP client integration for checking a company's LinkedIn presence and extracting relevant signals (employee count, company age, activity) as the second vetting pipeline stage.
- Implement Google Reviews reputation analysis pipeline stage: Build the HTTP client integration with the Google Places/Reviews API to analyze business reputation as the third vetting pipeline stage.
- Implement credit signals pipeline stage: Build the HTTP client integration with a credit scoring/bureau API to retrieve credit signals as the fourth vetting pipeline stage.
- Implement composite scoring engine (GREEN/YELLOW/RED): Build the composite scoring engine that aggregates results from all four pipeline stages into a final GREEN/YELLOW/RED lead score with detailed breakdown.
- Build vetting pipeline orchestrator: Implement the pipeline orchestrator that sequentially or concurrently runs all vetting stages, collects results, invokes the scoring engine, and persists the final VettingResult.
- Implement REST endpoints for vetting operations: Create the three REST API endpoints: POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id with proper request validation and response formatting.
- Add Prometheus metrics, health checks, and OpenAPI documentation: Integrate Prometheus metrics collection, liveness/readiness health check endpoints, and generate OpenAPI specification for the vetting service.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.