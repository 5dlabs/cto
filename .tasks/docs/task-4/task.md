## Implement Finance Service (Rex - Rust/Axum)

### Objective
Build the Finance Service handling invoicing, payments, AR/AP, payroll, multi-currency support, and Stripe integration. Second service in the Rex Cargo workspace, sharing common crates with the Catalog service.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Add `finance` crate to existing Cargo workspace at `services/rust/finance`.
2. SQLx migrations for `finance` schema:
   - `invoices` table: id, project_id, org_id, invoice_number (auto-generated sequence), status (enum: draft/sent/viewed/paid/overdue/cancelled), issued_at, due_at, currency (VARCHAR(3)), subtotal_cents, tax_cents, total_cents, paid_amount_cents, stripe_invoice_id, created_at, updated_at.
   - `payments` table: id, invoice_id, amount_cents, currency, method (enum: cash/check/wire/card/stripe), stripe_payment_id, received_at.
   - `payroll_entries` table: id, org_id, employee_id, period_start, period_end, amount_cents, currency, type (employee/contractor), notes.
   - `currency_rates` table: id, base_currency, target_currency, rate (DECIMAL(12,6)), fetched_at.
   - `tax_rules` table: id, jurisdiction, tax_type (GST/HST/sales_tax), rate_percent, effective_from.
3. Implement REST endpoints per PRD:
   - **Invoices**: POST/GET list/GET by id/POST send/POST paid — all under `/api/v1/invoices`
   - `POST /api/v1/invoices` accepts project_id, line items, currency, tax jurisdiction → computes subtotal, tax, total using tax_rules.
   - `POST /api/v1/invoices/:id/send` — if Stripe configured, create Stripe Invoice via API; mark status as `sent`.
   - `POST /api/v1/invoices/:id/paid` — record payment, update paid_amount_cents, transition status to `paid` if fully paid.
   - **Payments**: POST/GET list/GET by invoice — under `/api/v1/payments`
   - **Reports**: GET revenue, aging, cashflow, profitability — under `/api/v1/finance/reports/`
     - Revenue: aggregate paid invoices by period (month/quarter/year)
     - Aging: group unpaid invoices by days overdue (0-30, 31-60, 61-90, 90+)
     - Cashflow: payments received vs expenses (payroll) by period
     - Profitability: revenue minus costs per project_id
   - **Payroll**: GET by period, POST entry — under `/api/v1/payroll`
   - **Currency**: GET current rates — under `/api/v1/currency/rates`
4. Stripe integration:
   - Use `stripe-rust` crate (or `reqwest` with typed Stripe API calls)
   - Create Stripe Invoice when sending invoice
   - Webhook endpoint `POST /api/v1/webhooks/stripe` to handle `invoice.paid`, `payment_intent.succeeded` events → auto-update invoice status
   - Stripe API key from `sigma1-stripe-credentials` secret
5. Currency rate sync:
   - Background tokio task (or separate binary in workspace) that fetches rates from a free API (e.g., exchangerate.host or Open Exchange Rates) every hour
   - Stores rates in `currency_rates` table and caches in Valkey with 1hr TTL
   - Supports: USD, CAD, AUD, NZD as specified in PRD
6. Automated payment reminders:
   - Background task checks for invoices where `status = sent` and `due_at < now()` → transitions to `overdue`
   - Exposes data for Morgan to send reminder messages (query overdue invoices endpoint)
7. Tax calculation:
   - Seed tax_rules with Canadian GST (5%), HST by province (13% ON, 15% NS/NB/NL/PE), and placeholder US sales tax
   - Invoice creation auto-applies tax based on customer jurisdiction
8. OpenAPI spec via `utoipa`, served at `/api/v1/finance/openapi.json`.
9. GDPR endpoint: `DELETE /api/v1/gdpr/customer/:id` — anonymize invoice customer data (replace with 'DELETED'), retain financial records for legal compliance, return confirmation.
10. Reuse shared crate: health checks, metrics, rate limiting, error types, DB pool, API key auth middleware.
11. Dockerfile: same multi-stage pattern as catalog.
12. Kubernetes Deployment: namespace sigma1, replicas 2, envFrom ConfigMap, secret refs for DB + Stripe + service API keys, port 8082.

### Subtasks
- [ ] Scaffold finance crate in Cargo workspace with shared dependencies: Add the `finance` crate to the existing Rex Cargo workspace at `services/rust/finance`, configure Cargo.toml with dependencies (axum, sqlx, serde, utoipa, tokio, reqwest), and wire up the shared crate for health checks, metrics, error types, DB pool, and API key auth middleware. Set up the main.rs entrypoint with Axum router skeleton listening on port 8082.
- [ ] Create SQLx migrations for finance schema tables: Write SQLx migration files to create the `invoices`, `payments`, `payroll_entries`, `currency_rates`, and `tax_rules` tables in a `finance` schema with all columns, enums, indexes, and foreign keys as specified in the PRD.
- [ ] Implement tax calculation engine with seed data: Build the tax calculation module that determines applicable tax (GST/HST/sales tax) based on customer jurisdiction, and seed the `tax_rules` table with Canadian GST (5%), provincial HST rates, and US placeholder sales tax.
- [ ] Implement invoice status state machine: Build the invoice status state machine enforcing valid transitions (draft→sent→viewed→paid, draft→sent→overdue, any→cancelled) with rejection of invalid transitions.
- [ ] Implement invoice CRUD endpoints with tax calculation: Build the invoice REST endpoints: POST create (with line items and auto tax calculation), GET list (with filtering), GET by id, under `/api/v1/invoices`. Includes invoice number auto-generation.
- [ ] Implement invoice send and payment recording endpoints: Build the invoice action endpoints: POST send (mark as sent), POST paid (record payment with partial/full tracking and status transitions), and payment CRUD endpoints under `/api/v1/payments`.
- [ ] Implement Stripe integration for invoice creation and webhook handling: Integrate with the Stripe API to create Stripe Invoices when sending invoices, and implement the webhook endpoint to handle `invoice.paid` and `payment_intent.succeeded` events for automatic status updates.
- [ ] Implement financial reporting endpoints: Build the four financial report endpoints: revenue by period, aging buckets, cashflow, and project profitability under `/api/v1/finance/reports/`.
- [ ] Implement payroll endpoints and currency rate endpoints: Build the payroll entry CRUD endpoints under `/api/v1/payroll` and the currency rate query endpoint under `/api/v1/currency/rates`.
- [ ] Implement currency rate sync background task: Build the background tokio task that fetches currency exchange rates hourly from an external API and stores them in the database and Valkey cache.
- [ ] Implement automated overdue invoice detection background task: Build the background task that periodically checks for sent invoices past their due date and transitions them to overdue status, exposing overdue invoices for reminder queries.
- [ ] Implement GDPR anonymization endpoint: Build the GDPR endpoint `DELETE /api/v1/gdpr/customer/:id` that anonymizes customer-identifying data on invoices while preserving financial records for legal compliance.
- [ ] Generate OpenAPI spec and configure utoipa documentation: Configure utoipa to generate and serve the complete OpenAPI specification for the Finance Service at `/api/v1/finance/openapi.json`, covering all endpoints, request/response models, and error types.
- [ ] Create Dockerfile and Kubernetes deployment manifests: Build the multi-stage Dockerfile for the finance service and create Kubernetes Deployment, Service, and related manifests for namespace sigma1.