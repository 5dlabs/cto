<identity>
You are rex, the Rust 1.75+/Axum 0.7 implementation agent. You own task 4 end-to-end.
</identity>

<context>
<task_overview>
Task 4: Build Finance Service (Rex - Rust/Axum)
Implement the Finance microservice in Rust/Axum handling invoicing, payment recording, AR aging reports, payroll tracking, multi-currency support with live rate sync, and Stripe integration. Replaces QuickBooks/Xero for Sigma-1. Exposes REST endpoints, Prometheus metrics, health probes, and internal GDPR endpoints.
Priority: high
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Initialize Cargo workspace at services/finance. Dependencies: axum 0.7, tokio 1.x, sqlx 0.7 (postgres, uuid, chrono, rust_decimal), stripe 0.17 (async stripe-rust crate or reqwest-based Stripe REST client), redis 0.24, tower-http, prometheus 0.13, serde/serde_json, anyhow, tracing, chrono, rust_decimal.
2. Database migrations targeting finance schema. Tables: invoices (id UUID PK, project_id UUID, org_id UUID, invoice_number TEXT UNIQUE, status TEXT CHECK IN (draft,sent,viewed,paid,overdue,cancelled), issued_at TIMESTAMPTZ, due_at DATE, currency CHAR(3), subtotal_cents BIGINT, tax_cents BIGINT, total_cents BIGINT, paid_amount_cents BIGINT DEFAULT 0, stripe_invoice_id TEXT), invoice_line_items (id UUID PK, invoice_id UUID FK, description TEXT, quantity INT, unit_price_cents BIGINT, currency CHAR(3)), payments (id UUID PK, invoice_id UUID FK, amount_cents BIGINT, currency CHAR(3), method TEXT CHECK IN (cash,check,wire,card,stripe), stripe_payment_id TEXT, received_at TIMESTAMPTZ), payroll_entries (id UUID PK, period DATE, worker_id UUID, worker_name TEXT, worker_type TEXT CHECK IN (contractor,employee), amount_cents BIGINT, currency CHAR(3), description TEXT, created_at TIMESTAMPTZ), currency_rates (base CHAR(3), target CHAR(3), rate NUMERIC(18,8), fetched_at TIMESTAMPTZ, PRIMARY KEY (base, target)).
3. Implement handlers: POST /api/v1/invoices, GET /api/v1/invoices, GET /api/v1/invoices/:id, POST /api/v1/invoices/:id/send (updates status=sent, triggers email via optional SMTP), POST /api/v1/invoices/:id/paid (records payment, updates paid_amount_cents, sets status=paid if fully paid), POST /api/v1/payments, GET /api/v1/payments, GET /api/v1/payments/invoice/:id.
4. Reports: GET /api/v1/finance/reports/revenue?period=YYYY-MM returns sum of paid invoices grouped by month. GET /api/v1/finance/reports/aging returns unpaid invoices grouped by age buckets (0-30, 31-60, 61-90, 90+ days). GET /api/v1/finance/reports/cashflow returns monthly in/out. GET /api/v1/finance/reports/profitability returns per-project revenue minus payroll entries.
5. Payroll: GET /api/v1/payroll?period=YYYY-MM, POST /api/v1/payroll/entries.
6. Currency: GET /api/v1/currency/rates returns cached rates from Valkey (cache key: rates:v1). Background tokio::task spawned at startup syncs rates from exchangerate.host or similar free API every 6 hours, storing in both currency_rates table and Valkey with 6h TTL.
7. Stripe integration: on POST /api/v1/invoices/:id/send (if stripe_invoice_id null), create Stripe Invoice via API, store stripe_invoice_id. Stripe webhook endpoint POST /api/v1/webhooks/stripe validates signature, processes invoice.paid and payment_intent.succeeded events to update payment records.
8. Tax calculation: apply tax_rate based on currency/locale field on invoice; default CAD=13% HST, USD=0% (simplified — full tax engine deferred). Store tax_cents on invoice.
9. Automated payment reminders: tokio::spawn background task running daily, queries invoices WHERE status=sent AND due_at < NOW() - INTERVAL '1 day', updates status=overdue, enqueues reminder (log for now, real email/signal in Morgan integration).
10. JWT middleware, /metrics, /health/live, /health/ready, GDPR endpoints (/internal/gdpr/export/:org_id, /internal/gdpr/delete/:org_id — anonymize org_id on invoices and payroll). Kubernetes Deployment 2 replicas.
</implementation_plan>

<acceptance_criteria>
1. POST /api/v1/invoices with valid project_id and line items returns 201 with status=draft and computed total_cents = subtotal_cents + tax_cents.
2. POST /api/v1/invoices/:id/paid with amount equal to total_cents returns invoice status=paid.
3. GET /api/v1/finance/reports/aging returns JSON with buckets 0-30, 31-60, 61-90, 90+ each as numeric counts; overdue invoice (created 35 days ago, unpaid) appears in 31-60 bucket.
4. GET /api/v1/currency/rates returns JSON object with at least USD, CAD, AUD keys and non-zero rate values.
5. Stripe webhook POST /api/v1/webhooks/stripe with valid Stripe-Signature header and invoice.paid payload updates invoice status to paid (verify via GET /api/v1/invoices/:id).
6. Overdue reminder task: seed invoice with due_at = NOW()-2d and status=sent; after background task runs, status becomes overdue.
7. GET /health/ready returns
200. 8. cargo test passes with >= 80% coverage.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Initialize Cargo workspace and dependency manifest for Finance service: Create the Cargo workspace at services/finance with Cargo.toml declaring all required dependencies: axum, tokio, sqlx, stripe client, redis, tower-http, prometheus, serde, anyhow, tracing, chrono, rust_decimal.
- Write database migrations for all five Finance schema tables: Create SQL migration files targeting the finance schema for: invoices, invoice_line_items, payments, payroll_entries, and currency_rates tables.
- Implement invoice CRUD handlers with tax calculation and status state machine: Implement POST /api/v1/invoices (create with tax computation), GET /api/v1/invoices (list), GET /api/v1/invoices/:id, POST /api/v1/invoices/:id/send, and POST /api/v1/invoices/:id/paid handlers in Axum.
- Implement payment recording handlers: Implement POST /api/v1/payments, GET /api/v1/payments, and GET /api/v1/payments/invoice/:id handlers for recording and retrieving payment records.
- Implement Stripe invoice creation and webhook endpoint with signature validation: On POST /api/v1/invoices/:id/send when stripe_invoice_id is null, create a Stripe Invoice via the Stripe API and store the returned ID. Implement POST /api/v1/webhooks/stripe to validate Stripe-Signature and process invoice.paid and payment_intent.succeeded events.
- Implement financial report endpoints (revenue, aging, cashflow, profitability): Implement GET /api/v1/finance/reports/revenue, /aging, /cashflow, and /profitability query handlers returning aggregated financial data.
- Implement payroll entry handlers: Implement GET /api/v1/payroll?period=YYYY-MM and POST /api/v1/payroll/entries handlers for payroll entry management.
- Implement currency rate sync background task with Valkey caching: Implement the GET /api/v1/currency/rates endpoint returning cached rates from Valkey, and a tokio background task that fetches live rates from the chosen external API every 6 hours and stores them in both Valkey and the currency_rates table.
- Implement automated overdue invoice reminder background task: Implement a daily tokio background task that queries invoices with status=sent and due_at more than 1 day past, transitions them to status=overdue, and logs a reminder entry.
- Implement JWT authentication middleware: Add Tower/Axum middleware that validates JWT Bearer tokens on all /api/v1/* routes, rejecting requests without a valid token with HTTP 401.
- Add Prometheus metrics and health endpoints: Expose /metrics, /health/live, and /health/ready endpoints. Register custom Prometheus counters for invoice creation and payment recording.
- Implement GDPR export and delete endpoints: Implement GET /internal/gdpr/export/:org_id returning all finance data for an org and DELETE /internal/gdpr/delete/:org_id anonymizing org_id on invoices and payroll entries.
- Write Kubernetes Deployment and Service manifests for Finance service: Create Kubernetes Deployment (2 replicas) and ClusterIP Service manifests for the Finance service with correct port configuration, envFrom references, health probes, and resource limits.
- Write integration tests for Finance service end-to-end flows: Write integration tests covering invoice lifecycle, aging bucket correctness, Stripe webhook processing, and GDPR anonymization.
</subtasks>