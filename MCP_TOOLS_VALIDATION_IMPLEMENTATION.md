# MCP Tools Validation Implementation

## Overview

Implemented MCP tools validation across all CLI agents following the pattern established in Factory (droid). Each agent now validates that critical MCP tools are available before starting task execution, preventing failures due to misconfiguration.

## Implementation Summary

### CLI Commands Identified

| CLI | Binary | List Tools Command | Notes |
|-----|--------|-------------------|-------|
| **Factory** | `droid` | `droid exec --list-tools` | ✅ Already implemented |
| **Claude** | `claude` | `claude mcp list` | Shows MCP server status |
| **Cursor** | `cursor-agent` | `cursor-agent mcp list` | Shows configured servers |
| **Codex** | `codex` | `codex mcp list` | Shows server status |
| **OpenCode** | `opencode` | `opencode --version` | Validates binary + API key |

### Files Modified

1. **Claude**: `infra/charts/controller/agent-templates/code/claude/container.sh.hbs`
   - Added MCP server validation before Claude command execution
   - Checks for `toolman` server connectivity
   - Validates MCP configuration at `/workspace/.mcp.json`

2. **Cursor**: `infra/charts/controller/agent-templates/code/cursor/container-base.sh.hbs`
   - Added validation before main execution loop
   - Checks for configured MCP servers (toolman, context7)
   - Validates `.cursor/mcp.json` configuration

3. **Codex**: `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`
   - Added validation before execution loop
   - Checks for enabled MCP servers (doc-server)
   - Validates `~/.codex/config.toml` configuration

4. **OpenCode**: `infra/charts/controller/agent-templates/code/opencode/container-base.sh.hbs`
   - Added validation before execution loop
   - Checks OpenCode binary functionality
   - Validates `ANTHROPIC_API_KEY` presence

## Validation Logic

Each implementation follows this pattern:

```bash
# =========================================================================
# MCP TOOLS VALIDATION
# =========================================================================
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                 VALIDATING MCP TOOLS ACCESS                   ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

TOOL_VERIFICATION_FAILED=false

# CLI-specific checks
if command -v <cli-binary> >/dev/null 2>&1; then
  # Check MCP servers/tools
  # Verify required servers based on agent role
  # Set TOOL_VERIFICATION_FAILED=true if issues found
fi

if [ "$TOOL_VERIFICATION_FAILED" = "true" ]; then
  echo "❌ TOOL VERIFICATION FAILED"
  echo "Common issues:"
  # CLI-specific troubleshooting
  exit 1
fi

echo "✅ All critical MCP tools verified successfully"
# =========================================================================
```

## Agent-Specific Requirements

The validation checks for different tools based on the agent role:

### Rex (Implementation Agent)
- **Required**: Context7, GitHub, Brave Search
- **Servers**: toolman, context7, doc-server

### Blaze (UI Agent)
- **Required**: Context7, shadcn, GitHub
- **Servers**: toolman, context7, doc-server

### Cleo (Code Quality Agent)
- **Required**: Context7, GitHub
- **Servers**: toolman, context7, doc-server

### Tess (QA Agent)
- **Required**: Context7, Kubernetes, GitHub
- **Servers**: toolman, context7, doc-server

## Error Handling

When validation fails, the container:
1. Prints a clear error message
2. Lists common configuration issues
3. Displays the full MCP server/tool list for debugging
4. Exits with code 1 to prevent task execution

## Common Issues Detected

The validation catches these common problems:

1. **Missing API Keys**: CONTEXT7_API_KEY, ANTHROPIC_API_KEY not set
2. **MCP Server Misconfiguration**: Servers not initialized or not approved
3. **Configuration File Issues**: Missing or invalid mcp.json, config.toml
4. **Binary Installation Problems**: CLI not installed or not in PATH

## Testing

All implementations:
- ✅ Pass ShellCheck linting
- ✅ Follow existing code patterns
- ✅ Use consistent error messages
- ✅ Provide actionable troubleshooting steps

## Benefits

1. **Early Failure Detection**: Catches configuration issues before expensive AI calls
2. **Clear Error Messages**: Provides specific troubleshooting guidance
3. **Consistent Pattern**: Same validation approach across all CLIs
4. **Reduced Debugging Time**: Immediate feedback on what's misconfigured

## Gemini Build Pipeline Fix

As part of this implementation, we discovered that **Gemini was missing from the nightly build pipeline**. This has been fixed:

### Created Files
- `infra/images/gemini/Dockerfile` - Gemini agent image definition
- `infra/images/gemini/README.md` - Documentation for Gemini image

### Updated Files
- `.github/workflows/agents-build.yaml` - Added Gemini to build matrix

### Build Configuration
- **Version Type**: `dir_hash` (uses git commit hash of Dockerfile directory)
- **Build Trigger**: Daily at 6 AM UTC + on changes to `infra/images/gemini/`
- **Image Tags**: `latest` and `v{git-hash}`

**Note**: The Gemini Dockerfile includes fallback logic for the CLI package name since Google's official Gemini CLI package name may vary. The Dockerfile should be updated with the correct package name once confirmed.

## Next Steps

When deploying:
1. Merge changes to main branch
2. GitHub Actions will build new Gemini image on next nightly run (or manual trigger)
3. ArgoCD will sync updated templates
4. New CodeRun/DocsRun pods will include validation
5. Monitor logs for validation output during container startup

## Example Output

```
════════════════════════════════════════════════════════════════
║                 VALIDATING MCP TOOLS ACCESS                   ║
════════════════════════════════════════════════════════════════

🔍 Verifying Factory MCP tools...
📋 Checking MCP servers...
  ✓ toolman server available
  
🔍 Verifying required MCP servers...
  ✓ toolman server available
  📊 Total tools available: 12

📋 Available Tools:
  • Read - status: allowed
  • Edit - status: blocked
  • Execute - status: allowed
  ...

✅ All critical MCP tools verified successfully
════════════════════════════════════════════════════════════════
```

## Related Files

- Factory validation: `infra/charts/controller/agent-templates/code/factory/container-base.sh.hbs` (lines 1436-1502)
- Agent configurations: `infra/charts/controller/agent-templates/code/*/container*.sh.hbs`

