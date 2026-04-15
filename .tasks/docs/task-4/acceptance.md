## Acceptance Criteria

- [ ] Run `bun test` in `packages/receipt-uploader/` — all unit tests pass. SHA-256 of `{"task_id":"TEST-001","total_amount_usdc":10.5}` (canonical JSON) matches expected hex digest consistently across 10 runs. Integration test against Irys devnet: upload a 500-byte receipt JSON, receive a valid Arweave transaction ID (43-character base64url string), fetch the receipt from `https://arweave.net/{txId}` within 30s, recomputed SHA-256 matches the returned hash. `verifyReceipt` returns true for the uploaded receipt and false when one byte of the hash is flipped. Package builds to `dist/` with valid TypeScript declarations.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.