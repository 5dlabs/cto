<identity>
You are rex working on subtask 5004 of task 5.
</identity>

<context>
<scope>
Implement the async upload_receipt function that sends the receipt JSON to Arweave via the Bun receipt-uploader module (subprocess or HTTP call) and returns the Arweave transaction ID and SHA-256 hash.
</scope>
</context>

<implementation_plan>
1. In `crates/controller/src/tasks/code/billing.rs`, implement:
   `async fn upload_receipt(config: &BillingConfig, receipt: &serde_json::Value) -> Result<(String, [u8; 32])>`
2. SHA-256 hash computation:
   - Serialize receipt to canonical JSON string (sorted keys via serde_json::to_string).
   - Compute SHA-256 using `sha2::Sha256` digest.
   - Store as `[u8; 32]`.
3. Upload path — implement both strategies behind a config enum:
   **Subprocess strategy (default for hackathon):**
   - Use `tokio::process::Command` to run: `bun packages/receipt-uploader/scripts/upload.ts --receipt '<json>'`.
   - Parse stdout for Arweave transaction ID (expect JSON output like `{"arweave_tx_id": "..."}`).
   - Set a 30-second timeout via `tokio::time::timeout`.
   **HTTP strategy (if irys_upload_endpoint is a URL):**
   - POST receipt JSON to the endpoint.
   - Parse response for arweave_tx_id.
4. Error handling:
   - Return descriptive errors for: subprocess timeout, non-zero exit code, invalid stdout JSON, HTTP errors, missing arweave_tx_id in response.
   - All errors should be retryable (caller decides retry policy).
5. Return tuple: (arweave_tx_id: String, receipt_hash: [u8; 32]).
</implementation_plan>

<validation>
Unit test with a mock subprocess/HTTP: verify SHA-256 hash is computed correctly for a known receipt JSON (compare against a precomputed hash). Test that subprocess timeout after 30s returns a timeout error. Test that malformed subprocess output returns a parse error. Integration test (manual): call upload_receipt with a real receipt against the Bun uploader, verify returned arweave_tx_id is a valid 43-character base64url string, and the hash matches sha256 of the input JSON.
</validation>