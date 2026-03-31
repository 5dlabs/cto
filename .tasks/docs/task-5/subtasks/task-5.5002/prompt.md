Implement subtask 5002: Define VettingResult and LeadScore data models with SQLx migrations

## Objective
Create the PostgreSQL schema and Rust data models for vetting results, individual pipeline stage outputs, and the composite lead score.

## Steps
1. Add sqlx with the `postgres`, `runtime-tokio`, `migrate`, and `uuid` features.
2. Create SQL migration files under `migrations/`:
   - `CREATE TABLE vetting_results (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), org_id UUID NOT NULL, status VARCHAR(20) NOT NULL DEFAULT 'pending', business_verification JSONB, online_presence JSONB, reputation JSONB, credit_signals JSONB, composite_score VARCHAR(10), score_details JSONB, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now());`
   - Add indexes on `org_id` and `created_at`.
3. Define corresponding Rust structs: `VettingResult`, `BusinessVerification`, `OnlinePresence`, `ReputationAnalysis`, `CreditSignals`, `LeadScore` (with enum `ScoreGrade { Green, Yellow, Red }`).
4. Derive `sqlx::FromRow`, `Serialize`, `Deserialize` on all models.
5. Implement a `VettingRepository` with methods: `insert_result`, `get_by_id`, `get_by_org_id`, `get_credit_by_org_id`, `update_result`.
6. Run migrations programmatically at startup using `sqlx::migrate!().run(&pool).await`.

## Validation
Migrations run successfully against a test PostgreSQL instance. VettingRepository CRUD operations (insert, query by org_id, update) pass with correct data round-tripping. Rust models serialize/deserialize to/from JSON without data loss.