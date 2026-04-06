Implement subtask 1004: Provision S3/R2 buckets and access credentials

## Objective
Create S3-compatible object storage buckets for product images and event photos, and store access credentials as Kubernetes Secrets.

## Steps
1. Create two buckets: sigma1-product-images and sigma1-event-photos (via Cloudflare R2 API, Terraform, or manual setup depending on the chosen provider).
2. Configure CORS policies on each bucket to allow frontend access (GET from allowed origins).
3. Generate an API token / access key + secret key pair with read/write access to both buckets.
4. Create a Kubernetes Secret in the sigma1 namespace containing: S3_ENDPOINT, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_PRODUCT_IMAGES_BUCKET, S3_EVENT_PHOTOS_BUCKET.
5. Verify bucket accessibility by uploading and retrieving a test object using the AWS CLI (with --endpoint-url for R2).
6. Record the endpoint URL and bucket names for the ConfigMap.

## Validation
Upload a test file to each bucket and retrieve it successfully; verify the Kubernetes Secret contains all expected keys (endpoint, access key, secret key, bucket names); confirm CORS headers are returned on a preflight request.