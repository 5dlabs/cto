Implement task 6: Finance Service - Database & Core Invoicing API (Rex - Rust/Axum)

## Goal
Develop the core Finance service, establishing its PostgreSQL database schema and implementing initial API endpoints for invoice creation, listing, retrieval, and status updates. This lays the groundwork for financial operations.

## Task Context
- Agent owner: Rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize a new Rust Axum 0.7 project. 2. Define the PostgreSQL schema for `Invoice` and `Payment` data models, including `id`, `project_id`, `org_id`, `invoice_number`, `status`, `issued_at`, `due_at`, `currency`, `subtotal_cents`, `tax_cents`, `total_cents`, `paid_amount_cents`, `stripe_invoice_id` for invoices, and `id`, `invoice_id`, `amount_cents`, `currency`, `method`, `stripe_payment_id`, `received_at` for payments. Use `sqlx` for database interactions. 3. Implement API endpoints: `POST /api/v1/invoices` (create new invoice), `GET /api/v1/invoices` (list invoices), `GET /api/v1/invoices/:id` (get invoice details). 4. Implement endpoints for invoice status updates: `POST /api/v1/invoices/:id/send` (mark as sent), `POST /api/v1/invoices/:id/paid` (record payment). 5. Configure the service to connect to PostgreSQL using credentials from the 'sigma1-infra-endpoints' ConfigMap. Ensure proper error handling, input validation, and currency handling. Use Rust 1.75+.

## Acceptance Criteria
1. Deploy the service and verify it starts successfully, connecting to PostgreSQL. 2. Use `curl` or Postman to create a new invoice via `POST /api/v1/invoices` and verify it appears in `GET /api/v1/invoices` and `GET /api/v1/invoices/:id`. 3. Update an invoice status via `POST /api/v1/invoices/:id/send` and `POST /api/v1/invoices/:id/paid`, verifying the status changes in the database and via `GET /api/v1/invoices/:id`. 4. Verify API responses conform to expected JSON structures and handle invalid inputs gracefully. 5. Run `cargo test` and `cargo clippy` to ensure code quality and correctness.

## Subtasks
- Implement Finance Service - Database & Core Invoicing API (Rex - Rust/Axum): Develop the core Finance service, establishing its PostgreSQL database schema and implementing initial API endpoints for invoice creation, listing, retrieval, and status updates. This lays the groundwork for financial operations.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.