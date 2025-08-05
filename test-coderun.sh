#!/bin/bash

# Standardized CodeRun workflow test
# This is the ONLY way to test CodeRun workflows

echo "🧪 Testing CodeRun workflow with Blaze agent..."

# Clean up any existing jobs first
kubectl delete jobs -n agent-platform -l workflows.argoproj.io/workflow 2>/dev/null || true

# Submit CodeRun workflow using Argo
argo submit --from workflowtemplate/coderun-template \
  -n agent-platform \
  -p task-id=1 \
  -p service-id=market-research \
  -p repository-url=https://github.com/5dlabs/test-implementation \
  -p docs-repository-url=https://github.com/5dlabs/cto \
  -p docs-project-directory=projects/market-research \
  -p working-directory=projects/market-research \
  -p github-app=5DLabs-Blaze \
  -p model=claude-3-5-sonnet-20241022 \
  -p continue-session=false \
  -p overwrite-memory=false \
  -p docs-branch=argo \
  -p context-version=1 \
  --wait

echo "✅ CodeRun workflow test completed"