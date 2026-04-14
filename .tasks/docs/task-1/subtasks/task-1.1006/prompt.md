<identity>
You are rex working on subtask 1006 of task 1.
</identity>

<context>
<scope>
Implement the deposit and withdraw instructions with SPL token transfers between customer ATAs and the program vault PDA, including vault initialization logic.
</scope>
</context>

<implementation_plan>
1. Create `instructions/deposit.rs`:
   - Define `Deposit` accounts struct: customer (Signer), customer_balance (mut, PDA), customer_ata (mut, Account<TokenAccount>), vault (mut, Account<TokenAccount> — PDA seeds=[b"vault", mint.key()]), operator_config, mint (Account<Mint>), token_program.
   - Constraint: customer_ata.mint == operator_config.mint → InvalidMint.
   - Args: amount (u64).
   - Validation: require!(amount > 0, InvalidAmount).
   - Handler: Execute SPL token transfer from customer_ata to vault using CPI. Increment customer_balance.balance += amount and customer_balance.total_deposited += amount.
   - Emit Deposited event (define event struct here or import from events module).

2. Create `instructions/withdraw.rs`:
   - Define `Withdraw` accounts struct: customer (Signer), customer_balance (mut, PDA), customer_ata (mut), vault (mut), vault_authority (PDA signer), operator_config, mint, token_program.
   - Constraint: customer.key() == customer_balance.customer → Unauthorized.
   - Constraint: amount <= customer_balance.balance → InsufficientBalance.
   - Args: amount (u64).
   - Handler: Execute SPL token transfer from vault to customer_ata using PDA signer seeds `[b"vault", mint.key(), &[bump]]`. Decrement customer_balance.balance -= amount.
   - Emit Withdrawn event.

3. Ensure the vault token account is initialized in initialize_operator or via a separate `initialize_vault` instruction (use `init_if_needed` or `init` with the vault PDA seeds and token program). The vault authority is the PDA itself.

4. Register both instructions in lib.rs `#[program]` module.
</implementation_plan>

<validation>
Run `anchor build` — deposit and withdraw instructions compile. Verify SPL token CPI calls use correct signer seeds. Verify vault PDA derivation is consistent. Check that the IDL contains deposit and withdraw instructions with correct accounts and args.
</validation>