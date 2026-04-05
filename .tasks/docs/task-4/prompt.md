Implement task 4: Implement Finance Service (Rex - Rust/Axum)

## Goal
Create the Finance Service for invoicing, payments, payroll, and financial reporting, with Stripe integration and multi-currency support.

## Task Context
- Agent owner: Rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Rust 1.75+ Axum 0.7 project.", "Define Invoice, Payment, and Payroll models as per PRD.", "Implement endpoints: /api/v1/invoices, /api/v1/payments, /api/v1/finance/reports/*, /api/v1/payroll, /api/v1/currency/rates.", "Integrate with PostgreSQL for finance data.", "Integrate with Stripe API for payments (use secret from Kubernetes).", "Implement scheduled job for currency rate sync and cache in Redis.", "Add automated payment reminders and AR aging logic.", "Implement tax calculation for GST/HST, US sales tax, and international.", "Add Prometheus metrics and health endpoints.", "Document OpenAPI spec for endpoints."]}

## Acceptance Criteria
All endpoints return correct data as per OpenAPI spec; Stripe payments are processed and recorded; multi-currency invoices and reports are accurate; scheduled currency sync updates rates; AR aging and payroll reports generate correctly.

## Subtasks
- Initialize Rust/Axum project with finance data models and PostgreSQL schema: Set up the Rust 1.75+ Axum 0.7 project structure, define domain models for Invoice, Payment, and Payroll, and create PostgreSQL migrations for the finance schema.
- Implement invoice CRUD endpoints and AR aging logic: Build the /api/v1/invoices REST endpoints for creating, reading, updating, and listing invoices, plus accounts receivable aging report logic.
- Implement Stripe payment integration with webhook handling: Integrate with Stripe API for processing payments, recording payment results, and handling Stripe webhooks for asynchronous payment events.
- Implement multi-currency support with scheduled exchange rate sync: Build the currency rate sync scheduled job, Redis caching layer for rates, and multi-currency conversion utilities used across invoice and payment operations.
- Implement tax calculation engine for GST/HST, US sales tax, and international: Build a configurable tax calculation engine that supports Canadian GST/HST, US state sales tax, and international tax rules, integrated into invoice creation.
- Implement payroll endpoints and financial reporting: Build the payroll management endpoints and financial reporting endpoints including revenue, expense, and payroll summary reports.
- Add Prometheus metrics, health endpoints, and OpenAPI documentation: Implement observability with Prometheus metrics, health/readiness probes, and generate OpenAPI documentation for all finance endpoints.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.