#!/usr/bin/env bash
# Gate 3: create OVH Object Storage (S3-compat) bucket in GRA region.
#
# Decision (see gate3-decisions.md): OVH S3-GRA beats SeaweedFS+CF-tunnel for
# V100S workloads — zero egress since same DC as the inference node.
#
# Prereq:  op signin  (uses op://Automation/OVH Cloud API/* — create items
#          listed below before running)
#
# Env overrides:
#   BUCKET=cto-avatar-gra
#   REGION=GRA
#
# Required op items (create once after signin):
#   op://Automation/OVH S3-GRA Access Key/credential  (access key id)
#   op://Automation/OVH S3-GRA Secret/credential      (secret access key)
#
# If aws cli isn't installed:  brew install awscli

set -euo pipefail

BUCKET="${BUCKET:-cto-avatar-gra}"
REGION="${REGION:-gra}"   # lowercase for s3 endpoint
ENDPOINT="https://s3.${REGION}.perf.cloud.ovh.net"

command -v aws >/dev/null 2>&1 || { echo "aws cli missing (brew install awscli)" >&2; exit 2; }
op whoami --account my >/dev/null 2>&1 || { echo "op not signed in — run: op signin --account my" >&2; exit 2; }

AWS_ACCESS_KEY_ID="$(op read 'op://Automation/OVH S3-GRA Access Key/credential')"
AWS_SECRET_ACCESS_KEY="$(op read 'op://Automation/OVH S3-GRA Secret/credential')"
export AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_DEFAULT_REGION="$REGION"

echo "[info] ensuring bucket: s3://${BUCKET} at ${ENDPOINT}"
if aws --endpoint-url "$ENDPOINT" s3api head-bucket --bucket "$BUCKET" 2>/dev/null; then
  echo "[ok] bucket exists"
else
  aws --endpoint-url "$ENDPOINT" s3api create-bucket --bucket "$BUCKET" \
    --create-bucket-configuration "LocationConstraint=${REGION}"
  echo "[ok] bucket created"
fi

# Lifecycle: auto-expire Gate 1/2 validation artifacts after 14 days so we
# don't pay to store throwaway test mp4s.
aws --endpoint-url "$ENDPOINT" s3api put-bucket-lifecycle-configuration \
  --bucket "$BUCKET" \
  --lifecycle-configuration '{
    "Rules": [{
      "ID": "expire-gate-artifacts",
      "Status": "Enabled",
      "Filter": {"Prefix": "gate-artifacts/"},
      "Expiration": {"Days": 14}
    }]
  }' 2>/dev/null || echo "[warn] lifecycle config failed (non-fatal)"

echo "[ok] Gate 3 bucket ready: ${ENDPOINT}/${BUCKET}"
echo "     prefix layout:"
echo "       gate-artifacts/  (14d TTL, validation mp4s)"
echo "       models/          (persistent HF weights cache)"
echo "       outputs/         (persistent, prod inference)"
