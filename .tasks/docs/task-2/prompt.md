Implement task 2: Implement Equipment Catalog Service (Rex - Rust/Axum)

## Goal
Develop the Equipment Catalog microservice with all specified endpoints, real-time availability, and S3/R2 image serving. Integrate with PostgreSQL and Redis for data and caching.

## Task Context
- Agent owner: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust project with Axum 0.7, sqlx for PostgreSQL, and redis-rs for Redis integration.", "Define Product, Category, and Availability models as per PRD.", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout, /metrics, /health/live, /health/ready.", "Integrate S3/R2 for image URLs and serve via CDN.", "Implement rate limiting per tenant using Redis.", "Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.", "Write Prometheus metrics and health probes.", "Ensure all endpoints validate input and output schemas."]}

## Acceptance Criteria
All endpoints return correct data and status codes; product images are served via CDN; rate limiting is enforced; health and metrics endpoints are Prometheus-compatible; response times for availability check < 500ms.

## Subtasks
- Scaffold Rust/Axum project with dependencies and configuration: Initialize the Rust project with Cargo, configure Axum 0.7, sqlx (PostgreSQL), redis-rs, and set up the application configuration layer to consume environment variables from the sigma1-infra-endpoints ConfigMap.
- Define database models and create sqlx migrations for rms schema: Define the Product, Category, and Availability domain models in Rust and create sqlx migrations for the corresponding tables in the PostgreSQL rms schema.
- Implement core CRUD API endpoints for categories and products: Implement the category listing, product listing with filtering/pagination, and single product detail endpoints with input validation and JSON serialization.
- Implement real-time availability endpoint: Implement the product availability checking endpoint that queries current availability data and returns date-range availability for a given product.
- Implement S3/R2 image URL generation and CDN integration: Implement the logic to generate public CDN-backed URLs for product images stored in S3/R2, including pre-signed URL generation if needed for uploads.
- Implement Redis-based rate limiting per tenant: Implement rate limiting middleware using Redis sliding window counters, keyed by tenant ID, to protect all API endpoints.
- Implement Prometheus metrics and health probe endpoints: Add /metrics endpoint exposing Prometheus-compatible metrics and /health/live, /health/ready endpoints for Kubernetes probes.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.