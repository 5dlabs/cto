Implement subtask 4002: Define data models and PostgreSQL migrations for finance schema

## Objective
Create sqlx migrations and Rust data model structs for invoices, payments, payroll, currency rates, and all related enums in the finance PostgreSQL schema.

## Steps
1. Create migration 001_create_finance_schema.sql: CREATE SCHEMA IF NOT EXISTS finance;
2. Create migration 002_create_invoices.sql: invoices table with columns: id (UUID PK), project_id (UUID), customer_id (UUID), invoice_number (VARCHAR UNIQUE), status (enum: DRAFT, SENT, PAID, OVERDUE, CANCELLED, VOID), currency (VARCHAR(3)), subtotal (DECIMAL(12,2)), tax_amount (DECIMAL(12,2)), total (DECIMAL(12,2)), due_date (DATE), paid_at (TIMESTAMPTZ nullable), notes (TEXT), created_at, updated_at, deleted_at (nullable for soft-delete). Create invoice_line_items table: id, invoice_id (FK), description, quantity (DECIMAL), unit_price (DECIMAL), line_total (DECIMAL).
3. Create migration 003_create_payments.sql: payments table: id (UUID PK), invoice_id (FK), stripe_payment_intent_id (VARCHAR nullable), stripe_charge_id (VARCHAR nullable), amount (DECIMAL(12,2)), currency (VARCHAR(3)), status (enum: PENDING, PROCESSING, SUCCEEDED, FAILED, REFUNDED), payment_method (VARCHAR), idempotency_key (VARCHAR UNIQUE), created_at, updated_at.
4. Create migration 004_create_payroll.sql: payroll_records table: id, employee_id (UUID), period_start (DATE), period_end (DATE), gross_amount (DECIMAL), deductions (DECIMAL), net_amount (DECIMAL), currency, status (enum: DRAFT, APPROVED, PAID), paid_at, created_at, updated_at.
5. Create migration 005_create_currency_rates.sql: currency_rates table: id, base_currency (VARCHAR(3)), target_currency (VARCHAR(3)), rate (DECIMAL(18,8)), fetched_at (TIMESTAMPTZ), UNIQUE(base_currency, target_currency). Add index on (base_currency, target_currency, fetched_at DESC).
6. Create corresponding Rust structs in src/models/: Invoice, InvoiceLineItem, Payment, PayrollRecord, CurrencyRate with sqlx::FromRow derives, serde Serialize/Deserialize, and enum types.
7. Ensure all DECIMAL fields use rust_decimal::Decimal type.
8. Run migrations against test DB to verify.

## Validation
All migrations run successfully in order against a clean PostgreSQL database; Rust model structs compile and can be deserialized from sqlx queries; enum types map correctly between Rust and PostgreSQL; unique constraints and foreign keys are enforced.