Implement task 4: Develop Finance Service (Rex - Rust/Axum)

## Goal
Implement the Finance Service for invoicing, payments, and financial reporting. This service replaces traditional accounting software and integrates with the RMS for project-based invoicing.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1, 3

## Implementation Plan
1. Initialize a new Rust project targeting Rust 1.77.2.
2. Set up Axum 0.7.5.
3. Define `Invoice`, `Payment`, and `InvoiceStatus` data models. Implement database migrations for these schemas.
4. Implement endpoints:
    - `POST /api/v1/invoices`
    - `GET /api/v1/invoices`, `GET /api/v1/invoices/:id`
    - `POST /api/v1/invoices/:id/send`, `POST /api/v1/invoices/:id/paid`
    - `POST /api/v1/payments`, `GET /api/v1/payments`, `GET /api/v1/payments/invoice/:id`
    - Finance reports: `GET /api/v1/finance/reports/revenue`, `aging`, `cashflow`, `profitability`
    - Payroll: `GET /api/v1/payroll`, `POST /api/v1/payroll/entries`
    - Currency: `GET /api/v1/currency/rates`
5. Integrate with PostgreSQL using `sqlx` and Redis for currency rate caching, referencing the `sigma1-infra-endpoints` ConfigMap.
6. Implement basic Stripe integration for recording payments (initial focus on recording, not full payment processing).
7. Implement logic for quote-to-invoice conversion, requiring integration with the RMS service (Task 3) to fetch project details.
8. Implement a scheduled job for currency rate synchronization (e.g., using a background worker or cron job within the service).

## Acceptance Criteria
1. Deploy the service to Kubernetes and verify it starts successfully.
2. Create a mock project in RMS (Task 3) and then test invoice creation from that project ID.
3. Verify invoice status transitions (draft, sent, paid).
4. Test payment recording and retrieval for an invoice.
5. Confirm financial reports return data (even if mock data initially).
6. Verify currency rates are cached and updated by the scheduled job.
7. Ensure integration with RMS for project data retrieval is functional.

## Subtasks
- Implement Develop Finance Service (Rex - Rust/Axum): Implement the Finance Service for invoicing, payments, and financial reporting. This service replaces traditional accounting software and integrates with the RMS for project-based invoicing.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.