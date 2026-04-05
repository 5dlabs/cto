## Acceptance Criteria

- [ ] 1. Component unit tests (Vitest + React Testing Library): ProductCard renders name, image, price; AvailabilityCalendar shows available/unavailable dates; ChatWidget opens/closes correctly. 2. Page integration tests: `/equipment` page renders product grid after mocking catalog API; `/equipment/:id` shows product details and availability calendar. 3. Quote builder flow test: add 2 products, fill event details, submit, verify API call made with correct payload. 4. Chat widget test: mock WebSocket, send message, verify message appears in chat, verify typing indicator shows during response. 5. SEO test: verify Schema.org JSON-LD present on home page (Organization), equipment page (Product). Verify `llms.txt` returns plain text with correct content. 6. Accessibility audit: run axe-core on all pages, verify zero critical/serious violations. 7. Lighthouse CI: Performance > 90, Accessibility > 95, Best Practices > 90 on home and equipment pages. 8. Cloudflare Pages build: `npx @cloudflare/next-on-pages` completes without errors. 9. Responsive test: verify equipment grid renders correctly at mobile (375px), tablet (768px), desktop (1440px) widths.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.