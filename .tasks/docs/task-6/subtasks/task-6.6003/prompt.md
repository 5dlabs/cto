<identity>
You are nova working on subtask 6003 of task 6.
</identity>

<context>
<scope>
Implement the second half of demo.ts covering quality-met settlement (Step 5), quality-not-met settlement (Step 6), receipt verification (Step 7), withdrawal (Step 8), and the summary table (Step 9).
</scope>
</context>

<implementation_plan>
1. **Step 5 — Settle Task (Quality MET):**
   - Build receipt JSON: `{ task_id: "TASK-001", customer: customerPubkey, billing_items: [{description: "CodeRun: 15 min", quantity: 1, unit_price_usdc: "0.75", subtotal_usdc: "0.75"}, {description: "Infra: standard tier", ...}], total_amount_usdc: "10.00" }`.
   - Upload receipt to Arweave using the receipt-uploader module (import and call directly or shell out to the script).
   - Compute SHA-256 hash of the receipt JSON.
   - Call `program.methods.settleTask("TASK-001", new BN(10_000_000), Array.from(receiptHash), true).accounts({...}).signers([operator]).rpc()`.
   - Fetch and print: TaskReceipt PDA contents, customer balance (should be 90 USDC), calculate and print author earned (3 USDC = 30% of 10), treasury received (7 USDC = 70%), Arweave receipt URL.
   - Store arweave_tx_id_1 and receipt_hash_1.
2. **Step 6 — Settle Task (Quality NOT MET):**
   - Build receipt for "TASK-002" (similar structure).
   - Upload to Arweave. Store arweave_tx_id_2 and receipt_hash_2.
   - Call `settleTask("TASK-002", new BN(10_000_000), receiptHash2, false)`.
   - Fetch and print: TaskReceipt PDA shows amount=0, customer balance unchanged at 90 USDC, author earned 0.
3. **Step 7 — Verify Receipts:**
   - Fetch TaskReceipt PDA for TASK-001: verify receipt_hash matches computed hash.
   - Fetch TaskReceipt PDA for TASK-002: verify receipt_hash matches.
   - For each Arweave TX, fetch the data from `https://arweave.net/{txId}`, compute SHA-256, compare with on-chain hash.
   - Print ✓ or ✗ for each verification with colored output.
4. **Step 8 — Withdraw:**
   - Fetch current customer balance (90 USDC).
   - Call `program.methods.withdraw(new BN(90_000_000)).accounts({...}).signers([customer]).rpc()`.
   - Fetch and print final customer balance (0), customer ATA balance (should have received 90 USDC back).
5. **Step 9 — Summary:**
   - Print a formatted table (using chalk) with columns: Step, Action, TX Signature, Explorer Link, Status.
   - List all transactions from steps 1-8.
   - Print final account states: OperatorConfig, AgentPackage (task_count=2, success_count=1, total_earned=3_000_000), CustomerBalance (balance=0, total_deposited=100_000_000, total_spent=10_000_000).
   - Print total execution time.
   - Exit with code 0.
</implementation_plan>

<validation>
Run full `bun run demo` against devnet — all 10 steps complete, exit code 0, runtime under 120 seconds. Verify on-chain final state: AgentPackage has task_count=2, success_count=1, total_earned=3_000_000. CustomerBalance has balance=0, total_deposited=100_000_000, total_spent=10_000_000, task_count=2. TaskReceipt TASK-001: amount=10_000_000, quality_met=true, author_earned=3_000_000. TaskReceipt TASK-002: amount=0, quality_met=false, author_earned=0. Both Arweave URLs accessible and hashes match. Summary table printed with all transaction links.
</validation>