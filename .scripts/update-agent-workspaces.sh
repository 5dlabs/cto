#!/bin/bash
set -euo pipefail

# ============================================================================
# Update All Agent Templates to Use Task-Specific Workspace Directories
# ============================================================================
# This script updates all agent templates (45 files across 5 CLI types) to use
# task-specific workspace directories: /workspace/task-{id}
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATES_DIR="$ROOT_DIR/infra/charts/controller/templates"

echo "🔧 Updating agent templates for task-specific workspace isolation..."
echo ""

# Find all .hbs files in templates
FILES=$(find "$TEMPLATES_DIR" -name "*.sh.hbs" -type f)
FILE_COUNT=$(echo "$FILES" | wc -l | tr -d ' ')

echo "📋 Found $FILE_COUNT template files to update"
echo ""

UPDATED=0
SKIPPED=0

for file in $FILES; do
    BASENAME=$(basename "$file")
    REL_PATH="${file#$ROOT_DIR/}"
    
    # Check if file already has TASK_WORKSPACE variable
    if grep -q "TASK_WORKSPACE=" "$file"; then
        echo "⏭️  Skip: $REL_PATH (already updated)"
        ((SKIPPED++))
        continue
    fi
    
    echo "🔄 Update: $REL_PATH"
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Create temp file for atomic update
    TEMP_FILE=$(mktemp)
    
    # Apply transformations:
    # 1. Add TASK_WORKSPACE variable after the mkdir -p /workspace line
    # 2. Replace hardcoded /workspace paths with $TASK_WORKSPACE
    
    awk '
    BEGIN { task_workspace_added = 0 }
    
    # After mkdir -p /workspace, add TASK_WORKSPACE variable
    /mkdir -p \/workspace/ {
        print
        if (!task_workspace_added) {
            print ""
            print "# Task-specific workspace for parallel execution isolation"
            print "TASK_WORKSPACE=\"/workspace/task-{{task_id}}\""
            print "mkdir -p \"$TASK_WORKSPACE\""
            print "echo \"📁 Using task-specific workspace: $TASK_WORKSPACE\""
            task_workspace_added = 1
        }
        next
    }
    
    # Replace specific workspace patterns
    {
        # GIT_CONFIG_GLOBAL
        gsub(/export GIT_CONFIG_GLOBAL=\/workspace\/\.gitconfig/, "export GIT_CONFIG_GLOBAL=\"$TASK_WORKSPACE/.gitconfig\"")
        
        # CREDENTIALS_FILE
        gsub(/CREDENTIALS_FILE=\/workspace\/\.git-credentials/, "CREDENTIALS_FILE=\"$TASK_WORKSPACE/.git-credentials\"")
        
        # cd /workspace (but not in strings/comments that show the old path)
        if (!/echo.*\/workspace/ && !/Example:.*\/workspace/ && $0 !~ /#.*\/workspace/) {
            gsub(/cd \/workspace([^\/]|$)/, "cd \"$TASK_WORKSPACE\"")
        }
        
        # REPO_ROOT definition
        gsub(/REPO_ROOT="\/workspace\/\$REPO_NAME"/, "REPO_ROOT=\"$TASK_WORKSPACE/$REPO_NAME\"")
        
        # git safe.directory for /workspace
        if (/git config --global --add safe.directory \/workspace/) {
            gsub(/\/workspace/, "\"$TASK_WORKSPACE\"")
        }
        
        # Agent state directory
        gsub(/AGENT_STATE_DIR="\/workspace\/\.agent-state"/, "AGENT_STATE_DIR=\"$TASK_WORKSPACE/.agent-state\"")
        
        # MCP client config paths
        gsub(/TARGET_CFG="\${MCP_CLIENT_CONFIG:-\/workspace\/client-config.json}"/, "TARGET_CFG=\"${MCP_CLIENT_CONFIG:-$TASK_WORKSPACE/client-config.json}\"")
        gsub(/cp .* "\/workspace\/client-config.json"/, "cp \\$FACTORY_WORK_DIR/client-config.json \"$TASK_WORKSPACE/client-config.json\" 2>/dev/null || true")
        
        # Touch .agent_done file
        gsub(/touch \/workspace\/\.agent_done/, "touch \"$TASK_WORKSPACE/.agent_done\"")
        
        # Resolved workspace paths in conditionals
        gsub(/"\$resolved" = "\/workspace"/, "\"$resolved\" = \"$TASK_WORKSPACE\"")
        
        print
    }
    ' "$file" > "$TEMP_FILE"
    
    # Atomic replace
    mv "$TEMP_FILE" "$file"
    
    ((UPDATED++))
done

echo ""
echo "✅ Update complete!"
echo "   Updated: $UPDATED files"
echo "   Skipped: $SKIPPED files"
echo ""
echo "📦 Backups saved with .bak extension"
echo ""
echo "🧪 Next steps:"
echo "   1. Review changes: git diff infra/charts/controller/templates/"
echo "   2. Test with a sample workflow"
echo "   3. Clean up backups: find infra/charts/controller/templates/ -name '*.bak' -delete"

