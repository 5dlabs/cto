# Task 5: Enhance Rex Container for Remediation

## Overview
Create a separate, specialized Rex remediation container script to handle remediation mode with feedback processing and iteration limits. This dedicated container provides focused, targeted fixes rather than broad implementation, optimized specifically for addressing QA feedback while preserving working functionality.

## Technical Context
The existing Rex container is designed for initial feature implementation. Remediation requires a fundamentally different approach:

- **Implementation Mode**: Build features from scratch with broad context
- **Remediation Mode**: Fix specific issues with surgical precision

This task creates a separate remediation container that understands both the original requirements and specific feedback, enabling targeted fixes without breaking working functionality.

### Architecture Integration
- **Container Location**: New script at `infra/images/rex-remediation/` or `container-rex-remediation.sh`
- **Trigger Mechanism**: Activated via `REMEDIATION_MODE=true` environment variable
- **Integration**: Works with existing CodeRun CRD and play workflow
- **Authentication**: Uses existing GitHub App credentials and CLI tools

## Implementation Guide

### Step 1: Create Separate Rex Remediation Container Script

#### 1.1 Container Script Structure
Create a new dedicated remediation script at `infra/images/rex-remediation/container-rex-remediation.sh`:

```bash
#!/bin/bash
# Rex Remediation Container Script - Separate from normal Rex implementation
set -euo pipefail

# Strict mode validation - only for remediation
if [ "$REMEDIATION_MODE" != "true" ]; then
    echo "❌ Error: This script requires REMEDIATION_MODE=true"
    echo "This is the Rex REMEDIATION container, not the implementation container."
    exit 1
fi

# Initialize logging
echo "🔧 REX REMEDIATION MODE - Starting Fix Process"
echo "📊 Iteration: $ITERATION_COUNT/10"
echo "🎯 Task ID: $TASK_ID"
echo "📝 PR Number: $PR_NUMBER"

# Validate required environment variables
required_vars=("TASK_ID" "PR_NUMBER" "FEEDBACK_COMMENT_ID" "ITERATION_COUNT")
for var in "${required_vars[@]}"; do
    if [ -z "${!var:-}" ]; then
        echo "❌ Missing required environment variable: $var"
        exit 1
    fi
done

# Source common functions if available
if [ -f "/usr/local/bin/rex-common.sh" ]; then
    source "/usr/local/bin/rex-common.sh"
fi
```

#### 1.2 Script Directory Structure
```
infra/images/rex-remediation/
├── container-rex-remediation.sh     # Main remediation script
├── Dockerfile                       # Remediation-specific container
├── remediation-functions.sh         # Shared remediation utilities
└── templates/
    └── claude-remediation.md.template # Context template
```

### Step 2: Implement Original Task Context Fetching

#### 2.1 Task Context Retrieval
```bash
# Function: Fetch original task context
fetch_original_task_context() {
    echo "📋 Fetching original task context for Task $TASK_ID..."
    
    local task_context=""
    local task_found=false
    
    # Try multiple sources for task context
    local sources=(
        "/workspace/docs/task-${TASK_ID}.md"
        "/workspace/.taskmaster/docs/task-${TASK_ID}/task.md"
        "/workspace/README-task-${TASK_ID}.md"
    )
    
    for source in "${sources[@]}"; do
        if [ -f "$source" ]; then
            echo "✅ Found task context at: $source"
            task_context=$(cat "$source")
            task_found=true
            break
        fi
    done
    
    # Try Task Master API if files not found
    if [ "$task_found" != true ]; then
        echo "⚠️  Task context files not found, trying Task Master API..."
        if command -v curl >/dev/null 2>&1 && [ -n "${TASK_MASTER_API_URL:-}" ]; then
            task_context=$(curl -s "${TASK_MASTER_API_URL}/tasks/${TASK_ID}" | jq -r '.description // empty' || echo "")
            if [ -n "$task_context" ]; then
                task_found=true
                echo "✅ Retrieved task context from Task Master API"
            fi
        fi
    fi
    
    # Fallback message
    if [ "$task_found" != true ]; then
        echo "⚠️  Warning: Could not retrieve original task context"
        task_context="Original task context not available. Please refer to PR description and requirements."
    fi
    
    # Store for template usage
    export ORIGINAL_TASK_CONTEXT="$task_context"
    echo "📝 Task context retrieved (${#task_context} characters)"
}
```

### Step 3: Add GitHub API Comment Fetching for Feedback

#### 3.1 GitHub API Integration
```bash
# Function: Fetch feedback comment using GitHub CLI
fetch_feedback_comment() {
    echo "💬 Fetching feedback comment ID: $FEEDBACK_COMMENT_ID"
    
    # Validate gh CLI is available and authenticated
    if ! command -v gh >/dev/null 2>&1; then
        echo "❌ Error: GitHub CLI (gh) not found"
        exit 1
    fi
    
    # Test authentication
    if ! gh auth status >/dev/null 2>&1; then
        echo "❌ Error: GitHub CLI not authenticated"
        exit 1
    fi
    
    # Extract repository info from environment or git
    local repo_info
    if [ -n "${GITHUB_REPOSITORY:-}" ]; then
        repo_info="$GITHUB_REPOSITORY"
    else
        repo_info=$(git remote get-url origin | sed 's/.*github\.com[:/]\([^/]*\/[^/]*\)\.git.*/\1/' || echo "")
    fi
    
    if [ -z "$repo_info" ]; then
        echo "❌ Error: Could not determine repository information"
        exit 1
    fi
    
    echo "📍 Repository: $repo_info"
    
    # Fetch comment with retry logic
    local max_retries=3
    local retry_count=0
    local comment_data=""
    
    while [ $retry_count -lt $max_retries ]; do
        echo "🔄 Fetching comment (attempt $((retry_count + 1))/$max_retries)..."
        
        if comment_data=$(gh api "/repos/$repo_info/issues/comments/$FEEDBACK_COMMENT_ID" 2>/dev/null); then
            echo "✅ Successfully fetched feedback comment"
            break
        else
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $max_retries ]; then
                echo "⚠️  Retry in 2 seconds..."
                sleep 2
            fi
        fi
    done
    
    if [ -z "$comment_data" ]; then
        echo "❌ Error: Failed to fetch comment after $max_retries attempts"
        exit 1
    fi
    
    # Extract comment body and metadata
    export FEEDBACK_COMMENT_BODY=$(echo "$comment_data" | jq -r '.body // empty')
    export FEEDBACK_AUTHOR=$(echo "$comment_data" | jq -r '.user.login // empty')
    export FEEDBACK_CREATED_AT=$(echo "$comment_data" | jq -r '.created_at // empty')
    
    echo "👤 Feedback Author: $FEEDBACK_AUTHOR"
    echo "⏰ Created: $FEEDBACK_CREATED_AT"
    echo "📄 Comment Body Length: ${#FEEDBACK_COMMENT_BODY} characters"
}
```

### Step 4: Build Feedback Parser for Remediation Context

#### 4.1 Feedback Processing
```bash
# Function: Parse and extract feedback metadata
parse_feedback_content() {
    echo "🔍 Parsing feedback content for structured information..."
    
    local feedback="$FEEDBACK_COMMENT_BODY"
    
    # Extract severity if present
    local severity="Medium"  # Default
    if echo "$feedback" | grep -qi "severity.*critical\|critical.*severity"; then
        severity="Critical"
    elif echo "$feedback" | grep -qi "severity.*high\|high.*severity"; then
        severity="High"
    elif echo "$feedback" | grep -qi "severity.*low\|low.*severity"; then
        severity="Low"
    fi
    
    # Extract issue type
    local issue_type="General"
    if echo "$feedback" | grep -qi "bug\|error\|broken\|fails\|exception"; then
        issue_type="Bug"
    elif echo "$feedback" | grep -qi "performance\|slow\|optimization"; then
        issue_type="Performance"
    elif echo "$feedback" | grep -qi "security\|vulnerability\|auth"; then
        issue_type="Security"
    elif echo "$feedback" | grep -qi "documentation\|docs\|readme"; then
        issue_type="Documentation"
    fi
    
    # Extract specific issues (look for bullet points, numbered lists)
    local specific_issues=""
    specific_issues=$(echo "$feedback" | grep -E "^[[:space:]]*[-*+]|^[[:space:]]*[0-9]+\." | head -10)
    
    # Count checkboxes for progress tracking
    local total_checkboxes=$(echo "$feedback" | grep -c "- \[ \]\|- \[x\]" || echo "0")
    local completed_checkboxes=$(echo "$feedback" | grep -c "- \[x\]" || echo "0")
    
    # Export parsed data
    export FEEDBACK_SEVERITY="$severity"
    export FEEDBACK_ISSUE_TYPE="$issue_type"
    export FEEDBACK_SPECIFIC_ISSUES="$specific_issues"
    export FEEDBACK_TOTAL_CHECKBOXES="$total_checkboxes"
    export FEEDBACK_COMPLETED_CHECKBOXES="$completed_checkboxes"
    
    echo "📊 Parsed Feedback Summary:"
    echo "   Severity: $severity"
    echo "   Issue Type: $issue_type"
    echo "   Checkboxes: $completed_checkboxes/$total_checkboxes completed"
    echo "   Specific Issues: $(echo "$specific_issues" | wc -l) items"
}
```

### Step 5: Implement Iteration Limit Checking

#### 5.1 Iteration Validation and Escalation
```bash
# Function: Check iteration limits and handle escalation
check_iteration_limits() {
    echo "🔢 Checking iteration limits..."
    
    local max_iterations=10
    local current_iteration="$ITERATION_COUNT"
    
    echo "📈 Current iteration: $current_iteration/$max_iterations"
    
    if [ "$current_iteration" -ge "$max_iterations" ]; then
        echo "🚨 Maximum iterations reached! Escalating..."
        
        # Post escalation comment
        local escalation_comment=$(cat << EOF
## 🚨 Remediation Escalation - Max Iterations Reached

**Task ID**: $TASK_ID  
**Iteration**: $current_iteration/$max_iterations  
**Author**: @$FEEDBACK_AUTHOR

### Summary
Automated remediation has reached the maximum iteration limit without resolving all issues. Human intervention is now required.

### Current Status
- **Total Iterations**: $current_iteration
- **Last Feedback**: $(echo "$FEEDBACK_COMMENT_BODY" | head -3 | tr '\n' ' ')...
- **Escalation Time**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

### Next Steps
- [ ] Manual review by @platform-team or @cto
- [ ] Assess if requirements need clarification
- [ ] Consider if task complexity exceeds automation capabilities
- [ ] Manual implementation or guidance required

**Automation has been disabled for this task.**

cc: @platform-team @cto
EOF
        )
        
        # Post the escalation comment
        if gh pr comment "$PR_NUMBER" --body "$escalation_comment" 2>/dev/null; then
            echo "✅ Escalation comment posted successfully"
        else
            echo "⚠️  Warning: Failed to post escalation comment"
        fi
        
        echo "🛑 Terminating remediation process - manual intervention required"
        exit 1
    fi
    
    # Calculate remaining iterations
    local remaining=$((max_iterations - current_iteration))
    echo "⏳ Remaining iterations: $remaining"
    
    if [ "$remaining" -le 2 ]; then
        echo "⚠️  Warning: Approaching maximum iterations!"
    fi
}
```

### Step 6: Create Remediation-Specific AI Context

#### 6.1 Context Template Generation
```bash
# Function: Generate remediation-specific CLAUDE.md context
generate_remediation_context() {
    echo "📝 Generating remediation-specific AI context..."
    
    local claude_file="/workspace/CLAUDE.md"
    local timestamp=$(date -u '+%Y-%m-%d %H:%M:%S UTC')
    
    cat > "$claude_file" << EOF
# REMEDIATION MODE - Fix Required Issues

**Mode**: Remediation (Iteration $ITERATION_COUNT/10)  
**Task ID**: $TASK_ID  
**PR Number**: $PR_NUMBER  
**Generated**: $timestamp

## 🎯 Your Mission
You are in REMEDIATION mode. Your task is to **FIX SPECIFIC ISSUES** while preserving all working functionality. This is NOT a reimplementation - make targeted, surgical fixes only.

## 📋 Original Task Requirements
\`\`\`
$ORIGINAL_TASK_CONTEXT
\`\`\`

## 🔴 Issues to Fix (Priority: $FEEDBACK_SEVERITY)

**Feedback Author**: @$FEEDBACK_AUTHOR  
**Issue Type**: $FEEDBACK_ISSUE_TYPE  
**Created**: $FEEDBACK_CREATED_AT

### Feedback Details:
\`\`\`
$FEEDBACK_COMMENT_BODY
\`\`\`

### Specific Issues Identified:
$FEEDBACK_SPECIFIC_ISSUES

## 🛠️ Remediation Instructions

### Primary Objectives:
1. **Address ALL issues** mentioned in the feedback above
2. **Preserve working functionality** - don't break what already works
3. **Make targeted fixes** - avoid broad reimplementation
4. **Focus on the specific problems** identified by the reviewer
5. **Test your changes** to ensure issues are resolved

### Critical Guidelines:
- ✅ **DO**: Make surgical, targeted fixes to address specific issues
- ✅ **DO**: Preserve existing functionality that works correctly
- ✅ **DO**: Add or fix missing functionality mentioned in feedback
- ✅ **DO**: Improve code quality issues specifically called out
- ❌ **DON'T**: Reimplement features from scratch unless explicitly required
- ❌ **DON'T**: Change working code that isn't mentioned in the feedback
- ❌ **DON'T**: Add new features not requested in the feedback
- ❌ **DON'T**: Ignore any of the issues mentioned in the feedback

### Quality Standards:
- Code must pass all existing tests
- New functionality must include appropriate tests
- Documentation updates if interfaces change
- Follow existing code patterns and conventions

### Success Criteria:
You succeed when ALL feedback issues are addressed while maintaining existing functionality. The goal is a targeted fix, not a reimplementation.

---
**Iteration Context**: This is iteration $ITERATION_COUNT of $max_iterations. Focus on resolving the specific issues efficiently.
EOF

    echo "✅ Remediation context generated at: $claude_file"
    echo "📏 Context size: $(wc -c < "$claude_file") characters"
}
```

### Step 7: Configure Container Image and Deployment

#### 7.1 Dockerfile for Remediation Container
```dockerfile
# infra/images/rex-remediation/Dockerfile
FROM ubuntu:22.04

# Install required tools
RUN apt-get update && apt-get install -y \
    bash \
    curl \
    git \
    jq \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install -y gh

# Install Claude runner (placeholder - adjust based on actual installation)
COPY claude-runner /usr/local/bin/claude-runner
RUN chmod +x /usr/local/bin/claude-runner

# Copy remediation scripts
COPY container-rex-remediation.sh /usr/local/bin/container-rex-remediation.sh
COPY remediation-functions.sh /usr/local/bin/remediation-functions.sh
RUN chmod +x /usr/local/bin/container-rex-remediation.sh
RUN chmod +x /usr/local/bin/remediation-functions.sh

# Set working directory
WORKDIR /workspace

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/container-rex-remediation.sh"]
```

#### 7.2 Main Script Integration
```bash
# Complete main script flow
main() {
    echo "🚀 Starting Rex Remediation Process"
    
    # Step 1: Validate environment and check limits
    check_iteration_limits
    
    # Step 2: Fetch original task context
    fetch_original_task_context
    
    # Step 3: Fetch feedback comment
    fetch_feedback_comment
    
    # Step 4: Parse feedback content
    parse_feedback_content
    
    # Step 5: Generate AI context
    generate_remediation_context
    
    # Step 6: Execute Claude runner
    echo "🤖 Invoking Claude with remediation context..."
    exec /usr/local/bin/claude-runner
}

# Execute main function
main "$@"
```

### Step 8: Test Remediation Container End-to-End

#### 8.1 Testing Framework
Create comprehensive test suite at `infra/images/rex-remediation/tests/`:

```bash
#!/bin/bash
# test-remediation-container.sh

set -euo pipefail

# Test configuration
TEST_TASK_ID="test-42"
TEST_PR_NUMBER="123"
TEST_COMMENT_ID="456789"
TEST_ITERATION="3"

echo "🧪 Testing Rex Remediation Container"

test_environment_validation() {
    echo "Testing environment validation..."
    
    # Test missing REMEDIATION_MODE
    if ./container-rex-remediation.sh 2>&1 | grep -q "requires REMEDIATION_MODE=true"; then
        echo "✅ Environment validation works correctly"
    else
        echo "❌ Environment validation failed"
        exit 1
    fi
}

test_task_context_fetching() {
    echo "Testing task context fetching..."
    
    # Create test task file
    mkdir -p "/tmp/test-workspace/docs"
    echo "Test task description for Task $TEST_TASK_ID" > "/tmp/test-workspace/docs/task-$TEST_TASK_ID.md"
    
    # Test context retrieval
    export TASK_ID="$TEST_TASK_ID"
    if fetch_original_task_context && [ -n "$ORIGINAL_TASK_CONTEXT" ]; then
        echo "✅ Task context fetching works"
    else
        echo "❌ Task context fetching failed"
        exit 1
    fi
}

test_github_api_integration() {
    echo "Testing GitHub API integration..."
    
    # Mock gh command for testing
    cat > /tmp/mock-gh << 'EOF'
#!/bin/bash
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    exit 0
elif [ "$1" = "api" ]; then
    echo '{"body": "🔴 Required Changes\n\nThis is test feedback", "user": {"login": "test-user"}, "created_at": "2023-01-01T00:00:00Z"}'
    exit 0
fi
EOF
    chmod +x /tmp/mock-gh
    export PATH="/tmp:$PATH"
    
    # Test comment fetching
    export FEEDBACK_COMMENT_ID="$TEST_COMMENT_ID"
    if fetch_feedback_comment && [ -n "$FEEDBACK_COMMENT_BODY" ]; then
        echo "✅ GitHub API integration works"
    else
        echo "❌ GitHub API integration failed"
        exit 1
    fi
}

# Run all tests
main_test() {
    test_environment_validation
    test_task_context_fetching
    test_github_api_integration
    
    echo "🎉 All remediation container tests passed!"
}

main_test "$@"
```

## Integration Points

### CodeRun CRD Integration
The remediation container integrates with the existing play workflow through:

1. **Environment Variables**: Receives configuration via CodeRun spec
2. **GitHub Authentication**: Uses existing GitHub App credentials
3. **Workspace Access**: Mounts same workspace volume as normal Rex
4. **Output Generation**: Produces fixes that trigger existing sensors

### Sensor Integration
Works with the existing sensor ecosystem:

1. **Trigger**: Activated by remediation sensor detecting feedback
2. **Processing**: Runs independently of implementation agents
3. **Output**: Pushes fixes that trigger implementation-agent-remediation sensor
4. **Monitoring**: Integrates with existing observability stack

## Testing Strategy

### Unit Testing
1. **Script Functions**: Test all bash functions with mocked dependencies
2. **Environment Validation**: Test strict mode enforcement
3. **Context Generation**: Validate template generation with various inputs
4. **Error Handling**: Test failure scenarios and error messages

### Integration Testing
1. **GitHub API**: Test comment fetching with real API
2. **Container Build**: Verify Docker image builds and runs
3. **End-to-End**: Test complete remediation flow with CodeRun
4. **Authentication**: Verify GitHub App integration works

### Performance Testing
1. **Startup Time**: Measure container initialization
2. **Context Generation**: Test with large task descriptions
3. **Memory Usage**: Monitor resource consumption
4. **API Latency**: Test GitHub API response times

## Security Considerations

### Authentication
- Uses existing GitHub App credentials
- No additional secrets required
- Inherits RBAC from existing Rex containers

### Input Validation
- Validates all environment variables
- Sanitizes feedback content before processing
- Prevents code injection in template generation

### Resource Access
- Same workspace volume access as normal Rex
- No additional privileged access required
- Follows principle of least privilege

## Monitoring and Observability

### Logging
- Structured logging with clear prefixes
- Step-by-step progress indicators
- Error context and troubleshooting information

### Metrics
- Container startup success/failure
- Task context retrieval success rate
- GitHub API call success rate
- Iteration limit escalations

### Alerts
- Failed comment retrieval
- Authentication failures
- Iteration limit reached
- Container startup failures

## Success Criteria
- Remediation container activates only in REMEDIATION_MODE
- Successfully fetches original task context from multiple sources
- Reliably retrieves PR feedback via GitHub API
- Generates focused, targeted AI context for fixes
- Enforces iteration limits with proper escalation
- Integrates seamlessly with existing play workflow
- Produces targeted fixes rather than reimplementation
- All tests pass in CI/CD pipeline