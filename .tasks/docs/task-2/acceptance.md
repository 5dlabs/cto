## Acceptance Criteria

- [ ] 1. Deploy the service and verify it starts successfully, connecting to PostgreSQL. 2. Use `curl` or Postman to create a new category via `POST /api/v1/catalog/categories` and verify it appears in `GET /api/v1/catalog/categories`. 3. Create a new product via `POST /api/v1/catalog/products` and verify it appears in `GET /api/v1/catalog/products` and `GET /api/v1/catalog/products/:id`. 4. Update a product via `PATCH /api/v1/catalog/products/:id` and confirm changes are reflected. 5. Verify API responses conform to expected JSON structures and handle invalid inputs gracefully (e.g., 400 Bad Request for malformed data). 6. Run `cargo test` and `cargo clippy` to ensure code quality and correctness.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.