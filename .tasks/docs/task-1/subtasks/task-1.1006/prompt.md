<identity>
You are bolt working on subtask 1006 of task 1.
</identity>

<context>
<scope>
Write the devnet bootstrap script that airdrops SOL, creates a custom SPL token mint with 6 decimals, mints test tokens to customer wallets, and writes all addresses to config/devnet.json.
</scope>
</context>

<implementation_plan>
1. Create `scripts/setup-devnet.sh` with bash strict mode (`set -euo pipefail`).
2. Script parameters: operator keypair path (default: read from env or `./devnet-operator-keypair.json`), RPC URL (default: devnet).
3. Steps in the script:
   a. Airdrop 2 SOL to operator wallet (if balance < 2 SOL).
   b. Generate 2 test customer keypairs (ephemeral, stored in `config/test-customer-1.json` and `config/test-customer-2.json`). Add these to `.gitignore`.
   c. Airdrop 2 SOL to each test customer.
   d. Create a custom SPL token mint with 6 decimals: `spl-token create-token --decimals 6 --keypair <operator>`.
   e. Capture the mint address.
   f. Create associated token accounts for operator, customer-1, customer-2.
   g. Mint 1,000,000 test tokens (1,000,000 * 10^6 = 1_000_000_000_000 base units) to each customer.
   h. Write `config/devnet.json`:
      ```json
      {
        "mintAddress": "<MINT_PUBKEY>",
        "operatorPublicKey": "<OPERATOR_PUBKEY>",
        "testCustomer1": "<CUSTOMER1_PUBKEY>",
        "testCustomer2": "<CUSTOMER2_PUBKEY>",
        "decimals": 6,
        "network": "devnet"
      }
      ```
4. Make script executable: `chmod +x scripts/setup-devnet.sh`.
5. Add `config/test-customer-*.json` to `.gitignore`.
</implementation_plan>

<validation>
Run `scripts/setup-devnet.sh` end-to-end — exits 0. Verify `config/devnet.json` exists and contains all 6 fields. Verify `mintAddress` is a 44-character base58 string. Run `spl-token supply <mintAddress> --url devnet` — shows 2,000,000 (2M tokens minted). Run `spl-token balance --owner <customer1> <mint> --url devnet` — shows 1,000,000. Verify test customer keypair files are gitignored.
</validation>