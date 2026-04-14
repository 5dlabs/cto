<identity>
You are rex working on subtask 4003 of task 4.
</identity>

<context>
<scope>
Implement receipt JSON generation, SHA-256 hashing, and upload to S3-compatible object storage in billing/receipt.rs, with non-blocking error handling suitable for the controller's reconciliation loop.
</scope>
</context>

<implementation_plan>
1. Add S3 upload functionality to `crates/controller/src/billing/receipt.rs` (or a new `billing/storage.rs`):
   - Implement `pub async fn upload_receipt(receipt: &ReceiptJson, config: &SolanaBillingConfig) -> Result<(String, [u8; 32]), BillingError>`.
   - Steps:
     a. Serialize ReceiptJson to canonical JSON bytes via `serde_json::to_vec`.
     b. Compute SHA-256 hash of the JSON bytes using `sha2::Sha256`.
     c. Construct S3 key: `{config.receipt_s3_prefix}/{task_id}/{timestamp}.json`.
     d. Upload to S3 using the existing controller S3 client (check for `aws-sdk-s3`, `rust-s3`, or `object_store` crate in the workspace). If none exists, use a minimal HTTP PUT with presigned URL or add `rust-s3` as an optional dependency under the `solana-billing` feature.
     e. Return the full S3 URL and the SHA-256 hash.

2. Define `BillingError` enum in `billing/mod.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum BillingError {
       #[error("S3 upload failed: {0}")]
       S3Upload(String),
       #[error("Redis publish failed: {0}")]
       RedisPublish(String),
       #[error("Serialization failed: {0}")]
       Serialization(#[from] serde_json::Error),
   }
   ```

3. Ensure upload_receipt is non-blocking — use tokio::spawn or a timeout to prevent the controller from hanging if S3 is slow. Log errors with `tracing::warn!` but return `Err` (caller decides to continue).

4. Write unit tests:
   - Test that `upload_receipt` correctly computes the SHA-256 hash (mock the S3 upload, verify the hash output matches a precomputed value for the same receipt).
   - Test S3 key construction format.
   - Test error handling when S3 upload fails (mock returns error, function returns BillingError::S3Upload).
</implementation_plan>

<validation>
Run `cargo test --features solana-billing` — tests pass for: (1) SHA-256 hash computation produces correct hash for a known receipt JSON (compare against `echo -n '<json>' | sha256sum`), (2) S3 key is formatted correctly as `{prefix}/{task_id}/{timestamp}.json`, (3) BillingError::S3Upload is returned when upload mock fails. If using testcontainers with MinIO, also verify the actual upload/download round-trip.
</validation>