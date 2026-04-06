## Implement Finance Service (Rex - Rust/Axum)

### Objective
Create the Finance backend for invoicing, payments, payroll, and financial reporting, with Stripe integration and multi-currency support.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Set up Rust Axum service with PostgreSQL, Redis, and Stripe API keys from 'sigma1-infra-endpoints'", "Define Invoice, Payment, and Payroll models as per PRD", "Implement endpoints for invoices, payments, finance reports, payroll, and currency rates", "Integrate Stripe for payment processing and webhooks", "Schedule currency rate sync job and cache rates in Redis", "Implement automated payment reminders and AR aging logic", "Add tax calculation for GST/HST, US sales tax, and international", "Write OpenAPI spec and ensure endpoints are Effect-compatible for frontend consumption"]}

### Subtasks
- [ ] Scaffold Rust/Axum service with PostgreSQL and Redis connectivity: Initialize the Rust project with Axum 0.7, set up PostgreSQL connection pool via sqlx, Redis client, and health check endpoints, reading all connection details from the sigma1-infra-endpoints ConfigMap.
- [ ] Define database models and migrations for Invoice, Payment, and Payroll: Create PostgreSQL schema migrations for invoices, payments, payroll records, and currency rates using sqlx migrations, with multi-currency support built into the data model.
- [ ] Implement invoice and payment CRUD endpoints: Build Axum route handlers for creating, reading, updating, listing, and deleting invoices and payments, with proper validation, error handling, and JSON serialization.
- [ ] Integrate Stripe for payment processing and webhook handling: Implement Stripe payment intent creation, payment processing, and webhook handling to record Stripe payment events against invoices.
- [ ] Implement currency rate sync job with Redis caching: Build a scheduled background job that fetches current exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis for fast multi-currency conversions.
- [ ] Implement financial reporting endpoints and AR aging logic: Build endpoints for accounts receivable aging reports, revenue reports, and payroll summary reports with configurable date ranges and grouping.
- [ ] Implement payroll processing endpoints: Build endpoints for creating, approving, and processing payroll records, with support for hourly rates, deductions, and batch processing.
- [ ] Implement tax calculation engine for GST/HST, US sales tax, and international: Build a modular tax calculation service that computes applicable taxes based on jurisdiction, integrating into invoice creation and reporting.
- [ ] Implement automated payment reminder scheduling: Build a background job that identifies invoices approaching or past their due date and triggers payment reminder notifications.
- [ ] Generate OpenAPI specification and write integration tests: Generate an OpenAPI 3.0 spec for all Finance service endpoints using utoipa, and write comprehensive integration tests covering all routes with >80% code coverage.