# Agent Tool Verification Guide

## Overview

This guide provides instructions for agents to verify their MCP tools are available and working correctly during initialization.

## Verification Methods by CLI

### Claude Code CLI

Claude Code automatically lists available MCP servers and tools during initialization. Check the logs for:

```
✓ MCP servers initialized:
  - context7 (2 tools)
  - toolman (78 tools)
```

**No additional verification needed** - Claude will fail if tools aren't available.

### Factory CLI

Factory doesn't have a built-in tool listing command, but tools are documented in the agent's memory file.

**Verification Steps:**

1. **Check Memory File**
   ```bash
   cat agents.md | grep -A 20 "Toolman Tools"
   ```
   This shows all tools that should be available.

2. **Test Critical Tools**
   
   Add this to Factory agent initialization:
   ```bash
   # Test Context7 availability
   echo "🔍 Verifying Context7 tools..."
   if command -v npx >/dev/null 2>&1; then
       echo "✓ npx available"
       # Context7 should be spawnable
       timeout 5 npx -y @upstash/context7-mcp --help 2>&1 | head -1 || echo "⚠️ Context7 may not be available"
   fi
   
   # Test environment variables
   echo "🔍 Verifying environment variables..."
   if [ -n "${CONTEXT7_API_KEY:-}" ]; then
       echo "✓ CONTEXT7_API_KEY is set"
   else
       echo "❌ CONTEXT7_API_KEY is missing"
       exit 1
   fi
   ```

3. **Verify Tool Access in Prompt**
   
   Include in agent's initial prompt:
   ```
   Before starting the task, verify your tools are available:
   
   1. Check if you can access Context7:
      - Try: resolve-library-id with libraryName "tokio"
      - If it fails, report the error and exit
   
   2. Check if you can access GitHub tools:
      - Try: github_get_file_contents for a known file
      - If it fails, report the error and exit
   
   3. Only proceed with the task if all required tools are accessible
   ```

### Cursor CLI

Cursor CLI has tool verification built-in. Use the `/tools` command:

```bash
cursor agent
> /tools
```

This lists all available tools.

### Codex CLI

Codex lists tools in its configuration output. Check the startup logs for tool availability.

## Tool Verification Script for Factory

Add this to Factory container initialization scripts:

```bash
#!/bin/bash
# Tool Verification for Factory Agents

echo "════════════════════════════════════════════════════════════════"
echo "║                    TOOL VERIFICATION                          ║"
echo "════════════════════════════════════════════════════════════════"

VERIFICATION_FAILED=false

# Function to test tool availability
test_tool() {
    local tool_name="$1"
    local test_command="$2"
    
    echo -n "Testing $tool_name... "
    if eval "$test_command" >/dev/null 2>&1; then
        echo "✓"
    else
        echo "❌ FAILED"
        VERIFICATION_FAILED=true
    fi
}

# Test Context7
if [ -n "${CONTEXT7_API_KEY:-}" ]; then
    echo "✓ CONTEXT7_API_KEY is set"
    test_tool "Context7 package" "timeout 5 npx -y @upstash/context7-mcp --help"
else
    echo "❌ CONTEXT7_API_KEY is missing"
    VERIFICATION_FAILED=true
fi

# Test Toolman
if [ -n "${TOOLMAN_SERVER_URL:-}" ]; then
    echo "✓ TOOLMAN_SERVER_URL is set: ${TOOLMAN_SERVER_URL}"
    test_tool "Toolman connectivity" "timeout 5 curl -s ${TOOLMAN_SERVER_URL}/health"
else
    echo "⚠️ TOOLMAN_SERVER_URL not set (may not be required)"
fi

# Test GitHub CLI (if needed)
if command -v gh >/dev/null 2>&1; then
    echo "✓ GitHub CLI available"
    test_tool "GitHub authentication" "gh auth status"
else
    echo "ℹ️ GitHub CLI not available (may not be required)"
fi

# Test npm/npx
test_tool "npm/npx" "command -v npx"

# Test git
test_tool "git" "command -v git"

echo "════════════════════════════════════════════════════════════════"

if [ "$VERIFICATION_FAILED" = true ]; then
    echo "❌ Tool verification failed. Cannot proceed with task."
    echo "Please check the errors above and ensure all required tools are configured."
    exit 1
else
    echo "✅ All required tools verified successfully"
fi

echo "════════════════════════════════════════════════════════════════"
echo ""
```

## Agent-Specific Tool Requirements

### Rex (Rust Implementation)

**Critical Tools:**
- Context7 (local): `resolve-library-id`, `get-library-docs`
- GitHub (remote): `create_pull_request`, `push_files`, `create_branch`
- Brave Search (remote): `brave_web_search`

**Verification:**
```bash
# Test Context7
resolve-library-id with libraryName "tokio"

# Test GitHub
github_get_file_contents for README.md

# If either fails, exit with error
```

### Blaze (Frontend)

**Critical Tools:**
- Context7 (local): `resolve-library-id`, `get-library-docs`
- shadcn (remote): `list_components`, `get_component`
- GitHub (remote): `create_pull_request`, `push_files`

**Verification:**
```bash
# Test Context7
resolve-library-id with libraryName "react"

# Test shadcn
shadcn_list_components

# If either fails, exit with error
```

### Tess (QA/Testing)

**Critical Tools:**
- Context7 (local): `resolve-library-id`, `get-library-docs`
- Kubernetes (remote): All 18 tools
- GitHub (remote): `get_pull_request`, `create_pull_request_review`

**Verification:**
```bash
# Test Context7
resolve-library-id with libraryName "pytest"

# Test Kubernetes
kubernetes_listResources with Kind "Pod"

# If either fails, exit with error
```

### Cleo (Code Quality)

**Critical Tools:**
- Context7 (local): `resolve-library-id`, `get-library-docs`
- GitHub (remote): `get_pull_request`, `create_pull_request_review`

**Verification:**
```bash
# Test Context7
resolve-library-id with libraryName "clippy"

# Test GitHub
github_get_pull_request for current PR

# If either fails, exit with error
```

## Integration with Container Scripts

### For Factory Containers

Add to `container-base.sh.hbs`:

```bash
# After environment setup, before main execution
{{> verify-tools}}
```

Create partial template: `verify-tools.hbs`

```bash
echo "🔍 Verifying tool availability..."

# Test Context7 if configured
{{#if (has_local_server "context7")}}
if [ -z "${CONTEXT7_API_KEY:-}" ]; then
    echo "❌ CONTEXT7_API_KEY not set but Context7 is configured"
    exit 1
fi
echo "✓ Context7 configured with API key"
{{/if}}

# Test Toolman if configured
{{#if remote_tools}}
if [ -z "${TOOLMAN_SERVER_URL:-}" ]; then
    echo "❌ TOOLMAN_SERVER_URL not set but remote tools are configured"
    exit 1
fi
echo "✓ Toolman URL configured: ${TOOLMAN_SERVER_URL}"
{{/if}}

echo "✅ Tool verification complete"
```

### For Claude Containers

Claude automatically verifies tools during MCP initialization. No additional verification needed.

### For Cursor Containers

Add to container initialization:

```bash
# Verify MCP configuration
if [ -f "$MCP_CONFIG_PATH" ]; then
    echo "✓ MCP config found: $MCP_CONFIG_PATH"
    jq -e '.mcpServers' "$MCP_CONFIG_PATH" >/dev/null 2>&1 || {
        echo "❌ Invalid MCP config"
        exit 1
    }
else
    echo "❌ MCP config not found: $MCP_CONFIG_PATH"
    exit 1
fi
```

## Runtime Tool Verification

### For All Agents

Add this verification step in the agent's initial prompt:

```markdown
## Tool Verification (CRITICAL - Do this first!)

Before starting your task, verify your critical tools are accessible:

1. **Context7 Test:**
   ```
   Try to resolve a library relevant to your work:
   - Rex: resolve-library-id({ libraryName: "tokio" })
   - Blaze: resolve-library-id({ libraryName: "react" })
   - Cleo: resolve-library-id({ libraryName: "clippy" })
   
   If this fails, STOP and report:
   "❌ Context7 tools not available. Cannot proceed without documentation access."
   ```

2. **GitHub Test:**
   ```
   Try to get repository information:
   - github_get_file_contents for README.md
   
   If this fails, STOP and report:
   "❌ GitHub tools not available. Cannot proceed without repository access."
   ```

3. **Only proceed if all tests pass:**
   ```
   ✅ All tools verified. Proceeding with task...
   ```
```

## Failure Handling

If tool verification fails:

1. **Log the specific error**
   ```
   echo "❌ Tool verification failed: Context7 not available"
   echo "Error: Tool resolve-library-id returned: <error message>"
   ```

2. **Exit with clear status**
   ```
   exit 1
   ```

3. **Report in workflow logs**
   
   The workflow should capture the exit code and report tool verification failures clearly.

## Example: Factory Agent with Verification

```bash
#!/bin/bash
set -euo pipefail

echo "🤖 Starting Rex Implementation Agent"

# Environment verification
echo "🔍 Verifying environment..."
[ -n "${CONTEXT7_API_KEY:-}" ] || { echo "❌ CONTEXT7_API_KEY missing"; exit 1; }
[ -n "${GITHUB_TOKEN:-}" ] || { echo "❌ GITHUB_TOKEN missing"; exit 1; }
echo "✓ Environment variables set"

# Tool availability verification
echo "🔍 Verifying tool availability..."
timeout 5 npx -y @upstash/context7-mcp --help >/dev/null 2>&1 || {
    echo "❌ Context7 not available"
    exit 1
}
echo "✓ Context7 available"

# Start Factory CLI
echo "🚀 Starting Factory CLI..."
factory-cli --config factory-cli.json

# Factory will verify tools at runtime through MCP
# If tools fail, Factory will report errors in its output
```

## Summary

**For Claude:** Automatic verification ✅  
**For Factory:** Add verification script to container initialization  
**For Cursor:** Built-in tool listing  
**For Codex:** Startup logs show tool availability  

**Critical:** All agents should verify Context7 and GitHub tools before starting work, and **fail fast** if tools aren't available.

---

**Next Steps:**
1. Add verification script to Factory container templates
2. Add verification prompts to agent system prompts
3. Test with a simple task to ensure verification works
4. Monitor workflow logs for tool verification failures

