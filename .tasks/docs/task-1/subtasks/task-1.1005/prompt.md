<identity>
You are bolt working on subtask 1005 of task 1.
</identity>

<context>
<scope>
Create scripts/setup-devnet-usdc.sh that airdrops SOL to operator and test wallets, creates or references a devnet USDC mint, and mints test USDC tokens to relevant wallets.
</scope>
</context>

<implementation_plan>
1. Create `scripts/setup-devnet-usdc.sh` as a bash script. 2. Accept optional RPC URL argument (default to devnet or SOLANA_RPC_URL env var). 3. Airdrop 2+ SOL each to operator and test customer wallets using `solana airdrop`. 4. Check if the official devnet USDC mint (EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v) is accessible; if not, create a new SPL token mint with 6 decimals acting as mock USDC. 5. Create associated token accounts for operator and test wallets if they don't exist. 6. Mint 10,000 test USDC to operator wallet and 5,000 to each test customer wallet. 7. Print summary of balances. 8. Make script idempotent (safe to run multiple times). 9. Store the USDC mint address in a `.usdc-mint` file or export as env var for downstream use.
</implementation_plan>

<validation>
Run `scripts/setup-devnet-usdc.sh` against local validator or devnet — exits with code 0. Operator wallet has >1 SOL confirmed via `solana balance`. Operator wallet has >1000 test USDC confirmed via `spl-token balance`. Mock USDC mint address is valid and retrievable. Script is idempotent — running twice produces no errors.
</validation>