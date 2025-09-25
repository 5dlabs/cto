# MCP ToolMan Connection Issue Analysis

## Summary
We have **two distinct problems** preventing CLI documentation tools from working in the CLI-agnostic platform project, while they work perfectly in the trader project.

## Issue 1: ToolMan MCP Server Connection Failure

### Working vs Broken Comparison

**✅ TRADER (Working)**:
```json
"mcp_servers":[{"name":"toolman","status":"connected"}]
"tools":["Task","Bash",...,"mcp__toolman__agent_docs_birdeye_query","mcp__toolman__agent_docs_solana_query"...]
```

**❌ CLI-AGNOSTIC PLATFORM (Broken)**:
```json
"mcp_servers":[{"name":"toolman","status":"failed"}]
"tools":["Task","Bash",...] // NO mcp__toolman__* tools!
```

### What We Know
- **ToolMan server is running**: `toolman-6bff759697-chp7t   2/2   Running`
- **Service exists**: `toolman.agent-platform.svc.cluster.local:3000`
- **Network connectivity works**: curl to ToolMan succeeds (405 Method Not Allowed is expected)
- **Same MCP config**: Both use identical `/task-files/mcp.json` with ToolMan configuration
- **Working directory correct**: `/workspace/5dlabs-cto` (repo root)

### Direct Test Results
```bash
# From CLI-agnostic platform pod:
kubectl exec ... -- toolman --url http://toolman.agent-platform.svc.cluster.local:3000/mcp --working-dir /workspace
# Output: Error: missing field `command` at line 12 column 5
```

**This suggests a malformed JSON config is being loaded by toolman command**

## Issue 2: Client Config Debug Shows Only localServers

### The Problem
Container debug output shows:
```
🔍 DEBUG: First few lines of client config:
{
  "localServers": {
    "filesystem": {
      "enabled": true,
      "tools": [
        "read_file",
        "write_file",
        "list_directory",
        "search_files",
        "directory_tree"
```

### But Full File Contains Both
When we cat the full `/task-files/client-config.json`, it has:
```json
{
  "localServers": { ... },
  "remoteTools": [
    "brave_web_search",
    "context7_get_library_docs", 
    "rustdocs_query_rust_docs",
    "agent_docs_codex_query",      ✅
    "agent_docs_cursor_query",     ✅
    // ... all CLI tools present
  ]
}
```

### The Mystery
- **File has correct tools** when we cat it completely
- **Debug output cuts off** at line 10 (`head -10` in container script)
- **Claude Code init message shows NO CLI tools**

**Question**: Is Claude Code only reading the first 10 lines of the config file? Is there a parsing issue?

## Root Cause Analysis

### MCP Connection Issue
1. **ToolMan command fails** with "missing field `command`" error
2. This suggests toolman is reading a **malformed JSON config**
3. Could be reading wrong config file or parsing error

### Client Config Issue  
1. **Controller generates correct config** (we confirmed this)
2. **File contains correct tools** (we confirmed this)
3. **Claude Code doesn't see the tools** (mysterious)

## Potential Investigation Paths

### For MCP Connection Issue
1. **Check toolman logs**: What specific error is occurring?
2. **Test toolman with correct config**: Manually test with `/task-files/mcp.json`
3. **Compare environments**: What's different between trader and CLI-agnostic platform?

### For Client Config Issue
1. **Claude Code config loading**: How does Claude Code load client-config.json?
2. **File corruption**: Is there something wrong with the JSON structure?
3. **Reading/parsing logic**: Does Claude Code have a bug reading large configs?

## Files Investigated
- ✅ `/task-files/mcp.json` - Correct ToolMan config
- ✅ `/task-files/client-config.json` - Correct CLI tools (34 lines)
- ❌ `/workspace/5dlabs-cto/cli-agnostic-platform/.mcp.json` - REMOVED (task-master-ai only)
- ❌ `/workspace/5dlabs-cto/docs/.mcp.json` - REMOVED (task-master-ai only)
- ❌ CLI subdirectories `.kilo`, `.roo`, etc. - REMOVED (task-master-ai only)

## Next Steps Needed
1. **Get ToolMan connection logs**: Find why toolman command fails
2. **Test direct toolman connection**: With proper JSON config
3. **Compare trader vs CLI-agnostic platform environments**: Find the difference
4. **Debug Claude Code config loading**: Why doesn't it see remoteTools?

## Key Questions
1. **Why does trader work but CLI-agnostic platform doesn't?**
2. **What makes toolman command fail with "missing field 'command'"?**
3. **Is there a difference in how pods are started/configured?**
4. **Where is the filesystem localServers config actually coming from?**

