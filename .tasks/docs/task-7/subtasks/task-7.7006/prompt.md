<identity>
You are blaze working on subtask 7006 of task 7.
</identity>

<context>
<scope>
Build the /receipts/[taskId] detail page that displays all TaskReceipt fields, provides Solana explorer links, fetches the Arweave receipt JSON, and implements the hash verification button that re-hashes the fetched receipt to compare against the on-chain receipt_hash.
</scope>
</context>

<implementation_plan>
1. Create `src/app/receipts/[taskId]/page.tsx`:
   a. Extract taskId from route params.
   b. Derive TaskReceipt PDA from taskId + customer pubkey. Fetch account data.
   c. Display all fields: task_id, customer, amount, author_earned, protocol_fee, quality_met, agent_package, receipt_hash (as hex string), settled_at, status.
2. Add 'Verify on Solana' link: SolanaExplorerLink pointing to the TaskReceipt PDA address on devnet.
3. Implement 'View Itemized Receipt' section:
   a. Derive or look up Arweave URL from receipt_hash. Strategy: construct Arweave gateway URL (e.g., `https://arweave.net/{txId}`) — the mapping from receipt_hash to Arweave txId needs a convention (could be stored in receipt metadata or fetched from a known index).
   b. Fetch the JSON from Arweave gateway. Parse and display billing line items in a formatted list/table.
   c. Handle fetch errors gracefully (Arweave unavailable, invalid JSON).
4. Implement 'Verify Hash' button:
   a. On click: fetch the receipt JSON from Arweave.
   b. Compute SHA-256 hash of the fetched JSON content (use Web Crypto API: `crypto.subtle.digest('SHA-256', encodedData)`).
   c. Compare computed hash with on-chain receipt_hash (stored as [u8; 32]).
   d. Display green checkmark if match, red X if mismatch, with explanatory text.
5. Create `src/lib/verify.ts`: utility function for fetching Arweave data and computing hash comparison.
6. Handle edge cases: receipt not found (404 page), Arweave fetch timeout, hash mismatch explanation.
</implementation_plan>

<validation>
Navigate to a receipt detail page for a settled task. All TaskReceipt fields display correctly with proper formatting. 'Verify on Solana' link opens correct explorer page. 'Verify Hash' button fetches Arweave receipt, computes SHA-256, and shows green checkmark when hash matches on-chain receipt_hash. If Arweave is unreachable, a graceful error message appears instead of a crash. Billing line items from Arweave JSON render in a readable format.
</validation>