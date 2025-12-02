#!/bin/bash
# =============================================================================
# DEPRECATED: This script is deprecated in favor of unified-intake.sh.hbs
# =============================================================================
# This script only handles PRD parsing and task generation.
# Use unified-intake.sh.hbs instead, which combines PRD parsing, task generation,
# context enrichment via Firecrawl, and documentation generation in one operation.
#
# This file will be removed in a future release.
# =============================================================================
set -e

# Force output to be unbuffered
exec 2>&1
set -x  # Enable command tracing temporarily

# =============================================================================
# Error Handling - Prevent Silent Failures
# =============================================================================
# Write errors to shared PVC before exit to prevent silent failures (A2 alerts)
INTAKE_ERROR_DIR="${WORKSPACE_PVC:-/workspace}/intake-errors"
INTAKE_POD_NAME="${POD_NAME:-$(hostname)}"
INTAKE_START_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Function to log errors to shared storage
log_error_to_pvc() {
    local exit_code="$1"
    local line_no="$2"
    local command="$3"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    # Create error directory if it exists (shared PVC)
    mkdir -p "$INTAKE_ERROR_DIR" 2>/dev/null || true

    # Write error report
    local error_file="$INTAKE_ERROR_DIR/error-${INTAKE_POD_NAME}-$(date +%s).log"
    {
        echo "═══════════════════════════════════════════════════════════════"
        echo "INTAKE ERROR REPORT (Legacy Script)"
        echo "═══════════════════════════════════════════════════════════════"
        echo "Pod Name: $INTAKE_POD_NAME"
        echo "Start Time: $INTAKE_START_TIME"
        echo "Error Time: $timestamp"
        echo "Exit Code: $exit_code"
        echo "Failed at Line: $line_no"
        echo "Failed Command: $command"
        echo ""
        echo "Configuration:"
        echo "  Project: ${PROJECT_NAME:-unknown}"
        echo "  Repository: ${REPOSITORY_URL:-unknown}"
        echo "  Config File: ${CONFIG_FILE:-unknown}"
        echo ""
        echo "Environment:"
        echo "  PWD: $(pwd)"
        echo "  USER: $(whoami)"
        echo ""
        echo "Last 50 lines of output available in container logs"
        echo "═══════════════════════════════════════════════════════════════"
    } > "$error_file" 2>/dev/null || true

    echo "📝 Error logged to: $error_file" >&2
}

# Enhanced error trap that logs to PVC before exit
trap_handler() {
    local exit_code=$?
    local line_no=$1
    local command="$BASH_COMMAND"

    echo "" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2
    echo "❌ INTAKE FAILURE DETECTED (Legacy Script)" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2
    echo "❌ Exit code: $exit_code" >&2
    echo "❌ Line: $line_no" >&2
    echo "❌ Command: $command" >&2
    echo "❌ Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2

    # Log to PVC for persistence
    log_error_to_pvc "$exit_code" "$line_no" "$command"

    # Ensure output is flushed before exit
    sync 2>/dev/null || true
    sleep 1

    exit "$exit_code"
}

trap 'trap_handler $LINENO' ERR

echo "⚠️ DEPRECATED: This script is deprecated. Use unified-intake.sh.hbs instead."
echo "🚀 Starting Project Intake Process (Legacy Mode)"
echo "================================="
echo "📦 Pod: $INTAKE_POD_NAME"
echo "📅 Timestamp: $INTAKE_START_TIME"

# Debug: Show ALL environment variables related to our workflow
echo "🔍 DEBUG: Environment Variables Received:"
echo "  PRIMARY_MODEL: ${PRIMARY_MODEL:-[NOT SET]}"
echo "  PRIMARY_PROVIDER: ${PRIMARY_PROVIDER:-[NOT SET]}"
echo "  RESEARCH_MODEL: ${RESEARCH_MODEL:-[NOT SET]}"
echo "  RESEARCH_PROVIDER: ${RESEARCH_PROVIDER:-[NOT SET]}"
echo "  FALLBACK_MODEL: ${FALLBACK_MODEL:-[NOT SET]}"
echo "  FALLBACK_PROVIDER: ${FALLBACK_PROVIDER:-[NOT SET]}"
echo "  NUM_TASKS: ${NUM_TASKS:-[NOT SET]}"
echo "  EXPAND_TASKS: ${EXPAND_TASKS:-[NOT SET]}"
echo "  ANALYZE_COMPLEXITY: ${ANALYZE_COMPLEXITY:-[NOT SET]}"
echo "  GITHUB_APP: ${GITHUB_APP:-[NOT SET]}"
echo "================================="

# Load configuration from mounted ConfigMap
CONFIG_FILE="/intake-files/config.json"
PRD_FILE="/intake-files/prd.txt"
ARCH_FILE="/intake-files/architecture.md"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found at $CONFIG_FILE"
    exit 1
fi

# Debug: Show what's in the config file
echo "📄 Config file contents (from ConfigMap):"
cat "$CONFIG_FILE" || echo "Failed to cat config file"
echo ""
echo "📄 Parsed values from ConfigMap:"
echo "  primary_model from JSON: $(jq -r '.primary_model // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "  primary_provider from JSON: $(jq -r '.primary_provider // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "  research_model from JSON: $(jq -r '.research_model // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "  research_provider from JSON: $(jq -r '.research_provider // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "  fallback_model from JSON: $(jq -r '.fallback_model // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "  fallback_provider from JSON: $(jq -r '.fallback_provider // "[NOT IN JSON]"' "$CONFIG_FILE")"
echo "---"

# Preview PRD and Architecture to verify correctness
echo "📄 PRD file preview (first 40 lines):"
if [ -f "$PRD_FILE" ]; then
    head -40 "$PRD_FILE" || true
else
    echo "PRD file not found at $PRD_FILE"
fi
echo ""
if [ -f "$ARCH_FILE" ]; then
    echo "📐 Architecture file present: $ARCH_FILE"
else
    echo "📐 No architecture.md provided (optional)"
fi
echo "---"

# Parse configuration
echo "📋 Loading configuration from ConfigMap..."

# Parse each field with error handling
PROJECT_NAME=$(jq -r '.project_name' "$CONFIG_FILE" 2>/dev/null || echo "")
echo "  ✓ Project name: $PROJECT_NAME"

REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE" 2>/dev/null || echo "")
echo "  ✓ Repository URL: $REPOSITORY_URL"

# GITHUB_APP is now required from environment variables (no ConfigMap fallback)

# Parse granular model configuration (from Argo workflow parameters)
# NO FALLBACKS - if parameters not received, fail loudly to expose configuration issues

if [ -z "$PRIMARY_MODEL" ]; then
    echo "❌ PRIMARY_MODEL environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$RESEARCH_MODEL" ]; then
    echo "❌ RESEARCH_MODEL environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$FALLBACK_MODEL" ]; then
    echo "❌ FALLBACK_MODEL environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$PRIMARY_PROVIDER" ]; then
    echo "❌ PRIMARY_PROVIDER environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$RESEARCH_PROVIDER" ]; then
    echo "❌ RESEARCH_PROVIDER environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$FALLBACK_PROVIDER" ]; then
    echo "❌ FALLBACK_PROVIDER environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$NUM_TASKS" ]; then
    echo "❌ NUM_TASKS environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$EXPAND_TASKS" ]; then
    echo "❌ EXPAND_TASKS environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$ANALYZE_COMPLEXITY" ]; then
    echo "❌ ANALYZE_COMPLEXITY environment variable not set - configuration transmission failed"
    exit 1
fi

if [ -z "$GITHUB_APP" ]; then
    echo "❌ GITHUB_APP environment variable not set - configuration transmission failed"
    exit 1
fi

echo "  ✓ GitHub App: $GITHUB_APP"
echo "  ✓ Primary Model: $PRIMARY_MODEL ($PRIMARY_PROVIDER)"
echo "  ✓ Research Model: $RESEARCH_MODEL ($RESEARCH_PROVIDER)"
echo "  ✓ Fallback Model: $FALLBACK_MODEL ($FALLBACK_PROVIDER)"
echo "  ✓ Num tasks: $NUM_TASKS"
echo "  ✓ Expand tasks: $EXPAND_TASKS"
echo "  ✓ Analyze complexity: $ANALYZE_COMPLEXITY"

# Legacy MODEL variable for backward compatibility
MODEL="$PRIMARY_MODEL"

echo "🔍 Configuration summary:"
echo "  - Project: ${PROJECT_NAME:-[empty]}"
echo "  - Repository: ${REPOSITORY_URL:-[empty]}"
echo "  - GitHub App: ${GITHUB_APP:-[empty]}"
echo "  - Primary Model: ${PRIMARY_MODEL:-[empty]} (${PRIMARY_PROVIDER:-[empty]})"
echo "  - Research Model: ${RESEARCH_MODEL:-[empty]} (${RESEARCH_PROVIDER:-[empty]})"
echo "  - Fallback Model: ${FALLBACK_MODEL:-[empty]} (${FALLBACK_PROVIDER:-[empty]})"
echo "  - Num Tasks: ${NUM_TASKS:-[empty]}"
echo "  - Expand: ${EXPAND_TASKS:-[empty]}"
echo "  - Analyze: ${ANALYZE_COMPLEXITY:-[empty]}"

# Turn off command tracing after configuration parsing
set +x

# If project name is empty, try to extract from PRD
if [ -z "$PROJECT_NAME" ] || [ "$PROJECT_NAME" = "null" ]; then
    echo "📝 Extracting project name from PRD..."
    
    # Try to extract from first heading
    PROJECT_NAME=$(head -10 "$PRD_FILE" | grep -E "^#\s+" | head -1 | sed 's/^#\s*//' | \
                   sed 's/[^a-zA-Z0-9 -]//g' | tr '[:upper:]' '[:lower:]' | \
                   sed 's/ /-/g' | sed 's/--*/-/g' | sed 's/^-*//;s/-*$//')
    
    # Fallback to timestamp-based name
    if [ -z "$PROJECT_NAME" ]; then
        PROJECT_NAME="project-$(date +%Y%m%d-%H%M%S)"
    fi
    
    echo "✅ Using project name: $PROJECT_NAME"
fi

# Check for required environment variables
echo "🔍 Checking environment variables..."
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "⚠️ Warning: ANTHROPIC_API_KEY is not set"
fi

# Disable interactive Git prompts
export GIT_TERMINAL_PROMPT=0
export GIT_ASKPASS=/bin/true
export SSH_ASKPASS=/bin/true

# GitHub App authentication setup
if [ -n "$GITHUB_APP_PRIVATE_KEY" ] && [ -n "$GITHUB_APP_ID" ]; then
    echo "🔐 Setting up GitHub App authentication..."
    echo "  - GitHub App ID found: ${GITHUB_APP_ID:0:10}..."
    echo "  - GitHub App Private Key found: [REDACTED]"
    
    # Function to generate GitHub App token (reusing from container.sh logic)
    generate_github_token() {
        echo "Generating fresh GitHub App token..."
        
        # Create temporary private key file
        TEMP_KEY_FILE="/tmp/github-app-key.pem"
        echo "$GITHUB_APP_PRIVATE_KEY" > "$TEMP_KEY_FILE"
        chmod 600 "$TEMP_KEY_FILE"
        
        # Generate JWT token
        JWT_HEADER=$(printf '{"alg":"RS256","typ":"JWT"}' | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        NOW=$(date +%s)
        EXP=$((NOW + 600))
        JWT_PAYLOAD=$(printf '{"iat":%d,"exp":%d,"iss":"%s"}' "$NOW" "$EXP" "$GITHUB_APP_ID" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        JWT_SIGNATURE=$(printf '%s.%s' "$JWT_HEADER" "$JWT_PAYLOAD" | openssl dgst -sha256 -sign "$TEMP_KEY_FILE" -binary | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        JWT_TOKEN="$JWT_HEADER.$JWT_PAYLOAD.$JWT_SIGNATURE"
        
        # Get installation ID
        REPO_OWNER=$(echo "$REPOSITORY_URL" | sed -E 's|https://github.com/([^/]+)/.*|\1|')
        REPO_NAME=$(echo "$REPOSITORY_URL" | sed -E 's|https://github.com/[^/]+/([^/]+)(\.git)?|\1|')
        
        # Try repository installation first (follow redirects)
        INSTALLATION_RESPONSE=$(curl -s -L --retry 5 --retry-delay 2 --retry-connrefused \
            --connect-timeout 5 --max-time 12 \
            -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github+json" \
            "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/installation")

        INSTALLATION_ID=$(echo "$INSTALLATION_RESPONSE" | jq -r '.id')

        # Fallback: try organization installation if repo lookup failed
        if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
            echo "⚠️ Repo installation not found for $REPO_OWNER/$REPO_NAME, trying org installation..."
            ORG_INSTALLATION_RESPONSE=$(curl -s -L --retry 5 --retry-delay 2 --retry-connrefused \
                --connect-timeout 5 --max-time 12 \
                -H "Authorization: Bearer $JWT_TOKEN" \
                -H "Accept: application/vnd.github+json" \
                "https://api.github.com/orgs/$REPO_OWNER/installation")
            INSTALLATION_ID=$(echo "$ORG_INSTALLATION_RESPONSE" | jq -r '.id')
        fi

        if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
            echo "❌ Failed to get installation ID for $REPO_OWNER/$REPO_NAME"
            echo "Response (repo): $INSTALLATION_RESPONSE"
            echo "Response (org):  ${ORG_INSTALLATION_RESPONSE:-[none]}"
            return 1
        fi
        
        echo "Installation ID: $INSTALLATION_ID"
        
        # Generate installation access token
        GITHUB_TOKEN=$(curl -s -L --retry 5 --retry-delay 2 --retry-connrefused \
            --connect-timeout 5 --max-time 12 \
            -X POST \
            -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/app/installations/$INSTALLATION_ID/access_tokens" | jq -r '.token')
        
        if [ "$GITHUB_TOKEN" = "null" ] || [ -z "$GITHUB_TOKEN" ]; then
            echo "❌ Failed to generate GitHub token"
            return 1
        fi
        
        export GITHUB_TOKEN
        export GH_TOKEN="$GITHUB_TOKEN"
        
        # Configure git
        git config --global --replace-all credential.helper store
        echo "https://x-access-token:${GITHUB_TOKEN}@github.com" > ~/.git-credentials
        
        # Configure GitHub CLI
        echo "🔧 Configuring GitHub CLI..."
        echo "$GITHUB_TOKEN" | timeout 10 gh auth login --with-token || {
            echo "⚠️ gh auth login returned non-zero or timed out, but continuing..."
        }
        
        # Check auth status (this may return non-zero even when auth is valid)
        echo "🔍 Checking GitHub CLI auth status..."
        timeout 10 gh auth status || {
            echo "⚠️ gh auth status returned non-zero or timed out, but token is likely still valid"
        }
        
        echo "✅ GitHub authentication configured"
        return 0
    }
    
    # Initial token generation
    generate_github_token || exit 1
else
    echo "⚠️ GitHub App credentials not found, using default authentication"
fi

# Clone repository
echo "📦 Cloning repository: $REPOSITORY_URL"

# Validate repository URL
if [ -z "$REPOSITORY_URL" ] || [ "$REPOSITORY_URL" = "null" ]; then
    echo "❌ Repository URL is empty or null"
    exit 1
fi

CLONE_DIR="/tmp/repo-$(date +%s)"
echo "📂 Clone directory: $CLONE_DIR"
echo "🔍 Attempting git clone..."
git clone "$REPOSITORY_URL" "$CLONE_DIR" || {
    echo "❌ Git clone failed with exit code $?"
    echo "Repository URL: $REPOSITORY_URL"
    echo "Clone directory: $CLONE_DIR"
    exit 1
}

echo "✅ Repository cloned successfully"
cd "$CLONE_DIR"
echo "📂 Changed to clone directory: $(pwd)"

# Normalize project name for filesystem (lowercase, safe characters)
PROJECT_DIR_NAME=$(echo "$PROJECT_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g' | sed 's/--*/-/g' | sed 's/^-*//;s/-*$//')

# Set PROJECT_DIR to a subdirectory within the cloned repository
PROJECT_DIR="$CLONE_DIR/$PROJECT_DIR_NAME"

# Create project directory if it doesn't exist
if [ ! -d "$PROJECT_DIR" ]; then
    echo "📁 Creating project directory: $PROJECT_DIR_NAME"
    mkdir -p "$PROJECT_DIR"
fi

# Configure git identity
git config user.name "Project Intake Bot"
git config user.email "intake@5dlabs.com"

# Set up nvm environment if available (Claude Code image uses nvm)
if [ -s "/usr/local/nvm/nvm.sh" ]; then
    echo "🔧 Setting up nvm environment..."
    export NVM_DIR="/usr/local/nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
    echo "✅ nvm loaded, node version: $(node --version)"
fi

# Check if npm is available
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed or not in PATH"
    echo "🔍 PATH: $PATH"
    echo "🔍 Checking for node/npm in common locations..."
    
    # Check common locations
    for npm_path in /usr/local/nvm/versions/node/*/bin/npm /usr/bin/npm /usr/local/bin/npm; do
        if [ -f "$npm_path" ]; then
            echo "✅ Found npm at: $npm_path"
            # Add to PATH
            export PATH="$(dirname "$npm_path"):$PATH"
            break
        fi
    done
    
    # Final check
    if ! command -v npm &> /dev/null; then
        echo "❌ Cannot find npm after checking common locations"
        exit 1
    fi
fi

# TaskMaster is pre-installed in the agent image
echo "📦 TaskMaster is pre-installed in the agent image"
echo "📋 Node version: $(node --version)"
echo "📋 NPM version: $(npm --version)"

# Verify TaskMaster is available
if ! command -v task-master &> /dev/null; then
    echo "❌ task-master command not found in agent image"
    echo "🔍 PATH: $PATH"
    echo "🔍 Looking for task-master in common locations:"
    for path in /usr/local/bin /usr/bin /home/node/.npm-global/bin; do
        if [ -f "$path/task-master" ]; then
            echo "✅ Found task-master at: $path/task-master"
            export PATH="$path:$PATH"
            break
        fi
    done
    
    if ! command -v task-master &> /dev/null; then
        echo "❌ task-master not found after checking common locations"
        exit 1
    fi
fi

# Get the actual path to task-master
TASK_MASTER_PATH=$(which task-master)
echo "✅ TaskMaster found at: $TASK_MASTER_PATH"
echo "✅ TaskMaster version: $(task-master --version 2>/dev/null || echo 'version check failed')"

# Change to project directory
cd "$PROJECT_DIR"

# Set environment variables for TaskMaster
export TASKMASTER_LOG_LEVEL="debug"
export CI="true"  # This might help TaskMaster run in non-interactive mode
export TASKMASTER_AUTO_ACCEPT="true"

# Initialize TaskMaster
echo "🚀 Initializing TaskMaster project in $PROJECT_NAME..."
echo "📂 Current directory: $(pwd)"
echo "📂 Directory contents before init:"
ls -la

# Debug: Check if task-master command works
echo "🔍 Testing task-master command..."
task-master --version || echo "⚠️ task-master --version failed"
task-master --help > /dev/null 2>&1 || echo "⚠️ task-master --help failed"

# First attempt: Try clean init with Claude rules only
echo "🔍 Attempting TaskMaster init..."
# Use the full path to ensure we're calling the right binary
# --rules "claude" creates only Claude-specific files (CLAUDE.md)
"$TASK_MASTER_PATH" init --yes \
    --name "$PROJECT_NAME" \
    --description "Auto-generated project from intake pipeline" \
    --version "0.1.0" \
    --rules "claude" \
    --skip-install
INIT_EXIT_CODE=$?

echo "🔍 Init result: exit code $INIT_EXIT_CODE"

# Check if initialization was successful
if [ $INIT_EXIT_CODE -eq 0 ] && [ -d ".taskmaster" ]; then
    echo "✅ TaskMaster initialization successful!"
    echo "📂 Directory contents after init:"
    ls -la .taskmaster/
else
    echo "⚠️ TaskMaster init failed or didn't create .taskmaster directory"
    echo "📂 Current directory contents:"
    ls -la
    
    # Try alternative approach: init with minimal flags
    echo "🔧 Trying init with minimal flags..."
    task-master init --name "$PROJECT_NAME" --yes
    INIT_EXIT_CODE=$?
    
    if [ $INIT_EXIT_CODE -eq 0 ] && [ -d ".taskmaster" ]; then
        echo "✅ Minimal init method worked!"
    else
        echo "🔧 Final attempt: Manual directory creation as fallback..."
        
        # Create the .taskmaster directory structure manually as last resort
        echo "📁 Creating .taskmaster directory structure manually..."
        mkdir -p .taskmaster/docs
        mkdir -p .taskmaster/tasks
        mkdir -p .taskmaster/reports
        mkdir -p .taskmaster/templates
        
        # Create config.json with granular model configuration
        cat > .taskmaster/config.json << EOF
{
  "project": {
    "name": "$PROJECT_NAME",
    "description": "Auto-generated project from intake pipeline",
    "version": "0.1.0"
  },
  "models": {
    "main": {
      "provider": "$PRIMARY_PROVIDER",
      "modelId": "$PRIMARY_MODEL",
      "maxTokens": 64000,
      "temperature": 0.2
    },
    "research": {
      "provider": "$RESEARCH_PROVIDER",
      "modelId": "$RESEARCH_MODEL",
      "maxTokens": 32000,
      "temperature": 0.1
    },
    "fallback": {
      "provider": "$FALLBACK_PROVIDER",
      "modelId": "$FALLBACK_MODEL",
      "maxTokens": 8000,
      "temperature": 0.7
    }
  },
  "global": {
    "defaultTag": "master"
  }
}
EOF
        
        # Create empty tasks.json
        echo '{"tasks": []}' > .taskmaster/tasks/tasks.json
        
        echo "✅ Created .taskmaster directory structure manually"
    fi
fi

# Final check
if [ ! -d ".taskmaster" ]; then
    echo "❌ Failed to create .taskmaster directory after all attempts"
    echo "📂 Final directory contents:"
    ls -la
    exit 1
fi

echo "✅ TaskMaster setup complete"
echo "📂 Final .taskmaster contents:"
ls -la .taskmaster/

# Copy PRD and architecture files after initialization
echo "📋 Copying PRD and architecture files..."
# Ensure directories exist regardless of task-master version behavior
mkdir -p .taskmaster/docs .taskmaster/tasks
cp "$PRD_FILE" ".taskmaster/docs/prd.txt"
if [ -f "$ARCH_FILE" ] && [ -s "$ARCH_FILE" ]; then
    cp "$ARCH_FILE" ".taskmaster/docs/architecture.md"
fi

# Configure models with Claude Code and GPT-5 fallback
echo "🤖 Configuring AI models..."

# Test if OpenAI API key is valid by making a simple API call
OPENAI_VALID=false
if [ -n "$OPENAI_API_KEY" ]; then
  echo "🔍 Testing OpenAI API key validity..."
  # Test the API key with a simple models call
  if curl -s -H "Authorization: Bearer $OPENAI_API_KEY" \
          -H "Content-Type: application/json" \
          "https://api.openai.com/v1/models" > /dev/null 2>&1; then
    echo "✅ OpenAI API key is valid"
    OPENAI_VALID=true
  else
    echo "⚠️ OpenAI API key is invalid or expired, falling back to Claude only"
    OPENAI_VALID=false
  fi
fi

# Check if ANTHROPIC_API_KEY is available for Claude Code
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ ANTHROPIC_API_KEY is required for Claude Code but not set"
    exit 1
fi

# Configure Claude Code to use ANTHROPIC_API_KEY
echo "🔧 Configuring Claude Code authentication..."
# TaskMaster expects ANTHROPIC_API_KEY in environment, not config file
# Export it to ensure it's available to child processes
export ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY"

# Claude Code Workaround: Fix path resolution issue in @anthropic-ai/claude-code package
# The package looks for entrypoints/cli.js but the actual file is cli.js
fix_claude_code_path() {
    echo "🔧 Checking Claude Code path resolution..."

    # Find the claude-code package directory
    CLAUDE_CODE_DIR=$(find /usr/local/lib/node_modules -name "@anthropic-ai" -type d 2>/dev/null | head -1)
    if [ -z "$CLAUDE_CODE_DIR" ]; then
        CLAUDE_CODE_DIR=$(find /opt/homebrew/lib/node_modules -name "@anthropic-ai" -type d 2>/dev/null | head -1)
    fi

    if [ -n "$CLAUDE_CODE_DIR" ] && [ -d "$CLAUDE_CODE_DIR/claude-code" ]; then
        CLAUDE_PACKAGE_DIR="$CLAUDE_CODE_DIR/claude-code"
        echo "📍 Found Claude Code package at: $CLAUDE_PACKAGE_DIR"

        # Check if cli.js exists and entrypoints/cli.js is missing
        if [ -f "$CLAUDE_PACKAGE_DIR/cli.js" ] && [ ! -f "$CLAUDE_PACKAGE_DIR/entrypoints/cli.js" ]; then
            echo "🔧 Applying Claude Code path workaround..."
            mkdir -p "$CLAUDE_PACKAGE_DIR/entrypoints"
            cp "$CLAUDE_PACKAGE_DIR/cli.js" "$CLAUDE_PACKAGE_DIR/entrypoints/cli.js"
            echo "✅ Created symlink workaround for Claude Code CLI path"
        elif [ -f "$CLAUDE_PACKAGE_DIR/entrypoints/cli.js" ]; then
            echo "✅ Claude Code path already correct"
        else
            echo "⚠️ Claude Code CLI file not found at expected locations"
        fi
    else
        echo "⚠️ Claude Code package directory not found"
    fi
}

# Apply Claude Code path fix
fix_claude_code_path

# Also try to set up Claude Code config in case it's needed
# Try to create config directory, fallback to /tmp if permission denied
if mkdir -p ~/.config/claude-code 2>/dev/null; then
    CONFIG_DIR=~/.config/claude-code
    echo "✅ Using user config directory: $CONFIG_DIR"
else
    CONFIG_DIR=/tmp/claude-code-config
    mkdir -p $CONFIG_DIR
    echo "⚠️ Permission denied for user config, using temp directory: $CONFIG_DIR"
    # Set environment variable so Claude Code knows where to find config
    export CLAUDE_CONFIG_DIR="$CONFIG_DIR"
fi

cat > $CONFIG_DIR/config.json << EOF
{
  "apiKey": "$ANTHROPIC_API_KEY"
}
EOF

# Debug: Verify API key is set
echo "🔍 DEBUG: ANTHROPIC_API_KEY is ${ANTHROPIC_API_KEY:+[SET]}${ANTHROPIC_API_KEY:-[NOT SET]}"

# Claude Code Workaround: Smart provider selection based on available API keys
select_smart_providers() {
    echo "🧠 Selecting optimal providers based on available API keys..."

    # Check which API keys are available
    local anthropic_available=false
    local openai_available=false
    local perplexity_available=false

    if [ -n "$ANTHROPIC_API_KEY" ]; then
        # Test Anthropic API key validity
        if curl -s -H "x-api-key: $ANTHROPIC_API_KEY" \
                -H "Content-Type: application/json" \
                "https://api.anthropic.com/v1/messages" \
                -d '{"model": "claude-3-haiku-20240307", "max_tokens": 1, "messages": [{"role": "user", "content": "test"}]}' > /dev/null 2>&1; then
            anthropic_available=true
            echo "✅ Anthropic API key is valid"
        else
            echo "⚠️ Anthropic API key present but invalid"
        fi
    fi

    if [ -n "$OPENAI_API_KEY" ]; then
        # Test OpenAI API key validity
        if curl -s -H "Authorization: Bearer $OPENAI_API_KEY" \
                -H "Content-Type: application/json" \
                "https://api.openai.com/v1/models" > /dev/null 2>&1; then
            openai_available=true
            echo "✅ OpenAI API key is valid"
        else
            echo "⚠️ OpenAI API key present but invalid"
        fi
    fi

    if [ -n "$PERPLEXITY_API_KEY" ]; then
        perplexity_available=true
        echo "✅ Perplexity API key is available"
    fi

    # Set smart defaults prioritizing API-based providers over claude-code
    if [ "$anthropic_available" = true ]; then
        export PRIMARY_PROVIDER="${PRIMARY_PROVIDER:-anthropic}"
        export PRIMARY_MODEL="${PRIMARY_MODEL:-claude-3-5-sonnet-20241022}"
    elif [ "$openai_available" = true ]; then
        export PRIMARY_PROVIDER="${PRIMARY_PROVIDER:-openai}"
        export PRIMARY_MODEL="${PRIMARY_MODEL:-gpt-4o-mini}"
    else
        export PRIMARY_PROVIDER="${PRIMARY_PROVIDER:-claude-code}"
        export PRIMARY_MODEL="${PRIMARY_MODEL:-opus}"
        echo "⚠️ No valid API keys found, falling back to claude-code"
    fi

    # Research model - prefer OpenAI for research, fallback to Anthropic
    if [ "$openai_available" = true ]; then
        export RESEARCH_PROVIDER="${RESEARCH_PROVIDER:-openai}"
        export RESEARCH_MODEL="${RESEARCH_MODEL:-gpt-4o-mini}"
    elif [ "$anthropic_available" = true ]; then
        export RESEARCH_PROVIDER="${RESEARCH_PROVIDER:-anthropic}"
        export RESEARCH_MODEL="${RESEARCH_MODEL:-claude-3-5-sonnet-20241022}"
    else
        export RESEARCH_PROVIDER="${RESEARCH_PROVIDER:-claude-code}"
        export RESEARCH_MODEL="${RESEARCH_MODEL:-opus}"
    fi

    # Fallback model - use whatever is available
    if [ "$openai_available" = true ] && [ "$PRIMARY_PROVIDER" != "openai" ]; then
        export FALLBACK_PROVIDER="${FALLBACK_PROVIDER:-openai}"
        export FALLBACK_MODEL="${FALLBACK_MODEL:-gpt-4o-mini}"
    elif [ "$anthropic_available" = true ] && [ "$PRIMARY_PROVIDER" != "anthropic" ]; then
        export FALLBACK_PROVIDER="${FALLBACK_PROVIDER:-anthropic}"
        export FALLBACK_MODEL="${FALLBACK_MODEL:-claude-3-haiku-20240307}"
    else
        export FALLBACK_PROVIDER="${FALLBACK_PROVIDER:-claude-code}"
        export FALLBACK_MODEL="${FALLBACK_MODEL:-sonnet}"
    fi

    echo "🎯 Smart provider selection complete:"
    echo "  Primary: $PRIMARY_PROVIDER ($PRIMARY_MODEL)"
    echo "  Research: $RESEARCH_PROVIDER ($RESEARCH_MODEL)"
    echo "  Fallback: $FALLBACK_PROVIDER ($FALLBACK_MODEL)"
}

# Apply smart provider selection
select_smart_providers

# Set up dynamic provider selection for different operations
echo "✅ Configuring TaskMaster models: Primary=$PRIMARY_MODEL, Research=$RESEARCH_MODEL, Fallback=$FALLBACK_MODEL"

# Enable codebase analysis for research operations
export TASKMASTER_ENABLE_CODEBASE_ANALYSIS=true

if [ "$OPENAI_VALID" = true ]; then
  cat > .taskmaster/config.json << EOF
{
  "project": {
    "name": "$PROJECT_NAME",
    "description": "Auto-generated project from intake pipeline",
    "version": "0.1.0"
  },
  "models": {
    "main": {
      "provider": "$PRIMARY_PROVIDER",
      "modelId": "$PRIMARY_MODEL",
      "maxTokens": 64000,
      "temperature": 0.2
    },
    "research": {
      "provider": "$RESEARCH_PROVIDER",
      "modelId": "$RESEARCH_MODEL",
      "maxTokens": 32000,
      "temperature": 0.1
    },
    "fallback": {
      "provider": "$FALLBACK_PROVIDER",
      "modelId": "$FALLBACK_MODEL",
      "maxTokens": 8000,
      "temperature": 0.7
    }
  },
  "global": {
    "defaultTag": "master"
  }
}
EOF
else
  # Use configured providers for main/research, but no fallback if OpenAI unavailable
  echo "⚠️ OpenAI API key invalid/missing, configuring without OpenAI fallback"
  cat > .taskmaster/config.json << EOF
{
  "project": {
    "name": "$PROJECT_NAME",
    "description": "Auto-generated project from intake pipeline",
    "version": "0.1.0"
  },
  "models": {
    "main": {
      "provider": "$PRIMARY_PROVIDER",
      "modelId": "$PRIMARY_MODEL",
      "maxTokens": 64000,
      "temperature": 0.2
    },
    "research": {
      "provider": "$RESEARCH_PROVIDER",
      "modelId": "$RESEARCH_MODEL",
      "maxTokens": 32000,
      "temperature": 0.1
    },
    "fallback": {
      "provider": "$FALLBACK_PROVIDER",
      "modelId": "$FALLBACK_MODEL",
      "maxTokens": 8000,
      "temperature": 0.7
    }
  },
  "global": {
    "defaultTag": "master"
  }
}
EOF
fi

echo "✅ Claude Code configuration written"

# Debug: Show what TaskMaster config was written
echo "🔍 DEBUG: TaskMaster config contents:"
jq '.' .taskmaster/config.json || echo "Failed to display config"

# Parse PRD with research model for better analysis
echo "📄 Parsing PRD to generate tasks with Research model: $RESEARCH_MODEL ($RESEARCH_PROVIDER)..."
# Debug: Check if claude command is available (for claude-code provider)
if [ "$PRIMARY_PROVIDER" = "claude-code" ] || [ "$RESEARCH_PROVIDER" = "claude-code" ]; then
    echo "🔍 DEBUG: Checking claude-code availability..."
    which claude || echo "⚠️ claude command not found in PATH"
    echo "🔍 DEBUG: PATH=$PATH"
fi
# Claude Code Workaround: Enhanced error detection and logging
detect_claude_code_error() {
    local error_output="$1"
    local operation="$2"

    echo "🔍 Analyzing error for $operation..."

    # Check for known Claude Code error patterns
    if echo "$error_output" | grep -q "Claude Code executable not found"; then
        echo "🚨 Detected: Claude Code executable path issue"
        echo "   This is a known issue with @anthropic-ai/claude-code package"
        return 1
    elif echo "$error_output" | grep -q "entrypoints/cli.js"; then
        echo "🚨 Detected: Claude Code path resolution bug"
        echo "   Package is looking for entrypoints/cli.js instead of cli.js"
        return 2
    elif echo "$error_output" | grep -q "claude-code.*API.*error"; then
        echo "🚨 Detected: Claude Code API integration error"
        return 3
    elif echo "$error_output" | grep -q "ANTHROPIC_API_KEY"; then
        echo "🚨 Detected: Missing or invalid Anthropic API key"
        return 4
    else
        echo "🤔 Error pattern not recognized as Claude Code specific"
        return 0
    fi
}

# Log Claude Code diagnostics for troubleshooting
log_claude_code_diagnostics() {
    echo "🔍 Claude Code Diagnostics:"
    echo "  Node version: $(node --version 2>/dev/null || echo 'not found')"
    echo "  NPM version: $(npm --version 2>/dev/null || echo 'not found')"
    echo "  Claude command available: $(which claude 2>/dev/null || echo 'not found')"
    echo "  Claude Code package location: $(find /usr/local/lib/node_modules -name "@anthropic-ai" 2>/dev/null | head -1 || find /opt/homebrew/lib/node_modules -name "@anthropic-ai" 2>/dev/null | head -1 || echo 'not found')"
    echo "  ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:+[SET]}${ANTHROPIC_API_KEY:-[NOT SET]}"
    echo "  CLAUDE_CONFIG_DIR: ${CLAUDE_CONFIG_DIR:-[NOT SET]}"
}

# Claude Code Workaround: Helper functions for provider fallback
get_provider_fallback() {
    local provider="$1"
    case "$provider" in
        "claude-code") echo "anthropic" ;;
        "anthropic") echo "openai" ;;
        "openai") echo "anthropic" ;;
        *) echo "$provider" ;;
    esac
}

get_model_fallback() {
    local provider="$1"
    case "$provider" in
        "claude-code"|"anthropic") echo "claude-3-5-sonnet-20241022" ;;
        "openai") echo "gpt-4o-mini" ;;
        *) echo "claude-3-5-sonnet-20241022" ;;
    esac
}

# Claude Code Workaround: Add fallback mechanism for failed operations
configure_provider_fallback() {
    local operation="$1"
    local current_config="$2"

    echo "🔄 Attempting provider fallback for $operation..."

    # Extract current providers from config
    local main_provider=$(jq -r '.models.main.provider // "unknown"' "$current_config" 2>/dev/null)
    local research_provider=$(jq -r '.models.research.provider // "unknown"' "$current_config" 2>/dev/null)
    local fallback_provider=$(jq -r '.models.fallback.provider // "unknown"' "$current_config" 2>/dev/null)

    # Create new config with fallback providers
    local main_fallback_provider=$(get_provider_fallback "$main_provider")
    local main_fallback_model=$(get_model_fallback "$main_provider")
    local research_fallback_provider=$(get_provider_fallback "$research_provider")
    local research_fallback_model=$(get_model_fallback "$research_provider")
    local fallback_fallback_provider=$(get_provider_fallback "$fallback_provider")
    local fallback_fallback_model=$(get_model_fallback "$fallback_provider")

    local new_config=$(jq \
        --arg main_provider "${main_fallback_provider:-$main_provider}" \
        --arg main_model "${main_fallback_model:-claude-3-5-sonnet-20241022}" \
        --arg research_provider "${research_fallback_provider:-$research_provider}" \
        --arg research_model "${research_fallback_model:-claude-3-5-sonnet-20241022}" \
        --arg fallback_provider "${fallback_fallback_provider:-$fallback_provider}" \
        --arg fallback_model "${fallback_fallback_model:-gpt-4o-mini}" \
        '.models.main.provider = $main_provider |
         .models.main.modelId = $main_model |
         .models.research.provider = $research_provider |
         .models.research.modelId = $research_model |
         .models.fallback.provider = $fallback_provider |
         .models.fallback.modelId = $fallback_model' "$current_config")

    echo "$new_config" > "$current_config.tmp" && mv "$current_config.tmp" "$current_config"
    echo "✅ Applied provider fallback configuration"
}

# Use --research flag to use the configured research model
if ! task-master parse-prd \
    --input ".taskmaster/docs/prd.txt" \
    --force \
    --research 2>&1; then
    echo "❌ Failed to parse PRD with current configuration"

    # Log diagnostics for troubleshooting
    log_claude_code_diagnostics

    # Try fallback configuration
    configure_provider_fallback "PRD parsing" ".taskmaster/config.json"

    # Retry with fallback configuration
    if ! task-master parse-prd \
        --input ".taskmaster/docs/prd.txt" \
        --force \
        --research 2>&1; then
        echo "❌ PRD parsing failed even with fallback configuration"
        echo "📋 Final diagnostics before exit:"
        log_claude_code_diagnostics
        exit 1
    fi

    echo "✅ PRD parsing succeeded with fallback configuration"
fi

# Resolve tasks.json path (use default, fallback to discovery)
TASKS_FILE=".taskmaster/tasks/tasks.json"
if [ ! -f "$TASKS_FILE" ]; then
    ALT_TASKS_FILE=$(find .taskmaster -maxdepth 2 -name tasks.json | head -n 1 || true)
    if [ -n "$ALT_TASKS_FILE" ] && [ -f "$ALT_TASKS_FILE" ]; then
        TASKS_FILE="$ALT_TASKS_FILE"
    else
        echo "❌ tasks.json not found after parse"
        exit 1
    fi
fi

# ========================================
# Auto-Add Agent Hints
# ========================================
echo "🎯 Auto-detecting and adding agent routing hints..."

# Post-process tasks.json to add agentHint based on task content
jq '
  # Function to check if task should use frontend agent
  def is_frontend_task:
    (.title + " " + (.description // "") + " " + (.details // "")) 
    | test("frontend|react|component|ui|interface|styling|css|html|jsx|tsx|vue|angular|svelte|next\\.js|nuxt|page|layout|button|form|modal|navbar|header|footer|material-ui|mui|@mui|shadcn|component.*registry"; "i");
  
  # Function to check if task should use integration/testing agent
  def is_integration_task:
    (.title + " " + (.description // "") + " " + (.details // "")) 
    | test("test|testing|integration|e2e|end[- ]to[- ]end|qa|quality|verify|validation|cypress|playwright|jest|vitest|spec"; "i");
  
  # Apply to all tasks (handle both tagged and non-tagged formats)
  if .master.tasks then
    .master.tasks |= map(
      if .agentHint then
        # Keep existing agentHint if present
        .
      elif is_frontend_task then
        . + {"agentHint": "frontend"}
      elif is_integration_task then
        . + {"agentHint": "integration"}
      else
        # Backend/general tasks - no hint needed (defaults to Rex)
        .
      end
    )
  elif .tasks then
    .tasks |= map(
      if .agentHint then
        .
      elif is_frontend_task then
        . + {"agentHint": "frontend"}
      elif is_integration_task then
        . + {"agentHint": "integration"}
      else
        .
      end
    )
  else
    .
  end
' "$TASKS_FILE" > "$TASKS_FILE.tmp" && mv "$TASKS_FILE.tmp" "$TASKS_FILE"

echo "✅ Agent hints added based on task content"

# ========================================
# Append Integration Task
# ========================================
echo "📝 Appending integration & deployment verification task..."

# Use jq to append integration AND publishing tasks
jq '
  # Calculate the new task IDs (max existing ID + 1, + 2)
  ((.master.tasks // .tasks) | map(.id) | max + 1) as $integration_id |
  ($integration_id + 1) as $publishing_id |
  # Get all existing task IDs for dependencies
  ((.master.tasks // .tasks) | map(.id)) as $all_deps |
  # Append the integration task (depends on all tasks)
  if .master.tasks then
    .master.tasks += [{
      "id": $integration_id,
      "title": "Integration Tests & Validation",
      "description": "Execute comprehensive integration tests to verify all components work together correctly",
      "status": "pending",
      "dependencies": $all_deps,
      "priority": "high",
      "agentHint": "integration",
      "details": "## Integration Task Responsibilities\n\nThis task coordinates the final integration of all parallel development work.\n\n### 1. PR Merge Coordination\n- **Check PR Status**: Verify all dependent PRs are approved and passing checks\n  ```bash\n  gh pr list --state open --json number,title,mergeable,statusCheckRollup\n  ```\n- **Merge in Dependency Order**: Respect task dependencies when merging\n  - Example: If task 3 depends on task 1, merge PR for task 1 first\n  - Use: `gh pr merge <PR_NUMBER> --squash --delete-branch`\n- **Handle Conflicts**: If merge conflicts occur, document them and request manual intervention\n\n### 2. Integration Testing\n- **Backend Integration Tests**:\n  ```bash\n  # Run integration test suite\n  cargo test --test \"*\" -- --test-threads=1  # Rust\n  npm run test:integration                     # Node.js\n  pytest tests/integration/                    # Python\n  ```\n- **API Contract Testing**:\n  - Verify all API endpoints are accessible\n  - Check response schemas match expectations\n  - Test inter-service communication\n\n- **Frontend Integration** (if applicable):\n  - Run E2E tests with Playwright/Cypress\n  - Verify UI components render correctly\n  - Test user flows end-to-end\n  ```bash\n  npm run test:e2e  # If E2E tests exist\n  ```\n\n### 3. Deployment Verification\n- **Configuration Validation**:\n  - Check deployment manifests exist\n  - Verify environment variables are set\n  - Validate Kubernetes/Docker configs\n\n- **Service Health Checks**:\n  ```bash\n  # If deployed to Kubernetes\n  kubectl get pods -n <namespace>\n  kubectl get services -n <namespace>\n  \n  # Health endpoint checks\n  curl -f http://service:port/health\n  curl -f http://service:port/ready\n  ```\n\n- **Database Migrations** (if applicable):\n  - Verify migrations ran successfully\n  - Check schema is up to date\n\n### 4. Smoke Testing\n- Test critical user paths\n- Verify authentication/authorization\n- Check data persistence\n- Test error handling\n\n### 5. Reporting\n- **Generate Integration Summary**:\n  - List all merged PRs with links\n  - Show integration test results\n  - Document any issues found\n  - Confirm deployment readiness\n\n- **Create Deployment Checklist**:\n  - [ ] All PRs merged successfully\n  - [ ] Integration tests passing\n  - [ ] Health checks passing\n  - [ ] No critical issues found\n  - [ ] Ready for production deployment\n\n## Success Criteria\n✅ All dependent task PRs merged to main\n✅ Integration test suite passes\n✅ All services healthy and accessible\n✅ No critical bugs or conflicts detected\n✅ Deployment verification complete",
      "testStrategy": "Execute full integration test suite, verify all service health endpoints, run smoke tests on critical paths, and confirm deployment readiness with zero critical issues"
    }, {
      "id": $publishing_id,
      "title": "Publishing & Deployment",
      "description": "Merge all PRs, deploy application to Kubernetes, configure Ngrok ingress, and provide public access URLs",
      "status": "pending",
      "dependencies": [$integration_id],
      "priority": "critical",
      "agentHint": "deployment",
      "details": "See publishing task template for complete deployment instructions",
      "testStrategy": "Merge all PRs, build and deploy to Kubernetes, configure Ngrok ingress, verify public URL accessibility, run smoke tests, and provide deployment report with access URLs"
    }]
  else
    .tasks += [{
      "id": $integration_id,
      "title": "Integration Tests & Validation",
      "description": "Execute comprehensive integration tests to verify all components work together correctly",
      "status": "pending",
      "dependencies": $all_deps,
      "priority": "high",
      "agentHint": "integration",
      "details": "## Integration Task Responsibilities\n\nThis task coordinates the final integration of all parallel development work.\n\n### 1. PR Merge Coordination\n- **Check PR Status**: Verify all dependent PRs are approved and passing checks\n  ```bash\n  gh pr list --state open --json number,title,mergeable,statusCheckRollup\n  ```\n- **Merge in Dependency Order**: Respect task dependencies when merging\n  - Example: If task 3 depends on task 1, merge PR for task 1 first\n  - Use: `gh pr merge <PR_NUMBER> --squash --delete-branch`\n- **Handle Conflicts**: If merge conflicts occur, document them and request manual intervention\n\n### 2. Integration Testing\n- **Backend Integration Tests**:\n  ```bash\n  # Run integration test suite\n  cargo test --test \"*\" -- --test-threads=1  # Rust\n  npm run test:integration                     # Node.js\n  pytest tests/integration/                    # Python\n  ```\n- **API Contract Testing**:\n  - Verify all API endpoints are accessible\n  - Check response schemas match expectations\n  - Test inter-service communication\n\n- **Frontend Integration** (if applicable):\n  - Run E2E tests with Playwright/Cypress\n  - Verify UI components render correctly\n  - Test user flows end-to-end\n  ```bash\n  npm run test:e2e  # If E2E tests exist\n  ```\n\n### 3. Deployment Verification\n- **Configuration Validation**:\n  - Check deployment manifests exist\n  - Verify environment variables are set\n  - Validate Kubernetes/Docker configs\n\n- **Service Health Checks**:\n  ```bash\n  # If deployed to Kubernetes\n  kubectl get pods -n <namespace>\n  kubectl get services -n <namespace>\n  \n  # Health endpoint checks\n  curl -f http://service:port/health\n  curl -f http://service:port/ready\n  ```\n\n- **Database Migrations** (if applicable):\n  - Verify migrations ran successfully\n  - Check schema is up to date\n\n### 4. Smoke Testing\n- Test critical user paths\n- Verify authentication/authorization\n- Check data persistence\n- Test error handling\n\n### 5. Reporting\n- **Generate Integration Summary**:\n  - List all merged PRs with links\n  - Show integration test results\n  - Document any issues found\n  - Confirm deployment readiness\n\n- **Create Deployment Checklist**:\n  - [ ] All PRs merged successfully\n  - [ ] Integration tests passing\n  - [ ] Health checks passing\n  - [ ] No critical issues found\n  - [ ] Ready for production deployment\n\n## Success Criteria\n✅ All dependent task PRs merged to main\n✅ Integration test suite passes\n✅ All services healthy and accessible\n✅ No critical bugs or conflicts detected\n✅ Deployment verification complete",
      "testStrategy": "Execute full integration test suite, verify all service health endpoints, run smoke tests on critical paths, and confirm deployment readiness with zero critical issues"
    }, {
      "id": $publishing_id,
      "title": "Publishing & Deployment",
      "description": "Merge all PRs, deploy application to Kubernetes, configure Ngrok ingress, and provide public access URLs",
      "status": "pending",
      "dependencies": [$integration_id],
      "priority": "critical",
      "agentHint": "deployment",
      "details": "See publishing task template for complete deployment instructions",
      "testStrategy": "Merge all PRs, build and deploy to Kubernetes, configure Ngrok ingress, verify public URL accessibility, run smoke tests, and provide deployment report with access URLs"
    }]
  end
' "$TASKS_FILE" > "$TASKS_FILE.tmp" && mv "$TASKS_FILE.tmp" "$TASKS_FILE"

FINAL_TASK_ID=$(jq '(.master.tasks // .tasks) | map(.id) | max' "$TASKS_FILE")
INTEGRATION_ID=$((FINAL_TASK_ID - 1))
echo "✅ Integration task appended (ID: $INTEGRATION_ID)"
echo "   This task will run integration tests after all implementation tasks complete"
echo "✅ Publishing task appended (ID: $FINAL_TASK_ID)"
echo "   This task will merge all PRs, deploy to Kubernetes, and provide Ngrok URLs"

# ========================================

# Analyze complexity if requested
if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    echo "🔍 Analyzing task complexity..."
    mkdir -p .taskmaster/reports
    if ! task-master analyze-complexity --file "$TASKS_FILE" 2>&1; then
        echo "❌ analyze-complexity failed with current configuration"

        # Log diagnostics for troubleshooting
        log_claude_code_diagnostics

        # Try fallback configuration
        configure_provider_fallback "complexity analysis" ".taskmaster/config.json"

        # Retry with fallback configuration
        if ! task-master analyze-complexity --file "$TASKS_FILE" 2>&1; then
            echo "❌ Complexity analysis failed even with fallback configuration"
            echo "📋 Final diagnostics before exit:"
            log_claude_code_diagnostics
            exit 1
        fi

        echo "✅ Complexity analysis succeeded with fallback configuration"
    fi
fi

# Expand tasks if requested (switch to regular Claude API for faster expansion)
if [ "$EXPAND_TASKS" = "true" ]; then
    echo "🌳 Expanding tasks with subtasks using Claude API..."

    # Switch ONLY the main provider to regular Claude API for faster expansion
    # Keep research with Claude Code for any research operations
    if [ "$OPENAI_VALID" = true ]; then
        # Read current config and update only the main provider
        if [ -f ".taskmaster/config.json" ]; then
            # Use jq to update only the main provider in the existing config
            jq --arg provider "$PRIMARY_PROVIDER" --arg model "$PRIMARY_MODEL" '.models.main = {
                "provider": $provider,
                "modelId": $model,
                "maxTokens": 64000,
                "temperature": 0.2
            }' .taskmaster/config.json > .taskmaster/config_temp.json && mv .taskmaster/config_temp.json .taskmaster/config.json
            echo "✅ Updated main provider to $PRIMARY_PROVIDER ($PRIMARY_MODEL), kept research with $RESEARCH_PROVIDER"
        else
            echo "⚠️ Config file not found, using default Claude API config"
            cat > .taskmaster/config.json << EOF
{
  "project": {
    "name": "$PROJECT_NAME",
    "description": "Auto-generated project from intake pipeline",
    "version": "0.1.0"
  },
  "models": {
    "main": {
      "provider": "$PRIMARY_PROVIDER",
      "modelId": "$PRIMARY_MODEL",
      "maxTokens": 64000,
      "temperature": 0.2
    },
    "research": {
      "provider": "$RESEARCH_PROVIDER",
      "modelId": "$RESEARCH_MODEL",
      "maxTokens": 32000,
      "temperature": 0.1
    },
    "fallback": {
      "provider": "$FALLBACK_PROVIDER",
      "modelId": "$FALLBACK_MODEL",
      "maxTokens": 8000,
      "temperature": 0.7
    }
  },
  "global": {
    "defaultTag": "master"
  }
}
EOF
        fi
    fi

    if ! task-master expand --all --force --file "$TASKS_FILE" 2>&1; then
        echo "❌ expand failed with current configuration"

        # Log diagnostics for troubleshooting
        log_claude_code_diagnostics

        # Try fallback configuration
        configure_provider_fallback "task expansion" ".taskmaster/config.json"

        # Retry with fallback configuration
        if ! task-master expand --all --force --file "$TASKS_FILE" 2>&1; then
            echo "❌ Task expansion failed even with fallback configuration"
            echo "📋 Final diagnostics before exit:"
            log_claude_code_diagnostics
            exit 1
        fi

        echo "✅ Task expansion succeeded with fallback configuration"
    fi
fi

# Review and align tasks with architecture using Claude
echo "🤖 Reviewing tasks against architecture with Claude..."
if [ -f ".taskmaster/docs/architecture.md" ]; then
    # Check if claude command is available
    if command -v claude &> /dev/null; then
        echo "✅ Claude command found"

        # Create a prompt for Claude to review tasks
        cat > /tmp/review-prompt.md <<'EOF'
Please review the tasks.json file against the architecture.md document and ensure they are properly aligned.

Your task is to:
1. Cross-reference all tasks in tasks.json with the architecture diagram
2. Identify any missing tasks that are implied by the architecture
3. Identify any tasks that don't align with the architecture
4. Update, add, or remove tasks as needed to ensure full alignment

Important:
- Make direct edits to the tasks.json file
- Ensure all architectural components have corresponding tasks
- Ensure task dependencies match the architectural flow
- Preserve the existing task structure and IDs where possible
- Add clear details and implementation notes based on the architecture

Files to review:
- .taskmaster/tasks/tasks.json (the task list)
- .taskmaster/docs/architecture.md (the architecture reference)

Make the necessary modifications directly to ensure the tasks and architecture are fully aligned.
EOF

        # Run Claude Code to review and update tasks
        echo "🔍 Running Claude Code review..."
        # Set Claude config directory
        export CLAUDE_CONFIG_DIR="$CONFIG_DIR"
        if [ -s "/tmp/review-prompt.md" ]; then
          echo "📝 Processing review prompt with Claude Code..."
          # Claude Code is interactive and doesn't support programmatic JSON output
          # Skip this step for now as it's causing timeouts and failures
          echo "⚠️ Claude Code review skipped - interactive tool not suitable for automated processing"
          echo "📋 TaskMaster parsing should handle architecture alignment during PRD processing"
        else
          echo "⚠️ Review prompt file missing or empty; skipping Claude Code review"
        fi

        echo "✅ Task review complete (skipped)"
    else
        echo "⚠️ Claude command not found, skipping architecture alignment"
    fi
else
    echo "⚠️ No architecture.md file found, skipping architecture alignment"
fi

# Generate task files
echo "📝 Generating individual task files..."
task-master generate

# Create summary file
echo "📊 Creating project summary..."
cat > README.md <<EOF
# $PROJECT_NAME

Auto-generated project from intake pipeline.

## Project Structure

- **.taskmaster/** - TaskMaster configuration and tasks
  - **docs/** - Source documents (PRD, architecture)
  - **tasks/** - Generated task definitions
- **docs/** - Individual task documentation

## Getting Started

1. Review the generated tasks in .taskmaster/tasks/tasks.json
2. Use task-master list to view all tasks
3. Use task-master next to get the next task to work on
4. Implement tasks using the orchestrator workflow

## Generated Statistics

- Total tasks: $(jq '.tasks | length' .taskmaster/tasks/tasks.json 2>/dev/null || echo "N/A")
- Model used: $MODEL
- Generated on: $(date)

## Source Documents

- [Product Requirements](/.taskmaster/docs/prd.txt)
$([ -f ".taskmaster/docs/architecture.md" ] && echo "- [Architecture](/.taskmaster/docs/architecture.md)")
EOF

# Commit changes
echo "💾 Committing project structure..."
cd "$CLONE_DIR"
git add -A

# Build commit message properly
COMMIT_MSG="feat: initialize project $PROJECT_NAME

- Automated project intake via orchestrator
- Parsed PRD and architecture documents
- Generated TaskMaster task breakdown
- Created standardized project structure
- Set up CI/CD workflows and templates

🤖 Auto-generated by project intake workflow
- Model: $MODEL
- Tasks: $NUM_TASKS targets"

if [ "$EXPAND_TASKS" = "true" ]; then
    COMMIT_MSG="$COMMIT_MSG
- Expanded with subtasks"
fi

if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    COMMIT_MSG="$COMMIT_MSG
- Complexity analysis performed"
fi

git commit -m "$COMMIT_MSG"

# Create branch and push
# Use a hyphenated prefix to avoid collisions when a flat ref named 'intake' exists remotely
# Also prefer the sanitized, lowercase project directory name for the branch component
BRANCH_NAME="intake-${PROJECT_DIR_NAME}-$(date +%Y%m%d-%H%M%S)"
echo "🌿 Creating branch: $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"
git push -u origin "$BRANCH_NAME"

# Create pull request
echo "🔀 Creating pull request..."

# Build PR body inline to avoid function issues
ARCH_INCLUDED=""
if [ -f "$PROJECT_DIR/.taskmaster/docs/architecture.md" ]; then
    ARCH_INCLUDED="- ✅ Architecture document included"
fi

COMPLEXITY_DONE=""
if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    COMPLEXITY_DONE="- ✅ Complexity analysis performed"
fi

EXPANSION_DONE=""
if [ "$EXPAND_TASKS" = "true" ]; then
    EXPANSION_DONE="- ✅ Tasks expanded with subtasks"
fi

TASK_COUNT=$(jq '.tasks | length' "$PROJECT_DIR/.taskmaster/tasks/tasks.json" 2>/dev/null || echo "N/A")

# Build PR body using a temporary file to avoid bash interpretation issues
PR_BODY_FILE="/tmp/pr_body_$$.txt"
cat > "$PR_BODY_FILE" << 'EOF'
## 🎉 Project Intake: PROJECT_NAME_PLACEHOLDER

This PR contains the auto-generated project structure and tasks.

### 📋 What was processed:
- ✅ PRD document parsed
ARCH_INCLUDED_PLACEHOLDER
- ✅ TaskMaster initialized
- ✅ Tasks generated (target: NUM_TASKS_PLACEHOLDER)
COMPLEXITY_DONE_PLACEHOLDER
EXPANSION_DONE_PLACEHOLDER
- ✅ Project structure created

### 🏗️ Generated Structure:
```
PROJECT_DIR_PLACEHOLDER/
├── .taskmaster/
│   ├── docs/
│   │   ├── prd.txt
│   │   └── architecture.md
│   ├── tasks/
│   │   └── tasks.json
│   └── config.json
├── docs/
│   ├── task-1/
│   │   └── task.md
│   └── ...
└── README.md
```

### 🤖 Configuration:
- **Model**: MODEL_PLACEHOLDER
- **Tasks Generated**: TASK_COUNT_PLACEHOLDER
- **Complexity Analysis**: ANALYZE_COMPLEXITY_PLACEHOLDER
- **Task Expansion**: EXPAND_TASKS_PLACEHOLDER

### 🎯 Next Steps:
1. Review the generated tasks
2. Merge this PR to add the project
3. Use orchestrator workflows to implement tasks
EOF

# Replace placeholders with actual values using sed (using | as delimiter to avoid issues with forward slashes in paths)
sed -i "s|PROJECT_NAME_PLACEHOLDER|$PROJECT_NAME|g" "$PR_BODY_FILE"
sed -i "s|ARCH_INCLUDED_PLACEHOLDER|$ARCH_INCLUDED|g" "$PR_BODY_FILE"
sed -i "s|NUM_TASKS_PLACEHOLDER|$NUM_TASKS|g" "$PR_BODY_FILE"
sed -i "s|COMPLEXITY_DONE_PLACEHOLDER|$COMPLEXITY_DONE|g" "$PR_BODY_FILE"
sed -i "s|EXPANSION_DONE_PLACEHOLDER|$EXPANSION_DONE|g" "$PR_BODY_FILE"
sed -i "s|PROJECT_DIR_PLACEHOLDER|$PROJECT_DIR|g" "$PR_BODY_FILE"
sed -i "s|MODEL_PLACEHOLDER|$MODEL|g" "$PR_BODY_FILE"
sed -i "s|TASK_COUNT_PLACEHOLDER|$TASK_COUNT|g" "$PR_BODY_FILE"
sed -i "s|ANALYZE_COMPLEXITY_PLACEHOLDER|$ANALYZE_COMPLEXITY|g" "$PR_BODY_FILE"
sed -i "s|EXPAND_TASKS_PLACEHOLDER|$EXPAND_TASKS|g" "$PR_BODY_FILE"

# Read the final PR body
PR_BODY=$(cat "$PR_BODY_FILE")
rm -f "$PR_BODY_FILE"

# Refresh GitHub token before PR creation
if [ -n "$GITHUB_APP_PRIVATE_KEY" ]; then
    echo "🔄 Refreshing GitHub token for PR creation..."
    generate_github_token
fi

gh pr create \
    --title "🚀 Project Intake: $PROJECT_NAME" \
    --body "$PR_BODY" \
    --head "$BRANCH_NAME" \
    --base main || {
        echo "⚠️ Failed to create PR, but branch has been pushed"
        echo "Branch: $BRANCH_NAME"
        echo "You can create the PR manually"
    }

echo "✅ Project intake complete!"
echo "================================="
echo "Project: $PROJECT_NAME"
echo "Location: $PROJECT_DIR"
echo "Branch: $BRANCH_NAME"
echo "Repository: $REPOSITORY_URL"
