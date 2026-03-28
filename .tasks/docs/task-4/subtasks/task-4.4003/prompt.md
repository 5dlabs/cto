Implement subtask 4003: Implement Invoice and Payment API endpoints

## Objective
Develop API endpoints for creating, retrieving, and managing invoices and payments, including status transitions.

## Steps
1. Implement handlers for `POST /api/v1/invoices` to create new invoices.2. Implement handlers for `GET /api/v1/invoices` and `GET /api/v1/invoices/:id`.3. Implement handlers for `POST /api/v1/invoices/:id/send` and `POST /api/v1/invoices/:id/paid`.4. Implement handlers for `POST /api/v1/payments`, `GET /api/v1/payments`, `GET /api/v1/payments/invoice/:id`.

## Validation
1. Use `curl` to create an invoice, then retrieve it.2. Test invoice status transitions (send, paid).3. Create a payment and verify it's linked to an invoice and retrievable.