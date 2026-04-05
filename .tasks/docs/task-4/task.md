## Build Finance Service (Rex - Rust/Axum)

### Objective
Implement the Finance Service handling invoicing, payments, payroll, multi-currency support, and Stripe integration. Uses integer cents with explicit currency fields (per D11). Built within the Cargo workspace established by the Equipment Catalog service.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1, 2

### Implementation Details
1. Add service crate to existing workspace: `sigma1-services/services/finance/`
   - Depend on shared-auth, shared-db, shared-error, shared-observability crates
2. Database migrations in `finance` schema:
   - `invoices` table: id (UUID PK), project_id (UUID, NOT FK — cross-service reference via API), org_id, invoice_number (UNIQUE, auto-generated sequence), status (enum: draft/sent/viewed/paid/overdue/cancelled), issued_at, due_at, currency (VARCHAR(3)), subtotal_cents (BIGINT), tax_cents (BIGINT), total_cents (BIGINT), paid_amount_cents (BIGINT default 0), stripe_invoice_id (nullable), created_at, updated_at
   - `invoice_line_items` table: id, invoice_id (FK), description, quantity, unit_price_cents, subtotal_cents
   - `payments` table: id (UUID PK), invoice_id (FK), amount_cents (BIGINT), currency, method (enum: cash/check/wire/card/stripe), stripe_payment_id (nullable), received_at, created_at
   - `payroll_entries` table: id, employee_id (UUID), period_start, period_end, type (employee/contractor), hours, rate_cents, total_cents, currency, status, created_at
   - `currency_rates` table: base_currency, target_currency, rate (DECIMAL(20,10)), fetched_at, PRIMARY KEY (base_currency, target_currency)
   - Indexes: invoices(project_id), invoices(status), invoices(due_at), payments(invoice_id)
3. Implement Axum 0.7 endpoints:
   - `POST /api/v1/invoices` — create invoice (accepts project_id, line items, currency, due_at)
   - `GET /api/v1/invoices` — list with filters (status, date range, overdue), paginated
   - `GET /api/v1/invoices/:id` — full invoice with line items and payment history
   - `POST /api/v1/invoices/:id/send` — mark as sent, trigger Stripe invoice creation if payment method is card/stripe
   - `POST /api/v1/invoices/:id/paid` — record payment, update paid_amount_cents, auto-transition status to Paid if fully paid
   - `POST /api/v1/payments` — record standalone payment
   - `GET /api/v1/payments` — list payments with filters
   - `GET /api/v1/payments/invoice/:id` — payments for specific invoice
   - `GET /api/v1/finance/reports/revenue?period=` — revenue report (monthly/quarterly/yearly aggregation)
   - `GET /api/v1/finance/reports/aging` — AR aging buckets (current, 30, 60, 90+ days)
   - `GET /api/v1/finance/reports/cashflow` — cash inflows/outflows by period
   - `GET /api/v1/finance/reports/profitability` — per-project profitability (revenue - costs)
   - `GET /api/v1/payroll?period=` — payroll summary for period
   - `POST /api/v1/payroll/entries` — add payroll entry
   - `GET /api/v1/currency/rates` — current cached rates
   - Health and metrics endpoints via shared crates
4. Stripe integration module:
   - Use `stripe-rust` crate
   - Create Stripe Invoice when invoice is sent with card/stripe method
   - Webhook handler `POST /api/v1/webhooks/stripe` for payment_intent.succeeded → auto-record payment
   - Idempotency: store stripe_invoice_id and stripe_payment_id to prevent duplicates
5. Currency rate sync:
   - Background tokio task running every 6 hours
   - Fetch rates from exchangerate.host or similar free API
   - Store in currency_rates table, cache in Valkey with 6-hour TTL
   - Support: USD, CAD, AUD, NZD, EUR, GBP
6. Tax calculation module:
   - Configurable tax rates per jurisdiction (GST/HST for Canada, sales tax for US)
   - Store tax config as JSONB in a `tax_configurations` table
   - Calculate tax_cents on invoice creation based on customer jurisdiction
7. Automated overdue detection:
   - Background task checks invoices where due_at < now() AND status = 'sent', transitions to 'overdue'
   - Runs every hour
8. All monetary arithmetic uses `rust_decimal` crate, stored as i64 cents in DB (per D11).
9. Kubernetes Deployment: namespace `sigma1`, 2 replicas, envFrom sigma1-infra-endpoints.
10. Invoice number generation: sequential per-org prefix (e.g., PE-2024-0001).

### Subtasks
- [ ] Scaffold finance service crate within Cargo workspace: Create the finance service crate under sigma1-services/services/finance/ with proper Cargo.toml dependencies on shared-auth, shared-db, shared-error, shared-observability crates. Set up the main.rs with Axum 0.7 server bootstrap, router skeleton, graceful shutdown, and health/metrics endpoints via shared crates.
- [ ] Implement database migrations for all finance schema tables: Create SQLx migrations for the finance schema including invoices, invoice_line_items, payments, payroll_entries, currency_rates, and tax_configurations tables with all specified columns, enums, indexes, and constraints.
- [ ] Implement invoice CRUD endpoints with line items and invoice number generation: Build the invoice domain model, repository layer, and Axum handlers for creating, listing, and retrieving invoices including line items and sequential per-org invoice number generation.
- [ ] Implement invoice status state machine and send endpoint: Build the invoice status state machine (draft→sent→viewed→paid→overdue→cancelled) with validation, and implement the POST /api/v1/invoices/:id/send endpoint that marks an invoice as sent.
- [ ] Implement payment recording endpoints with partial payment support: Build payment CRUD endpoints including standalone payment recording, invoice-linked payments with automatic status transitions on full payment, and partial payment tracking.
- [ ] Implement Stripe integration module with invoice creation and webhook handler: Build the Stripe integration module using stripe-rust crate: create Stripe invoices when an invoice is sent with card/stripe method, and handle payment_intent.succeeded webhooks with idempotency.
- [ ] Implement tax calculation module with configurable jurisdiction rates: Build the tax calculation module supporting configurable per-jurisdiction tax rates (GST/HST for Canada, sales tax for US) stored in the tax_configurations table, and integrate it into invoice creation.
- [ ] Implement financial reporting endpoints (revenue, AR aging, cashflow, profitability): Build the four financial reporting endpoints: revenue aggregation by period, accounts receivable aging buckets, cash flow by period, and per-project profitability.
- [ ] Implement payroll endpoints for entry management and period summaries: Build payroll entry creation and period-based summary endpoints including support for employee and contractor types.
- [ ] Implement currency rate sync background task with Valkey caching: Build the background tokio task that fetches exchange rates every 6 hours from an external API, stores them in the currency_rates table, and caches them in Valkey with a 6-hour TTL.
- [ ] Implement automated overdue invoice detection background task: Build the hourly background task that detects invoices past their due date and transitions them from sent/viewed to overdue status.
- [ ] Implement Kubernetes deployment manifest for finance service: Create the Kubernetes Deployment, Service, and related manifests for the finance service in the sigma1 namespace with 2 replicas and envFrom sigma1-infra-endpoints ConfigMap.