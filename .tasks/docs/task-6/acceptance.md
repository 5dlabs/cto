## Acceptance Criteria

- [ ] 1. Deploy the service and verify it starts successfully, connecting to PostgreSQL. 2. Use `curl` or Postman to create a new invoice via `POST /api/v1/invoices` and verify it appears in `GET /api/v1/invoices` and `GET /api/v1/invoices/:id`. 3. Update an invoice status via `POST /api/v1/invoices/:id/send` and `POST /api/v1/invoices/:id/paid`, verifying the status changes in the database and via `GET /api/v1/invoices/:id`. 4. Verify API responses conform to expected JSON structures and handle invalid inputs gracefully. 5. Run `cargo test` and `cargo clippy` to ensure code quality and correctness.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.