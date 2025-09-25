#!/bin/bash
set -e

echo "🔄 Updating Helm templates..."
echo "============================="

# Navigate to controller chart directory
cd "$(dirname "$0")/../infra/charts/controller"

# Regenerate templates
echo "📝 Regenerating static ConfigMap..."
./scripts/generate-agent-templates-configmap.sh

# Check if there are changes
if git diff --quiet templates/agent-templates-static.yaml; then
    echo "✅ Templates are already up to date"
    exit 0
fi

# Show what changed
echo ""
echo "📄 Template changes detected:"
git diff --stat templates/agent-templates-static.yaml

echo ""
echo "🔍 Summary of changes:"
echo "- Updated templates from agent-templates/ source files"
echo "- New checksum: $(grep 'templates-checksum:' templates/agent-templates-static.yaml | awk '{print $2}' | tr -d '"')"

# Add and commit changes
echo ""
echo "📝 Committing updated templates..."
git add templates/agent-templates-static.yaml
git commit -m "chore: update agent templates static ConfigMap

- Regenerated from agent-templates/ source files
- Includes latest container script validation changes
- Ensures ArgoCD deploys current templates"

echo ""
echo "✅ Templates updated and committed!"
echo "🚀 Push to trigger deployment: git push"
