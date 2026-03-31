## Decision Points

- Image zoom/pan approach: Should the comparison view use CSS transforms for a lightweight zoom, or integrate a third-party image viewer library (e.g., react-zoom-pan-pinch)? CSS transforms are simpler but limited; a library adds a dependency but provides smoother UX with touch support.
- Polling vs SSE for deliberation status updates: Should the detail page poll the API on an interval for status changes, or use Server-Sent Events for real-time push updates? Polling is simpler but adds latency; SSE requires backend support that may or may not exist from Task 3.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js