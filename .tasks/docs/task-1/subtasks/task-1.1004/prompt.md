Implement subtask 1004: Provision S3/R2 buckets and configure access credentials

## Objective
Provision S3-compatible object storage buckets for product images and event photos, and store access credentials and endpoint URLs as Kubernetes Secrets.

## Steps
1. Create two buckets in the chosen S3-compatible provider (R2 or S3): sigma1-product-images, sigma1-event-photos.
2. Configure bucket policies for appropriate access (private with signed URLs, or public read for CDN).
3. Generate access key and secret key for programmatic access.
4. Create a Kubernetes Secret in the sigma1 namespace containing: S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY, S3_PRODUCT_IMAGES_BUCKET, S3_EVENT_PHOTOS_BUCKET.
5. Record the S3_URL endpoint for later ConfigMap aggregation.

## Validation
Verify the Kubernetes Secret exists with all required keys. From a test pod, use the AWS CLI (or compatible tool) with the stored credentials to list buckets and perform a test upload/download to each bucket.