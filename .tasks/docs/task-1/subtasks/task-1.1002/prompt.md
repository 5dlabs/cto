<identity>
You are bolt working on subtask 1002 of task 1.
</identity>

<context>
<scope>
Create the cto-pay-receipts bucket in the existing SeaweedFS deployment in the cto namespace, and create a K8s Job manifest in infra/gitops/ to ensure idempotent bucket creation.
</scope>
</context>

<implementation_plan>
1. Connect to the existing SeaweedFS S3 API endpoint at `http://seaweedfs-s3.cto.svc.cluster.local:8333`.
2. Create bucket `cto-pay-receipts` using either the SeaweedFS S3-compatible API (`aws s3 mb s3://cto-pay-receipts --endpoint-url ...`) or `weed shell` command.
3. Create a K8s Job YAML manifest at `infra/gitops/cto-pay-receipts-init.yaml` that:
   - Uses an image with the AWS CLI or a lightweight S3 client.
   - Runs `aws s3 mb s3://cto-pay-receipts --endpoint-url http://seaweedfs-s3.cto.svc.cluster.local:8333` (idempotent — mb returns success if bucket exists).
   - Runs in namespace `cto` with appropriate RBAC.
4. Alternatively, create a shell script at `infra/scripts/create-seaweedfs-bucket.sh` that can be run manually or by the Job.
5. Verify the bucket is accessible and empty after creation.
</implementation_plan>

<validation>
Run `aws s3 ls s3://cto-pay-receipts --endpoint-url http://seaweedfs-s3.cto.svc.cluster.local:8333` — returns empty listing (no error). Run `aws s3 cp /dev/null s3://cto-pay-receipts/test-object --endpoint-url ...` then delete — confirms write access. K8s Job manifest passes `kubectl apply --dry-run=client`.
</validation>