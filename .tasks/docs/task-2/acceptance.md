## Acceptance Criteria for Task 2

1. Run `anchor build` — compiles with zero warnings under `clippy::pedantic`.
2. Verify IDL generation: `target/idl/cto_pay.json` exists and contains `initialize_operator` instruction with `protocol_fee_bps` arg.
3. Verify all 3 account types appear in IDL with correct field names and types.
4. Verify `TaskReceipt` uses `task_id_hash` as `[u8; 32]`, NOT a String field.
5. Verify `OperatorConfig` contains `mint: Pubkey` field.
6. Verify error codes appear in IDL.
7. Unit test: `OperatorConfig::INIT_SPACE` matches manual size calculation (100).
8. Unit test: `CustomerBalance::INIT_SPACE` matches manual calculation (105).
9. Unit test: `TaskReceipt::INIT_SPACE` matches manual calculation (154).

_Generated from task metadata (LLM fallback)._