## Decision Points

- Headless browser library choice: Playwright vs Puppeteer-core — Playwright has better cross-browser support but heavier install; Puppeteer-core is lighter but Bun compatibility is less tested. Need to validate which library works reliably under the Bun runtime.
- Screenshot capture execution model: Should screenshots be captured synchronously within the request lifecycle, or offloaded to a background job/queue? Synchronous is simpler but blocks the deliberation creation response; async requires a job runner (which may not exist yet).
- Variant rendering strategy: How are generated variants rendered for screenshot capture? Are they served as local HTML files, injected into the original page via DOM manipulation, or hosted on a temporary local server? This affects both the capture pipeline architecture and the fidelity of variant snapshots.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia