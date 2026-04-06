Implement subtask 4002: Define database models and migrations for Invoice, Payment, and Payroll

## Objective
Create PostgreSQL schema migrations for invoices, payments, payroll records, and currency rates using sqlx migrations, with multi-currency support built into the data model.

## Steps
1. Create migrations directory and use sqlx-cli for migration management.
2. Migration 001_create_invoices: invoices table with fields (id UUID PK, client_id, project_id, invoice_number unique, status enum (draft/sent/paid/overdue/cancelled), line_items JSONB, subtotal DECIMAL(19,4), tax_amount DECIMAL(19,4), total DECIMAL(19,4), currency VARCHAR(3), due_date DATE, issued_date DATE, paid_date DATE nullable, notes TEXT, created_at, updated_at). Add indexes on status, client_id, due_date.
3. Migration 002_create_payments: payments table (id UUID PK, invoice_id FK, amount DECIMAL(19,4), currency VARCHAR(3), stripe_payment_intent_id, stripe_charge_id, status enum (pending/succeeded/failed/refunded), method enum (card/bank_transfer/check), paid_at TIMESTAMP, created_at). Index on invoice_id, stripe_payment_intent_id.
4. Migration 003_create_payroll: payroll_records table (id UUID PK, crew_member_id, period_start DATE, period_end DATE, hours_worked DECIMAL(8,2), rate DECIMAL(10,2), gross_pay DECIMAL(19,4), deductions JSONB, net_pay DECIMAL(19,4), currency VARCHAR(3), status enum (draft/approved/paid), paid_at, created_at). Index on crew_member_id, period.
5. Migration 004_create_currency_rates: currency_rates table (id, base_currency, target_currency, rate DECIMAL(18,8), fetched_at TIMESTAMP, PRIMARY KEY (base_currency, target_currency, fetched_at)).
6. Define Rust structs (models) for each entity using sqlx::FromRow.
7. Implement repository trait and impl for each model with CRUD operations.

## Validation
Migrations run forward and backward cleanly against a test PostgreSQL instance; Rust model structs deserialize correctly from database rows; repository CRUD operations (create, get by ID, list with pagination, update, delete) work correctly; DECIMAL precision is preserved; currency fields accept valid ISO 4217 codes.