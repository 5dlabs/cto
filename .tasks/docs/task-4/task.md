## Develop Finance Service (Rex - Rust/Axum)

### Objective
Implement the Finance Service for invoicing, payments, and financial reporting. This service replaces traditional accounting software and integrates with the RMS for project-based invoicing.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1, 3

### Implementation Details
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

### Subtasks
- [ ] Implement Develop Finance Service (Rex - Rust/Axum): Implement the Finance Service for invoicing, payments, and financial reporting. This service replaces traditional accounting software and integrates with the RMS for project-based invoicing.