## Acceptance Criteria

- [ ] Run `bun run build` in `app/dashboard/` — Next.js production build succeeds with zero errors. Run `bun run lint` — no ESLint errors. Run `bun run dev` and manually verify: (1) Wallet connect flow works with Phantom (devnet). (2) After connecting a wallet with an existing CustomerBalance, the dashboard shows correct balance matching on-chain state. (3) `/receipts` page lists TaskReceipt accounts for the connected wallet — verify at least 2 receipts appear after running the demo CLI (Task 6). (4) Receipt detail page shows correct amount, quality_met status, and Arweave verification link. (5) Clicking 'Verify Hash' re-fetches the Arweave receipt and shows green checkmark when hash matches. (6) `/agents` page shows registered AgentPackage cards with correct stats. (7) Deposit 1 USDC via `/deposit` — transaction succeeds, balance updates. (8) All Solana explorer links open to the correct addresses on devnet. Lighthouse score > 80 for performance on the home page.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.