## Acceptance Criteria for Task 3

1. Run `anchor build` — compiles with zero warnings.
2. IDL contains all 4 new instructions: `create_customer_account`, `deposit`, `withdraw`, `update_spending_caps` with correct args.
3. Verify `create_customer_account` IDL shows `max_per_task` and `max_per_day` parameters.
4. Verify `deposit` and `withdraw` IDL show `amount` parameter.
5. Code review: confirm `withdraw` does NOT check `operator_config.paused`.
6. Code review: all SPL token transfers use `anchor_spl::token::Transfer` CPI, not raw invoke.
7. Code review: every `u64` operation uses `checked_*` with error propagation.
8. Code review: `customer_token_account.mint == operator_config.mint` validation present in deposit and withdraw account structs.

_Generated from task metadata (LLM fallback)._