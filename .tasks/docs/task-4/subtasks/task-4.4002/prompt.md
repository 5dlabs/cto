Implement subtask 4002: Implement database migrations for all finance schema tables

## Objective
Create SQLx migrations for the finance schema including invoices, invoice_line_items, payments, payroll_entries, currency_rates, and tax_configurations tables with all specified columns, enums, indexes, and constraints.

## Steps
1. Create migration files under `services/finance/migrations/`.
2. Migration 1: `CREATE SCHEMA IF NOT EXISTS finance;`
3. Migration 2: Create enum types — `finance.invoice_status` (draft, sent, viewed, paid, overdue, cancelled), `finance.payment_method` (cash, check, wire, card, stripe), `finance.payroll_type` (employee, contractor), `finance.payroll_status` (pending, approved, paid).
4. Migration 3: `finance.invoices` table — id UUID PK DEFAULT gen_random_uuid(), project_id UUID NOT NULL, org_id UUID NOT NULL, invoice_number VARCHAR(50) UNIQUE NOT NULL, status finance.invoice_status DEFAULT 'draft', issued_at TIMESTAMPTZ, due_at TIMESTAMPTZ NOT NULL, currency VARCHAR(3) NOT NULL DEFAULT 'USD', subtotal_cents BIGINT NOT NULL DEFAULT 0, tax_cents BIGINT NOT NULL DEFAULT 0, total_cents BIGINT NOT NULL DEFAULT 0, paid_amount_cents BIGINT NOT NULL DEFAULT 0, stripe_invoice_id VARCHAR(255), created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now(). Add indexes on (project_id), (org_id, status), (due_at), (stripe_invoice_id) WHERE stripe_invoice_id IS NOT NULL.
5. Migration 4: `finance.invoice_line_items` — id UUID PK, invoice_id UUID FK→invoices ON DELETE CASCADE, description TEXT NOT NULL, quantity DECIMAL(10,4) NOT NULL, unit_price_cents BIGINT NOT NULL, subtotal_cents BIGINT NOT NULL, sort_order INT DEFAULT 0.
6. Migration 5: `finance.payments` — id UUID PK, invoice_id UUID FK→invoices, amount_cents BIGINT NOT NULL, currency VARCHAR(3) NOT NULL, method finance.payment_method NOT NULL, stripe_payment_id VARCHAR(255), received_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ DEFAULT now(). Index on (invoice_id), unique partial index on (stripe_payment_id) WHERE stripe_payment_id IS NOT NULL.
7. Migration 6: `finance.payroll_entries` — id UUID PK, employee_id UUID NOT NULL, period_start DATE NOT NULL, period_end DATE NOT NULL, type finance.payroll_type NOT NULL, hours DECIMAL(8,2), rate_cents BIGINT NOT NULL, total_cents BIGINT NOT NULL, currency VARCHAR(3) NOT NULL DEFAULT 'USD', status finance.payroll_status DEFAULT 'pending', created_at TIMESTAMPTZ DEFAULT now().
8. Migration 7: `finance.currency_rates` — base_currency VARCHAR(3), target_currency VARCHAR(3), rate DECIMAL(20,10) NOT NULL, fetched_at TIMESTAMPTZ NOT NULL, PRIMARY KEY (base_currency, target_currency).
9. Migration 8: `finance.tax_configurations` — id UUID PK, jurisdiction_code VARCHAR(20) UNIQUE NOT NULL, jurisdiction_name TEXT NOT NULL, tax_type VARCHAR(50) NOT NULL, rate DECIMAL(8,6) NOT NULL, config JSONB DEFAULT '{}', active BOOLEAN DEFAULT true, created_at TIMESTAMPTZ DEFAULT now().
10. Migration 9: `finance.invoice_number_counters` — org_id UUID NOT NULL, year INT NOT NULL, next_number INT NOT NULL DEFAULT 1, PRIMARY KEY (org_id, year).
11. Run `sqlx migrate run` against test DB to verify all migrations apply cleanly.

## Validation
Run migrations against a clean PostgreSQL database. Verify all tables exist in the finance schema with correct columns and types using `\d finance.invoices` etc. Verify indexes exist. Verify enum types are created. Run migrations twice to confirm idempotency (no errors on re-run).