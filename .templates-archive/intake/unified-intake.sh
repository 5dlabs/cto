#!/bin/bash
# =============================================================================
# Unified Intake Script - PRD Parsing + Documentation Generation
# =============================================================================
# This script combines the intake (PRD parsing, TaskMaster setup) and docs
# generation workflows into a single operation.
#
# Uses the Rust `tasks` crate for task management (TaskMaster-compatible).
#
# Phases:
#   1. Repository setup and GitHub authentication
#   2. Task initialization and PRD parsing (using tasks crate)
#   3. Context enrichment via Firecrawl (optional)
#   4. Documentation generation via Claude
#   5. Single PR creation with complete project structure
# =============================================================================

set -e

# Force output to be unbuffered
exec 2>&1

# =============================================================================
# Error Handling - Prevent Silent Failures
# =============================================================================
INTAKE_ERROR_DIR="${WORKSPACE_PVC:-/workspace}/intake-errors"
INTAKE_POD_NAME="${POD_NAME:-$(hostname)}"
INTAKE_START_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)

log_error_to_pvc() {
    local exit_code="$1"
    local line_no="$2"
    local command="$3"
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    mkdir -p "$INTAKE_ERROR_DIR" 2>/dev/null || true

    local error_file="$INTAKE_ERROR_DIR/error-${INTAKE_POD_NAME}-$(date +%s).log"
    {
        echo "═══════════════════════════════════════════════════════════════"
        echo "INTAKE ERROR REPORT"
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
        echo "═══════════════════════════════════════════════════════════════"
    } > "$error_file" 2>/dev/null || true

    echo "📝 Error logged to: $error_file" >&2
}

trap_handler() {
    local exit_code=$?
    local line_no=$1
    local command="$BASH_COMMAND"

    echo "" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2
    echo "❌ INTAKE FAILURE DETECTED" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2
    echo "❌ Exit code: $exit_code" >&2
    echo "❌ Line: $line_no" >&2
    echo "❌ Command: $command" >&2
    echo "❌ ═══════════════════════════════════════════════════════════════" >&2

    log_error_to_pvc "$exit_code" "$line_no" "$command"
    sync 2>/dev/null || true
    sleep 1

    exit "$exit_code"
}

trap 'trap_handler $LINENO' ERR

echo "🚀 Starting Unified Intake Process (tasks crate)"
echo "================================================="
echo "📍 Script version: unified-intake v2.0.0 (Rust)"
echo "📅 Timestamp: $INTAKE_START_TIME"
echo "📦 Pod: $INTAKE_POD_NAME"

# =============================================================================
# Phase 1: Configuration and Environment Setup
# =============================================================================
echo ""
echo "📋 Phase 1: Configuration and Environment Setup"
echo "================================================"

CONFIG_FILE="/intake-files/config.json"
PRD_FILE="/intake-files/prd.txt"
ARCH_FILE="/intake-files/architecture.md"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found at $CONFIG_FILE"
    exit 1
fi

echo "📄 Loading configuration..."
PROJECT_NAME=$(jq -r '.project_name' "$CONFIG_FILE")
REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE")
GITHUB_APP=$(jq -r '.github_app' "$CONFIG_FILE")

# Model configuration
PRIMARY_MODEL=$(jq -r '.primary_model' "$CONFIG_FILE")
PRIMARY_PROVIDER=$(jq -r '.primary_provider' "$CONFIG_FILE")
RESEARCH_MODEL=$(jq -r '.research_model' "$CONFIG_FILE")
RESEARCH_PROVIDER=$(jq -r '.research_provider' "$CONFIG_FILE")
FALLBACK_MODEL=$(jq -r '.fallback_model' "$CONFIG_FILE")
FALLBACK_PROVIDER=$(jq -r '.fallback_provider' "$CONFIG_FILE")

# Unified intake parameters
DOCS_MODEL=$(jq -r '.docs_model // .primary_model' "$CONFIG_FILE")
ENRICH_CONTEXT=$(jq -r '.enrich_context // true' "$CONFIG_FILE")
INCLUDE_CODEBASE=$(jq -r '.include_codebase // false' "$CONFIG_FILE")

# Task generation parameters
NUM_TASKS=$(jq -r '.num_tasks // 50' "$CONFIG_FILE")
EXPAND_TASKS=$(jq -r '.expand_tasks // true' "$CONFIG_FILE")
ANALYZE_COMPLEXITY=$(jq -r '.analyze_complexity // true' "$CONFIG_FILE")

echo "  ✓ Project: $PROJECT_NAME"
echo "  ✓ Repository: $REPOSITORY_URL"
echo "  ✓ GitHub App: $GITHUB_APP"
echo "  ✓ Primary Model: $PRIMARY_MODEL ($PRIMARY_PROVIDER)"
echo "  ✓ Research Model: $RESEARCH_MODEL ($RESEARCH_PROVIDER)"
echo "  ✓ Docs Model: $DOCS_MODEL"
echo "  ✓ Context Enrichment: $ENRICH_CONTEXT"

# Disable interactive Git prompts
export GIT_TERMINAL_PROMPT=0
export GIT_ASKPASS=/bin/true
export SSH_ASKPASS=/bin/true

# =============================================================================
# Phase 1.1: GitHub App Authentication
# =============================================================================
echo ""
echo "🔐 Setting up GitHub App authentication..."

generate_github_token() {
    echo "Generating fresh GitHub App token..."
    
    if [ -z "$GITHUB_APP_PRIVATE_KEY" ] || [ -z "$GITHUB_APP_ID" ]; then
        echo "❌ GITHUB_APP_PRIVATE_KEY or GITHUB_APP_ID not found"
        return 1
    fi
    
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
    
    INSTALLATION_RESPONSE=$(curl -s -L --retry 5 --retry-delay 2 \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/installation")
    
    INSTALLATION_ID=$(echo "$INSTALLATION_RESPONSE" | jq -r '.id')
    
    if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
        ORG_RESPONSE=$(curl -s -L --retry 5 --retry-delay 2 \
            -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github+json" \
            "https://api.github.com/orgs/$REPO_OWNER/installation")
        INSTALLATION_ID=$(echo "$ORG_RESPONSE" | jq -r '.id')
    fi
    
    if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
        echo "❌ Failed to get installation ID"
        rm -f "$TEMP_KEY_FILE"
        return 1
    fi
    
    GITHUB_TOKEN=$(curl -s -L --retry 5 --retry-delay 2 -X POST \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/app/installations/$INSTALLATION_ID/access_tokens" | jq -r '.token')
    
    rm -f "$TEMP_KEY_FILE"
    
    if [ "$GITHUB_TOKEN" = "null" ] || [ -z "$GITHUB_TOKEN" ]; then
        echo "❌ Failed to generate GitHub token"
        return 1
    fi
    
    export GITHUB_TOKEN
    export GH_TOKEN="$GITHUB_TOKEN"
    export TOKEN_GENERATED_AT=$(date +%s)
    
    git config --global --replace-all credential.helper store
    echo "https://x-access-token:${GITHUB_TOKEN}@github.com" > ~/.git-credentials
    
    echo "$GITHUB_TOKEN" | timeout 10 gh auth login --with-token 2>/dev/null || true
    
    echo "✅ GitHub authentication configured"
    return 0
}

if [ -n "$GITHUB_APP_PRIVATE_KEY" ] && [ -n "$GITHUB_APP_ID" ]; then
    generate_github_token || exit 1
else
    echo "⚠️ GitHub App credentials not found"
    exit 1
fi

# =============================================================================
# Phase 2: Repository Clone and Tasks Setup (using Rust tasks crate)
# =============================================================================
echo ""
echo "📦 Phase 2: Repository Clone and Tasks Setup"
echo "=============================================="

CLONE_DIR="/tmp/repo-$(date +%s)"
echo "📂 Cloning repository to: $CLONE_DIR"
git clone "$REPOSITORY_URL" "$CLONE_DIR" || {
    echo "❌ Git clone failed"
    exit 1
}
cd "$CLONE_DIR"

git config user.name "Unified Intake Bot"
git config user.email "intake@5dlabs.com"

PROJECT_DIR_NAME=$(echo "$PROJECT_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g' | sed 's/--*/-/g' | sed 's/^-*//;s/-*$//')
PROJECT_DIR="$CLONE_DIR/$PROJECT_DIR_NAME"

mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

# Initialize Tasks project using Rust tasks crate
echo "🚀 Initializing Tasks project..."
tasks init --yes || {
    echo "⚠️ Tasks init failed, creating structure manually..."
    mkdir -p .taskmaster/docs .taskmaster/tasks .taskmaster/reports
}

# Copy PRD and architecture
mkdir -p .taskmaster/docs
cp "$PRD_FILE" ".taskmaster/docs/prd.txt"
if [ -f "$ARCH_FILE" ] && [ -s "$ARCH_FILE" ]; then
    cp "$ARCH_FILE" ".taskmaster/docs/architecture.md"
fi

# Configure model for tasks crate
# The tasks crate reads ANTHROPIC_API_KEY and OPENAI_API_KEY from environment
# Model selection is done via --model flag

echo "✅ Tasks project initialized"

# =============================================================================
# Phase 2.1: Parse PRD and Generate Tasks (using tasks crate)
# =============================================================================
echo ""
echo "📄 Parsing PRD to generate tasks..."
echo "  → Provider: $PRIMARY_PROVIDER"
echo "  → Model: $PRIMARY_MODEL"
echo "  → Num Tasks: $NUM_TASKS"
echo "  → This may take several minutes..."
echo ""

# Start a background progress indicator
(
    i=0
    while true; do
        i=$((i + 1))
        echo "  ⏳ Tasks running... (${i}m elapsed)"
        sleep 60
    done
) &
PROGRESS_PID=$!

# Run parse-prd using Rust tasks crate
# Map provider to model flag
TASKS_MODEL=""
if [ "$PRIMARY_PROVIDER" = "anthropic" ]; then
    TASKS_MODEL="$PRIMARY_MODEL"
elif [ "$PRIMARY_PROVIDER" = "openai" ]; then
    TASKS_MODEL="$PRIMARY_MODEL"
fi

tasks parse-prd ".taskmaster/docs/prd.txt" \
    --num-tasks "$NUM_TASKS" \
    --research \
    ${TASKS_MODEL:+--model "$TASKS_MODEL"} 2>&1 || {
    kill $PROGRESS_PID 2>/dev/null || true
    echo "❌ Failed to parse PRD"
    exit 1
}

kill $PROGRESS_PID 2>/dev/null || true

# Resolve tasks file path (tasks crate uses .taskmaster/tasks/tasks.json)
TASKS_FILE=".taskmaster/tasks/tasks.json"
if [ ! -f "$TASKS_FILE" ]; then
    TASKS_FILE=$(find .taskmaster -maxdepth 2 -name tasks.json | head -n 1)
fi

if [ ! -f "$TASKS_FILE" ]; then
    echo "❌ tasks.json not found after parsing"
    exit 1
fi

echo "✅ Tasks generated: $TASKS_FILE"

# Analyze complexity if requested
if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    echo "🔍 Analyzing task complexity..."
    mkdir -p .taskmaster/reports
    tasks analyze-complexity \
        ${TASKS_MODEL:+--model "$TASKS_MODEL"} || echo "⚠️ Complexity analysis failed"
fi

# Expand tasks if requested
if [ "$EXPAND_TASKS" = "true" ]; then
    echo "🌳 Expanding tasks with subtasks..."
    tasks expand-all --force \
        ${TASKS_MODEL:+--model "$TASKS_MODEL"} || echo "⚠️ Task expansion failed"
fi

# Add agent hints based on task content
echo "🎯 Adding agent routing hints..."
jq '
  def is_frontend_task:
    (.title + " " + (.description // "") + " " + (.details // "")) 
    | test("frontend|react|component|ui|interface|styling|css|html|jsx|tsx"; "i");
  
  def is_integration_task:
    (.title + " " + (.description // "") + " " + (.details // "")) 
    | test("test|testing|integration|e2e|end.to.end|qa|quality"; "i");
  
  if .tasks then
    .tasks |= map(
      if .agentHint then .
      elif is_frontend_task then . + {"agentHint": "frontend"}
      elif is_integration_task then . + {"agentHint": "integration"}
      else . end)
  else . end
' "$TASKS_FILE" > "$TASKS_FILE.tmp" && mv "$TASKS_FILE.tmp" "$TASKS_FILE"

echo "✅ Agent hints added"

# Generate individual task files (tasks crate generate command)
echo "📝 Generating individual task files..."
tasks generate

# =============================================================================
# Phase 3: Context Enrichment via Firecrawl (Optional)
# =============================================================================
if [ "$ENRICH_CONTEXT" = "true" ]; then
    echo ""
    echo "🔗 Phase 3: Context Enrichment via Firecrawl"
    echo "============================================="
    
    URLS=$(grep -oP 'https?://[^\s<>"]+' "$PRD_FILE" 2>/dev/null | sort -u | head -10)
    
    if [ -n "$URLS" ]; then
        echo "📋 Found URLs in PRD:"
        echo "$URLS" | head -5
        
        CONTEXT_FILE=".taskmaster/docs/enriched-context.md"
        echo "# Enriched Context from PRD References" > "$CONTEXT_FILE"
        echo "" >> "$CONTEXT_FILE"
        echo "This context was automatically extracted from URLs referenced in the PRD." >> "$CONTEXT_FILE"
        echo "Generated at: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$CONTEXT_FILE"
        echo "" >> "$CONTEXT_FILE"
        
        echo "## Referenced URLs" >> "$CONTEXT_FILE"
        echo "" >> "$CONTEXT_FILE"
        for url in $URLS; do
            echo "- $url" >> "$CONTEXT_FILE"
        done
        
        echo "✅ Context enrichment file created"
    else
        echo "ℹ️ No URLs found in PRD, skipping context enrichment"
    fi
else
    echo ""
    echo "ℹ️ Context enrichment disabled"
fi

# =============================================================================
# Phase 4: Documentation Generation
# =============================================================================
echo ""
echo "📚 Phase 4: Documentation Generation"
echo "====================================="

TASK_COUNT=$(jq '.tasks | length' "$TASKS_FILE")
echo "📋 Processing $TASK_COUNT tasks..."

jq -c '.tasks[]' "$TASKS_FILE" | while IFS= read -r task_json; do
    task_id=$(echo "$task_json" | jq -r '.id')
    title=$(echo "$task_json" | jq -r '.title // "No Title"')
    description=$(echo "$task_json" | jq -r '.description // ""')
    details=$(echo "$task_json" | jq -r '.details // ""')
    test_strategy=$(echo "$task_json" | jq -r '.testStrategy // ""')
    priority=$(echo "$task_json" | jq -r '.priority // "medium"')
    dependencies=$(echo "$task_json" | jq -r '.dependencies // [] | join(", ")')
    
    if [ -z "$task_id" ] || [ "$task_id" = "null" ]; then
        continue
    fi
    
    echo "  📝 Generating docs for Task $task_id: $title"
    
    task_dir=".taskmaster/docs/task-$task_id"
    mkdir -p "$task_dir"
    
    # Generate task.md
    cat > "$task_dir/task.md" << TASK_MD
# Task $task_id: $title

## Overview
$description

## Priority
$priority

## Dependencies
${dependencies:-None}

## Implementation Details
$details

## Test Strategy
$test_strategy
TASK_MD

    # Infer role from task
    inferred_role="Senior Software Engineer"
    if echo "$title $description" | grep -qiE 'frontend|ui|react|component|css|tailwind|design'; then
        inferred_role="Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX"
    elif echo "$title $description" | grep -qiE 'backend|api|server|database|rust|postgres'; then
        inferred_role="Senior Backend Engineer with expertise in Rust, APIs, and database systems"
    elif echo "$title $description" | grep -qiE 'devops|deploy|kubernetes|helm|infra|ci/cd'; then
        inferred_role="Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD"
    elif echo "$title $description" | grep -qiE 'test|qa|quality|validation'; then
        inferred_role="Senior QA Engineer with expertise in test automation and quality assurance"
    elif echo "$title $description" | grep -qiE 'security|auth|encryption|oauth'; then
        inferred_role="Senior Security Engineer with expertise in authentication and secure coding"
    fi

    # Generate prompt.md
    cat > "$task_dir/prompt.md" << PROMPT_MD
# Implementation Prompt for Task $task_id

## Role (Persona)
You are a $inferred_role. Your primary responsibility is implementing Task $task_id with production-quality code that follows best practices and project conventions.

## Context (Background)
**Task Priority:** $priority
**Dependencies:** ${dependencies:-None (this task can start immediately)}
**Scope:** $description

This task is part of a larger project. Review the PRD and architecture documents in \`.taskmaster/docs/\` to understand the full context before implementing.

## Task (Specific Instructions)
**Objective:** $title

### Requirements
$details

### Constraints & Formatting
- **Code Style:** Match existing codebase patterns and conventions
- **Output Format:** Pull request with clear, atomic commits
- **Documentation:** Update relevant docs and inline comments
- **Testing:** Include unit tests for new functionality
- **PR Title Format:** \`feat(task-$task_id): $title\`

## Acceptance Criteria
$test_strategy

## Self-Critique Checklist
Before submitting your PR, verify each item:

- [ ] **Functionality:** Does the implementation meet all requirements?
- [ ] **Edge Cases:** Are boundary conditions and error states handled?
- [ ] **Security:** Are there any potential security vulnerabilities?
- [ ] **Performance:** Is the solution efficient for expected scale?
- [ ] **Maintainability:** Is the code readable and well-documented?
- [ ] **Testing:** Do all tests pass? Is coverage adequate?
- [ ] **Conventions:** Does the code follow project style guidelines?
PROMPT_MD

    # Generate acceptance-criteria.md
    cat > "$task_dir/acceptance-criteria.md" << AC_MD
# Acceptance Criteria for Task $task_id

## Task
$title

## Criteria

### Functional Requirements
$details

### Testing Requirements
$test_strategy

### Definition of Done
- [ ] All functional requirements implemented
- [ ] Tests written and passing
- [ ] Code reviewed and approved
- [ ] Documentation updated
AC_MD

done

echo "✅ Documentation generated for all tasks"

# =============================================================================
# Phase 5: Create Pull Request
# =============================================================================
echo ""
echo "🔀 Phase 5: Creating Pull Request"
echo "=================================="

cd "$CLONE_DIR"

BRANCH_NAME="intake-${PROJECT_DIR_NAME}-$(date +%Y%m%d-%H%M%S)"
echo "🌿 Creating branch: $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"

git add -A

COMMIT_MSG="feat: unified intake for $PROJECT_NAME

- Parsed PRD and generated tasks (using tasks crate)
- Created comprehensive documentation
- Added agent routing hints
- Generated individual task files

🤖 Auto-generated by unified intake workflow (Rust)
- Model: $PRIMARY_MODEL
- Tasks: $(jq '.tasks | length' "$PROJECT_DIR/$TASKS_FILE") generated"

if [ "$EXPAND_TASKS" = "true" ]; then
    COMMIT_MSG="$COMMIT_MSG
- Expanded with subtasks"
fi

if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    COMMIT_MSG="$COMMIT_MSG
- Complexity analysis performed"
fi

if [ "$ENRICH_CONTEXT" = "true" ]; then
    COMMIT_MSG="$COMMIT_MSG
- Context enrichment enabled"
fi

git commit -m "$COMMIT_MSG"

echo "📤 Pushing branch..."
git push -u origin "$BRANCH_NAME"

# Refresh token if needed
if [ -n "$GITHUB_APP_PRIVATE_KEY" ]; then
    NOW=$(date +%s)
    TOKEN_AGE=$((NOW - ${TOKEN_GENERATED_AT:-0}))
    if [ $TOKEN_AGE -gt 3000 ]; then
        generate_github_token
    fi
fi

echo "📝 Creating pull request..."

PR_BODY="## 🎉 Unified Intake: $PROJECT_NAME

This PR contains the complete project structure generated by the unified intake workflow.

### 📋 What was processed:
- ✅ PRD document parsed
$([ -f "$PROJECT_DIR/.taskmaster/docs/architecture.md" ] && echo "- ✅ Architecture document included")
- ✅ Tasks initialized (using Rust tasks crate)
- ✅ Tasks generated ($(jq '.tasks | length' "$PROJECT_DIR/$TASKS_FILE") tasks)
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- ✅ Complexity analysis performed")
$([ "$EXPAND_TASKS" = "true" ] && echo "- ✅ Tasks expanded with subtasks")
$([ "$ENRICH_CONTEXT" = "true" ] && echo "- ✅ Context enrichment enabled")
- ✅ Documentation generated (task.md, prompt.md, acceptance-criteria.md)

### 🏗️ Generated Structure:
\`\`\`
$PROJECT_DIR_NAME/
├── .taskmaster/
│   ├── config.json
│   ├── docs/
│   │   ├── prd.txt
│   │   ├── architecture.md
│   │   └── task-*/
│   │       ├── task.md
│   │       ├── prompt.md
│   │       └── acceptance-criteria.md
│   └── tasks/
│       └── tasks.json
└── README.md
\`\`\`

### 🤖 Configuration:
- **Primary Model**: $PRIMARY_MODEL ($PRIMARY_PROVIDER)
- **Research Model**: $RESEARCH_MODEL ($RESEARCH_PROVIDER)
- **Docs Model**: $DOCS_MODEL
- **Context Enrichment**: $ENRICH_CONTEXT

### 🎯 Next Steps:
1. Review the generated tasks and documentation
2. Merge this PR to add the project
3. Use \`cto play\` to implement tasks

🤖 Auto-generated by unified intake workflow (Rust tasks crate)"

gh pr create \
    --title "🚀 Unified Intake: $PROJECT_NAME" \
    --body "$PR_BODY" \
    --head "$BRANCH_NAME" \
    --base main || {
    echo "⚠️ Failed to create PR, but branch has been pushed"
    echo "Branch: $BRANCH_NAME"
    echo "You can create the PR manually"
}

echo ""
echo "✅ Unified intake complete!"
echo "================================="
echo "Project: $PROJECT_NAME"
echo "Location: $PROJECT_DIR"
echo "Branch: $BRANCH_NAME"
echo "Repository: $REPOSITORY_URL"
echo "Tasks generated: $(jq '.tasks | length' "$PROJECT_DIR/$TASKS_FILE")"

