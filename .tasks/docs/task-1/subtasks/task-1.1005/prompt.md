<identity>
You are rex working on subtask 1005 of task 1.
</identity>

<context>
<scope>
Implement the initialize_operator, create_customer_account, register_agent_package, and update_agent_package instructions with full validation and Anchor constraints.
</scope>
</context>

<implementation_plan>
1. Create `instructions/initialize_operator.rs`:
   - Define `InitializeOperator` accounts struct: authority (Signer), operator_config (init, PDA seeds=[b"operator_config"], payer=authority), system_program, mint (Account<Mint>), treasury_ata (Account<TokenAccount> — validated as associated token account for treasury + mint).
   - Args: treasury (Pubkey), protocol_fee_bps (u16).
   - Handler: set operator_config.authority = authority.key(), .treasury = treasury, .mint = mint.key(), .protocol_fee_bps = protocol_fee_bps, .paused = false.

2. Create `instructions/create_customer_account.rs`:
   - Define `CreateCustomerAccount` accounts struct: customer (Signer), customer_balance (init, PDA seeds=[b"customer_balance", customer.key()], payer=customer), system_program.
   - Args: max_per_task (u64), max_per_day (u64).
   - Handler: set all fields, counters to 0, created_at from Clock sysvar.

3. Create `instructions/register_agent_package.rs`:
   - Define `RegisterAgentPackage` accounts struct: author (Signer), agent_package (init, PDA seeds=[b"agent_package", &hash_seed(&package_id)], payer=author), operator_config, author_ata (Account<TokenAccount> — validated as ATA for author + operator_config.mint, NOT created), system_program, clock (Sysvar<Clock>).
   - Args: package_id (String), split_bps (u16), source_uri (String).
   - Validation: require!(split_bps <= 10000, InvalidSplitBps).
   - Handler: set all fields, counters to 0, registered_at from clock, active = true.

4. Create `instructions/update_agent_package.rs`:
   - Define `UpdateAgentPackage` accounts struct: author (Signer), agent_package (mut).
   - Constraint: author.key() == agent_package.author → Unauthorized.
   - Args: source_uri (Option<String>), split_bps (Option<u16>), active (Option<bool>).
   - Handler: update only provided fields. Validate split_bps <= 10000 if provided.

5. Register all four instructions in lib.rs `#[program]` module.
6. Export all instruction modules from `instructions/mod.rs`.
</implementation_plan>

<validation>
Run `anchor build` — all four instructions compile. Verify the generated IDL contains initialize_operator, create_customer_account, register_agent_package, and update_agent_package with correct argument types. Verify PDA seeds in the IDL match specifications.
</validation>