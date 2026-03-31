Implement task 4: Implement Finance Service (Rex - Rust/Axum)

## Goal
Create the Finance microservice for invoicing, payments, payroll, and financial reporting. Integrates with Stripe and supports multi-currency operations.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+, Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust Axum project with PostgreSQL, Redis, and Stripe API integration.", "Define data models for Invoice, Payment, Payroll, and related enums.", "Implement endpoints: /api/v1/invoices, /api/v1/payments, /api/v1/finance/reports/*, /api/v1/payroll, /api/v1/currency/rates.", "Add scheduled job for currency rate sync and cache in Redis.", "Implement automated payment reminders and AR aging logic.", "Integrate Stripe for payment creation and webhook handling.", "Add Prometheus metrics and health checks.", "Document OpenAPI spec."]}

## Acceptance Criteria
All endpoints return correct data for seeded invoices/payments. Stripe integration processes test payments. Currency rates update on schedule. Reports generate within 5 seconds. Health and metrics endpoints respond.

## Subtasks
- Scaffold Rust/Axum project with dependency configuration: Initialize the Finance service Cargo project with Axum 0.7, configure crate dependencies for PostgreSQL (sqlx), Redis, Stripe SDK, serde, tokio, and establish the application entrypoint with Axum router skeleton, graceful shutdown, and configuration loading from environment variables via the infra-endpoints ConfigMap (envFrom).
- Define data models and SQLx migrations for Invoice, Payment, Payroll, and financial enums: Create the PostgreSQL schema migrations and corresponding Rust struct models for all finance domain entities: Invoice, InvoiceLineItem, Payment, PayrollRecord, CurrencyRate, and supporting enums (InvoiceStatus, PaymentStatus, PaymentMethod, PayrollFrequency, Currency).
- Implement invoice CRUD endpoints: Build the /api/v1/invoices endpoints for creating, reading, updating, listing (with filtering/pagination), and deleting invoices, including line item management.
- Implement payment CRUD endpoints: Build the /api/v1/payments endpoints for recording, listing, and managing payments, including linking payments to invoices and updating invoice status on payment completion.
- Implement Stripe payment creation and webhook handling: Integrate with the Stripe API for creating PaymentIntents, handling checkout flows, and processing Stripe webhooks with idempotent event handling.
- Implement payroll endpoints: Build the /api/v1/payroll endpoints for creating, listing, and managing payroll records.
- Implement financial reporting endpoints: Build the /api/v1/finance/reports/* endpoints for revenue reports, AR aging, profit/loss summaries, and expense breakdowns.
- Implement automated payment reminders and AR aging logic: Build the background job that detects overdue invoices, updates their status, and triggers payment reminder notifications.
- Implement currency rate sync scheduled job with Redis caching: Build the background job that periodically fetches currency exchange rates from an external API and caches them in Redis, plus the /api/v1/currency/rates endpoint.
- Add Prometheus metrics, health checks, and OpenAPI documentation: Instrument the Finance service with Prometheus metrics, implement health/readiness endpoints, and generate OpenAPI specification.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.