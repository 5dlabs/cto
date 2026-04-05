Implement subtask 1004: Provision S3/R2 buckets for product images and event photos

## Objective
Create and configure S3-compatible object storage buckets (Cloudflare R2 or AWS S3) for product images and event photos, including access credentials and CORS policies.

## Steps
1. Decide on provider (R2 or S3) per dp-2 decision.
2. Create two buckets: `sigma1-product-images` and `sigma1-event-photos`.
3. Configure CORS policies to allow GET from the web domain and PUT from admin endpoints.
4. Create an access key pair (or R2 API token) with scoped permissions (read/write to these buckets only).
5. Store the access key ID, secret key, bucket names, and endpoint URL in a Kubernetes Secret named `sigma1-s3-credentials` in the `databases` namespace.
6. Record the S3_URL and bucket names for ConfigMap creation.
7. Upload a test object and verify retrieval via the endpoint URL.

## Validation
Verify both buckets exist and are accessible. Upload a test file and retrieve it via the S3-compatible endpoint. Confirm the Kubernetes Secret `sigma1-s3-credentials` contains valid access keys, endpoint URL, and bucket names.