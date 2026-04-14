<identity>
You are rex working on subtask 1008 of task 1.
</identity>

<context>
<scope>
Write and execute the 6 smoke tests as specified in the test strategy: initialize operator, create customer, register agent package, deposit, withdraw, and pause.
</scope>
</context>

<implementation_plan>
1. Create test file `tests/cto-billing.ts` using Anchor's Mocha test framework.
2. Setup: Configure provider, program from IDL, create mock USDC mint, create ATAs for operator treasury and test customer, airdrop SOL to test wallets.
3. Test 1 — Initialize Operator: Call initialize_operator with authority, treasury, mock mint, protocol_fee_bps=500. Fetch OperatorConfig and assert authority, treasury, mint, protocol_fee_bps=500, paused=false.
4. Test 2 — Create Customer Account: Call create_customer_account with max_per_task=1000000, max_per_day=5000000. Fetch CustomerBalance and assert customer pubkey matches signer, balance=0, caps are set correctly.
5. Test 3 — Register Agent Package: Call register_agent_package with package_id="test-pkg-1", split_bps=3000, source_uri="https://example.com/pkg". Fetch AgentPackage and assert author, split_bps=3000, source_uri, active=true, counters=0.
6. Test 4 — Deposit: Mint 100 USDC (100 * 10^6 base units) to customer ATA. Call deposit(100_000_000). Fetch CustomerBalance and assert balance=100_000_000, total_deposited=100_000_000.
7. Test 5 — Withdraw: Call withdraw(50_000_000). Fetch CustomerBalance and assert balance=50_000_000. Verify customer ATA balance increased by 50_000_000.
8. Test 6 — Pause: Call pause(). Fetch OperatorConfig and assert paused=true.
9. Run all tests with `anchor test` and verify all 6 pass within 30 seconds.
10. Verify `target/idl/cto_billing.json` contains all 9 instruction definitions.
11. Verify `target/types/cto_billing.ts` is generated.
</implementation_plan>

<validation>
Execute `anchor test` — all 6 smoke tests pass with green checkmarks. Total execution time < 30 seconds on local validator. Verify IDL file exists at target/idl/cto_billing.json and contains 9 instruction definitions. Verify TypeScript types file exists at target/types/cto_billing.ts.
</validation>