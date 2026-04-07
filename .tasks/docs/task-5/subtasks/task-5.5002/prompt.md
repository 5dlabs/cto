Implement subtask 5002: Write sqlx database migrations for vetting schema

## Objective
Create sqlx migration files under services/customer-vetting/migrations/ targeting the vetting PostgreSQL schema. Migrations must create both the vetting_requests and vetting_results tables with all columns, constraints, and indexes described in the task details.

## Steps
Create migrations/0001_create_vetting_requests.sql: id UUID PK DEFAULT gen_random_uuid(), org_id UUID NOT NULL, status TEXT NOT NULL CHECK (status IN ('pending','running','completed','failed')), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), completed_at TIMESTAMPTZ, error_text TEXT. Create migrations/0002_create_vetting_results.sql: id UUID PK DEFAULT gen_random_uuid(), org_id UUID UNIQUE NOT NULL, business_verified BOOL, opencorporates_data JSONB, linkedin_exists BOOL, linkedin_followers INT, google_reviews_rating FLOAT4, google_reviews_count INT, credit_score INT, risk_flags TEXT[], final_score TEXT CHECK (final_score IN ('GREEN','YELLOW','RED')), vetted_at TIMESTAMPTZ, updated_at TIMESTAMPTZ NOT NULL DEFAULT now(). Add index on vetting_requests(org_id) and vetting_results(org_id). Use sqlx::migrate!() macro in main.rs to run migrations at startup.

## Validation
Run `sqlx migrate run` against a local Postgres instance in the vetting schema; verify both tables exist with correct column types using `\d vetting_results` and `\d vetting_requests` in psql. Run migrations a second time and confirm idempotency (no error).