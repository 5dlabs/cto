Implement subtask 1004: Configure S3/R2 bucket access credentials

## Objective
Create Kubernetes secrets in the 'sigma1' namespace to securely store S3/R2 bucket access credentials for image storage.

## Steps
1. Obtain S3/R2 access key ID and secret access key.2. Create a Kubernetes secret (e.g., `sigma1-s3-credentials`) in the 'sigma1' namespace using `kubectl create secret generic` with the credentials.

## Validation
Verify the secret exists in the 'sigma1' namespace using `kubectl get secret sigma1-s3-credentials -n sigma1` and confirm it contains the expected keys (without exposing values).