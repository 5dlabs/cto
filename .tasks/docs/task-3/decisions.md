## Decision Points

- Circuit breaker scope: Should deposit and withdraw instructions be blocked when the program is paused, or only settlement and refund? The PRD implies only settlement/refund, but this needs explicit confirmation before implementing Group 9 tests.
- Daily cap rollover testing: The local validator may not support Clock manipulation via `warp_to_slot` or `set_clock` — decide whether to test daily cap rollover via direct clock manipulation, or by mocking the day-number derivation logic, or by accepting the daily cap enforcement is only tested within a single day boundary.

## Coordination Notes

- Agent owner: tess
- Primary stack: Anchor/TypeScript/Mocha