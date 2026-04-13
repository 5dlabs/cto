## Acceptance Criteria for Task 5

Run `anchor test` (or `bun test tests/happy-path/`) — all tests pass. Minimum 15 happy-path test cases covering: 1 initialize_operator, 6 customer lifecycle (create, deposit x2, withdraw partial, withdraw full, update caps), 5 settlement (single settle, multi settle, fee verification, daily cap tracking, daily reset with slot warp), 2 refund (refund + withdraw refunded funds), 1 full end-to-end loop. Test suite completes in under 30 seconds using Bankrun. Zero tests skipped.

_Generated from task metadata (LLM fallback)._