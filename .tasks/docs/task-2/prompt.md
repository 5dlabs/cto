Implement task 2: Implement Equipment Catalog Service API (Rex - Rust/Axum)

## Goal
Develop the Equipment Catalog microservice with endpoints for product/category listing, availability checks, and machine-readable APIs for Morgan and other agents. Supports 533+ products and real-time queries.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+, Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust Axum project with PostgreSQL and Redis clients using connection strings from ConfigMap.", "Define data models for Product, Category, and Availability as per PRD.", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout.", "Integrate S3/R2 for image URLs in product responses.", "Add rate limiting middleware using Redis.", "Implement Prometheus metrics and health probes.", "Seed database with 533+ products and 24 categories.", "Document OpenAPI spec for all endpoints."]}

## Acceptance Criteria
All endpoints return correct data for seeded products/categories. Availability check returns <500ms for 95th percentile. Rate limiting enforced per tenant. Prometheus metrics and health endpoints respond as expected.

## Subtasks
- Initialize Rust Axum project scaffold with PostgreSQL client: Create the Rust workspace and Axum 0.7 application skeleton with PostgreSQL connection pool (sqlx or deadpool-postgres) configured from the infra ConfigMap environment variables. Set up the project structure with layered architecture (handlers, services, repositories, models).
- Initialize Redis client for caching and rate limiting: Add Redis/Valkey client to the Axum application, reading connection parameters from the infra ConfigMap, and make the connection pool available in Axum shared state.
- Define data models and database migrations for Product, Category, and Availability: Create SQLx migrations for the catalog schema including categories, products, and availability tables. Define corresponding Rust structs with serde Serialize/Deserialize derives.
- Seed database with 533+ products and 24 categories: Create a seed script or migration that populates the catalog database with all 24 equipment categories and 533+ products with realistic rental equipment data.
- Implement repository layer for catalog queries: Create repository modules with database query functions for categories, products (list, detail, search), and availability lookups using sqlx.
- Implement core catalog REST endpoints (categories, products, availability): Build Axum route handlers for the public-facing catalog API: category listing, product listing with pagination/filtering, product detail, and product availability check.
- Implement machine-readable agent API endpoints (equipment-api/catalog and checkout): Build the agent-facing API endpoints at /api/v1/equipment-api/catalog and /api/v1/equipment-api/checkout designed for Morgan and other AI agents to consume programmatically.
- Integrate S3/R2 image URL generation in product responses: Add a service layer that constructs fully qualified image URLs for product and category images using the configured object storage (S3 or R2) base URL.
- Implement rate limiting middleware using Redis: Add an Axum middleware layer that enforces per-tenant or per-IP rate limiting using Redis as the backing store with a sliding window or token bucket algorithm.
- Implement Prometheus metrics and health probe endpoints: Add /metrics endpoint exporting Prometheus-format metrics and /healthz, /readyz endpoints for Kubernetes liveness and readiness probes.
- Generate OpenAPI specification for all catalog endpoints: Document all API endpoints with an OpenAPI 3.0 specification, either auto-generated from code using utoipa or manually authored, and serve it at a discovery endpoint.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.