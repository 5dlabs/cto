# Regression #3: Breaking Change in AgentTools Field Name

**Date:** 2025-11-23  
**Found During:** E2E testing review  
**Severity:** CRITICAL - Config loading broken  
**Related:** MCP validation changes

---

## Problem Statement

The `rename = "remoteTools"` attribute in `AgentTools` struct breaks loading of all existing `cto-config.json` files.

**Error that would occur:**
```
Failed to parse repository config: missing field `remoteTools`
```

---

## Root Cause

The `AgentTools` struct has conflicting requirements:

**In the code:**
```rust
struct AgentTools {
    #[serde(default, rename = "remoteTools")]  // ❌ Expects camelCase
    remote: Vec<String>,
}
```

**In ALL cto-config files:**
```json
"tools": {
  "remote": [  // ❌ Uses lowercase
    "brave_search_brave_web_search"
  ]
}
```

**Files affected:**
- `cto-config.json` - Uses `"remote"`
- `cto-config-example.json` - Uses `"remote"`
- `cto-config.template.json` - Uses `"remote"`

**Different schema:**
- `client-config.json` - Uses `"remoteTools"` (camelCase)
- `toolman-client/client-config.json` - Uses `"remoteTools"`

---

## Impact

**Without fix:**
- ❌ All `cto-config.json` files fail to load
- ❌ Repository-specific configurations broken
- ❌ Falls back to platform config (loses customization)
- ❌ Tools configuration incorrect

**Symptoms:**
```
⚠️  Failed to parse repository config: missing field `remoteTools`
ℹ️  Using platform default configuration (no repository config found)
```

---

## Fix

**Remove the `rename` attribute** to maintain backward compatibility:

```rust
// Before (BREAKING):
struct AgentTools {
    #[serde(default, rename = "remoteTools")]  // ❌ Breaks cto-config.json
    remote: Vec<String>,
}

// After (FIXED):
struct AgentTools {
    #[serde(default)]  // ✅ Uses field name "remote" in JSON
    remote: Vec<String>,
}
```

**Why this works:**
- `cto-config.json` files already use `"remote"` ✅
- Struct field is named `remote` ✅
- No rename needed - they match!

**Client config files:**
- Need to be updated from `"remoteTools"` → `"remote"`
- OR use a different struct type for client configs

---

## Alternative Considered

**Option A:** Keep `rename = "remoteTools"` and update all config files  
**Verdict:** ❌ Breaking change, requires migration

**Option B:** Remove `rename` attribute  
**Verdict:** ✅ Backward compatible, matches existing files

**Option C:** Use separate structs for cto-config vs client-config  
**Verdict:** More complex, not needed

---

## Files Changed

- `mcp/src/main.rs` - Removed `rename = "remoteTools"` attribute
- `E2E_REGRESSION_3_BREAKING_RENAME.md` - This documentation

---

## Client Config Migration

If `client-config.json` files need updating:

```json
// Before:
{
  "remoteTools": ["tool1", "tool2"]
}

// After:
{
  "remote": ["tool1", "tool2"]
}
```

---

## Testing

**Verify config loading:**
```bash
# Should load successfully
cto-mcp < test-request.json

# Check logs for:
✅ Repository configuration loaded successfully
(not)
⚠️  Failed to parse repository config: missing field remoteTools
```

---

## Related Issues

- Part of MCP validation changes
- Introduced in feat/mcp-tools-validation branch
- Related to controller compatibility
- Part of E2E regression testing

---

## Prevention

**Lesson learned:**
- Test config loading with actual config files
- Verify field names match between code and configs
- Check both serialization AND deserialization
- Test with multiple config file variants
