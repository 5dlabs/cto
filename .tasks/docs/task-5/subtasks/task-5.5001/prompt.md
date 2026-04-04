Implement subtask 5001: Create vetting crate and SQLx database migrations

## Objective
Add the `vetting` crate to the Cargo workspace at `services/rust/vetting` and create SQLx migrations for the `vetting_results` and `vetting_requests` tables with all specified columns, types, and indexes.

## Steps
1. Create `services/rust/vetting` directory with `Cargo.toml` referencing workspace dependencies (axum 0.7, sqlx with postgres feature, serde, uuid, chrono, tokio, reqwest, utoipa).
2. Add the crate to the root `Cargo.toml` workspace members list.
3. Create SQLx migration for `vetting_results` table: org_id UUID PRIMARY KEY, business_verified BOOL NOT NULL DEFAULT false, opencorporates_data JSONB, linkedin_exists BOOL NOT NULL DEFAULT false, linkedin_followers INT DEFAULT 0, google_reviews_rating FLOAT, google_reviews_count INT DEFAULT 0, credit_score INT, risk_flags TEXT[] NOT NULL DEFAULT '{}', final_score VARCHAR(6) NOT NULL, vetted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), raw_responses JSONB NOT NULL DEFAULT '{}'.
4. Create SQLx migration for `vetting_requests` table: id UUID PRIMARY KEY DEFAULT gen_random_uuid(), org_id UUID NOT NULL, requested_by UUID NOT NULL, status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','running','completed','failed')), started_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, error_message TEXT, FOREIGN KEY org_id REFERENCES vetting_results or allow standalone.
5. Add indexes: idx_vetting_requests_org_id, idx_vetting_requests_status, idx_vetting_results_final_score.
6. Define Rust domain structs: VettingResult, VettingRequest with sqlx::FromRow derives and serde Serialize/Deserialize.
7. Verify migrations run cleanly against the dev PostgreSQL instance.

## Validation
Run `sqlx migrate run` against a test database. Verify both tables are created with correct columns via `\d vetting_results` and `\d vetting_requests`. Insert sample rows and verify constraints (e.g., invalid status value is rejected, UUID generation works).