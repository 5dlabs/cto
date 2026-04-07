Implement subtask 4002: Write database migrations for all five Finance schema tables

## Objective
Create SQL migration files targeting the finance schema for: invoices, invoice_line_items, payments, payroll_entries, and currency_rates tables.

## Steps
Use sqlx migrate with files in finance-service/migrations/. Migration 001: CREATE SCHEMA IF NOT EXISTS finance. Migration 002: invoices table with all columns, CHECK constraints for status enum, DEFAULT 0 for paid_amount_cents. Migration 003: invoice_line_items with FK to invoices. Migration 004: payments with FK to invoices and CHECK constraint on method. Migration 005: payroll_entries with CHECK on worker_type. Migration 006: currency_rates with composite PRIMARY KEY (base, target). Add indexes: invoices(status), invoices(org_id), invoices(due_at) for aging queries, payments(invoice_id). Also add migration 007 for finance.gdpr_deletions audit table (entity TEXT, deleted_org_id UUID, deleted_at TIMESTAMPTZ). Use UP/DOWN pairs. Run via `sqlx migrate run`.

## Validation
`sqlx migrate run` against a fresh finance schema applies all 7 migrations without error. `sqlx migrate revert` repeatedly down to 0 drops all tables cleanly. `psql -c '\dt finance.*'` lists all 6 domain tables plus gdpr_deletions.