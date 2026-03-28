Implement task 4: Develop Finance Service (Rex - Rust/Axum)

## Goal
Implement the Finance Service for invoicing, payments, and financial reporting. This service replaces traditional accounting software and integrates with the RMS for project-based invoicing.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1, 3

## Implementation Plan
1. Initialize a new Rust project targeting Rust 1.77.2.2. Set up Axum 0.7.5.3. Define `Invoice`, `Payment`, and `InvoiceStatus` data models. Implement database migrations for these schemas.4. Implement endpoints:    - `POST /api/v1/invoices`    - `GET /api/v1/invoices`, `GET /api/v1/invoices/:id`    - `POST /api/v1/invoices/:id/send`, `POST /api/v1/invoices/:id/paid`    - `POST /api/v1/payments`, `GET /api/v1/payments`, `GET /api/v1/payments/invoice/:id`    - Finance reports: `GET /api/v1/finance/reports/revenue`, `aging`, `cashflow`, `profitability`5. Integrate with PostgreSQL using `sqlx`.6. Implement basic Stripe integration for recording payments (initial focus on recording, not full payment processing).7. Implement logic for quote-to-invoice conversion, requiring integration with the RMS service (Task 3) to fetch project details.

## Acceptance Criteria
1. Deploy the service to Kubernetes and verify it starts successfully.2. Create a mock project in RMS (Task 3) and then test invoice creation from that project ID.3. Verify invoice status transitions (draft, sent, paid).4. Test payment recording and retrieval for an invoice.5. Confirm financial reports return data (even if mock data initially).6. Ensure integration with RMS for project data retrieval is functional.

## Subtasks
- Initialize Rust project, Axum, and define finance data models: Set up a new Rust project for the Finance Service, configure Axum, and define `Invoice`, `Payment`, `InvoiceStatus` data models with `sqlx` migrations.
- Integrate PostgreSQL for data persistence: Connect the service to PostgreSQL using `sqlx` for data persistence, utilizing the `sigma1-infra-endpoints` ConfigMap.
- Implement Invoice and Payment API endpoints: Develop API endpoints for creating, retrieving, and managing invoices and payments, including status transitions.
- Implement financial reporting API endpoints: Develop API endpoints for various financial reports (revenue, aging, cashflow, profitability).
- Implement basic Stripe integration for payment recording: Implement basic Stripe integration for recording payment events, focusing on capturing payment details rather than full processing.
- Implement RMS integration for quote-to-invoice conversion: Implement logic for quote-to-invoice conversion, requiring integration with the RMS service (Task 3) to fetch project details.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.