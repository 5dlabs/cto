Implement subtask 4001: Initialize Rust/Axum project with finance data models and PostgreSQL schema

## Objective
Set up the Rust 1.75+ Axum 0.7 project structure, define domain models for Invoice, Payment, and Payroll, and create PostgreSQL migrations for the finance schema.

## Steps
1. Initialize Cargo workspace with `cargo init`. Add dependencies: axum 0.7, tokio, serde, sqlx (with postgres feature), tower, tower-http. 2. Create project structure: /src/main.rs (entrypoint), /src/routes/ (endpoint handlers), /src/models/ (domain types), /src/db/ (repository layer), /src/services/ (business logic), /src/config.rs (env/ConfigMap loading). 3. Define models: Invoice (id, customer_id, line_items, currency, subtotal, tax, total, status, due_date, created_at), Payment (id, invoice_id, amount, currency, stripe_payment_intent_id, status, method, paid_at), Payroll (id, employee_id, period_start, period_end, gross_amount, deductions, net_amount, currency, status). 4. Create SQL migrations using sqlx-cli: invoices, invoice_line_items, payments, payroll_records, currency_rates tables. Use NUMERIC type for monetary columns. 5. Implement database connection pool initialization from ConfigMap PostgreSQL connection string. 6. Implement repository traits and implementations for each domain.

## Validation
Project compiles with `cargo build`; migrations run against test PostgreSQL; all tables created with correct column types (NUMERIC for money); repository CRUD operations pass integration tests; connection pool initializes from environment variables.