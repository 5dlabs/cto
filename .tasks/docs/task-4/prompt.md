Implement task 4: Implement Finance Service (Rex - Rust/Axum)

## Goal
Create the Finance API for invoicing, payments, payroll, and financial reporting, including Stripe integration and multi-currency support.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps":["Initialize Rust 1.75+ project with Axum 0.7, using POSTGRES_URL, REDIS_URL, and Stripe API key from ConfigMap/secrets.","Define Invoice, Payment, and related models as per PRD.","Implement endpoints for invoices, payments, finance reports, payroll, and currency rates.","Integrate Stripe for payment processing and webhooks.","Implement scheduled job for currency rate sync, caching rates in Redis.","Add automated payment reminders and AR aging logic.","Ensure quote-to-invoice conversion is atomic and auditable.","Add Prometheus metrics and health endpoints."]}

## Acceptance Criteria
All endpoints return correct data. Stripe payments are processed and recorded. Currency rates are updated and cached. Automated reminders and AR aging reports are generated. Health and metrics endpoints are available.

## Subtasks
- Initialize Rust/Axum project with database schema and migrations: Set up the Rust 1.75+ project with Axum 0.7, SQLx for PostgreSQL, Redis client, and create all database migrations for the finance domain (invoices, payments, payroll, currency_rates, audit_log).
- Implement invoice CRUD endpoints with quote-to-invoice atomic conversion: Build the invoice management endpoints including create, read, update, list with filtering, and the atomic quote-to-invoice conversion flow that marks the source opportunity and creates the invoice in a single database transaction.
- Integrate Stripe for payment processing and webhook handling: Implement Stripe payment intent creation, confirmation handling, and webhook endpoint for processing asynchronous payment events (succeeded, failed, refunded).
- Implement multi-currency support with scheduled rate sync and Redis caching: Build the currency rate synchronization job that fetches exchange rates from an external API on a schedule, stores them in PostgreSQL, and caches current rates in Redis for fast lookups during invoice and payment operations.
- Implement automated payment reminders and AR aging logic: Build the accounts receivable aging report generation and automated payment reminder system that identifies overdue invoices, categorizes them into aging buckets, and triggers reminder notifications.
- Implement payroll endpoints: Build the payroll management endpoints for creating, reviewing, and processing payroll records linked to crew members and projects.
- Add Prometheus metrics and health endpoints: Instrument the Finance service with Prometheus metrics for request counts, latencies, error rates, and business metrics, plus health and readiness probes.
- End-to-end finance workflow tests: Write comprehensive integration tests covering the full invoice lifecycle, Stripe payment flow, currency conversion, AR aging, payroll, and cross-cutting audit trail.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.