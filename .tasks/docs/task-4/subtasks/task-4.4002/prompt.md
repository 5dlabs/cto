<identity>
You are nova working on subtask 4002 of task 4.
</identity>

<context>
<scope>
Build the core uploader module: createIrysClient for Solana wallet integration, uploadReceipt with canonical JSON + SHA-256 + Irys tagging, verifyReceipt for hash comparison, and fundIrysNode for devnet upload credits.
</scope>
</context>

<implementation_plan>
1. Create `src/uploader.ts`:
   - `createIrysClient(solanaKeypair: Keypair, rpcUrl: string, irysNode?: string): Promise<Irys>`: Initialize Irys SDK with the Solana wallet. Use `@irys/sdk` constructor with `token: 'solana'`, `key: solanaKeypair.secretKey`, `config: { providerUrl: rpcUrl }`. Default irysNode to `https://devnet.irys.xyz`. Return the initialized client.
   - `uploadReceipt(client: Irys, receipt: TaskReceipt): Promise<{ txId: string; hash: Uint8Array; arweaveUrl: string }>`:
     a. Import `canonicalJsonSerialize` and `computeSha256` from utils.
     b. Serialize receipt: `const jsonStr = canonicalJsonSerialize(receipt as unknown as Record<string, unknown>)`.
     c. Convert to bytes: `const jsonBytes = new TextEncoder().encode(jsonStr)`.
     d. Compute hash: `const hash = computeSha256(jsonBytes)`.
     e. Define tags array: `[{ name: 'Content-Type', value: 'application/json' }, { name: 'App-Name', value: 'cto-billing' }, { name: 'Task-Id', value: receipt.task_id }, { name: 'Version', value: '1' }]`.
     f. Upload: `const response = await client.upload(Buffer.from(jsonBytes), { tags })`.
     g. Return `{ txId: response.id, hash, arweaveUrl: 'https://arweave.net/' + response.id }`.
   - `verifyReceipt(arweaveUrl: string, expectedHash: Uint8Array): Promise<boolean>`:
     a. Fetch receipt: `const response = await fetch(arweaveUrl)`.
     b. Get body as bytes: `const body = new Uint8Array(await response.arrayBuffer())`.
     c. Compute SHA-256 of body.
     d. Compare byte-by-byte with expectedHash, return boolean.
2. Create `src/fund.ts`:
   - `fundIrysNode(client: Irys, amountInLamports: number): Promise<void>`: Call `await client.fund(amountInLamports)`. Log the funded amount and remaining balance. This is needed before uploads on devnet.
3. Update `src/index.ts` to re-export all functions from `uploader.ts` and `fund.ts`: `createIrysClient`, `uploadReceipt`, `verifyReceipt`, `fundIrysNode`.
4. Create `scripts/test-upload.ts` as a standalone Bun script: load a keypair from env or file, create client, fund if needed, upload a sample receipt, log txId and arweaveUrl, verify the upload. This serves as a manual smoke test.
</implementation_plan>

<validation>
Import all exported functions — TypeScript compiles without errors. `createIrysClient` with a devnet keypair and RPC URL returns an Irys instance without throwing. `uploadReceipt` with a mock/stub Irys client (for unit testing) calls the client's upload method with correct tags and buffer. `verifyReceipt` with a known Arweave URL returns the expected boolean. `scripts/test-upload.ts` runs end-to-end on devnet when given a funded keypair.
</validation>