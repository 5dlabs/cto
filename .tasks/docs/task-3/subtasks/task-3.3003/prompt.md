<identity>
You are rex working on subtask 3003 of task 3.
</identity>

<context>
<scope>
Create the withdraw instruction that transfers SPL tokens from the program vault back to the customer's token account, signed by the vault PDA authority. Crucially, this instruction does NOT check pause state — it is a safety valve allowing customers to always exit.
</scope>
</context>

<implementation_plan>
Create `instructions/withdraw.rs`. Define the `Withdraw` Anchor accounts struct with: `customer` (Signer), `customer_balance` (mut, has_one = customer), `customer_token_account` (mut, Account<TokenAccount> with mint constraint against operator_config.mint), `vault` (mut, Account<TokenAccount>), `operator_config` (Account read-only), `vault_authority` (UncheckedAccount or SystemAccount, PDA seeds = `[VAULT_SEED, operator_config.key().as_ref()]` with bump), `token_program`. Handler args: `amount: u64`. Logic: (1) Do NOT check paused — this is the PRD safety valve. (2) Check `amount > 0` → ZeroAmount. (3) Check `customer_balance.balance >= amount` → InsufficientBalance. (4) Build PDA signer seeds: `&[VAULT_SEED, operator_config.key().as_ref(), &[vault_authority_bump]]`. (5) Build `anchor_spl::token::Transfer` CPI context with_signer using the PDA seeds, from = vault, to = customer_token_account. (6) Call `anchor_spl::token::transfer(cpi_ctx, amount)?`. (7) Update: `balance = balance.checked_sub(amount).ok_or(CtoPayError::ArithmeticOverflow)?`. Ensure vault_authority bump is obtained from ctx.bumps. Export from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles. IDL contains `withdraw` instruction with `amount` (u64) arg. Critical code review: confirm NO `operator_config.paused` check exists in withdraw handler or account constraints. Code review: vault PDA signer seeds match `[VAULT_SEED, operator_config.key().as_ref(), &[bump]]`. Code review: SPL transfer uses `anchor_spl::token::Transfer` with CPI signer. Code review: balance debit uses checked_sub.
</validation>