## Acceptance Criteria

- [ ] Run `npm run demo:localnet` against a running `solana-test-validator` with the cto-billing program deployed. The script completes all 10 phases without errors in under 120 seconds. Verify specific outputs: (1) Phase 5 shows author_earned = 3.00 USDC and treasury_received = 7.00 USDC, (2) Phase 6 shows amount_charged = 0.00 and 'Quality NOT MET' message, (3) Phase 8 receipt table displays all 3 tasks with correct amounts and quality indicators, (4) Phase 9 shows withdrawal of exactly 185.00 USDC (200 deposited - 10 - 5 charged), (5) All logged Explorer links contain valid base58 transaction signatures. Run `npx tsc --noEmit` — TypeScript compilation passes with zero errors.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.