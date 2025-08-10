#!/bin/bash
set -e

# Force output to be unbuffered
exec 2>&1
set -x  # Enable command tracing temporarily

# Add error trap for debugging
trap 'echo "❌ Error occurred at line $LINENO with exit code $?. Last command: $BASH_COMMAND"; exit 1' ERR

echo "🚀 Starting Project Intake Process"
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
echo "📄 Config file contents:"
cat "$CONFIG_FILE" || echo "Failed to cat config file"
echo ""
echo "---"

# Parse configuration
echo "📋 Loading configuration from ConfigMap..."

# Parse each field with error handling
PROJECT_NAME=$(jq -r '.project_name' "$CONFIG_FILE" 2>/dev/null || echo "")
echo "  ✓ Project name: $PROJECT_NAME"

REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE" 2>/dev/null || echo "")
echo "  ✓ Repository URL: $REPOSITORY_URL"

GITHUB_APP=$(jq -r '.github_app' "$CONFIG_FILE" 2>/dev/null || echo "")
echo "  ✓ GitHub App: $GITHUB_APP"

MODEL=$(jq -r '.model' "$CONFIG_FILE" 2>/dev/null || echo "claude-3-5-sonnet-20241022")
echo "  ✓ Model: $MODEL"

NUM_TASKS=$(jq -r '.num_tasks' "$CONFIG_FILE" 2>/dev/null || echo "10")
echo "  ✓ Num tasks: $NUM_TASKS"

EXPAND_TASKS=$(jq -r '.expand_tasks' "$CONFIG_FILE" 2>/dev/null || echo "false")
echo "  ✓ Expand tasks: $EXPAND_TASKS"

ANALYZE_COMPLEXITY=$(jq -r '.analyze_complexity' "$CONFIG_FILE" 2>/dev/null || echo "false")
echo "  ✓ Analyze complexity: $ANALYZE_COMPLEXITY"

echo "🔍 Configuration summary:"
echo "  - Project: ${PROJECT_NAME:-[empty]}"
echo "  - Repository: ${REPOSITORY_URL:-[empty]}"
echo "  - GitHub App: ${GITHUB_APP:-[empty]}"
echo "  - Model: ${MODEL:-[empty]}"
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
        
        INSTALLATION_ID=$(curl -s -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/installation" | jq -r '.id')
        
        if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
            echo "❌ Failed to get installation ID for $REPO_OWNER/$REPO_NAME"
            return 1
        fi
        
        echo "Installation ID: $INSTALLATION_ID"
        
        # Generate installation access token
        GITHUB_TOKEN=$(curl -s -X POST \
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

# Install TaskMaster globally
echo "📦 Installing TaskMaster..."
echo "📋 Node version: $(node --version)"
echo "📋 NPM version: $(npm --version)"

# Check if we're in the Claude Code container
if [ -d "/usr/local/share/npm-global" ] && [ -w "/usr/local/share/npm-global" ]; then
    echo "✅ Detected Claude Code container environment"
    export NPM_CONFIG_PREFIX=/usr/local/share/npm-global
    export PATH=$PATH:/usr/local/share/npm-global/bin
    NPM_BIN="/usr/local/share/npm-global/bin"
else
    echo "🔍 Using default npm global location"
    # Let npm use its default global location
    NPM_BIN=$(npm bin -g 2>/dev/null || echo "/usr/local/bin")
fi

npm install -g task-master-ai@latest || {
    echo "❌ TaskMaster installation failed"
    echo "🔍 Trying with --force flag..."
    npm install -g task-master-ai@latest --force || exit 1
}

# Verify installation location
echo "🔍 NPM global bin directory: $NPM_BIN"
echo "🔍 Current PATH: $PATH"

# Add npm global bin to PATH if not already there
if [[ ":$PATH:" != *":$NPM_BIN:"* ]]; then
    export PATH="$NPM_BIN:$PATH"
    echo "🔍 Added $NPM_BIN to PATH"
fi

# Verify installation
if ! command -v task-master &> /dev/null; then
    echo "❌ task-master command not found after installation"
    echo "🔍 PATH: $PATH"
    echo "🔍 Looking for task-master in npm bin:"
    ls -la "$NPM_BIN" | grep -i task || echo "Not found in $NPM_BIN"
    exit 1
fi

# Get the actual path to task-master
TASK_MASTER_PATH=$(which task-master)
echo "✅ TaskMaster installed at: $TASK_MASTER_PATH"
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

# First attempt: Try clean init with all flags
echo "🔍 Attempting TaskMaster init with full flags..."
# Use the full path to ensure we're calling the right binary
"$TASK_MASTER_PATH" init --yes \
    --name "$PROJECT_NAME" \
    --description "Auto-generated project from intake pipeline" \
    --version "0.1.0" \
    --rules "cursor" \
    --skip-install \
    --aliases
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
        
        # Create a minimal config.json file
        cat > .taskmaster/config.json << EOF
{
  "project": {
    "name": "$PROJECT_NAME",
    "description": "Auto-generated project from intake pipeline",
    "version": "0.1.0"
  },
  "models": {
    "main": "claude-3-5-sonnet-20241022",
    "research": "claude-3-5-sonnet-20241022",
    "fallback": "claude-3-5-sonnet-20241022"
  },
  "parameters": {
    "maxTokens": 8000,
    "temperature": 0.7
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

# Configure models
echo "🤖 Configuring AI models..."
task-master models --set-main "$MODEL"
task-master models --set-research "$MODEL"  # Use same model for research to avoid Perplexity requirement
task-master models --set-fallback "claude-3-5-sonnet-20241022"

# Parse PRD
echo "📄 Parsing PRD to generate tasks..."
task-master parse-prd \
    --input ".taskmaster/docs/prd.txt" \
    --force || {
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

# Expand tasks if requested
if [ "$EXPAND_TASKS" = "true" ]; then
    echo "🌳 Expanding tasks with subtasks..."
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

        # Run Claude to review and update tasks
        echo "🔍 Running Claude review..."
        claude -p --output-format stream-json --verbose --model "$MODEL" /tmp/review-prompt.md || {
            echo "⚠️ Claude review failed, but continuing..."
        }
        
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
git commit -m "🚀 Initialize project: $PROJECT_NAME

Automated project intake:
- Parsed PRD and architecture documents
- Generated TaskMaster tasks
- Created project structure
- Model: $MODEL
- Tasks: $NUM_TASKS targets
$([ "$EXPAND_TASKS" = "true" ] && echo "- Expanded with subtasks")
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- Complexity analysis performed")
"

# Create branch and push
BRANCH_NAME="intake/$PROJECT_NAME-$(date +%Y%m%d-%H%M%S)"
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
