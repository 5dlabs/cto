<identity>
You are rex working on subtask 3007 of task 3.
</identity>

<context>
<scope>
Write comprehensive Rust unit tests covering the payment split arithmetic edge cases, overflow protection, rounding behavior, and boundary conditions across all settle_task and refund_task code paths.
</scope>
</context>

<implementation_plan>
1. Create a test module in `programs/cto-billing/src/instructions/settle_task.rs` (or a dedicated `tests/` module) with `#[cfg(test)]`:
   - Test split calculation helper function (extract the split logic into a pure function for unit testing):
     a. `split_bps=0, amount=1_000_000` → author_share=0, platform_share=1_000_000.
     b. `split_bps=10000, amount=1_000_000` → author_share=1_000_000, platform_share=0.
     c. `split_bps=3000, amount=100` → author_share=3, platform_share=97 (floor division).
     d. `split_bps=3333, amount=1` → author_share=0, platform_share=1.
     e. `split_bps=5000, amount=999_999` → author_share=499_999, platform_share=500_000 (verify no off-by-one).
     f. `split_bps=1, amount=u64::MAX` → verify no overflow with u128 intermediate.
     g. `split_bps=9999, amount=u64::MAX` → verify no overflow.
   - Test that author_share + platform_share always equals amount for all tested inputs.
2. Add integration-level tests in `tests/` (TypeScript/Mocha with Anchor):
   - Test the 10 scenarios from the parent test_strategy if not already covered by subtasks 3003-3006.
   - Verify IDL has all expected instructions: register_agent_package, update_agent_package, settle_task, refund_task (plus existing instructions from Task 2).
3. Run `anchor build && anchor test` to verify everything compiles and all tests pass.
4. Ensure the extracted split function uses the chosen overflow strategy (u128 intermediate) consistently.
</implementation_plan>

<validation>
Run `cargo test` in the program crate — all unit tests for split calculation pass, covering 0 bps, 10000 bps, fractional amounts, u64::MAX overflow safety, and author_share + platform_share == amount invariant. Run `anchor test` — full integration suite passes with all 10 scenarios from the parent task test_strategy verified. IDL inspection confirms all instructions are present.
</validation>