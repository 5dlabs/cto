#!/bin/bash
# fix-postgres-argocd-visibility.sh
# 
# This script patches PostgreSQL operator-managed resources with ownerReferences
# so they appear in ArgoCD's resource tree view. This addresses a limitation
# where the PostgreSQL operator doesn't set ownerReferences on child resources.
#
# Usage: ./fix-postgres-argocd-visibility.sh <postgresql-name> <namespace>
# Example: ./fix-postgres-argocd-visibility.sh vector-postgres databases

set -euo pipefail

POSTGRES_NAME="${1:-}"
NAMESPACE="${2:-databases}"

if [ -z "$POSTGRES_NAME" ]; then
    echo "Usage: $0 <postgresql-name> [namespace]"
    echo "Example: $0 vector-postgres databases"
    exit 1
fi

echo "Fixing ArgoCD visibility for PostgreSQL cluster: $POSTGRES_NAME in namespace: $NAMESPACE"

# Get the PostgreSQL CRD UID for ownerReference
POSTGRES_UID=$(kubectl get postgresql "$POSTGRES_NAME" -n "$NAMESPACE" -o jsonpath='{.metadata.uid}')

if [ -z "$POSTGRES_UID" ]; then
    echo "Error: PostgreSQL resource $POSTGRES_NAME not found in namespace $NAMESPACE"
    exit 1
fi

echo "PostgreSQL UID: $POSTGRES_UID"

# Create ownerReference JSON
OWNER_REF=$(cat <<EOF
{
  "metadata": {
    "ownerReferences": [
      {
        "apiVersion": "acid.zalan.do/v1",
        "kind": "postgresql", 
        "name": "$POSTGRES_NAME",
        "uid": "$POSTGRES_UID",
        "controller": false,
        "blockOwnerDeletion": false
      }
    ]
  }
}
EOF
)

echo "Patching PostgreSQL resources with ownerReferences..."

# Patch StatefulSet
if kubectl get statefulset "$POSTGRES_NAME" -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "  - StatefulSet: $POSTGRES_NAME"
    kubectl patch statefulset "$POSTGRES_NAME" -n "$NAMESPACE" --type='merge' -p="$OWNER_REF"
else
    echo "  - StatefulSet $POSTGRES_NAME not found, skipping"
fi

# Patch master service
if kubectl get service "$POSTGRES_NAME" -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "  - Service: $POSTGRES_NAME"
    kubectl patch service "$POSTGRES_NAME" -n "$NAMESPACE" --type='merge' -p="$OWNER_REF"
else
    echo "  - Service $POSTGRES_NAME not found, skipping"
fi

# Patch replica service
if kubectl get service "$POSTGRES_NAME-repl" -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "  - Service: $POSTGRES_NAME-repl"
    kubectl patch service "$POSTGRES_NAME-repl" -n "$NAMESPACE" --type='merge' -p="$OWNER_REF"
else
    echo "  - Service $POSTGRES_NAME-repl not found, skipping"
fi

# Patch connection pooler deployment if it exists
if kubectl get deployment "$POSTGRES_NAME-pooler" -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "  - Deployment: $POSTGRES_NAME-pooler"
    kubectl patch deployment "$POSTGRES_NAME-pooler" -n "$NAMESPACE" --type='merge' -p="$OWNER_REF"
else
    echo "  - Deployment $POSTGRES_NAME-pooler not found, skipping"
fi

echo ""
echo "✅ Successfully patched PostgreSQL resources with ownerReferences"
echo ""
echo "To refresh ArgoCD and see the resources, run:"
echo "kubectl patch application database-instances -n argocd -p '{\"metadata\":{\"annotations\":{\"argocd.argoproj.io/refresh\":\"hard\"}}}' --type merge"
echo ""
echo "Note: This is a workaround for a PostgreSQL operator limitation."
echo "The QuestDB operator properly sets ownerReferences, but PostgreSQL operator doesn't."