Implement subtask 4001: Initialize Rust/Axum project with database schema and migrations

## Objective
Set up the Rust 1.75+ project with Axum 0.7, SQLx for PostgreSQL, Redis client, and create all database migrations for the finance domain (invoices, payments, payroll, currency_rates, audit_log).

## Steps
1. Initialize Cargo project: `cargo init finance-service`.
2. Add dependencies in Cargo.toml: axum 0.7, tokio, serde/serde_json, sqlx (with postgres and runtime-tokio features), redis (or deadpool-redis), tower, tower-http, tracing, tracing-subscriber.
3. Create `src/main.rs` with Axum app setup: load POSTGRES_URL, REDIS_URL from environment (populated via envFrom referencing infra ConfigMap), STRIPE_SECRET_KEY from Kubernetes secret.
4. Set up SQLx connection pool (PgPool) and Redis connection pool.
5. Create `migrations/` directory with SQLx migrations:
   - `001_invoices.sql`: invoices table (id UUID, opportunity_id UUID, customer_id UUID, status VARCHAR [draft/sent/paid/overdue/cancelled], currency VARCHAR(3), subtotal BIGINT, tax BIGINT, total BIGINT, due_date DATE, issued_at TIMESTAMP, paid_at TIMESTAMP, created_at, updated_at), invoice_line_items (id, invoice_id, description, quantity, unit_price BIGINT, amount BIGINT).
   - `002_payments.sql`: payments table (id UUID, invoice_id UUID, stripe_payment_intent_id, amount BIGINT, currency, status, method, processed_at, created_at).
   - `003_payroll.sql`: payroll_records (id, crew_member_id, period_start, period_end, hours_worked DECIMAL, rate BIGINT, currency, gross_amount BIGINT, deductions BIGINT, net_amount BIGINT, status, created_at).
   - `004_currency_rates.sql`: currency_rates (id, base_currency, target_currency, rate DECIMAL(18,8), fetched_at TIMESTAMP, created_at).
   - `005_audit_log.sql`: finance_audit_log (id, entity_type, entity_id, action, actor, old_value JSONB, new_value JSONB, created_at).
6. Create module structure: `src/config.rs`, `src/db.rs`, `src/routes/mod.rs`, `src/models/mod.rs`, `src/error.rs`.
7. Implement graceful shutdown with tokio signal handling.
8. Run `sqlx migrate run` on startup.

## Validation
Project compiles with `cargo build`. Server starts and binds to configured port. SQLx migrations run successfully against a test PostgreSQL instance. Connection pools for PostgreSQL and Redis are established without errors.