<identity>
You are tess working on subtask 8002 of task 8.
</identity>

<context>
<scope>
Write the operator lifecycle test suite covering initialization with valid and invalid params, duplicate initialization prevention, authorization checks for pause/unpause operations, and protocol fee validation.
</scope>
</context>

<implementation_plan>
1. Create `tests/01-operator.test.ts` with describe block 'Operator Lifecycle'.
2. Before-all: create funded authority wallet, derive operator PDA.
3. Test: 'initializes operator with valid params' — call initialize_operator with protocol_fee_bps=500 (5%), treasury pubkey. Assert PDA created, fields match: authority, treasury, protocol_fee_bps=500, paused=false.
4. Test: 'fails to initialize operator twice' — call initialize_operator again with same authority. Assert error (PDA already exists / account already initialized).
5. Test: 'fails when non-authority tries to pause' — create a different wallet, call pause with non-authority signer. Assert Unauthorized error.
6. Test: 'authority pauses and unpauses' — call pause with authority → fetch operator, assert paused=true. Call unpause → assert paused=false.
7. Test: 'fails to initialize with protocol_fee_bps > 10000' — use a fresh operator PDA (or this may need a separate test program deploy scenario). If PDA-locked, test with a different seed. Assert InvalidFee or similar error.
8. Each test should clean up or use unique PDAs where needed to avoid interference.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'Operator Lifecycle'` — all 5 tests pass. Verify on-chain state after each mutation matches expected values. Error tests produce the correct Anchor error codes.
</validation>