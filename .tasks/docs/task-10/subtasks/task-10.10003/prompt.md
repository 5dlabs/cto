<identity>
You are atlas working on subtask 10003 of task 10.
</identity>

<context>
<scope>
Add the devnet deployment job to the CI workflow with manual trigger support, operator keypair secret handling, SOL airdrop, anchor deploy command, and a post-deployment smoke test that runs basic CLI operations against devnet.
</scope>
</context>

<implementation_plan>
1. Add `deploy-devnet` job to ci.yml:
   - Set `needs: [anchor-test]` to depend on passing tests.
   - Add trigger: `workflow_dispatch` for manual deployment. Optionally also trigger on push to `main` branch.
   - Configure required secrets: `OPERATOR_KEYPAIR` (JSON byte array format: `[12,34,56,...]`), `SOLANA_DEVNET_RPC_URL` (or default to `https://api.devnet.solana.com`).
2. Decode operator keypair:
   - `echo '${{ secrets.OPERATOR_KEYPAIR }}' > /tmp/operator.json`
   - Set permissions: `chmod 600 /tmp/operator.json`
   - Configure Solana CLI: `solana config set --url devnet --keypair /tmp/operator.json`
3. Ensure deployment SOL:
   - `solana airdrop 2 --url devnet` (may need retry logic as devnet airdrop can be flaky).
   - Add fallback: retry up to 3 times with 10s delay.
4. Deploy program:
   - Download `cto_billing.so` artifact from anchor-build.
   - Run `anchor deploy --provider.cluster devnet --provider.wallet /tmp/operator.json`.
   - Capture the deployed program ID from output.
   - Alternatively: `solana program deploy target/deploy/cto_billing.so --url devnet --keypair /tmp/operator.json`.
5. Record deployment manifest:
   - Create `deployment-manifest.json` with: program_id, deploy_tx_signature, timestamp, deployer_pubkey, cluster.
   - Upload as artifact.
6. Run smoke test:
   - Execute CLI commands against devnet: initialize operator (if not already), create a customer account, deposit a small USDC amount (or skip if devnet USDC not available — use SOL equivalent or mock).
   - Verify each command succeeds.
   - Run `solana program show <program_id> --url devnet` to confirm deployed status.
7. Clean up: shred operator keypair file. `shred -u /tmp/operator.json`.
8. Update `Anchor.toml` with deployed program ID in `[programs.devnet]` section — this may need a follow-up commit or be documented as a manual step.
</implementation_plan>

<validation>
Manually trigger the deploy-devnet workflow. The job decodes the operator keypair, airdrops SOL, deploys the program to devnet, and records the program ID. `solana program show <program_id> --url devnet` confirms 'Deployed' status. Smoke test completes (at minimum: program show succeeds). deployment-manifest.json artifact is uploaded with valid program ID and TX signature. Operator keypair is cleaned up (not persisted in artifacts or logs).
</validation>