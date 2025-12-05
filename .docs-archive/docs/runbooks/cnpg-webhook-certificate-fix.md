# CNPG Webhook Certificate Fix

## Symptom

ArgoCD sync fails for CNPG-managed resources (Cluster, Backup, etc.) with
an error like:

```text
Internal error occurred: failed calling webhook "mcluster.cnpg.io":
failed to call webhook: Post "https://cnpg-webhook-service...":
tls: failed to verify certificate: x509: certificate signed by unknown authority
```

The operator logs may also show:

```text
http: TLS handshake error from 10.244.0.0:XXXXX: remote error: tls: bad certificate
```

## Root Cause

CloudNativePG operator manages its own self-signed CA and webhook
certificates. When these certificates are rotated (either manually or during
operator restarts), the webhook configurations need to be updated with the
new CA bundle.

If ArgoCD is configured to manage the CNPG operator without ignoring the
caBundle field, it will overwrite the operator's certificate updates with
stale values from the Helm chart or previous sync state.

## Diagnosis

Verify the CA bundles don't match:

```bash
# Check if CA in webhook config matches CA in secret
# Cross-platform md5 hash function
if command -v md5sum &> /dev/null; then
    md5_hash() { md5sum | awk '{print $1}'; }
elif command -v md5 &> /dev/null; then
    md5_hash() { md5 | awk '{print $NF}'; }
else
    echo "ERROR: Neither md5sum nor md5 command found"
    exit 1
fi

echo "Secret CA:" && \
kubectl get secret cnpg-ca-secret -n infra \
    -o jsonpath='{.data.ca\.crt}' | base64 -d | md5_hash

echo "Mutating webhook CA:" && \
kubectl get mutatingwebhookconfiguration \
    cnpg-mutating-webhook-configuration \
    -o jsonpath='{.webhooks[0].clientConfig.caBundle}' | base64 -d | md5_hash
```

If the hashes differ, the webhooks have stale certificates.

## Immediate Fix

Run the automated fix script:

```bash
./scripts/fix-cnpg-webhook-certs.sh
```

Or manually patch the webhooks:

```bash
NEW_CA=$(kubectl get secret cnpg-ca-secret -n infra -o jsonpath='{.data.ca\.crt}')

kubectl patch mutatingwebhookconfiguration \
    cnpg-mutating-webhook-configuration \
    --type='json' \
    -p="[{\"op\": \"replace\", \"path\": \"/webhooks/0/clientConfig/caBundle\", \
         \"value\": \"${NEW_CA}\"}]"

kubectl patch validatingwebhookconfiguration \
    cnpg-validating-webhook-configuration \
    --type='json' \
    -p="[{\"op\": \"replace\", \"path\": \"/webhooks/0/clientConfig/caBundle\", \
         \"value\": \"${NEW_CA}\"}]"
```

## Permanent Fix

The ArgoCD application for CNPG should have `ignoreDifferences` configured
to prevent overwriting the CA bundles:

```yaml
ignoreDifferences:
  - group: admissionregistration.k8s.io
    kind: MutatingWebhookConfiguration
    name: cnpg-mutating-webhook-configuration
    jsonPointers:
      - /webhooks/0/clientConfig/caBundle
      - /webhooks/1/clientConfig/caBundle
      - /webhooks/2/clientConfig/caBundle
      - /webhooks/3/clientConfig/caBundle
  - group: admissionregistration.k8s.io
    kind: ValidatingWebhookConfiguration
    name: cnpg-validating-webhook-configuration
    jsonPointers:
      - /webhooks/0/clientConfig/caBundle
      - /webhooks/1/clientConfig/caBundle
      - /webhooks/2/clientConfig/caBundle
      - /webhooks/3/clientConfig/caBundle
      - /webhooks/4/clientConfig/caBundle
```

This configuration is included in
`infra/gitops/applications/cloudnative-pg-operator.yaml`.

## Prevention

1. Ensure the ArgoCD application has the `ignoreDifferences` config above
2. If upgrading CNPG, be prepared to run the fix script after upgrade
3. Monitor for TLS handshake errors in operator logs

## Related Files

- `infra/gitops/applications/cloudnative-pg-operator.yaml` - ArgoCD config
- `scripts/fix-cnpg-webhook-certs.sh` - Automated fix script
