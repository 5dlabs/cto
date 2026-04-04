## Decision Points

- WebSocket connection for Morgan chat: connect directly to Morgan agent backend, or route through an API gateway / BFF endpoint? This affects CORS, auth, and deployment topology.
- Quote builder submission target: submit directly to RMS opportunities API endpoint, or route through Morgan agent for conversational follow-up? The two flows have different UX and integration implications.
- R2 CDN image loader: use a custom Next.js loader pointing at R2 public bucket URL, or use Cloudflare Image Resizing / Image Transformations for on-the-fly optimization? Impacts performance and cost.
- Effect 3.x + TanStack Query integration pattern: wrap Effect programs inside TanStack Query's queryFn, or build a custom Effect-native caching/fetching layer? No widely established pattern exists for this combination.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React 19/Next.js 15 + Effect 3.x