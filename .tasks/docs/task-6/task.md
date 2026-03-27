## Finance Service - Database & Core Invoicing API (Rex - Rust/Axum)

### Objective
Develop the core Finance service, establishing its PostgreSQL database schema and implementing initial API endpoints for invoice creation, listing, retrieval, and status updates. This lays the groundwork for financial operations.

### Ownership
- Agent: Rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize a new Rust Axum 0.7 project. 2. Define the PostgreSQL schema for `Invoice` and `Payment` data models, including `id`, `project_id`, `org_id`, `invoice_number`, `status`, `issued_at`, `due_at`, `currency`, `subtotal_cents`, `tax_cents`, `total_cents`, `paid_amount_cents`, `stripe_invoice_id` for invoices, and `id`, `invoice_id`, `amount_cents`, `currency`, `method`, `stripe_payment_id`, `received_at` for payments. Use `sqlx` for database interactions. 3. Implement API endpoints: `POST /api/v1/invoices` (create new invoice), `GET /api/v1/invoices` (list invoices), `GET /api/v1/invoices/:id` (get invoice details). 4. Implement endpoints for invoice status updates: `POST /api/v1/invoices/:id/send` (mark as sent), `POST /api/v1/invoices/:id/paid` (record payment). 5. Configure the service to connect to PostgreSQL using credentials from the 'sigma1-infra-endpoints' ConfigMap. Ensure proper error handling, input validation, and currency handling. Use Rust 1.75+.

### Subtasks
- [ ] Implement Finance Service - Database & Core Invoicing API (Rex - Rust/Axum): Develop the core Finance service, establishing its PostgreSQL database schema and implementing initial API endpoints for invoice creation, listing, retrieval, and status updates. This lays the groundwork for financial operations.