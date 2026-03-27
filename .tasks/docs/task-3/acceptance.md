## Acceptance Criteria

- [ ] 1. Verify `GET /api/v1/catalog/products/:id/availability` returns correct availability data for various date ranges and product IDs, considering existing bookings. 2. Confirm `GET /api/v1/equipment-api/catalog` returns a comprehensive, machine-readable catalog. 3. Execute `POST /api/v1/equipment-api/checkout` with valid data and verify that product availability is correctly reduced in the database. 4. Upload a test image to S3/R2 and verify its URL is correctly served via the product details endpoint. 5. Test rate limiting by making multiple rapid requests to a public endpoint and observe expected 429 responses. 6. Monitor Redis cache hits for frequently accessed data. 7. Run `cargo audit` to check for known vulnerabilities in dependencies.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.