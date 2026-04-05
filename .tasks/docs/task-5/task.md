## Build Customer Vetting Service (Rex - Rust/Axum)

### Objective
Build the Customer Vetting Service that runs automated background research on prospects through a multi-stage pipeline: business verification (OpenCorporates), online presence (LinkedIn), reputation (Google Reviews), credit signals, and final GREEN/YELLOW/RED scoring.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Add `vetting` crate to existing Cargo workspace at `services/rust/vetting`.
2. SQLx migrations for `vetting` schema:
   - `vetting_results` table: org_id (UUID PK or FK), business_verified (BOOL), opencorporates_data (JSONB), linkedin_exists (BOOL), linkedin_followers (INT), google_reviews_rating (FLOAT), google_reviews_count (INT), credit_score (INT nullable), risk_flags (TEXT[]), final_score (VARCHAR(6) — GREEN/YELLOW/RED), vetted_at (TIMESTAMPTZ), raw_responses (JSONB — store all API responses for audit).
   - `vetting_requests` table: id, org_id, requested_by, status (pending/running/completed/failed), started_at, completed_at, error_message.
3. Implement REST endpoints:
   - `POST /api/v1/vetting/run` — accepts org_id and company_name/domain. Spawns async vetting pipeline. Returns 202 Accepted with request ID.
   - `GET /api/v1/vetting/:org_id` — returns latest VettingResult or 404.
   - `GET /api/v1/vetting/credit/:org_id` — returns credit signals subset.
4. Vetting pipeline (5 stages, executed concurrently where possible):
   - **Stage 1 — Business Verification**: Call OpenCorporates API (`/companies/search`). Verify company exists, check incorporation status, extract directors. API key from `sigma1-external-apis` secret.
   - **Stage 2 — Online Presence**: Check LinkedIn Company API (if available) or fallback to LinkedIn company page scraping via headless request. Check for website existence via DNS/HTTP probe.
   - **Stage 3 — Reputation**: Query Google Business Profile API (preferred) or Google Places API for reviews and rating. Fallback: structured Google search scraping. Extract rating and review count.
   - **Stage 4 — Credit Signals**: Call commercial credit API (design as a trait `CreditProvider` with pluggable implementations). Initial implementation can be a stub returning UNKNOWN if no API selected (per open question #4). Log that credit data is unavailable.
   - **Stage 5 — Final Scoring**: Weighted algorithm:
     - business_verified: 30 points
     - linkedin_exists && followers > 50: 20 points
     - google_reviews_rating >= 4.0 && count >= 10: 20 points
     - credit_score > 650: 20 points (or 10 if unavailable)
     - No risk_flags: 10 points
     - Total >= 70 → GREEN, 40-69 → YELLOW, < 40 → RED
5. Each external API call uses `reqwest` with:
   - Timeout: 10 seconds per call
   - Retry: 2 retries with exponential backoff
   - Circuit breaker pattern: if API fails 5 times in 5 minutes, mark stage as unavailable rather than failing entire pipeline
6. Store all raw API responses in `raw_responses` JSONB for audit trail.
7. GDPR endpoint: `DELETE /api/v1/gdpr/customer/:id` — delete all vetting data for org_id, return confirmation.
8. Reuse shared crate: health, metrics, rate limiting, DB pool, API key auth.
9. OpenAPI spec via `utoipa` at `/api/v1/vetting/openapi.json`.
10. Dockerfile + Kubernetes Deployment: namespace sigma1, replicas 2, envFrom ConfigMap, secret refs, port 8083.

### Subtasks
- [ ] Create vetting crate and SQLx database migrations: Add the `vetting` crate to the Cargo workspace at `services/rust/vetting` and create SQLx migrations for the `vetting_results` and `vetting_requests` tables with all specified columns, types, and indexes.
- [ ] Implement REST endpoint scaffolding and async pipeline dispatch: Build the Axum router with POST /api/v1/vetting/run (202 Accepted), GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id endpoints. Wire up the 202 pattern that spawns the vetting pipeline as a background tokio task and updates vetting_requests status.
- [ ] Build HTTP client utilities: timeout, retry with backoff, and circuit breaker: Implement a reusable HTTP client wrapper providing per-request 10-second timeouts, 2 retries with exponential backoff, and a circuit breaker that opens after 5 failures in 5 minutes, marking the stage as unavailable instead of failing the pipeline.
- [ ] Implement Stage 1: Business Verification via OpenCorporates API: Build the OpenCorporates integration stage that searches for a company by name, verifies existence and incorporation status, extracts directors, and returns structured results with raw response capture.
- [ ] Implement Stage 2: Online Presence Check (LinkedIn and Website): Build the online presence stage that checks for a company's LinkedIn page existence and follower count, and probes for website existence via DNS/HTTP.
- [ ] Implement Stage 3: Reputation Check via Google Reviews/Places API: Build the reputation stage that queries Google Business Profile or Google Places API to extract review rating and review count for the company.
- [ ] Implement Stage 4: Credit Signals with CreditProvider trait and stub: Design the CreditProvider trait for pluggable credit API implementations and build the initial stub implementation that returns UNKNOWN/unavailable credit data with appropriate logging.
- [ ] Implement Stage 5: Final scoring algorithm with weighted point system: Build the scoring algorithm that takes results from all 5 stages, applies the weighted 100-point system (30/20/20/20/10), and produces a GREEN/YELLOW/RED classification.
- [ ] Build async pipeline orchestrator with concurrent stage execution: Implement the pipeline orchestrator that runs stages 1-4 concurrently via tokio::join!, feeds results into the scoring stage, persists VettingResult to the database, and updates VettingRequest status throughout.
- [ ] Implement GDPR deletion endpoint: Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all vetting data (vetting_results and vetting_requests) for a given org_id and returns confirmation.
- [ ] Generate OpenAPI spec with utoipa: Add utoipa annotations to all endpoints and data models, and expose the OpenAPI JSON spec at /api/v1/vetting/openapi.json.
- [ ] Create Dockerfile and Kubernetes deployment manifest: Build the multi-stage Dockerfile for the vetting service and create the Kubernetes Deployment, Service, and related manifests for namespace sigma1 with proper secret and ConfigMap references.