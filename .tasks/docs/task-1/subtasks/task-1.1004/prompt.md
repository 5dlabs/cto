Implement subtask 1004: Provision S3/R2 buckets and configure access credentials

## Objective
Create S3/R2 buckets for product images and event photos, configure access keys, and store credentials as Kubernetes secrets.

## Steps
1. Using the chosen object storage provider (Cloudflare R2 or AWS S3 per dp-4), create two buckets: 'sigma1-product-images' and 'sigma1-event-photos'. 2. Configure bucket policies: product-images should allow public read (for CDN serving); event-photos can be private. 3. Generate API access keys (Access Key ID + Secret Access Key) with scoped permissions to these buckets. 4. Create a Kubernetes Secret in the 'sigma1' namespace containing: S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_ENDPOINT, S3_REGION, S3_PRODUCT_BUCKET, S3_EVENT_BUCKET. 5. Record the S3/R2 endpoint URL for inclusion in the aggregated ConfigMap.

## Validation
Verify buckets exist via API listing; upload a test object to each bucket and retrieve it; verify the Kubernetes Secret is created with all required keys populated.