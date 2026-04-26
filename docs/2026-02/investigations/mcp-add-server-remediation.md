# Morgan Intake MCP Tools Fix - Implementation Summary

## Problem

Morgan's intake jobs were loading the wrong MCP tools:
- **Loaded**: `pg_aiguide_*`, `openmemory_*`, `better_auth_*`, `solana_*`, `ai_elements_*` (generic platform tools)
- **Needed**: `firecrawl_*` (URL scraping), `context7_*` (library docs), `openmemory_*` (memory)

## Root Causes (TWO ISSUES)

### Issue 1: No Default Agent Tools

The controller's `generate_client_config` function had no fallback for agent-specific default tools. When:
1. No `remoteTools` was specified in the CodeRun spec
2. No agent config existed in Helm values (`agentCliConfigs: {}` was empty)

It fell back to an empty `remoteTools` array.

### Issue 2: Environment Variable Mismatch

The `tools-client` binary (MCP bridge) was looking for:
- `MCP_TOOLS_CONFIG` env var
- `tools-config.json` file

But the controller was setting:
- `MCP_CLIENT_CONFIG` env var
- `client-config.json` file

This mismatch caused the tools-client to not find the config, defaulting to "include all tools".

## Solution

Added built-in default tools based on agent + run_type in the controller. This follows the principle that the **cto-config should define agent tools**, and the controller now has built-in knowledge of what tools each agent needs by default.

### Changes Made

#### 1. Controller: Added `get_default_agent_tools()` function

**File**: [`crates/controller/src/tasks/code/templates.rs`](../../../crates/controller/src/tasks/code/templates.rs)

New function that returns default MCP tools based on agent and run_type:

```rust
fn get_default_agent_tools(github_app: &str, run_type: &str) -> Vec<String> {
    match (agent, run_type) {
        // Morgan: intake/documentation - needs URL scraping, library docs, and memory
        ("morgan", "intake" | "documentation") => vec![
            "mcp_tools_firecrawl_*",
            "mcp_tools_context7_*",
            "mcp_tools_openmemory_*",
        ],
        // Implementation agents: need GitHub, context docs, and memory
        ("rex" | "grizz" | "nova" | "blaze" | "tap" | "spark", "implementation" | "coder") => vec![
            "mcp_tools_github_*",
            "mcp_tools_context7_*",
            "mcp_tools_firecrawl_*",
            "mcp_tools_openmemory_*",
        ],
        // ... other agents
    }
}
```

#### 2. Controller: Updated `generate_client_config()` fallback logic

Now calls `get_default_agent_tools()` before falling back to empty tools:

```rust
// 4) Fall back to built-in defaults based on agent + run_type
let default_tools = Self::get_default_agent_tools(github_app, run_type);
if !default_tools.is_empty() {
    // Use default tools for this agent/run_type combination
    return json!({ "remoteTools": default_tools, "localServers": {} });
}
// 5) No defaults available → minimal empty config
```

#### 3. Suppressed corepack JavaScript warning

**File**: [`templates/_shared/partials/node-env.sh.hbs`](../../../templates/_shared/partials/node-env.sh.hbs)

Changed corepack enable to suppress all output:

```bash
# Before:
corepack enable 2>&1 | grep -v "EINVAL\|readlink\|already exists" || true

# After:
corepack enable >/dev/null 2>&1 || true
```

### Tests Added

- `test_get_default_agent_tools_morgan_intake` - Verifies Morgan gets firecrawl, context7, openmemory
- `test_get_default_agent_tools_rex_implementation` - Verifies Rex gets github tools
- `test_get_default_agent_tools_bolt_infra` - Verifies Bolt gets kubernetes tools
- `test_get_default_agent_tools_unknown_agent` - Verifies unknown agents get empty tools

## Verification

After deploying, run a new intake and verify:
1. Log shows `firecrawl_*`, `context7_*`, `openmemory_*` tools in the init message
2. Morgan can successfully use Firecrawl to scrape URLs
3. Morgan can lookup library documentation via Context7

### 4. Fixed tools-client env var mismatch

**File**: [`crates/tools/src/client.rs`](../../../crates/tools/src/client.rs)

Updated `load_client_config()` to check both legacy and controller-standard paths:

```rust
// Priority:
// 1. MCP_TOOLS_CONFIG env var (legacy)
// 2. MCP_CLIENT_CONFIG env var (controller standard)
// 3. working_dir/client-config.json (controller standard)
// 4. working_dir/tools-config.json (legacy)
// 5. ./tools-config.json (fallback)
```

## Why This Approach?

**Rejected approach**: Adding `remoteTools` directly to the intake WorkflowTemplate. This would work but:
- Hardcodes tools in YAML instead of centralized config
- Duplicates tool definitions across multiple places
- Harder to maintain and update

**Chosen approach**: Built-in defaults in controller because:
- Single source of truth for agent tool requirements
- Consistent with cto-config pattern (tools defined per agent)
- Easy to extend for new agents/run_types
- Works automatically without modifying WorkflowTemplates
