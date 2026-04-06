Implement task 2: Implement Equipment Catalog Service API (Rex - Rust/Axum)

## Goal
Develop the Equipment Catalog backend service to provide product listings, availability checks, and machine-readable APIs for Morgan and the website.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust Axum project with PostgreSQL and Redis clients using connection strings from 'sigma1-infra-endpoints' ConfigMap", "Define Product, Category, and Availability models as per PRD", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout, /metrics, /health/live, /health/ready", "Integrate S3/R2 for image URLs in product responses", "Add rate limiting middleware using Redis", "Ensure endpoints are filterable and support pagination", "Write Prometheus metrics and health probes", "Document OpenAPI spec for all endpoints"]}

## Acceptance Criteria
Call each endpoint and verify correct data structure, filtering, and pagination; check rate limiting is enforced; confirm Prometheus metrics and health endpoints respond as expected; verify image URLs resolve to S3/R2 objects.

## Subtasks
- Scaffold Rust/Axum project with infrastructure client setup and health endpoints: Initialize the Equipment Catalog Rust project with Axum 0.7 scaffolding, PostgreSQL (sqlx) and Redis client pools configured from the sigma1-infra-endpoints ConfigMap, and implement /health/live, /health/ready, and /metrics endpoints.
- Define data models and create database migrations: Define Product, Category, and Availability domain models in Rust and create sqlx database migrations for the catalog schema tables in the rms PostgreSQL schema.
- Implement public catalog CRUD endpoints with filtering and pagination: Implement the public-facing catalog API endpoints: GET /api/v1/catalog/categories, GET /api/v1/catalog/products (with filtering and pagination), GET /api/v1/catalog/products/:id, and GET /api/v1/catalog/products/:id/availability.
- Implement machine-readable Morgan API endpoints with rate limiting: Implement the machine-readable API endpoints for the Morgan AI agent (/api/v1/equipment-api/catalog and /api/v1/equipment-api/checkout) and add Redis-based rate limiting middleware.
- Generate OpenAPI specification and write integration tests: Document all Equipment Catalog API endpoints with an OpenAPI 3.0 specification and write comprehensive integration tests covering all endpoints, filtering, pagination, error cases, and rate limiting.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.