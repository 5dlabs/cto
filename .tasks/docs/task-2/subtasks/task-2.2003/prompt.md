<identity>
You are rex working on subtask 2003 of task 2.
</identity>

<context>
<scope>
Create the initialize_operator instruction handler and its Accounts struct, including vault token account initialization via anchor_spl, protocol_fee_bps validation, and OperatorConfig PDA initialization.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-pay/src/instructions/mod.rs` re-exporting `initialize_operator`.
2. Create `programs/cto-pay/src/instructions/initialize_operator.rs`:
3. Define the `InitializeOperator` Accounts struct:
   ```rust
   #[derive(Accounts)]
   pub struct InitializeOperator<'info> {
       #[account(mut)]
       pub authority: Signer<'info>,
       
       #[account(
           init,
           payer = authority,
           space = 8 + OperatorConfig::INIT_SPACE,
           seeds = [OPERATOR_CONFIG_SEED],
           bump,
       )]
       pub operator_config: Account<'info, OperatorConfig>,
       
       #[account(
           init,
           payer = authority,
           seeds = [VAULT_SEED, operator_config.key().as_ref()],
           bump,
           token::mint = mint,
           token::authority = operator_config,
       )]
       pub vault: Account<'info, TokenAccount>,
       
       pub mint: Account<'info, Mint>,
       
       /// CHECK: Validated as token account with correct mint
       #[account(
           constraint = treasury.mint == mint.key() @ CtoPayError::InvalidMint,
       )]
       pub treasury: Account<'info, TokenAccount>,
       
       pub system_program: Program<'info, System>,
       pub token_program: Program<'info, Token>,
       pub rent: Sysvar<'info, Rent>,
   }
   ```
4. Implement the handler function:
   ```rust
   pub fn handler(ctx: Context<InitializeOperator>, protocol_fee_bps: u16) -> Result<()> {
       require!(protocol_fee_bps <= 10_000, CtoPayError::InvalidFeeBps);
       
       let config = &mut ctx.accounts.operator_config;
       config.authority = ctx.accounts.authority.key();
       config.treasury = ctx.accounts.treasury.key();
       config.mint = ctx.accounts.mint.key();
       config.protocol_fee_bps = protocol_fee_bps;
       config.paused = false;
       config.bump = ctx.bumps.operator_config;
       
       Ok(())
   }
   ```
5. Use `anchor_spl::token::{Token, TokenAccount, Mint}` imports.
6. Ensure NO use of `init_if_needed` — only `init`.
7. Ensure the vault token account authority is the `operator_config` PDA (the program can sign for it via PDA seeds in future transfer instructions).
8. Add appropriate `use` statements for constants, errors, and state types.
</implementation_plan>

<validation>
Run `anchor build` — compiles with zero errors and zero warnings under clippy::pedantic. Verify IDL contains `initialize_operator` instruction with `protocol_fee_bps: u16` argument. Verify IDL accounts list includes `authority`, `operatorConfig`, `vault`, `mint`, `treasury`, `systemProgram`, `tokenProgram`, `rent`. Verify vault account in IDL shows it is a token account type.
</validation>