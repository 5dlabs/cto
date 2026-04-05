Implement task 2: Implement Equipment Catalog Service (Rex - Rust/Axum)

## Goal
Develop the Equipment Catalog API for product listing, availability, and machine-readable endpoints. This service powers the catalog, quoting, and AI agent workflows.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps":["Initialize Rust 1.75+ project with Axum 0.7, referencing the sigma1-infra-endpoints ConfigMap for POSTGRES_URL, REDIS_URL, and S3_URL.","Define Product, Category, and Availability models as per PRD.","Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout, /metrics, /health/live, /health/ready.","Integrate Redis for rate limiting and caching.","Serve product images from S3/R2 via signed URLs.","Implement admin endpoints (POST/PATCH) with RBAC (admin-only).","Add Prometheus metrics and health probes.","Ensure <500ms response time for availability checks."]}

## Acceptance Criteria
All endpoints return correct data for 533+ products and 24 categories. Rate limiting is enforced. Product images are served via S3/R2. /metrics and health endpoints are accessible. Availability checks complete in <500ms under load.

## Subtasks
- Initialize Rust/Axum project with database connection and schema migrations: Set up the Rust 1.75+ project with Axum 0.7, configure database connectivity from the sigma1-infra-endpoints ConfigMap, and create SQLx migrations for Product, Category, and Availability models.
- Implement public catalog read endpoints: Build the public read-only REST endpoints for browsing the equipment catalog: list categories, list products (with filtering/pagination), get product by ID, and check product availability.
- Implement machine-readable equipment-api endpoints: Build the machine-readable API endpoints (/equipment-api/catalog and /equipment-api/checkout) designed for AI agent consumption with structured, deterministic response formats.
- Integrate Redis for rate limiting and response caching: Add Redis integration to the Equipment Catalog service for API rate limiting and caching of catalog read responses to meet performance requirements.
- Implement S3/R2 signed URL generation for product images: Add S3-compatible signed URL generation for serving product images securely, and implement the image URL resolution in product API responses.
- Implement admin CRUD endpoints with RBAC authorization: Build admin-only endpoints for creating and updating products and categories, protected by role-based access control middleware.
- Add Prometheus metrics, health probes, and performance validation: Implement Prometheus metrics exposition, Kubernetes health probes (liveness and readiness), and validate that availability checks complete in <500ms.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.