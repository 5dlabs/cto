# Anchor Framework

Deep expertise in the Anchor framework for Solana program development.

## Project Structure
```
programs/
  my-program/
    src/
      lib.rs          # Program entry, declare_id!, #[program] module
      state.rs        # Account structs with #[account]
      instructions/   # Instruction handlers
      errors.rs       # #[error_code] enum
Anchor.toml           # Config: cluster, program IDs, test command
tests/                # TypeScript integration tests
migrations/           # Deploy scripts
```

## Key Macros & Attributes

### Program Declaration
```rust
use anchor_lang::prelude::*;

declare_id!("YourProgramId11111111111111111111111111111");

#[program]
pub mod my_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        ctx.accounts.state.data = data;
        ctx.accounts.state.authority = ctx.accounts.authority.key();
        Ok(())
    }
}
```

### Account Validation with Constraints
```rust
#[derive(Accounts)]
#[instruction(data: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + State::INIT_SPACE,
        seeds = [b"state", authority.key().as_ref()],
        bump,
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

### Account Data
```rust
#[account]
#[derive(InitSpace)]
pub struct State {
    pub authority: Pubkey,    // 32 bytes
    pub data: u64,            // 8 bytes
    #[max_len(200)]
    pub name: String,         // 4 + 200 bytes
    pub bump: u8,             // 1 byte
}
```

### Errors
```rust
#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("The provided data exceeds the maximum allowed.")]
    DataTooLarge,
}
```

## Common Constraints
| Constraint | Purpose |
|-----------|---------|
| `init` | Create and initialize account |
| `init_if_needed` | Create only if doesn't exist (requires `feature = "init-if-needed"`) |
| `mut` | Account is mutable |
| `seeds = [..]`, `bump` | PDA derivation and validation |
| `has_one = field` | Validates `account.field == other_account.key()` |
| `constraint = expr` | Custom boolean constraint |
| `close = target` | Close account, send lamports to target |
| `realloc = size, payer = x, zero = bool` | Resize account data |
| `token::mint = m, token::authority = a` | SPL token account validation |
| `associated_token::mint = m, associated_token::authority = a` | ATA validation |

## CPI (Cross-Program Invocation)
```rust
// Calling another Anchor program
let cpi_ctx = CpiContext::new(
    ctx.accounts.other_program.to_account_info(),
    other_program::cpi::accounts::DoSomething {
        authority: ctx.accounts.authority.to_account_info(),
    },
);
other_program::cpi::do_something(cpi_ctx, args)?;

// CPI with PDA signer
let seeds = &[b"auth", &[bump]];
let signer_seeds = &[&seeds[..]];
let cpi_ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
```

## Testing with Bankrun
```typescript
import { startAnchor } from 'solana-bankrun';
import { BankrunProvider } from 'anchor-bankrun';

const context = await startAnchor(".", [], []);
const provider = new BankrunProvider(context);
const program = new Program(IDL, provider);

await program.methods
  .initialize(new BN(42))
  .accounts({ state: statePda, authority: provider.wallet.publicKey })
  .rpc();

const state = await program.account.state.fetch(statePda);
assert.equal(state.data.toNumber(), 42);
```

## Build & Deploy
```bash
anchor build                          # Compile programs
anchor test                           # Build + test (localnet)
anchor test --skip-local-validator    # Test against running validator
anchor deploy --provider.cluster devnet  # Deploy to devnet
anchor idl init <program-id> --filepath target/idl/my_program.json  # Publish IDL
anchor verify <program-id>            # Verify on-chain matches local build
```

## Version Compatibility
- Anchor v0.30+ uses `declare_id!` and `#[program]`
- Anchor v0.29+ introduced `InitSpace` derive macro
- Always match `anchor-lang` version with `anchor-cli` version
- Use `avm` (Anchor Version Manager) to switch versions
