Implement task 5: Build Customer Vetting Service (Rex - Rust/Axum)

## Goal
Implement the Customer Vetting microservice in Rust/Axum. Orchestrates a multi-step background research pipeline: OpenCorporates business verification, LinkedIn presence check, Google Reviews sentiment, credit signal lookup, and produces a GREEN/YELLOW/RED lead score. Persists results for Morgan and RMS to consume. Isolated from Finance per D8 decision.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize Cargo workspace at services/customer-vetting. Dependencies: axum 0.7, tokio 1.x, sqlx 0.7, reqwest 0.11 (async, json, tls), serde/serde_json, redis 0.24, uuid, chrono, anyhow, tracing, prometheus 0.13.
2. Database migrations targeting vetting schema. Tables: vetting_results (id UUID PK, org_id UUID UNIQUE, business_verified BOOL, opencorporates_data JSONB, linkedin_exists BOOL, linkedin_followers INT, google_reviews_rating FLOAT4, google_reviews_count INT, credit_score INT, risk_flags TEXT[], final_score TEXT CHECK IN (GREEN,YELLOW,RED), vetted_at TIMESTAMPTZ, updated_at TIMESTAMPTZ), vetting_requests (id UUID PK, org_id UUID, status TEXT CHECK IN (pending,running,completed,failed), created_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, error_text TEXT).
3. POST /api/v1/vetting/run: accepts {org_id, company_name, jurisdiction_code, website_url?}. Creates vetting_request row with status=pending, spawns tokio::spawn pipeline, returns 202 Accepted with request_id.
4. Vetting pipeline (async, runs in background task): Step 1 — OpenCorporates: GET https://api.opencorporates.com/v0.4/companies/search?q={name}&jurisdiction_code={code}&api_token=OPENCORPORATES_API_KEY. Parse company status, registered address, directors. Step 2 — LinkedIn: GET LinkedIn API /v2/companies?q=universalName&universalName={slug} using LINKEDIN_CLIENT_ID/SECRET OAuth2 client_credentials flow. Check followers, employee count. Step 3 — Google Reviews: GET Places API /maps/api/place/findplacefromtext with name, then GET /maps/api/place/details for rating and user_ratings_total. Step 4 — Credit signals: stub with mock data (real credit API integration deferred — use Equifax/Dun&Bradstreet in Phase 2); log YELLOW flag for stubs. Step 5 — Scoring: business_verified(40%) + linkedin_score(20%) + reviews_score(20%) + credit_score(20%) → weighted sum → GREEN (>=70), YELLOW (40-69), RED (<40). Update vetting_results and vetting_request status=completed.
5. GET /api/v1/vetting/:org_id returns vetting_result or 404. GET /api/v1/vetting/credit/:org_id returns credit_score subset.
6. Cache completed vetting results in Valkey with 24h TTL (key: vetting:org:{org_id}) to avoid repeated API calls for same org.
7. Error handling: each pipeline step wrapped in timeout (10s per step), failure sets risk_flag and continues to next step. If all external steps fail, return YELLOW with flags.
8. JWT middleware, /metrics, /health/live (always 200), /health/ready (postgres + redis ping). GDPR: GET /internal/gdpr/export/:org_id returns vetting_results row; DELETE /internal/gdpr/delete/:org_id deletes vetting_results and vetting_requests rows, logs deletion to audit schema. Kubernetes Deployment 2 replicas.

## Acceptance Criteria
1. POST /api/v1/vetting/run returns 202 with request_id within 1 second (pipeline runs async). 2. After async pipeline completes (poll GET /api/v1/vetting/:org_id up to 30s), response contains final_score of GREEN, YELLOW, or RED and vetted_at timestamp. 3. Scoring unit test: business_verified=true, linkedin_followers=500, google_reviews_rating=4.2, google_reviews_count=20, credit_score=700 should produce GREEN. 4. If OpenCorporates returns 404 for org, risk_flags contains business_not_found and final_score is RED or YELLOW. 5. Second call to POST /api/v1/vetting/run for same org_id within 24h returns cached result (verify by checking Valkey key exists via redis-cli GET vetting:org:{id}). 6. DELETE /internal/gdpr/delete/:org_id returns 200; subsequent GET /api/v1/vetting/:org_id returns 404. 7. GET /health/ready returns 200. 8. cargo test passes with >= 80% coverage.

## Subtasks
- Initialize Cargo workspace and declare all dependencies: Create the Cargo workspace at services/customer-vetting with a single crate. Add all required dependencies to Cargo.toml: axum 0.7, tokio 1.x (full features), sqlx 0.7 (postgres, uuid, chrono, migrate), reqwest 0.11 (json, tls), serde/serde_json, redis 0.24 (tokio-comp), uuid (v4), chrono, anyhow, tracing, tracing-subscriber, prometheus 0.13.
- Write sqlx database migrations for vetting schema: Create sqlx migration files under services/customer-vetting/migrations/ targeting the vetting PostgreSQL schema. Migrations must create both the vetting_requests and vetting_results tables with all columns, constraints, and indexes described in the task details.
- Implement POST /api/v1/vetting/run handler with async pipeline orchestration: Implement the Axum route POST /api/v1/vetting/run. The handler accepts {org_id, company_name, jurisdiction_code, website_url?}, inserts a vetting_requests row with status=pending, spawns a tokio::spawn background task for the pipeline, and immediately returns HTTP 202 Accepted with the request_id. Also implement GET /api/v1/vetting/:org_id and GET /api/v1/vetting/credit/:org_id retrieval endpoints.
- Implement OpenCorporates and LinkedIn pipeline steps with per-step timeout: Implement the first two steps of the vetting background pipeline: OpenCorporates company search and LinkedIn company presence check. Each step must complete within a 10-second timeout; on timeout or HTTP error, set the corresponding risk_flag and continue to the next step.
- Implement Google Reviews pipeline step and weighted scoring algorithm: Implement the third pipeline step (Google Places API for reviews) and the weighted scoring function that aggregates all four step results into a GREEN/YELLOW/RED final_score. The scoring logic must be unit-testable independently of HTTP calls.
- Implement Valkey result caching with 24h TTL: After the pipeline completes and writes to vetting_results, serialize the result and store it in Valkey under key vetting:org:{org_id} with a 24-hour TTL. On POST /api/v1/vetting/run, check the cache first; if a cached result exists and is not stale, skip the pipeline and return the cached request_id.
- Implement JWT middleware, health endpoints, Prometheus metrics, and GDPR endpoints: Wire up JWT authentication middleware for all non-internal routes, implement /health/live, /health/ready, /metrics, and the GDPR export/delete internal endpoints. Configure Kubernetes Deployment manifest for 2 replicas.
- Write integration and unit test suite targeting >= 80% coverage: Write the full cargo test suite covering the scoring algorithm unit tests, async pipeline integration tests with mocked HTTP, cache hit/miss behavior, GDPR deletion flow, and async 202 polling convergence.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.