<identity>
You are tess working on subtask 3002 of task 3.
</identity>

<context>
<scope>
Implement Mocha describe blocks for Groups 1-3 covering initialize_operator, create_customer_account, update_spending_caps, register_agent_package, update_agent_package instructions, including all happy paths and error cases (re-init, duplicate PDA, non-author update, invalid split_bps).
</scope>
</context>

<implementation_plan>
1. **Group 1: Initialization** — `describe('Initialization', ...)`:
   - `it('initializes operator config')`: Call initialize_operator with treasury pubkey, mock USDC mint, protocol_fee_bps=0. Fetch the operator config PDA and assert all fields (treasury, mint, fee_bps, paused=false).
   - `it('fails on re-initialization')`: Call initialize_operator again with same seeds. Assert it throws with expected Anchor error (account already initialized / PDA constraint violation).

2. **Group 2: Customer Account Management** — `describe('Customer Account Management', ...)`:
   - `it('creates customer account')`: Call create_customer_account for customer keypair with max_per_task=50_000_000, max_per_day=200_000_000. Fetch PDA, verify max values, balance=0, total_deposited=0, total_spent=0.
   - `it('updates spending caps')`: Call update_spending_caps with new values (max_per_task=75_000_000, max_per_day=300_000_000). Fetch and verify.
   - `it('fails creating duplicate customer account')`: Call create_customer_account again for same customer. Assert error.

3. **Group 3: Agent Package Registration** — `describe('Agent Package Registration', ...)`:
   - `it('registers a package')`: Call register_agent_package with id='test-agent-v1', split_bps=3000, source_uri='https://github.com/test/agent', author=author keypair. Fetch PDA, verify author, split_bps=3000, active=true, total_earned=0, task_count=0, success_count=0.
   - `it('updates package split_bps')`: Call update_agent_package, change split_bps to 2500. Fetch and verify.
   - `it('deactivates package')`: Call update with active=false. Fetch, verify active=false.
   - `it('fails update by non-author')`: Call update_agent_package signed by second_author. Assert Unauthorized error.
   - `it('fails register with split_bps=10001')`: Call register_agent_package with split_bps=10001. Assert InvalidSplitBps error.

4. Use `assert` or `chai.expect` for all assertions. Use try/catch with `assert.fail` for expected-error tests, verifying the Anchor error code matches.
</implementation_plan>

<validation>
Run `anchor test` and verify Groups 1-3 all pass: 2 tests in Group 1 (init success, re-init fail), 3 tests in Group 2 (create, update caps, duplicate fail), 5 tests in Group 3 (register, update split, deactivate, non-author fail, invalid split fail). Total: 10 tests passing with correct assertion messages. Verify error codes match the on-chain program's defined error enum.
</validation>