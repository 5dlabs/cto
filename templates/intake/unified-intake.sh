#!/bin/bash
set -euo pipefail

# =========================================================================
# Unified Intake Script
# This script runs the tasks CLI to parse PRD and generate documentation
# =========================================================================

echo "🚀 Starting Unified Intake Process"
echo "================================="
echo "📍 Script version: unified-intake v2.0.0 (tasks CLI)"
echo "📅 Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "📦 Pod: ${HOSTNAME:-unknown}"
echo ""

# =========================================================================
# Phase 1: Configuration and Environment Setup
# =========================================================================
echo "📋 Phase 1: Configuration and Environment Setup"
echo "================================================"

CONFIG_FILE="/intake-files/config.json"

# Support both .txt and .md PRD files
if [ -f "/intake-files/prd.txt" ]; then
    PRD_FILE="/intake-files/prd.txt"
elif [ -f "/intake-files/prd.md" ]; then
    PRD_FILE="/intake-files/prd.md"
else
    echo "❌ No PRD file found (tried prd.txt, prd.md)"
    exit 1
fi

ARCH_FILE="/intake-files/architecture.md"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found at $CONFIG_FILE"
    exit 1
fi

echo "📄 Loading configuration..."
PROJECT_NAME=$(jq -r '.project_name' "$CONFIG_FILE")
REPOSITORY_URL=$(jq -r '.repository_url // ""' "$CONFIG_FILE")
GITHUB_APP=$(jq -r '.github_app // "5DLabs-Morgan"' "$CONFIG_FILE")
PRIMARY_MODEL=$(jq -r '.primary_model // "claude-sonnet-4-5-20250929"' "$CONFIG_FILE")
NUM_TASKS=$(jq -r '.num_tasks // 15' "$CONFIG_FILE")
EXPAND_TASKS=$(jq -r '.expand_tasks // true' "$CONFIG_FILE")
ANALYZE_COMPLEXITY=$(jq -r '.analyze_complexity // true' "$CONFIG_FILE")
GITHUB_DEFAULT_ORG=$(jq -r '.github_default_org // "5dlabs"' "$CONFIG_FILE")
GITHUB_VISIBILITY=$(jq -r '.github_visibility // "private"' "$CONFIG_FILE")

echo "  ✓ Project: $PROJECT_NAME"
if [ -n "$REPOSITORY_URL" ] && [ "$REPOSITORY_URL" != "null" ]; then
    echo "  ✓ Repository: $REPOSITORY_URL (existing)"
else
    echo "  ✓ Repository: Will create new repo in $GITHUB_DEFAULT_ORG"
fi
echo "  ✓ GitHub App: $GITHUB_APP"
echo "  ✓ Model: $PRIMARY_MODEL"
echo "  ✓ Tasks: ~$NUM_TASKS"
echo ""

# =========================================================================
# Phase 2: GitHub Authentication
# =========================================================================
echo "🔐 Setting up GitHub App authentication..."

# Generate GitHub token from app credentials
if [ -n "${GITHUB_APP_ID:-}" ] && [ -n "${GITHUB_APP_PRIVATE_KEY:-}" ]; then
    echo "Generating fresh GitHub App token..."
    
    # Create JWT
    NOW=$(date +%s)
    IAT=$((NOW - 60))
    EXP=$((NOW + 600))
    
    HEADER=$(echo -n '{"alg":"RS256","typ":"JWT"}' | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    PAYLOAD=$(echo -n "{\"iat\":${IAT},\"exp\":${EXP},\"iss\":\"${GITHUB_APP_ID}\"}" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    SIGNATURE=$(echo -n "${HEADER}.${PAYLOAD}" | openssl dgst -sha256 -sign <(echo "$GITHUB_APP_PRIVATE_KEY") | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    JWT="${HEADER}.${PAYLOAD}.${SIGNATURE}"
    
    # Get installation token
    INSTALLATIONS=$(curl -s -H "Authorization: Bearer $JWT" -H "Accept: application/vnd.github+json" "https://api.github.com/app/installations")
    INSTALLATION_ID=$(echo "$INSTALLATIONS" | jq -r '.[0].id')
    
    if [ -n "$INSTALLATION_ID" ] && [ "$INSTALLATION_ID" != "null" ]; then
        TOKEN_RESPONSE=$(curl -s -X POST -H "Authorization: Bearer $JWT" -H "Accept: application/vnd.github+json" "https://api.github.com/app/installations/${INSTALLATION_ID}/access_tokens")
        GITHUB_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.token')
        
        if [ -n "$GITHUB_TOKEN" ] && [ "$GITHUB_TOKEN" != "null" ]; then
            export GITHUB_TOKEN
            git config --global credential.helper "!f() { echo \"password=$GITHUB_TOKEN\"; }; f"
            git config --global url."https://x-access-token:${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"
            echo "✅ GitHub authentication configured"
        else
            echo "⚠️ Could not get installation token, using default auth"
        fi
    else
        echo "⚠️ Could not get installation ID, using default auth"
    fi
else
    echo "⚠️ GitHub App credentials not provided, using default auth"
fi
echo ""

# =========================================================================
# Phase 3: Repository Setup (Clone existing or Create new)
# =========================================================================
echo "📦 Phase 2: Repository Setup"
echo "============================="

CLONE_DIR="/tmp/repo-$(date +%s)"

# Normalize project/repo name from the project title
REPO_NAME=$(echo "$PROJECT_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g' | sed 's/--*/-/g' | sed 's/^-*//;s/-*$//')

if [ -n "$REPOSITORY_URL" ] && [ "$REPOSITORY_URL" != "null" ]; then
    # Use existing repository
    echo "📂 Cloning existing repository: $REPOSITORY_URL"
    git clone "$REPOSITORY_URL" "$CLONE_DIR" || exit 1
    
    # For existing repos, create a subdirectory for the project
    PROJECT_DIR_NAME="$REPO_NAME"
    PROJECT_DIR="$CLONE_DIR/$PROJECT_DIR_NAME"
    mkdir -p "$PROJECT_DIR"
    CREATED_NEW_REPO=false
else
    # Create new repository
    FULL_REPO_NAME="${GITHUB_DEFAULT_ORG}/${REPO_NAME}"
    echo "🆕 Creating new repository: $FULL_REPO_NAME"
    
    # Check if repo already exists
    if gh repo view "$FULL_REPO_NAME" &>/dev/null; then
        echo "  ⚠️ Repository already exists, cloning..."
        REPOSITORY_URL="https://github.com/${FULL_REPO_NAME}"
        git clone "$REPOSITORY_URL" "$CLONE_DIR" || exit 1
        CREATED_NEW_REPO=false
    else
        # Create new repo with visibility setting
        echo "  → Creating $GITHUB_VISIBILITY repository..."
        gh repo create "$FULL_REPO_NAME" --"$GITHUB_VISIBILITY" --clone --description "Generated from Linear intake: $PROJECT_NAME" "$CLONE_DIR" || {
            echo "❌ Failed to create repository"
            exit 1
        }
        REPOSITORY_URL="https://github.com/${FULL_REPO_NAME}"
        echo "  ✅ Created repository: $REPOSITORY_URL"
        CREATED_NEW_REPO=true
    fi
    
    # For new repos, work directly in the root
    PROJECT_DIR_NAME=""
    PROJECT_DIR="$CLONE_DIR"
fi

cd "$CLONE_DIR" || exit 1

git config user.name "Morgan Intake"
git config user.email "morgan@5dlabs.com"

# Create project directory if needed
if [ -n "$PROJECT_DIR_NAME" ]; then
    mkdir -p "$PROJECT_DIR"
fi
cd "$PROJECT_DIR" || exit 1

echo "  ✓ Working directory: $PROJECT_DIR"

# =========================================================================
# Phase 4: Prepare Input Files
# =========================================================================
echo ""
echo "📝 Setting up input files..."

mkdir -p .tasks/docs
cp "$PRD_FILE" ".tasks/docs/prd.txt"
[ -f "$ARCH_FILE" ] && [ -s "$ARCH_FILE" ] && cp "$ARCH_FILE" ".tasks/docs/architecture.md"

# =========================================================================
# Phase 5: Run Tasks CLI Intake
# =========================================================================
echo ""
echo "🚀 Phase 3: Running tasks CLI intake"
echo "====================================="

# Check if tasks CLI is available
TASKS_AVAILABLE=false
if command -v tasks &> /dev/null; then
    echo "✓ tasks CLI found: $(which tasks)"
    tasks --version 2>&1 || true
    TASKS_AVAILABLE=true
else
    echo "⚠️ tasks CLI not available, using fallback mode"
    echo "  Note: For full functionality, ensure tasks binary is in the image"
fi

if [ "$TASKS_AVAILABLE" = "true" ]; then
    # Build intake command
    INTAKE_CMD="tasks intake --prd .tasks/docs/prd.txt --num-tasks $NUM_TASKS"
    
    # Add architecture if present
    [ -f ".tasks/docs/architecture.md" ] && INTAKE_CMD="$INTAKE_CMD --architecture .tasks/docs/architecture.md"
    
    # Add model if specified
    [ -n "$PRIMARY_MODEL" ] && [ "$PRIMARY_MODEL" != "null" ] && INTAKE_CMD="$INTAKE_CMD --model $PRIMARY_MODEL"
    
    # Optionally skip expansion/analysis
    [ "$EXPAND_TASKS" = "false" ] && INTAKE_CMD="$INTAKE_CMD --no-expand"
    [ "$ANALYZE_COMPLEXITY" = "false" ] && INTAKE_CMD="$INTAKE_CMD --no-analyze"
    
    echo "  → Running: $INTAKE_CMD"
    eval "$INTAKE_CMD" || exit 1
    
    # Verify tasks were generated
    TASKS_FILE=".tasks/tasks/tasks.json"
    if [ ! -f "$TASKS_FILE" ]; then
        echo "❌ tasks.json not found at $TASKS_FILE"
        exit 1
    fi
    
    TASK_COUNT=$(jq '.tasks | length' "$TASKS_FILE")
else
    # Fallback mode: Create basic structure without tasks CLI
    echo "📝 Creating basic intake structure (fallback mode)..."
    
    mkdir -p .tasks/tasks .tasks/reports
    
    # Create a placeholder tasks.json
    cat > .tasks/tasks/tasks.json << 'TASKS_JSON'
{
  "tasks": [
    {
      "id": 1,
      "title": "PRD Review Required",
      "description": "The tasks CLI is not available. Please use the PRD in .tasks/docs/prd.txt to manually create tasks or run tasks CLI locally.",
      "status": "pending",
      "priority": "high"
    }
  ],
  "metadata": {
    "generated_at": "TIMESTAMP_PLACEHOLDER",
    "fallback_mode": true,
    "message": "Generated in fallback mode - tasks CLI not available"
  }
}
TASKS_JSON
    
    # Replace timestamp placeholder
    sed -i "s/TIMESTAMP_PLACEHOLDER/$(date -u +%Y-%m-%dT%H:%M:%SZ)/" .tasks/tasks/tasks.json 2>/dev/null || \
    sed -i '' "s/TIMESTAMP_PLACEHOLDER/$(date -u +%Y-%m-%dT%H:%M:%SZ)/" .tasks/tasks/tasks.json
    
    TASK_COUNT=1
    echo "⚠️ Created placeholder tasks.json (fallback mode)"
    echo "  Note: Run 'tasks intake' locally for full task generation"
fi
echo "✅ Generated $TASK_COUNT tasks with documentation"

# =========================================================================
# Phase 6: Create Pull Request
# =========================================================================
echo ""
echo "🔀 Phase 4: Creating Pull Request"
echo "=================================="

cd "$CLONE_DIR" || exit 1

BRANCH_NAME="intake-${PROJECT_DIR_NAME}-$(date +%Y%m%d-%H%M%S)"
git checkout -b "$BRANCH_NAME"
git add -A
git commit -m "feat: intake for $PROJECT_NAME

- $TASK_COUNT tasks generated
- XML + Markdown documentation per task
- Agent routing hints added
- Complexity analysis: $ANALYZE_COMPLEXITY
- Task expansion: $EXPAND_TASKS

🤖 Generated by Morgan (tasks CLI v2)"

git push -u origin "$BRANCH_NAME"

gh pr create \
    --title "🚀 Intake: $PROJECT_NAME" \
    --body "## Intake: $PROJECT_NAME

### Generated Structure
\`\`\`
$PROJECT_DIR_NAME/
├── .tasks/
│   ├── docs/
│   │   ├── prd.txt
│   │   ├── architecture.md (if provided)
│   │   └── task-*/
│   │       ├── prompt.xml
│   │       ├── prompt.md
│   │       └── acceptance.md
│   ├── tasks/
│   │   └── tasks.json
│   └── reports/
│       └── complexity-report.json
\`\`\`

### Stats
- **Tasks**: $TASK_COUNT
- **Model**: $PRIMARY_MODEL
- **Expansion**: $EXPAND_TASKS
- **Complexity**: $ANALYZE_COMPLEXITY

🤖 Generated by Morgan (tasks CLI v2)" \
    --head "$BRANCH_NAME" \
    --base main || echo "⚠️ PR creation failed, branch pushed: $BRANCH_NAME"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ ✅ Intake completed successfully!"
echo "════════════════════════════════════════════════════════════════"

