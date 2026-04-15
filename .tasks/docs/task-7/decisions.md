## Decision Points

- Arweave URL resolution strategy: should the receipt_hash be used to derive the Arweave gateway URL directly, or should there be an off-chain mapping service/index that maps receipt_hash to Arweave transaction IDs?
- UI component library choice: use shadcn/ui for rapid development (adds init complexity, opinionated styling) or build lightweight custom Tailwind components?
- Static export vs SSR: using `output: 'export'` means all Solana RPC calls happen client-side only — is this acceptable for the hackathon, or should some data be fetched server-side for SEO/performance?

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js