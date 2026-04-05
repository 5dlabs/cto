Implement subtask 4002: Create SQLx migrations for finance schema tables

## Objective
Write SQLx migration files to create the `invoices`, `payments`, `payroll_entries`, `currency_rates`, and `tax_rules` tables in a `finance` schema with all columns, enums, indexes, and foreign keys as specified in the PRD.

## Steps
1. Create migration directory `services/rust/finance/migrations/`.
2. Migration 001: Create custom enums — `invoice_status` (draft, sent, viewed, paid, overdue, cancelled), `payment_method` (cash, check, wire, card, stripe), `payroll_type` (employee, contractor), `tax_type_enum` (gst, hst, sales_tax).
3. Migration 002: `invoices` table — `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `project_id UUID NOT NULL`, `org_id UUID NOT NULL`, `invoice_number VARCHAR(50) UNIQUE NOT NULL`, `status invoice_status NOT NULL DEFAULT 'draft'`, `issued_at TIMESTAMPTZ`, `due_at TIMESTAMPTZ`, `currency VARCHAR(3) NOT NULL DEFAULT 'CAD'`, `subtotal_cents BIGINT NOT NULL DEFAULT 0`, `tax_cents BIGINT NOT NULL DEFAULT 0`, `total_cents BIGINT NOT NULL DEFAULT 0`, `paid_amount_cents BIGINT NOT NULL DEFAULT 0`, `stripe_invoice_id VARCHAR(255)`, `customer_name VARCHAR(255)`, `customer_email VARCHAR(255)`, `jurisdiction VARCHAR(50)`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`. Add indexes on `org_id`, `project_id`, `status`, `due_at`.
4. Migration 003: `invoice_line_items` table — `id UUID PRIMARY KEY`, `invoice_id UUID REFERENCES invoices(id)`, `description TEXT`, `quantity DECIMAL(10,2)`, `unit_price_cents BIGINT`, `amount_cents BIGINT`. (Needed for storing line items.)
5. Migration 004: `payments` table — all columns as specified, `invoice_id UUID REFERENCES invoices(id)`, index on `invoice_id`.
6. Migration 005: `payroll_entries` table — all columns as specified, indexes on `org_id`, `employee_id`, `period_start/period_end`.
7. Migration 006: `currency_rates` table — all columns, unique constraint on `(base_currency, target_currency)`, index on `fetched_at`.
8. Migration 007: `tax_rules` table — all columns, index on `(jurisdiction, tax_type, effective_from)`.
9. Run `sqlx migrate run` against a test database and verify all tables created.
10. Generate SQLx offline query data with `cargo sqlx prepare`.

## Validation
Run `sqlx migrate run` against a clean PostgreSQL database and verify all 7 migrations apply without errors. Verify each table exists with correct columns using `\d+ table_name`. Verify enums are created. Run `sqlx migrate run` a second time to confirm idempotency (no errors on re-run).