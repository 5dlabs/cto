## Acceptance Criteria for Task 1

1. Clone the cto-pay repo. Run `anchor build` — compiles without errors (program skeleton).
2. Run `bun install` at root — all dependencies resolve.
3. Verify `cli/package.json` has correct bin entry and dependencies.
4. In the CTO cluster: `kubectl get configmap cto-pay-infra-endpoints -n cto -o yaml` — contains all 5 keys (SEAWEEDFS_ENDPOINT, SEAWEEDFS_BUCKET, SOLANA_RPC_URL, SOLANA_OPERATOR_KEYPAIR_PATH, CTO_PAY_ENABLED).
5. Verify SeaweedFS bucket: `aws s3 ls s3://cto-pay-receipts --endpoint-url $SEAWEEDFS_ENDPOINT` returns empty (no error).
6. Verify operator keypair secret: `kubectl get secret cto-pay-operator-keypair -n cto` exists and contains key `operator-keypair.json`.
7. Run `scripts/setup-devnet.sh` — creates SPL mint, `config/devnet.json` contains mint address (44-character base58 string).

_Generated from task metadata (LLM fallback)._