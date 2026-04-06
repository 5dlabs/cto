## Implement Finance Service (Rex - Rust/Axum)

### Objective
Develop the Finance microservice for invoicing, payments, payroll, and financial reporting. Integrate with PostgreSQL, Stripe, and Redis for currency rates.

### Ownership
- Agent: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust project with Axum 0.7, sqlx for PostgreSQL, and redis-rs for Redis.", "Define Invoice, Payment, and related models as per PRD.", "Implement endpoints: /api/v1/invoices, /api/v1/payments, /api/v1/finance/reports/*, /api/v1/payroll, /api/v1/currency/rates.", "Integrate Stripe API for payment processing.", "Implement scheduled job for currency rate sync using Redis as cache.", "Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.", "Implement Prometheus metrics and health endpoints.", "Ensure all endpoints validate input and output schemas."]}

### Subtasks
- [ ] Initialize Rust/Axum project scaffolding with sqlx and finance schema migrations: Set up the Rust project with Axum 0.7, sqlx for PostgreSQL, redis-rs, and create the `finance` schema migrations for all finance-related tables.
- [ ] Implement Invoice model and CRUD endpoints: Define the Invoice domain model, repository layer, and implement /api/v1/invoices endpoints (create, get, list, update, delete).
- [ ] Implement Payment model and CRUD endpoints: Define the Payment domain model, repository layer, and implement /api/v1/payments endpoints for recording and querying payments.
- [ ] Integrate Stripe API for payment processing: Implement a Stripe client module and wire it into the payment flow for creating payment intents, handling webhooks, and recording Stripe payment outcomes.
- [ ] Implement payroll endpoints: Build the /api/v1/payroll endpoints for creating, approving, and managing payroll records.
- [ ] Implement financial reporting endpoints: Build the /api/v1/finance/reports/* endpoints for revenue, expense, and summary financial reports.
- [ ] Implement currency rate sync scheduled job with Redis caching: Build a background job that periodically fetches currency exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis.
- [ ] Implement Prometheus metrics and health/readiness endpoints: Add Prometheus metrics instrumentation to all endpoints and implement /healthz and /readyz endpoints for the Finance service.