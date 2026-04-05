## Build Customer Vetting Service (Rex - Rust/Axum)

### Objective
Implement the Customer Vetting Service providing automated background research on prospects via OpenCorporates, LinkedIn, Google Reviews, and credit signal APIs. Produces a weighted GREEN/YELLOW/RED lead score. Built within the shared Cargo workspace.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1, 2

### Implementation Details
1. Add service crate: `sigma1-services/services/customer-vetting/`
   - Depend on shared-auth, shared-db, shared-error, shared-observability crates
2. Database migrations in `vetting` schema:
   - `vetting_results` table: id (UUID PK), org_id (UUID), business_verified (BOOL), opencorporates_data (JSONB nullable), linkedin_exists (BOOL), linkedin_followers (INT default 0), google_reviews_rating (REAL nullable), google_reviews_count (INT default 0), credit_score (INT nullable), risk_flags (TEXT[]), final_score (VARCHAR(6): GREEN/YELLOW/RED), vetted_at (TIMESTAMPTZ), created_at
   - `vetting_requests` table: id (UUID PK), org_id, org_name, org_domain (nullable), status (pending/in_progress/completed/failed), requested_at, completed_at
   - Indexes: vetting_results(org_id), vetting_requests(org_id, status)
3. Implement Axum 0.7 endpoints:
   - `POST /api/v1/vetting/run` — accepts { org_id, org_name, org_domain? }, kicks off async vetting pipeline, returns request ID + 202 Accepted
   - `GET /api/v1/vetting/:org_id` — returns latest VettingResult for org
   - `GET /api/v1/vetting/credit/:org_id` — returns credit signals subset
   - Health and metrics endpoints
4. Vetting pipeline (async background task via tokio::spawn):
   - Step 1 — Business Verification:
     - Call OpenCorporates API `/companies/search?q={org_name}`
     - Parse response: company exists, jurisdiction, good_standing, directors
     - Store in opencorporates_data JSONB
     - Set business_verified = true if company found and in good standing
   - Step 2 — Online Presence:
     - LinkedIn company lookup via API or scraping proxy
     - Check if company page exists, follower count
     - Set linkedin_exists, linkedin_followers
   - Step 3 — Reputation:
     - Google Places API search for business, extract reviews rating and count
     - Store google_reviews_rating, google_reviews_count
   - Step 4 — Credit Signals:
     - Integration point for commercial credit API (e.g., Creditsafe)
     - Behind feature flag — if not configured, skip and log warning
     - Store credit_score if available
   - Step 5 — Final Score calculation:
     - Weighted algorithm:
       - business_verified: 30 points
       - linkedin_exists && followers > 50: 20 points
       - google_reviews_rating >= 4.0: 20 points
       - google_reviews_count >= 10: 10 points
       - credit_score >= 600: 20 points (or 10 points if unavailable but other signals positive)
     - GREEN: >= 70 points
     - YELLOW: 40-69 points
     - RED: < 40 points
     - risk_flags populated for each failed check
5. External API client modules with retry logic (tokio-retry, exponential backoff):
   - `OpenCorporatesClient` — configurable base URL, API key from ExternalSecret
   - `LinkedInClient` — configurable, gracefully degrades if API unavailable
   - `GooglePlacesClient` — uses Google Places API key
   - `CreditClient` — feature-flagged, returns None if not configured
6. All external API calls have 10-second timeouts and circuit breaker pattern (track consecutive failures, back off after 5).
7. Cache vetting results in Valkey for 24 hours (keyed by org_id) to avoid re-running expensive pipeline.
8. GDPR: implement `DELETE /api/v1/vetting/:org_id` to purge all vetting data for an organization.
9. Kubernetes Deployment: namespace `sigma1`, 1 replica (lower traffic), envFrom sigma1-infra-endpoints.

### Subtasks
- [ ] Scaffold customer-vetting service crate in Cargo workspace: Create the `sigma1-services/services/customer-vetting/` crate with Cargo.toml, main.rs, and module structure. Wire up dependencies on shared-auth, shared-db, shared-error, shared-observability workspace crates. Configure Axum 0.7 application skeleton with health endpoint and tracing initialization.
- [ ] Create database migrations for vetting schema: Write SQLx migrations to create the `vetting` schema with `vetting_results` and `vetting_requests` tables including all specified columns, types, defaults, and indexes.
- [ ] Implement OpenCorporatesClient with retry and circuit breaker: Build the OpenCorporates external API client module with reqwest, exponential backoff retry via tokio-retry, 10-second timeouts, and a circuit breaker that trips after 5 consecutive failures.
- [ ] Implement LinkedInClient with graceful degradation: Build the LinkedIn company lookup client module with retry logic, 10-second timeouts, circuit breaker, and graceful degradation when the API is unavailable or unconfigured.
- [ ] Implement GooglePlacesClient with retry and circuit breaker: Build the Google Places API client module to search for a business and extract reviews rating and count, with retry logic, 10-second timeouts, and circuit breaker.
- [ ] Implement CreditClient with feature flag gating: Build the commercial credit API client module that is feature-flagged and returns None when not configured, with retry and circuit breaker when active.
- [ ] Extract shared circuit breaker module: Extract the circuit breaker pattern into a shared module `src/clients/circuit_breaker.rs` used by all four external API clients, avoiding code duplication.
- [ ] Implement weighted scoring algorithm and risk_flags population: Build the scoring module that takes raw vetting data from all pipeline steps and computes the weighted GREEN/YELLOW/RED score with risk_flags array.
- [ ] Implement async vetting pipeline orchestrator: Build the async pipeline that orchestrates the 5 vetting steps (OpenCorporates, LinkedIn, Google Places, Credit, Scoring), updates request status, stores results, and handles partial failures gracefully.
- [ ] Implement Valkey caching layer for vetting results: Add a caching layer using Valkey (Redis-compatible) to cache vetting results for 24 hours per org_id, avoiding redundant external API calls.
- [ ] Implement API endpoints (POST run, GET result, GET credit, DELETE GDPR): Build all Axum 0.7 route handlers: POST /api/v1/vetting/run (202 Accepted with async pipeline), GET /api/v1/vetting/:org_id, GET /api/v1/vetting/credit/:org_id, DELETE /api/v1/vetting/:org_id for GDPR.
- [ ] Write comprehensive unit and integration tests: Create the full test suite covering scoring algorithm test vectors, pipeline integration with mocked APIs, cache behavior, GDPR deletion, and circuit breaker verification.
- [ ] Create Kubernetes deployment manifest for customer-vetting service: Write the Kubernetes Deployment, Service, and related manifests for the customer-vetting service in the sigma1 namespace with 1 replica and envFrom sigma1-infra-endpoints ConfigMap.