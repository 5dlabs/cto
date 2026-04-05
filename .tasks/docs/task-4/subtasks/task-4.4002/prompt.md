Implement subtask 4002: Implement invoice CRUD endpoints and AR aging logic

## Objective
Build the /api/v1/invoices REST endpoints for creating, reading, updating, and listing invoices, plus accounts receivable aging report logic.

## Steps
1. Implement Axum route handlers in /src/routes/invoices.rs: POST /api/v1/invoices (create), GET /api/v1/invoices (list with filters: status, customer, date range), GET /api/v1/invoices/:id (get by ID), PUT /api/v1/invoices/:id (update), POST /api/v1/invoices/:id/send (mark as sent, trigger reminder scheduling). 2. Implement invoice line item management: each invoice contains line items with description, quantity, unit_price, tax_rate. Calculate subtotal, tax, and total on create/update. 3. Implement AR aging logic in /src/services/aging.rs: categorize outstanding invoices into buckets (Current, 1-30 days, 31-60 days, 61-90 days, 90+ days overdue). 4. Add endpoint GET /api/v1/finance/reports/ar-aging that returns the aging report with totals per bucket. 5. Implement automated payment reminder logic: identify invoices approaching or past due date, expose a function that can be called by a scheduled task to generate reminder notifications (store reminder records). 6. Implement proper error handling with consistent JSON error responses.

## Validation
Invoice CRUD returns correct HTTP status codes and data; line item calculations are accurate to the cent; AR aging correctly categorizes invoices into time buckets; overdue invoices appear in the correct aging bucket; reminders are generated for past-due invoices.