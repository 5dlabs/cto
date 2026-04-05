## Implement Finance Service (Rex - Rust/Axum)

### Objective
Create the Finance service for invoicing, payments, payroll, and financial reporting, including Stripe integration and multi-currency support. Enables automated quote-to-invoice and payment flows.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust 1.75+ project with Axum 0.7, sqlx for PostgreSQL, and redis-rs for currency rate cache.", "Define data models for Invoice, Payment, and related enums as per PRD.", "Implement endpoints for invoices, payments, finance reports, payroll, and currency rates.", "Integrate with Stripe API for payment processing and webhooks.", "Implement scheduled job for currency rate sync.", "Reference connection strings and Stripe API key from 'sigma1-infra-endpoints' ConfigMap and secrets.", "Write unit and integration tests for all endpoints and Stripe flows."]}

### Subtasks
- [ ] Scaffold Rust/Axum project with dependencies and infrastructure config: Initialize a Rust 1.75+ project with Axum 0.7 web framework, sqlx for PostgreSQL, redis-rs for caching, and configuration loading from the sigma1-infra-endpoints ConfigMap.
- [ ] Define data models and PostgreSQL migrations for finance schema: Create sqlx migrations and Rust data model structs for invoices, payments, payroll, currency rates, and all related enums in the finance PostgreSQL schema.
- [ ] Implement invoice CRUD endpoints and business logic: Build Axum REST endpoints for creating, reading, updating, listing invoices and invoice line items, including status transitions and invoice number generation.
- [ ] Implement Stripe payment processing integration: Build payment endpoints that integrate with the Stripe API for creating payment intents, processing payments against invoices, and recording payment outcomes.
- [ ] Implement Stripe webhook handler for asynchronous payment events: Build a secure Stripe webhook endpoint that processes payment lifecycle events (succeeded, failed, refunded) and updates payment/invoice records accordingly.
- [ ] Implement payroll and financial reporting endpoints: Build REST endpoints for payroll management and financial reporting including revenue summaries, payment reports, and outstanding invoice tracking.
- [ ] Implement scheduled currency rate sync job with Redis caching: Build a background job that periodically fetches exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis for fast lookups.
- [ ] Write integration tests for all finance endpoints and Stripe flows: Create comprehensive integration tests covering invoice lifecycle, payment processing with Stripe mocks, payroll workflows, currency sync, and financial reports.