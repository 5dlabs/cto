## Implement Equipment Catalog Service API (Rex - Rust/Axum)

### Objective
Develop the Equipment Catalog Service with endpoints for product/category listing, product details, availability checks, and machine-readable APIs for Morgan and other agents.

### Ownership
- Agent: Rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust 1.75+ Axum 0.7 project with Effect integration for schema validation.", "Define Product, Category, and Availability models as per PRD.", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout.", "Integrate with PostgreSQL for catalog and availability data (use connection string from ConfigMap).", "Integrate Redis for rate limiting and caching.", "Implement S3/R2 image serving for product images.", "Add Prometheus metrics and health endpoints.", "Enforce <500ms response time for availability checks.", "Add tenant-based rate limiting.", "Document OpenAPI spec for endpoints."]}

### Subtasks
- [ ] Initialize Rust/Axum project with database connection pooling and configuration: Scaffold the Rust 1.75+ Axum 0.7 project with Cargo workspace structure, configure environment-based settings loading from the sigma1-infra-endpoints ConfigMap, and set up PostgreSQL connection pooling using sqlx or deadpool-postgres.
- [ ] Define database schema and domain models for Product, Category, and Availability: Create SQL migration files for the catalog schema (products, categories, availability tables) and corresponding Rust struct models with sqlx FromRow derives and serde serialization.
- [ ] Implement category and product listing/detail CRUD endpoints: Implement the core REST endpoints for categories listing, products listing with filtering/pagination, and product detail retrieval.
- [ ] Implement availability check endpoint with performance optimization: Implement the GET /api/v1/catalog/products/:id/availability endpoint with a strict <500ms p95 response time requirement, using optimized queries and Redis caching.
- [ ] Implement S3/R2 image URL generation for product images: Implement image URL construction and optional pre-signed URL generation for product images stored in S3/R2, integrated into product detail responses.
- [ ] Integrate Redis client for caching and rate limiting infrastructure: Set up the Redis connection pool and create reusable caching and rate limiting utility modules that other handlers will consume.
- [ ] Implement machine-readable equipment-api endpoints for Morgan agent: Implement the /api/v1/equipment-api/catalog and /api/v1/equipment-api/checkout endpoints designed for consumption by the Morgan AI agent and other automated systems.
- [ ] Add Prometheus metrics and health check endpoints: Implement /health, /ready, and /metrics endpoints for Kubernetes probes and Prometheus scraping.
- [ ] Generate OpenAPI specification and integration tests: Generate an OpenAPI 3.0 specification for all catalog endpoints and write integration tests verifying endpoint behavior against the spec.