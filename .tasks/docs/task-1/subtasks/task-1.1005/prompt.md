<identity>
You are bolt working on subtask 1005 of task 1.
</identity>

<context>
<scope>
Generate a Solana keypair for devnet operator use, airdrop SOL to it, store it securely in 1Password for the secrets pipeline, and ensure it is gitignored.
</scope>
</context>

<implementation_plan>
1. Run `solana-keygen new -o devnet-operator-keypair.json --no-bip39-passphrase` locally.
2. Record the public key for reference.
3. Fund on devnet: `solana airdrop 5 --keypair devnet-operator-keypair.json --url devnet`. Retry if rate-limited.
4. Verify balance: `solana balance --keypair devnet-operator-keypair.json --url devnet` shows ≥ 5 SOL.
5. Upload the keypair JSON content to the 1Password item `cto-pay-operator-keypair` created in subtask 1003. This ensures the ExternalSecret pipeline delivers this specific devnet keypair to the cluster.
6. Verify `devnet-operator-keypair.json` is listed in `.gitignore` (should already be there from subtask 1001).
7. Delete the local `devnet-operator-keypair.json` after uploading to 1Password.
8. Document the devnet operator public key in `config/devnet.json` under `operatorPublicKey` field.
</implementation_plan>

<validation>
Verify `solana balance` for the operator pubkey on devnet shows ≥ 5 SOL. Verify 1Password item `cto-pay-operator-keypair` contains valid 64-integer JSON array. Verify `devnet-operator-keypair.json` is in `.gitignore`. Verify `config/devnet.json` contains `operatorPublicKey` as a 44-character base58 string.
</validation>