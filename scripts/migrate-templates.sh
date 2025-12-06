#!/bin/bash
# =============================================================================
# Template Migration Script
# Consolidates templates/ and templates/ into single templates/
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "🔄 Template Migration Script"
echo "============================"
echo ""

# 1. Archive old templates
echo "📦 Step 1: Archiving templates/ to .templates-archive/"
if [ -d "templates" ] && [ ! -L "templates" ]; then
    mv templates .templates-archive
    echo "   ✓ templates/ archived"
else
    echo "   ⚠️ templates/ not found or is a symlink"
fi

# 2. Copy clis/ from archive to templates
echo ""
echo "📁 Step 2: Copying CLI templates"
if [ -d ".templates-archive/clis" ]; then
    cp -r .templates-archive/clis templates/
    echo "   ✓ clis/ copied"
fi

# 3. Copy agents/ identity templates
echo ""
echo "📁 Step 3: Copying agent identity templates"
if [ -d ".templates-archive/agents" ]; then
    cp -r .templates-archive/agents templates/
    echo "   ✓ agents/ copied"
fi

# 4. Create legacy/ for templates still needed by controller
echo ""
echo "📁 Step 4: Creating legacy/ for backward compatibility"
mkdir -p templates/legacy

# Copy code templates that are still referenced
if [ -d ".templates-archive/code" ]; then
    cp -r .templates-archive/code templates/legacy/
    echo "   ✓ legacy/code/ created"
fi

# Copy shared templates
if [ -d ".templates-archive/shared" ]; then
    cp -r .templates-archive/shared templates/legacy/
    echo "   ✓ legacy/shared/ created"
fi

# Copy review templates
if [ -d ".templates-archive/review" ]; then
    cp -r .templates-archive/review templates/legacy/
    echo "   ✓ legacy/review/ created"
fi

# Copy remediate templates if they exist
if [ -d ".templates-archive/remediate" ]; then
    cp -r .templates-archive/remediate templates/legacy/
    echo "   ✓ legacy/remediate/ created"
fi

# Copy healer templates
if [ -d ".templates-archive/healer" ]; then
    cp -r .templates-archive/healer templates/legacy/
    echo "   ✓ legacy/healer/ created"
fi

# Copy docs templates
if [ -d ".templates-archive/docs" ]; then
    cp -r .templates-archive/docs templates/legacy/
    echo "   ✓ legacy/docs/ created"
fi

# Copy pm templates
if [ -d ".templates-archive/pm" ]; then
    cp -r .templates-archive/pm templates/legacy/
    echo "   ✓ legacy/pm/ created"
fi

# Copy intake templates
if [ -d ".templates-archive/intake" ]; then
    cp -r .templates-archive/intake templates/legacy/
    echo "   ✓ legacy/intake/ created"
fi

# 5. Update Helm chart symlink
echo ""
echo "🔗 Step 5: Updating Helm chart symlink"
rm -f infra/charts/controller/templates
ln -s ../../../templates infra/charts/controller/templates
echo "   ✓ Symlink updated: infra/charts/controller/templates → ../../../templates"

# 6. Create templates/ symlink for any remaining references
echo ""
echo "🔗 Step 6: Creating templates/ symlink for compatibility"
if [ ! -e "templates" ]; then
    ln -s templates templates
    echo "   ✓ templates → templates symlink created"
else
    echo "   ⚠️ templates already exists, skipping symlink"
fi

echo ""
echo "✅ Migration complete!"
echo ""
echo "📂 New structure:"
echo "   templates/"
echo "   ├── _shared/          # Shared partials (new)"
echo "   ├── clis/             # CLI config templates"
echo "   ├── agents/           # Agent identity templates"
echo "   ├── legacy/           # Old templates (for migration)"
echo "   ├── rex/              # Rex agent jobs"
echo "   ├── blaze/            # Blaze agent jobs"
echo "   └── ...               # Other agents"
echo ""
echo "   templates → templates (symlink)"
echo "   .templates-archive/   # Original templates (backup)"



