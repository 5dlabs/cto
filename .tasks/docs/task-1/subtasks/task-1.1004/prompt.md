Implement subtask 1004: Provision S3/R2 buckets and create access key secrets

## Objective
Provision S3/R2 object storage buckets for product images and social photos, and store the access keys as Kubernetes secrets in the databases namespace.

## Steps
1. Provision two buckets (via Cloudflare R2 dashboard/API or AWS S3 CLI depending on dp-5 decision):
   - sigma1-product-images: for equipment catalog product images
   - sigma1-social-photos: for social media event photos
2. Create IAM/API tokens with read-write access scoped to these two buckets.
3. Create Kubernetes Secret in databases namespace:
   - name: sigma1-s3-credentials
   - data: S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_ENDPOINT_URL, S3_PRODUCT_IMAGES_BUCKET=sigma1-product-images, S3_SOCIAL_PHOTOS_BUCKET=sigma1-social-photos, S3_REGION
4. If using R2, the endpoint will be https://<account-id>.r2.cloudflarestorage.com.
5. Record the S3 endpoint URL for the ConfigMap.

## Validation
Verify secret sigma1-s3-credentials exists in databases namespace with all expected keys. From a debug pod, use AWS CLI with the credentials to `aws s3 ls s3://sigma1-product-images --endpoint-url <endpoint>` and confirm bucket access.