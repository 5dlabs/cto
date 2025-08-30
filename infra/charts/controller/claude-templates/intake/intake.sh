#!/bin/bash
set -e

# Force output to be unbuffered
exec 2>&1
set -x  # Enable command tracing temporarily

# Add error trap for debugging
trap 'echo "❌ Error occurred at line $LINENO with exit code $?. Last command: $BASH_COMMAND"; exit 1' ERR

echo "🚀 Starting Project Intake Process"
echo "================================="

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
            export PATH="$(dirname $npm_path):$PATH"
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
cat .taskmaster/config.json | jq '.' || echo "Failed to display config"

# Parse PRD with research model for better analysis
echo "📄 Parsing PRD to generate tasks with Research model: $RESEARCH_MODEL ($RESEARCH_PROVIDER)..."
# Debug: Check if claude command is available (for claude-code provider)
if [ "$PRIMARY_PROVIDER" = "claude-code" ] || [ "$RESEARCH_PROVIDER" = "claude-code" ]; then
    echo "🔍 DEBUG: Checking claude-code availability..."
    which claude || echo "⚠️ claude command not found in PATH"
    echo "🔍 DEBUG: PATH=$PATH"
fi
# Use --research flag to use the configured research model
task-master parse-prd \
    --input ".taskmaster/docs/prd.txt" \
    --force \
    --research || {
    echo "❌ Failed to parse PRD"
    exit 1
}

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

# Analyze complexity if requested
if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    echo "🔍 Analyzing task complexity..."
    mkdir -p .taskmaster/reports
    task-master analyze-complexity --file "$TASKS_FILE" || {
        echo "❌ analyze-complexity failed"
        exit 1
    }
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

    task-master expand --all --force --file "$TASKS_FILE" || {
        echo "❌ expand failed"
        exit 1
    }
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
          # Claude Code uses simpler command line arguments
          timeout 300 claude --model "$MODEL" < /tmp/review-prompt.md > /tmp/claude-output.json 2>/tmp/claude-error.log || {
              echo "⚠️ Claude Code review failed (exit code: $?), but continuing..."
              echo "Error log:" && cat /tmp/claude-error.log 2>/dev/null || echo "No error log available"
          }
          # Check if we got a valid response
          if [ -s "/tmp/claude-output.json" ]; then
              echo "✅ Claude Code review completed successfully"
          else
              echo "⚠️ Claude Code review produced no output"
          fi
        else
          echo "⚠️ Review prompt file missing or empty; skipping Claude Code review"
        fi
        
        echo "✅ Task review complete"
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

1. Review the generated tasks in \`.taskmaster/tasks/tasks.json\`
2. Use \`task-master list\` to view all tasks
3. Use \`task-master next\` to get the next task to work on
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
git commit -m "feat: initialize project $PROJECT_NAME

- Automated project intake via orchestrator
- Parsed PRD and architecture documents
- Generated TaskMaster task breakdown
- Created standardized project structure
- Set up CI/CD workflows and templates

🤖 Auto-generated by project intake workflow"
- Model: $MODEL
- Tasks: $NUM_TASKS targets
$([ "$EXPAND_TASKS" = "true" ] && echo "- Expanded with subtasks")
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- Complexity analysis performed")
"

# Create branch and push
# Use a hyphenated prefix to avoid collisions when a flat ref named 'intake' exists remotely
# Also prefer the sanitized, lowercase project directory name for the branch component
BRANCH_NAME="intake-${PROJECT_DIR_NAME}-$(date +%Y%m%d-%H%M%S)"
echo "🌿 Creating branch: $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"
git push -u origin "$BRANCH_NAME"

# Create pull request
echo "🔀 Creating pull request..."
PR_BODY="## 🎉 Project Intake: $PROJECT_NAME

This PR contains the auto-generated project structure and tasks.

### 📋 What was processed:
- ✅ PRD document parsed
$([ -f "$PROJECT_DIR/.taskmaster/docs/architecture.md" ] && echo "- ✅ Architecture document included")
- ✅ TaskMaster initialized
- ✅ Tasks generated (target: $NUM_TASKS)
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- ✅ Complexity analysis performed")
$([ "$EXPAND_TASKS" = "true" ] && echo "- ✅ Tasks expanded with subtasks")
- ✅ Project structure created

### 🏗️ Generated Structure:
\`\`\`
$PROJECT_DIR/
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
\`\`\`

### 🤖 Configuration:
- **Model**: $MODEL
- **Tasks Generated**: $(jq '.tasks | length' "$PROJECT_DIR/.taskmaster/tasks/tasks.json" 2>/dev/null || echo "N/A")
- **Complexity Analysis**: $ANALYZE_COMPLEXITY
- **Task Expansion**: $EXPAND_TASKS

### 🎯 Next Steps:
1. Review the generated tasks
2. Merge this PR to add the project
3. Use orchestrator workflows to implement tasks
"

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
