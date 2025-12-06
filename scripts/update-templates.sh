#!/bin/bash
set -e

echo "🔄 Updating Helm templates..."
echo "============================="

# Navigate to controller chart directory
cd "$(dirname "$0")/../infra/charts/controller"

# Regenerate templates
echo "📝 Regenerating split ConfigMaps..."
./scripts/generate-templates-configmaps-split.sh

# Check if there are changes
TEMPLATES_CHANGED=false
for template in templates/templates-*.yaml; do
    if ! git diff --quiet "$template" 2>/dev/null; then
        TEMPLATES_CHANGED=true
        break
    fi
done

if [ "$TEMPLATES_CHANGED" = "false" ]; then
    echo "✅ Templates are already up to date"
    exit 0
fi

# Show what changed
echo ""
echo "📄 Template changes detected:"
git diff --stat templates/templates-*.yaml

echo ""
echo "🔍 Summary of changes:"
echo "- Updated templates from templates/ source files"
for template in templates/templates-*.yaml; do
    if ! git diff --quiet "$template" 2>/dev/null; then
        checksum=$(grep 'templates-checksum:' "$template" | awk '{print $2}' | tr -d '"')
        echo "- $(basename "$template"): checksum $checksum"
    fi
done

# Add and commit changes
echo ""
echo "📝 Committing updated templates..."
git add templates/templates-*.yaml
git commit -m "chore: update agent templates ConfigMaps

- Regenerated from templates/ source files
- Includes latest container script validation changes
- Ensures ArgoCD deploys current templates"

echo ""
echo "✅ Templates updated and committed!"
echo "🚀 Push to trigger deployment: git push"
