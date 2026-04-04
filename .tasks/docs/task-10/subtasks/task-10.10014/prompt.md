Implement subtask 10014: GDPR deletion orchestrator: implement Rust CLI binary

## Objective
Build a Rust CLI binary (new Cargo workspace member `gdpr-orchestrator`) that accepts a customer ID, calls GDPR deletion endpoints on each service in order, and writes a structured audit log.

## Steps
Step-by-step:
1. In the Rex Cargo workspace, add a new member: `crates/gdpr-orchestrator/Cargo.toml` with dependencies: `clap` (CLI args), `reqwest` (HTTP client), `serde`/`serde_json`, `sqlx` (PostgreSQL), `tokio`, `uuid`, `tracing`.
2. Implement `main.rs`:
   - Parse `--customer-id <UUID>` via clap
   - Read service URLs from env vars: `VETTING_URL`, `SOCIAL_URL`, `FINANCE_URL`, `RMS_URL`, `CATALOG_URL`
   - Call each service's GDPR deletion endpoint in order: Vetting → Social → Finance → RMS → Catalog
   - For each call: POST to `{SERVICE_URL}/api/gdpr/delete` with JSON body `{"customer_id": "<uuid>"}`
   - Collect response status code and body for each
   - If any service returns non-2xx, log error and continue to report all statuses, but set exit code to 1
3. Write audit log:
   - Connect to PostgreSQL using `DATABASE_URL` env var
   - Insert into `audit.gdpr_deletions` table: `(request_id UUID DEFAULT gen_random_uuid(), customer_id UUID, service TEXT, status INT, response JSONB, completed_at TIMESTAMPTZ DEFAULT now())`
   - One row per service call
4. Exit 0 only if all services returned 2xx.
5. Add `gdpr-orchestrator` to workspace `Cargo.toml` members list.

## Validation
Run `cargo build -p gdpr-orchestrator` and verify it compiles. Run with `--help` and verify usage is printed. Mock all service endpoints locally (e.g., with a simple HTTP server returning 204) and run the CLI with a test UUID. Verify it exits 0 and that audit rows are written to the database. Run with one service returning 500, verify exit code is 1 and error is logged.