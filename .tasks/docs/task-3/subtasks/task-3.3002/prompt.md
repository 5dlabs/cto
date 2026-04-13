<identity>
You are rex working on subtask 3002 of task 3.
</identity>

<context>
<scope>
Create the deposit instruction that transfers SPL tokens from the customer's token account to the program vault and credits the customer's on-chain balance using checked arithmetic.
</scope>
</context>

<implementation_plan>
Create `instructions/deposit.rs`. Define the `Deposit` Anchor accounts struct with: `customer` (Signer), `customer_balance` (mut, has_one = customer), `customer_token_account` (mut, Account<TokenAccount> with constraint `customer_token_account.mint == operator_config.mint @ CtoPayError::InvalidMint`), `vault` (mut, Account<TokenAccount>), `operator_config` (Account read-only), `token_program` (Program<Token>). Handler args: `amount: u64`. Logic: (1) Check `!operator_config.paused` → ProgramPaused. (2) Check `amount > 0` → ZeroAmount. (3) Build `anchor_spl::token::Transfer` CPI context with authority = customer signer, from = customer_token_account, to = vault. (4) Call `anchor_spl::token::transfer(cpi_ctx, amount)?`. (5) Update customer_balance: `balance = balance.checked_add(amount).ok_or(CtoPayError::ArithmeticOverflow)?`, `total_deposited = total_deposited.checked_add(amount).ok_or(CtoPayError::ArithmeticOverflow)?`. Export from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles. IDL contains `deposit` instruction with `amount` (u64) arg. Code review: SPL transfer uses `anchor_spl::token::Transfer` (not raw invoke). Code review: mint constraint is `customer_token_account.mint == operator_config.mint`. Code review: pause check present. Code review: both balance and total_deposited use checked_add.
</validation>