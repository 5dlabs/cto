<identity>
You are tess working on subtask 5002 of task 5.
</identity>

<context>
<scope>
Write initialize.test.ts covering OperatorConfig PDA creation/verification and customer-lifecycle.test.ts covering account creation, single/multiple deposits, partial/full withdrawals, and spending cap updates.
</scope>
</context>

<implementation_plan>
1. Create `tests/happy-path/initialize.test.ts`:
   - Test 'initialize_operator creates OperatorConfig PDA with correct fields': Call `initializeProgram()`, fetch the OperatorConfig account, assert authority === operator.publicKey, treasury matches, mint matches, protocol_fee_bps === 500, paused === false.
   - Test 'vault token account is created and owned by program PDA': After initialization, fetch the vault ATA, verify its owner is the program-derived vault authority PDA and its mint matches the configured mint.
   - Test 'OperatorConfig PDA is deterministic': Re-derive the PDA using seeds and program ID, assert it matches the PDA returned during initialization.

2. Create `tests/happy-path/customer-lifecycle.test.ts`:
   - Test 'create_customer_account initializes CustomerBalance with zero balance and correct caps': Call `createCustomer()` with max_per_task=50_000_000, max_per_day=200_000_000. Fetch CustomerBalance PDA. Assert balance===0, max_per_task===50_000_000, max_per_day===200_000_000, total_deposited===0, total_spent===0, task_count===0, created_at > 0.
   - Test 'deposit increases customer balance and vault balance': Deposit 50_000_000. Fetch CustomerBalance, assert balance===50_000_000, total_deposited===50_000_000. Fetch vault token account, assert its balance increased by 50_000_000.
   - Test 'multiple deposits accumulate correctly': Deposit 50_000_000 then 30_000_000. Assert balance===80_000_000, total_deposited===80_000_000.
   - Test 'partial withdraw decreases customer balance': Deposit 100_000_000, withdraw 40_000_000. Assert balance===60_000_000. Assert customer's ATA increased by 40_000_000.
   - Test 'full withdraw leaves zero balance': Deposit 100_000_000, withdraw 100_000_000. Assert balance===0.
   - Test 'update_spending_caps changes caps correctly': Create customer with initial caps, call `update_spending_caps` with new values (max_per_task=75_000_000, max_per_day=300_000_000). Fetch CustomerBalance, assert new caps are stored.

3. All assertions use strict equality (===). Use `describe`/`it` blocks with business-scenario names.
</implementation_plan>

<validation>
Run `bun test tests/happy-path/initialize.test.ts tests/happy-path/customer-lifecycle.test.ts` — all 9 tests pass. Verify 3 initialize tests and 6 customer lifecycle tests execute with zero failures. Each test independently sets up its own context via helpers.
</validation>