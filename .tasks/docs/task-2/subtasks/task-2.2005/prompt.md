Implement subtask 2005: Implement S3/R2 signed URL generation for product images

## Objective
Add S3-compatible signed URL generation for serving product images securely, and implement the image URL resolution in product API responses.

## Steps
1. Add `aws-sdk-s3` or `rust-s3` crate to Cargo.toml.
2. Initialize an S3 client in AppState using S3_URL, bucket name, and credentials from environment/secrets.
3. Create a utility function `generate_signed_url(image_key: &str, expiry: Duration) -> String` that generates a pre-signed GET URL for an object in the product images bucket.
4. Default expiry: 1 hour.
5. Update the Product response serialization to include a `image_url` field that calls this function using the product's `image_key`.
6. Handle missing image_key gracefully (return null or a default placeholder URL).
7. For the equipment-api/catalog endpoint, batch-generate signed URLs for all products in the response.
8. Ensure signed URL generation does not add significant latency (pre-sign is a local crypto operation, no network call).

## Validation
Verify product responses include a valid `image_url` field. Verify the signed URL is accessible and returns the correct image from S3/R2. Verify products without images return null or a placeholder. Verify URL expiry works (URL becomes inaccessible after expiry period).