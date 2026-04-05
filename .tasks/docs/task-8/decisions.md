## Decision Points

- Hero section media: should the home page use a looping background video or a static image carousel? Video increases visual impact but significantly impacts LCP and performance scores.
- Dark theme scope: should the site be dark-theme-only (matching lighting/production industry aesthetic) or support a light/dark toggle? This affects the entire design system token structure.
- WebSocket reconnection strategy for chat widget: simple exponential backoff vs. a library like `reconnecting-websocket`? Needs alignment with Morgan's /ws/chat endpoint behavior on reconnect (does it replay missed messages or just resume?).
- Quote builder state persistence: should in-progress quotes be persisted to localStorage so users don't lose work on accidental navigation/refresh, or rely solely on in-memory React state?
- Availability calendar data granularity: does the 90-day availability view need per-hour slot resolution or just per-day available/unavailable status? This affects API contract and calendar component complexity.

## Coordination Notes

- Agent owner: blaze
- Primary stack: Next.js 15/React 19/Effect