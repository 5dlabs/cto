## Acceptance Criteria

- [ ] Run `npm run build` — Next.js production build completes with zero errors and zero TypeScript errors. Run `npm run dev` and verify at http://localhost:3000: (1) Page loads in under 3 seconds with dark background and Solana purple/green accent colors visible, (2) Header shows CTO logo, 'Devnet' badge, and wallet connect button, (3) Clicking wallet connect opens modal showing Phantom and Solflare options, (4) Split-panel layout is visible with left (60%) and right (40%) panels on a 1920×1080 viewport, (5) Panels stack vertically on 375px mobile viewport. Run Lighthouse accessibility audit — score ≥ 80. Run `npx tsc --noEmit` — zero type errors. Verify `useProgram()` hook instantiates without errors when wallet is connected (use Phantom devnet with test SOL).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.