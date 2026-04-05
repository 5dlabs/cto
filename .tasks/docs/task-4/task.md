## Implement Finance Service (Rex - Rust/Axum)

### Objective
Create the Finance Service for invoicing, payments, payroll, and financial reporting, with Stripe integration and multi-currency support.

### Ownership
- Agent: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust project with Axum 0.7, connect to PostgreSQL, Redis, and Stripe using ConfigMap.", "Define Invoice, Payment, and related models as per PRD.", "Implement endpoints for invoices, payments, finance reports, payroll, and currency rates.", "Integrate Stripe API for payment processing and webhooks.", "Implement scheduled job for currency rate sync.", "Add Prometheus metrics and health endpoints.", "Write database migrations for finance schema.", "Document OpenAPI spec for all endpoints."]}

### Subtasks
- [ ] Initialize Rust/Axum project with PostgreSQL and Redis connectivity: Set up the Rust project with Axum 0.7 framework, configure database connection pools for PostgreSQL (via sqlx) and Redis, and establish the module structure for the finance service.
- [ ] Write database migrations for finance schema: Create SQL migration files for all finance domain tables: invoices, invoice_line_items, payments, payroll_entries, currency_rates, and supporting tables.
- [ ] Define data models and repository layer for finance entities: Implement Rust structs for all finance domain entities (Invoice, Payment, PayrollEntry, CurrencyRate) with serde and sqlx derives, plus repository functions for database CRUD operations.
- [ ] Implement invoice and payment CRUD endpoints: Build Axum route handlers for invoice creation, retrieval, listing, status updates, and payment recording endpoints with proper validation and error handling.
- [ ] Integrate Stripe API for payment processing and webhook handling: Implement Stripe PaymentIntent creation for invoices, payment confirmation flow, and Stripe webhook endpoint to handle asynchronous payment events.
- [ ] Implement payroll endpoints with multi-currency support: Build Axum route handlers for payroll entry creation, approval workflow, and payment recording, with amounts stored and displayed in the crew member's configured currency.
- [ ] Implement scheduled currency rate sync job: Create a background task that periodically fetches current exchange rates from an external API and stores them in the currency_rates table.
- [ ] Implement financial reporting endpoints: Build reporting endpoints for revenue summaries, outstanding invoices, payment history, and payroll cost reports with date range filtering.
- [ ] Add Prometheus metrics, health endpoints, and OpenAPI documentation: Instrument the service with Prometheus metrics, implement readiness/liveness probes, and generate/document the OpenAPI specification for all endpoints.
- [ ] Write integration tests for end-to-end finance workflows: Create comprehensive integration tests covering the full invoice-to-payment lifecycle, Stripe webhook processing, payroll workflow, and financial reporting accuracy.