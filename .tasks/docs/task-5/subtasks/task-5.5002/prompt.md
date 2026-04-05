Implement subtask 5002: Create database migrations for vetting schema

## Objective
Write SQLx migrations to create the `vetting` schema with `vetting_results` and `vetting_requests` tables including all specified columns, types, defaults, and indexes.

## Steps
1. Create migration file in `migrations/` directory.
2. `CREATE SCHEMA IF NOT EXISTS vetting;`
3. Create `vetting.vetting_requests` table:
   - id UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - org_id UUID NOT NULL
   - org_name TEXT NOT NULL
   - org_domain TEXT
   - status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','in_progress','completed','failed'))
   - requested_at TIMESTAMPTZ NOT NULL DEFAULT now()
   - completed_at TIMESTAMPTZ
4. Create `vetting.vetting_results` table:
   - id UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - org_id UUID NOT NULL
   - business_verified BOOL NOT NULL DEFAULT false
   - opencorporates_data JSONB
   - linkedin_exists BOOL NOT NULL DEFAULT false
   - linkedin_followers INT NOT NULL DEFAULT 0
   - google_reviews_rating REAL
   - google_reviews_count INT NOT NULL DEFAULT 0
   - credit_score INT
   - risk_flags TEXT[] NOT NULL DEFAULT '{}'
   - final_score VARCHAR(6) NOT NULL CHECK (final_score IN ('GREEN','YELLOW','RED'))
   - vetted_at TIMESTAMPTZ NOT NULL DEFAULT now()
   - created_at TIMESTAMPTZ NOT NULL DEFAULT now()
5. Create indexes: `vetting_results_org_id_idx` on vetting_results(org_id), `vetting_requests_org_id_status_idx` on vetting_requests(org_id, status).
6. Create corresponding Rust model structs in `src/models/` with `sqlx::FromRow` derives for both tables.
7. Create repository module `src/db.rs` with functions: insert_request, update_request_status, insert_result, get_latest_result_by_org, get_credit_by_org, delete_all_by_org.

## Validation
Run `sqlx migrate run` against a test database successfully. Verify tables exist with `\dt vetting.*`. Insert and query sample rows via repository functions in a test.