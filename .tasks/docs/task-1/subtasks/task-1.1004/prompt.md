Implement subtask 1004: Provision S3/R2 buckets for product images and event photos

## Objective
Create S3-compatible object storage buckets for product images and event photos, configure access credentials, and expose bucket endpoints for downstream services.

## Steps
1. Determine provider from decision point (Cloudflare R2 or AWS S3). For R2: use wrangler CLI or Terraform to create two buckets: 'sigma1-product-images' and 'sigma1-event-photos'.
2. Create an R2 API token (or S3 IAM user) with read/write access scoped to these two buckets.
3. Store credentials in a Kubernetes Secret 'sigma1-s3-credentials' in the sigma1 namespace: S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_ENDPOINT, S3_REGION.
4. Configure CORS on both buckets to allow GET from the web domain.
5. If using R2, configure a custom domain or public bucket URL for CDN-backed reads.
6. Document the bucket names and public read URLs.

## Validation
Verify buckets exist by listing them via the S3-compatible CLI (aws s3 ls --endpoint-url <endpoint>); upload a test file and retrieve it via the public CDN URL; confirm the Kubernetes Secret contains all four required keys.