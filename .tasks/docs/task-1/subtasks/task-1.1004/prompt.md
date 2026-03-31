Implement subtask 1004: Provision S3/R2 object storage bucket and access credentials

## Objective
Create an S3-compatible object storage bucket (Cloudflare R2 or AWS S3 based on decision point dp-3) for product images, event photos, and other binary assets. Generate access keys and store them as Kubernetes secrets.

## Steps
1. Based on the resolved dp-3 decision, provision a bucket named `sigma1-assets` on the chosen provider (R2 or S3).
2. For Cloudflare R2: Use the Cloudflare API or dashboard to create the bucket. Generate an R2 API token with read/write access scoped to the bucket.
3. For AWS S3: Use Terraform/CLI to create the bucket with versioning disabled (v1), server-side encryption enabled. Create an IAM user or role with a policy granting s3:GetObject, s3:PutObject, s3:DeleteObject, s3:ListBucket on the bucket ARN.
4. Create a Kubernetes Secret in the `sigma1` namespace named `sigma1-s3-credentials` containing: `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_REGION`.
5. If using R2, set S3_ENDPOINT to the R2 S3-compatible endpoint URL.
6. Verify connectivity from within the cluster by running a test pod that lists the bucket contents using the AWS CLI.

## Validation
Verify the Kubernetes secret `sigma1-s3-credentials` exists in the sigma1 namespace with all required keys. Spin up a temporary pod with the AWS CLI, mount the secret, and run `aws s3 ls s3://sigma1-assets --endpoint-url $S3_ENDPOINT` to confirm connectivity and empty bucket listing. Upload a test file and verify it can be retrieved.