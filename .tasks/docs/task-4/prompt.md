<identity>
You are nova, the Bun/TypeScript implementation agent. You own task 4 end-to-end.
</identity>

<context>
<task_overview>
Task 4: Build Arweave/Irys Receipt Upload Module (Nova - Bun/TypeScript)
Create a TypeScript module that takes an itemized receipt JSON object, uploads it to Arweave via Irys (formerly Bundlr) for permanent decentralized storage per §3a design principles, and returns the content-addressed transaction ID and SHA-256 hash for on-chain reference. This module is consumed by both the controller integration (Task 5) and the CLI demo (Task 6).
Priority: high
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Create directory `packages/receipt-uploader/` with a Bun-compatible TypeScript package:
   - `package.json` with name `@cto-billing/receipt-uploader`, type module.
   - Dependencies: `@irys/sdk` (latest v0.2.x), `@solana/web3.js` (^1.95.x), `bs58`, `crypto` (Node built-in).
   - `tsconfig.json` targeting ESNext with NodeNext module resolution.
2. Define the receipt schema in `src/types.ts`:
   ```typescript
   interface TaskReceipt {
     task_id: string;
     customer_pubkey: string;
     timestamp: string; // ISO 8601
     billing_items: BillingItem[];
     total_amount_usdc: number; // human-readable (6 decimals)
     agent_package_id?: string;
     quality_met: boolean;
     coderun_duration_seconds?: number;
     infra_tier?: 'standard' | 'high-mem' | 'gpu';
   }
   interface BillingItem {
     category: 'coderun' | 'infra_compute' | 'ai_tokens';
     description: string;
     quantity: number;
     unit_price_usdc: number;
     subtotal_usdc: number;
   }
   ```
3. Implement `src/uploader.ts`:
   - `createIrysClient(solanaKeypair, rpcUrl, irysNode?)`: Initialize Irys SDK with Solana wallet. Default to devnet node (`https://devnet.irys.xyz`).
   - `uploadReceipt(client, receipt: TaskReceipt): Promise<{ txId: string; hash: Uint8Array; arweaveUrl: string }>`:
     a. Serialize receipt to canonical JSON (sorted keys via `JSON.stringify(receipt, Object.keys(receipt).sort())`).
     b. Compute SHA-256 hash of the JSON bytes.
     c. Upload to Irys with tags: `Content-Type: application/json`, `App-Name: cto-billing`, `Task-Id: {task_id}`, `Version: 1`.
     d. Return `{ txId: irysResponse.id, hash: sha256Bytes, arweaveUrl: 'https://arweave.net/' + irysResponse.id }`.
   - `verifyReceipt(arweaveUrl: string, expectedHash: Uint8Array): Promise<boolean>`: Fetch the receipt from Arweave, recompute SHA-256, compare to expected hash.
4. Implement `src/index.ts` exporting all public types and functions.
5. Add a `src/fund.ts` utility: `fundIrysNode(client, amount)` to fund the Irys node with SOL for upload credits (needed for devnet testing).
6. Write tests in `tests/uploader.test.ts`:
   - Unit test: SHA-256 hash computation is deterministic for same input.
   - Unit test: Canonical JSON serialization produces consistent output.
   - Integration test (requires devnet): Upload a sample receipt, retrieve it, verify hash matches.
   - Test: verifyReceipt returns true for valid receipt, false for tampered receipt.
7. Export the package for consumption: add `"main": "dist/index.js"`, `"types": "dist/index.d.ts"` and a `build` script (`bun build src/index.ts --outdir dist`).
8. Add a `scripts/test-upload.ts` standalone script for manual verification.
</implementation_plan>

<acceptance_criteria>
Run `bun test` in `packages/receipt-uploader/` — all unit tests pass. SHA-256 of `{"task_id":"TEST-001","total_amount_usdc":10.5}` (canonical JSON) matches expected hex digest consistently across 10 runs. Integration test against Irys devnet: upload a 500-byte receipt JSON, receive a valid Arweave transaction ID (43-character base64url string), fetch the receipt from `https://arweave.net/{txId}` within 30s, recomputed SHA-256 matches the returned hash. `verifyReceipt` returns true for the uploaded receipt and false when one byte of the hash is flipped. Package builds to `dist/` with valid TypeScript declarations.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Scaffold Package Structure, TypeScript Configuration, and Core Type Definitions: Create the packages/receipt-uploader/ directory with package.json, tsconfig.json, and define the TaskReceipt and BillingItem TypeScript interfaces along with the canonical JSON serialization utility function.
- Implement Irys Client Initialization, Receipt Upload, Verification, and Funding Utilities: Build the core uploader module: createIrysClient for Solana wallet integration, uploadReceipt with canonical JSON + SHA-256 + Irys tagging, verifyReceipt for hash comparison, and fundIrysNode for devnet upload credits.
- Write Unit and Integration Tests, Validate Build Output and TypeScript Declarations: Create the full test suite covering deterministic hashing, canonical JSON consistency, Irys devnet integration (upload + retrieve + verify), tamper detection, and validate the package builds correctly with TypeScript declarations.
</subtasks>