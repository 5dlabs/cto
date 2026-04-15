<identity>
You are rex working on subtask 2006 of task 2.
</identity>

<context>
<scope>
Implement deposit (customer ATA → program vault) and withdraw (program vault → customer ATA) instructions with SPL token transfer_checked, pause checks, and balance tracking.
</scope>
</context>

<implementation_plan>
1. Create `deposit.rs`: Define `Deposit` accounts struct — customer (Signer), customer_balance (mut, has_one = customer), customer_ata (mut, token::authority = customer, token::mint = usdc_mint), vault (mut, token::authority = vault_authority PDA seeds [b"vault"]), vault_authority (PDA seeds [b"vault"]), usdc_mint, operator_config (to check paused), token_program. Args: amount (u64). Handler: require!(!operator_config.paused, ProgramPaused), require!(amount > 0, InvalidAmount), call transfer_checked from customer_ata to vault for amount with usdc_mint decimals (6), increment customer_balance.balance and total_deposited using checked_add. 2. Create `withdraw.rs`: Define `Withdraw` accounts struct — same pattern but vault_authority needs bump for PDA signing. Handler: require!(!operator_config.paused, ProgramPaused), require!(amount > 0, InvalidAmount), require!(customer_balance.balance >= amount, InsufficientBalance), CPI transfer_checked from vault to customer_ata signed by vault_authority PDA, decrement customer_balance.balance. 3. Initialize vault token account in initialize_operator or as a separate init_vault instruction if the vault doesn't exist yet — decide on creation strategy. 4. Register both instructions. 5. Call maybe_reset_daily_spending in both deposit/withdraw paths (needed for withdraw to track against daily caps if applicable).
</implementation_plan>

<validation>
Deposit 1000 USDC — customer_balance.balance = 1000, vault SPL balance = 1000, customer ATA decreased by 1000. Deposit 0 fails with InvalidAmount. Deposit while paused fails with ProgramPaused. Withdraw 500 — customer_balance.balance = 500, customer ATA increased by 500, vault decreased by 500. Withdraw more than balance fails with InsufficientBalance. Withdraw while paused fails with ProgramPaused. Multiple deposits accumulate correctly via checked_add.
</validation>