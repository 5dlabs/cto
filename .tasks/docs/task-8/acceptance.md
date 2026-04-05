## Acceptance Criteria

- [ ] 1. Component test: Quote builder renders all 4 steps, adding 3 items via useFieldArray produces correct line items array and total price calculation. 2. Component test: Equipment catalog table renders 20 items, category filter reduces results, search input triggers debounced API call. 3. Component test: Chat widget opens, sends message via WebSocket mock, displays streamed response character by character. 4. Integration test: /equipment page fetches from Equipment Catalog API, renders product cards with images, prices in correct format. 5. E2E test (Playwright): navigate Home → /equipment → click product → /equipment/:id → click Add to Quote → /quote → fill form → submit, verify API call made with correct payload. 6. Accessibility test: axe-core scan on /, /equipment, /quote pages reports zero critical/serious violations. 7. Lighthouse test: / page scores >= 90 on Performance, Accessibility, SEO. 8. Session continuity test: open chat widget, send message, navigate to /equipment, verify chat history persists. 9. llms.txt test: GET /llms.txt returns text/plain with service description and equipment category listing.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.