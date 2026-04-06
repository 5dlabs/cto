Implement subtask 2005: Implement S3/R2 image URL generation and CDN integration

## Objective
Implement the logic to generate public CDN-backed URLs for product images stored in S3/R2, including pre-signed URL generation if needed for uploads.

## Steps
1. Create src/services/storage.rs:
   - Define a StorageService struct holding the S3 endpoint, bucket name, access credentials, and optional CDN base URL.
   - Implement `pub fn public_image_url(&self, image_key: &str) -> String` that returns the CDN URL for a product image. If CDN is configured (CDN_BASE_URL env var), return `{CDN_BASE_URL}/{image_key}`. Otherwise fall back to `{S3_ENDPOINT_URL}/{bucket}/{image_key}`.
   - Implement `pub async fn generate_upload_url(&self, image_key: &str) -> Result<String>` for pre-signed PUT URLs (for admin image uploads) using the aws-sdk-s3 crate or rusoto.
2. Add aws-sdk-s3 to Cargo.toml (or s3 crate for lighter dependency).
3. Integrate StorageService into AppState.
4. Modify ProductResponse DTO to compute image_url from image_key using StorageService.
5. Ensure the image_key stored in DB is a relative path (e.g., `products/{product_id}/main.webp`) and the full URL is computed at response time.

## Validation
Unit test: public_image_url correctly constructs CDN URL when CDN_BASE_URL is set and S3 fallback URL when it's not. Integration test: generate_upload_url returns a valid pre-signed URL (verify by attempting a HEAD request or checking URL structure). Product API responses include correctly formed image_url fields.