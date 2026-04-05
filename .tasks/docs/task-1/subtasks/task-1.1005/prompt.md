Implement subtask 1005: Configure Cloudflare R2 bucket and credentials

## Objective
Provision the sigma1-assets R2 bucket with the required sub-prefix structure and store R2 API credentials as a Kubernetes Secret (or ExternalSecret).

## Steps
1. Determine provisioning method (Cloudflare operator CR or Terraform — see decision point).
2. If using Terraform:
   a. Write a Terraform resource `cloudflare_r2_bucket` with name `sigma1-assets`.
   b. Create R2 API token scoped to this bucket with read/write permissions.
   c. Output the R2 endpoint URL and credentials.
3. If using Cloudflare operator:
   a. Create the appropriate CR for R2 bucket provisioning.
4. R2 doesn't have native folder concepts, but document the sub-prefix convention: `products/`, `social/`, `portfolio/`.
5. Store credentials as a Kubernetes Secret `sigma1-r2-credentials` in the `sigma1` namespace with keys:
   - `R2_ACCESS_KEY_ID`
   - `R2_SECRET_ACCESS_KEY`
   - `R2_ENDPOINT` (e.g., `https://<account-id>.r2.cloudflarestorage.com`)
6. Alternatively, if using ExternalSecrets, create the secret in the external store and reference it via an ExternalSecret CR (covered in subtask 1006).

## Validation
R2 bucket `sigma1-assets` is accessible via the S3-compatible API using the stored credentials. A test `PUT` and `GET` operation to `products/test.txt` succeeds. Secret `sigma1-r2-credentials` exists in `sigma1` namespace with all 3 keys populated.