<identity>
You are nova working on subtask 4001 of task 4.
</identity>

<context>
<scope>
Create the packages/receipt-uploader/ directory with package.json, tsconfig.json, and define the TaskReceipt and BillingItem TypeScript interfaces along with the canonical JSON serialization utility function.
</scope>
</context>

<implementation_plan>
1. Create `packages/receipt-uploader/` directory.
2. Create `package.json` with: name `@cto-billing/receipt-uploader`, version `0.1.0`, type `module`, main `dist/index.js`, types `dist/index.d.ts`, scripts for `build` (`bun build src/index.ts --outdir dist --target node`) and `test` (`bun test`).
3. Add dependencies: `@irys/sdk` (^0.2.0), `@solana/web3.js` (^1.95.0), `bs58` (^6.0.0). Add devDependencies: `bun-types`.
4. Create `tsconfig.json`: target ESNext, module NodeNext, moduleResolution NodeNext, strict true, outDir dist, rootDir src, declaration true, declarationDir dist.
5. Create `src/types.ts` with the `TaskReceipt` and `BillingItem` interfaces exactly as specified in the parent task details. Export both interfaces.
6. Create `src/utils.ts` with a `canonicalJsonSerialize(obj: Record<string, unknown>): string` function that performs deterministic JSON serialization using sorted keys. Implement a recursive key-sorting approach that handles nested objects and arrays correctly. Also export a `computeSha256(data: string | Uint8Array): Uint8Array` function using Node's `crypto.createHash('sha256')`.
7. Create `src/index.ts` that re-exports types from `types.ts` and utilities from `utils.ts` (uploader exports will be added in next subtask).
8. Run `bun install` and `bun build` to verify the scaffold compiles cleanly.
</implementation_plan>

<validation>
Run `bun install` — all dependencies resolve. Run `bun build` — compiles to dist/ with index.js and index.d.ts present. Import TaskReceipt type in a scratch file — type-checks correctly. Call canonicalJsonSerialize with `{b: 2, a: 1}` and verify output is `{"a":1,"b":2}`. Call computeSha256 with the same string 10 times — returns identical Uint8Array each time.
</validation>