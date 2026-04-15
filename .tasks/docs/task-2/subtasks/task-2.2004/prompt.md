<identity>
You are rex working on subtask 2004 of task 2.
</identity>

<context>
<scope>
Implement the three operator authority instructions: initialize_operator to create the OperatorConfig PDA, and pause/unpause to toggle the circuit breaker.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/instructions/mod.rs` with pub mod exports. 2. Create `initialize_operator.rs`: Define `InitializeOperator` accounts struct with `#[derive(Accounts)]` — operator (Signer, mut), operator_config (init, PDA seeds [b"operator_config"], payer = operator, space), system_program. Instruction handler sets authority = operator.key(), treasury from args (validate != Pubkey::default()), protocol_fee_bps from args (validate <= 10000), paused = false. 3. Create `pause.rs`: Define `Pause` accounts struct — authority (Signer), operator_config (mut, has_one = authority). Handler sets operator_config.paused = true. 4. Create `unpause.rs`: Define `Unpause` accounts struct — authority (Signer), operator_config (mut, has_one = authority). Handler sets operator_config.paused = false. 5. Register all three instructions in the `#[program]` module in lib.rs. 6. Use `require!` macros with appropriate BillingError variants for all validations.
</implementation_plan>

<validation>
anchor build compiles all instructions. initialize_operator with valid params creates PDA with correct fields. initialize_operator with zero treasury address fails with InvalidTreasury. initialize_operator with fee > 10000 fails with InvalidProtocolFee. pause by authority succeeds, sets paused = true. pause by non-authority fails with Unauthorized (Anchor constraint). unpause restores paused = false.
</validation>