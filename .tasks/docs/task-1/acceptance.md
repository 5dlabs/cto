## Acceptance Criteria

- [ ] Run `anchor build` in the workspace — compilation succeeds with zero errors. Run `anchor test --skip-deploy` — test runner initializes without error. Apply the Helm chart to a dev namespace — Solana validator pod reaches Ready state within 90s. Run `solana cluster-version --url http://solana-validator:8899` — returns a valid version string. Verify ConfigMap `cto-billing-infra-endpoints` exists and contains `SOLANA_RPC_URL`, `IRYS_RPC_URL` keys. Run `scripts/setup-devnet-usdc.sh` — operator wallet has >1 SOL and >1000 test USDC balance confirmed via `spl-token balance`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.