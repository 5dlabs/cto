## Acceptance Criteria

- [ ] 1. kubectl exec into Morgan pod and run morgan tool-test sigma1_catalog_search query=lights — response contains product array with name and day_rate fields. 2. sigma1_check_availability for a known product_id with valid date range returns availability object with quantity_available field. 3. sigma1_generate_quote with customer_id and line items creates opportunity in RMS — verify via GET /api/v1/opportunities/:id returns status=pending. 4. sigma1_vet_customer with valid org data returns 202 and polling returns final_score within 60s. 5. sigma1_create_invoice creates invoice in Finance — verify via GET /api/v1/invoices/:id returns total_cents > 0. 6. sigma1_gdpr_export returns aggregated JSON containing data from at least 3 services. 7. Signal-CLI connectivity: send test message to Morgan's phone number via Signal; verify Morgan pod logs show incoming message received. 8. Web chat endpoint: POST http://morgan-chat-svc:3000/chat with {message: 'hello'} returns {reply: <non-empty string>} within 10 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.