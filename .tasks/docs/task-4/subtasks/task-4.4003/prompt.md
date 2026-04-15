<identity>
You are nova working on subtask 4003 of task 4.
</identity>

<context>
<scope>
Create the full test suite covering deterministic hashing, canonical JSON consistency, Irys devnet integration (upload + retrieve + verify), tamper detection, and validate the package builds correctly with TypeScript declarations.
</scope>
</context>

<implementation_plan>
1. Create `tests/uploader.test.ts` with the following test cases using Bun's built-in test runner (`import { describe, it, expect } from 'bun:test'`):
   - **Unit: Deterministic SHA-256**: Compute SHA-256 of `'{"task_id":"TEST-001","total_amount_usdc":10.5}'` 10 times in a loop, assert all results are identical. Also verify the hex digest matches a pre-computed expected value.
   - **Unit: Canonical JSON serialization**: Create a TaskReceipt object, serialize it, then create the same object with keys in different order, serialize again — both produce identical JSON strings. Test with nested BillingItem arrays to verify array ordering is preserved while object keys are sorted.
   - **Unit: Canonical JSON handles edge cases**: Optional fields (agent_package_id undefined), empty billing_items array, special characters in description.
   - **Integration (devnet): Upload and retrieve**: Skip if `SKIP_INTEGRATION` env var is set. Load a test keypair, create Irys client on devnet, fund if balance < threshold, upload a sample 500-byte receipt, assert txId is a 43-character base64url string, wait up to 30s polling `https://arweave.net/{txId}`, fetch the content, recompute SHA-256, assert it matches the returned hash.
   - **Unit: verifyReceipt true case**: Mock fetch to return known bytes, verify returns true with matching hash.
   - **Unit: verifyReceipt false case (tampered)**: Flip one byte in the expected hash, verify returns false.
2. Run `bun test` and ensure all unit tests pass (integration tests may be skipped in CI without devnet access).
3. Run `bun run build` — verify `dist/index.js` exists and is valid JavaScript, `dist/index.d.ts` exists and exports all public types (TaskReceipt, BillingItem) and functions (createIrysClient, uploadReceipt, verifyReceipt, fundIrysNode).
4. Verify the package can be imported from another Bun project by creating a minimal consumer script that imports `@cto-billing/receipt-uploader` and type-checks correctly.
</implementation_plan>

<validation>
Run `bun test` — all unit tests pass (6+ test cases). SHA-256 determinism test produces identical hash across 10 iterations. Canonical JSON test confirms key-order independence. verifyReceipt tests confirm true for valid hash, false for tampered hash. If devnet is available, integration test uploads a receipt, retrieves it from Arweave within 30s, and hash verification succeeds. Run `bun run build` — dist/index.js and dist/index.d.ts exist. A consumer script `import { uploadReceipt, TaskReceipt } from '@cto-billing/receipt-uploader'` compiles without type errors.
</validation>