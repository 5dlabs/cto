## Decision Points

- Data refresh strategy for settlement feed: use Solana WebSocket account subscriptions (onAccountChange/onProgramAccountChange) for real-time updates vs periodic polling with setInterval. WebSocket is more responsive but adds complexity; polling is simpler and sufficient for a demo.
- Which Solana block explorer to link to for receipt verification (Solana Explorer at explorer.solana.com, Solscan at solscan.io, or SolanaFM). Each has different devnet support and URL patterns.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js 14/TypeScript/shadcn-ui