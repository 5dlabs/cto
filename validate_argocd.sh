#!/bin/bash

echo "Validating ArgoCD Application manifests..."
EXIT_CODE=0

for file in infra/gitops/applications/*.yaml; do
  [ -f "$file" ] || continue
  echo "Checking: $file"

  # Check if it's an ArgoCD Application
  KIND=$(yq eval '.kind' "$file")
  if [ "$KIND" != "Application" ]; then
    echo "  ❌ Error: Not an Application resource (kind: $KIND)"
    EXIT_CODE=1
    continue
  fi

  # Check API version
  API_VERSION=$(yq eval '.apiVersion' "$file")
  if [ "$API_VERSION" != "argoproj.io/v1alpha1" ]; then
    echo "  ❌ Error: Invalid apiVersion: $API_VERSION (expected: argoproj.io/v1alpha1)"
    EXIT_CODE=1
  fi

  # Check required fields
  NAME=$(yq eval '.metadata.name' "$file")
  if [ "$NAME" == "null" ]; then
    echo "  ❌ Error: Missing metadata.name"
    EXIT_CODE=1
  fi

  NAMESPACE=$(yq eval '.metadata.namespace' "$file")
  if [ "$NAMESPACE" == "null" ]; then
    echo "  ⚠️  Warning: Missing metadata.namespace (will default to 'default')"
  fi

  DEST_SERVER=$(yq eval '.spec.destination.server' "$file")
  DEST_NAME=$(yq eval '.spec.destination.name' "$file")
  if [ "$DEST_SERVER" == "null" ] && [ "$DEST_NAME" == "null" ]; then
    echo "  ❌ Error: Missing spec.destination.server or spec.destination.name"
    EXIT_CODE=1
  fi

  SOURCE=$(yq eval '.spec.source' "$file")
  SOURCES=$(yq eval '.spec.sources' "$file")
  if [ "$SOURCE" == "null" ] && [ "$SOURCES" == "null" ]; then
    echo "  ❌ Error: Missing spec.source or spec.sources"
    EXIT_CODE=1
  fi

  PROJECT=$(yq eval '.spec.project' "$file")
  if [ "$PROJECT" == "null" ]; then
    echo "  ⚠️  Warning: Missing spec.project (will default to 'default')"
  fi

  # If all checks pass for this file
  if [ $EXIT_CODE -eq 0 ]; then
    echo "  ✅ Valid ArgoCD Application"
  fi
done

exit $EXIT_CODE
