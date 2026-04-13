## Acceptance Criteria for Task 6

Run `bun test tests/edge-cases/` — all tests pass. Minimum 30 negative test cases covering: 7 spending cap enforcement, 7 authorization failures, 7 pause behavior (including withdraw/refund succeeding while paused), 5 overflow/arithmetic edge cases, 4 double-operation guards, 5 input validation. Every test asserts the specific `CtoPayError` variant (not just 'transaction failed'). Test suite completes in under 30 seconds using Bankrun. Combined with Task 5, total test count exceeds 45.

_Generated from task metadata (LLM fallback)._