# MCP Tools Configuration Regressions - Complete Summary

## Overview

Three separate regressions discovered in MCP tools configuration affecting parallel test workflows.

---

## ✅ Regression #1: MCP Server Not Reading Repository Config

### Problem
MCP server only loaded its own platform-level `cto-config.json`, causing all workflows to use default tool configurations instead of repository-specific settings.

### Evidence
- Blaze agent showing only 3 tools instead of 14+ defined in repository
- Rex agent showing tools that didn't match repository config

### Root Cause
```rust
// mcp/src/main.rs:load_cto_config()
let mut config_paths = vec![
    PathBuf::from("cto-config.json"),          // ❌ MCP server's directory
    PathBuf::from("../cto-config.json"),       // ❌ Parent directory
    // WORKSPACE_FOLDER_PATHS - not set in cluster ❌
];
```

### Fix
**PR #1597** - Added `load_repository_config()` function
- Reads cto-config.json from target repository path
- Falls back to workspace detection
- Uses repository config for agent tools, platform config as fallback
- Commit: `2955e3a1` on `fix/regressions` branch

### Status
✅ **FIXED** - Included in PR #1597

---

## ✅ Regression #2: Context7 Tools Hardcoded

### Problem
Context7 MCP tools were hardcoded in repository configs, preventing dynamic tool discovery via CLI `--list-tools`.

### Evidence
```json
// cto-parallel-test/cto-config.json (all agents)
{
  "remote": [
    "brave_search_brave_web_search",
    "context7_resolve-library-id",     // ❌ Hardcoded
    "context7_get-library-docs",       // ❌ Hardcoded
    ...
  ],
  "localServers": {}
}
```

### Root Cause
Context7 tools were manually added to repository configs instead of relying on CLI tool discovery mechanism.

### Fix
**cto-parallel-test repository** - Removed hardcoded Context7 tools
- Cleaned all 8 agents (morgan, rex, cleo, tess, blaze, cipher, atlas, bolt)
- Commit: `1b751c607` in cto-parallel-test/main branch
- Context7 tools now discovered via `--list-tools`

### Status
✅ **FIXED** - Committed to cto-parallel-test repository

---

## ⚠️ Regression #3: Cleo Fast-Path Skipping Quality Checks

### Problem
Cleo (quality agent) skips all quality checks if PR already has an approval, even if:
- Approval is from a different agent (not Cleo)
- Approval is stale from previous iteration
- New commits added after approval

### Evidence
```bash
# From Cleo log (task-6)
🔍 Checking if PR #12 already has approval...
✅ PR #12 already has 1 approval(s)
🚀 FAST-PATH: Skipping quality checks since PR is already approved
```

### Root Cause
Fast-path optimization added to Cleo container script:

```bash
# infra/charts/controller/agent-templates/code/claude/container-cleo.sh.hbs:1478-1500
if [ "${APPROVAL_COUNT:-0}" -gt 0 ]; then
  echo "🚀 FAST-PATH: Skipping quality checks since PR is already approved"
  SUCCESS=1  # ❌ Sets success without running checks
  break
fi
```

### Why This Is Wrong
1. **Approval might be from Blaze/Cipher/Tess**, not Cleo
2. **New commits could have been added** after approval
3. **Quality standards bypassed** - no fmt/clippy/test validation
4. **Breaks workflow contract** - Quality gate should always run

### Affected Templates
- `code/claude/container-cleo.sh.hbs` (Cleo quality agent)
- `code/claude/container-cipher.sh.hbs` (Cipher security agent)
- `code/factory/container-base.sh.hbs` (Factory CLI - all agents)
- `code/cursor/container-base.sh.hbs` (Cursor CLI - all agents)

### Fix Required
Remove the fast-path logic from all agent templates:

```bash
# DELETE lines 1475-1510 in container-cleo.sh.hbs
# DELETE similar sections in container-cipher.sh.hbs
# DELETE similar sections in factory/container-base.sh.hbs
# DELETE similar sections in cursor/container-base.sh.hbs
```

### Status
⚠️ **NOT FIXED** - Fast-path code not present in `fix/regressions` branch  
📍 **Location**: Likely in `develop` or `feat/mcp-tools-validation` branch

---

## Impact Summary

| Regression | Severity | Symptoms | Status |
|------------|----------|----------|--------|
| #1: Config not read | 🔴 Critical | Wrong tools, workflows hang | ✅ Fixed in PR #1597 |
| #2: Context7 hardcoded | 🟡 Medium | Prevents dynamic discovery | ✅ Fixed in cto-parallel-test |
| #3: Cleo fast-path | 🔴 Critical | Quality checks skipped entirely | ⚠️ Needs fix (different branch) |

---

## Next Steps

### Immediate (PR #1597)
1. ✅ PR created with Regressions #1 and #2 fixes
2. ⏳ Await review and merge
3. ⏳ Deploy to cluster
4. ⏳ Test with fresh workflow run

### Follow-up (Regression #3)
1. Find branch with fast-path code (likely `develop` or `feat/mcp-tools-validation`)
2. Remove fast-path logic from all agent templates
3. Create separate PR or add to #1597
4. Test Cleo actually runs quality checks

---

## Test Validation Checklist

After fixes deployed:

- [ ] Rex shows repository's tools from cto-config.json (not platform defaults)
- [ ] Blaze shows 14+ tools (not just 3)
- [ ] Context7 tools appear dynamically (not hardcoded)
- [ ] Cleo runs actual quality checks (fmt, clippy, test)
- [ ] Cleo doesn't skip checks when PR has approval
- [ ] Quality gate completes successfully

---

**PR**: https://github.com/5dlabs/cto/pull/1597  
**Branch**: `fix/regressions`  
**Date**: 2025-11-23
