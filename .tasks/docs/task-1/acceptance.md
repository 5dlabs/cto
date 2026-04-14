## Acceptance Criteria

- [ ] Run `anchor build` — compiles with zero errors and zero warnings. Verify `target/idl/cto_billing.json` exists and contains instruction definitions for all 9 instructions (initialize_operator, create_customer_account, register_agent_package, update_agent_package, deposit, withdraw, update_spending_caps, pause, unpause). Verify `target/types/cto_billing.ts` is generated. Run `anchor test` with a minimal smoke test that: (1) initializes operator config and reads it back with correct authority/treasury/mint, (2) creates a customer account with caps and verifies fields, (3) registers an agent package and verifies author/split_bps/source_uri, (4) deposits 100 USDC and verifies customer balance == 100, (5) withdraws 50 USDC and verifies balance == 50, (6) pauses and verifies paused == true. All 6 smoke tests pass on local validator in under 30 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.