#!/bin/bash
# fix-cnpg-webhook-certs.sh - Fix CNPG webhook certificate issues
#
# This script fixes the common issue where CNPG webhook CA bundles get out
# of sync with the actual CA certificate used by the operator.
#
# Symptoms:
#   - ArgoCD sync fails with: "failed calling webhook mcluster.cnpg.io:
#     tls: failed to verify certificate: x509: certificate signed by unknown authority"
#   - kubectl apply for CNPG Cluster resources fails with TLS errors
#
# Root cause:
#   - CNPG operator manages self-signed certificates
#   - ArgoCD may overwrite webhook configurations with stale CA bundles
#   - Certificate rotation can cause CA mismatches
#
# Usage:
#   ./scripts/fix-cnpg-webhook-certs.sh [--dry-run]

set -euo pipefail

NAMESPACE="${CNPG_NAMESPACE:-infra}"
DRY_RUN=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN="--dry-run=server"
            echo "Running in dry-run mode..."
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--dry-run]"
            echo "  --dry-run  Show what would be changed without applying"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Detect the correct md5 command for the platform and create a wrapper function
# that normalizes output to just the hash (no extra text)
if command -v md5sum &> /dev/null; then
    # Linux: md5sum outputs "<hash>  -" or "<hash>  <filename>"
    # We use awk to extract just the first field (the hash)
    md5_hash() {
        md5sum | awk '{print $1}'
    }
elif command -v md5 &> /dev/null; then
    # macOS: md5 outputs "MD5 (-) = <hash>" or "MD5 (<filename>) = <hash>"
    # We use awk to extract the last field (the hash)
    md5_hash() {
        md5 | awk '{print $NF}'
    }
else
    echo "ERROR: Neither md5sum (Linux) nor md5 (macOS) command found"
    exit 1
fi

echo "=== CNPG Webhook Certificate Fix ==="
echo "Namespace: ${NAMESPACE}"
echo ""

# Check if CNPG CA secret exists
if ! kubectl get secret cnpg-ca-secret -n "${NAMESPACE}" &>/dev/null; then
    echo "ERROR: cnpg-ca-secret not found in namespace ${NAMESPACE}"
    echo "Is CNPG operator installed?"
    exit 1
fi

# Get the correct CA bundle
NEW_CA=$(kubectl get secret cnpg-ca-secret -n "${NAMESPACE}" -o jsonpath='{.data.ca\.crt}')

if [[ -z "${NEW_CA}" ]]; then
    echo "ERROR: CA certificate is empty in cnpg-ca-secret"
    exit 1
fi

echo "Current CA bundle hashes:"
echo "  Secret CA:            $(echo "${NEW_CA}" | base64 -d | md5_hash)"
echo "  Mutating webhook CA:  $(kubectl get mutatingwebhookconfiguration cnpg-mutating-webhook-configuration -o jsonpath='{.webhooks[0].clientConfig.caBundle}' 2>/dev/null | base64 -d | md5_hash || echo 'not found')"
echo "  Validating webhook CA: $(kubectl get validatingwebhookconfiguration cnpg-validating-webhook-configuration -o jsonpath='{.webhooks[0].clientConfig.caBundle}' 2>/dev/null | base64 -d | md5_hash || echo 'not found')"
echo ""

# Check if fix is needed
CURRENT_CA=$(kubectl get mutatingwebhookconfiguration cnpg-mutating-webhook-configuration -o jsonpath='{.webhooks[0].clientConfig.caBundle}' 2>/dev/null || echo "")
if [[ "${CURRENT_CA}" == "${NEW_CA}" ]]; then
    echo "CA bundles already match - no fix needed!"
    exit 0
fi

echo "CA bundle mismatch detected - applying fix..."
echo ""

# Get webhook counts
MUTATING_COUNT=$(kubectl get mutatingwebhookconfiguration cnpg-mutating-webhook-configuration -o jsonpath='{.webhooks}' | jq length)
VALIDATING_COUNT=$(kubectl get validatingwebhookconfiguration cnpg-validating-webhook-configuration -o jsonpath='{.webhooks}' | jq length)

echo "Found ${MUTATING_COUNT} mutating webhooks, ${VALIDATING_COUNT} validating webhooks"

# Build patch for mutating webhooks
MUTATING_PATCH="["
for ((i=0; i<MUTATING_COUNT; i++)); do
    [[ $i -gt 0 ]] && MUTATING_PATCH+=","
    MUTATING_PATCH+="{\"op\": \"replace\", \"path\": \"/webhooks/${i}/clientConfig/caBundle\", \"value\": \"${NEW_CA}\"}"
done
MUTATING_PATCH+="]"

# Build patch for validating webhooks
VALIDATING_PATCH="["
for ((i=0; i<VALIDATING_COUNT; i++)); do
    [[ $i -gt 0 ]] && VALIDATING_PATCH+=","
    VALIDATING_PATCH+="{\"op\": \"replace\", \"path\": \"/webhooks/${i}/clientConfig/caBundle\", \"value\": \"${NEW_CA}\"}"
done
VALIDATING_PATCH+="]"

# Apply patches
echo "Patching mutating webhook configuration..."
kubectl patch mutatingwebhookconfiguration cnpg-mutating-webhook-configuration \
    ${DRY_RUN} \
    --type='json' \
    -p="${MUTATING_PATCH}"

echo "Patching validating webhook configuration..."
kubectl patch validatingwebhookconfiguration cnpg-validating-webhook-configuration \
    ${DRY_RUN} \
    --type='json' \
    -p="${VALIDATING_PATCH}"

echo ""
echo "=== Verification ==="
if [[ -z "${DRY_RUN}" ]]; then
    echo "New CA bundle hashes:"
    echo "  Mutating webhook CA:  $(kubectl get mutatingwebhookconfiguration cnpg-mutating-webhook-configuration -o jsonpath='{.webhooks[0].clientConfig.caBundle}' | base64 -d | md5_hash)"
    echo "  Validating webhook CA: $(kubectl get validatingwebhookconfiguration cnpg-validating-webhook-configuration -o jsonpath='{.webhooks[0].clientConfig.caBundle}' | base64 -d | md5_hash)"
    echo ""
    echo "Testing webhook connectivity..."
    if kubectl apply --dry-run=server -f - <<EOF 2>&1
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: test-webhook-check
  namespace: databases
spec:
  instances: 1
  storage:
    size: 1Gi
EOF
    then
        echo "✓ Webhook is working correctly!"
    else
        echo "✗ Webhook test failed - please investigate"
        exit 1
    fi
fi

echo ""
echo "Done! You can now retry your ArgoCD sync."

