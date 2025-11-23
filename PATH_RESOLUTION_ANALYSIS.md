# Path Resolution Problem - Deep Dive Analysis

## Problem Statement

The MCP server (`cto-mcp`) is failing to correctly identify which workspace/repository it should operate on when multiple Cursor windows are open. This causes:
- Wrong `cto-config.json` file being loaded
- Wrong `tasks.json` file being read
- Wrong repository being detected for auto-detection
- Workflows targeting the wrong repository

## Root Cause Analysis

### 1. How Cursor Sets Environment Variables

When Cursor launches an MCP server, it sets:
- `WORKSPACE_FOLDER_PATHS`: A comma-separated list of **ALL** open workspace folders across **ALL** Cursor windows
- `std::env::current_dir()`: May or may not be set to the "active" workspace

**Critical Finding**: If you have two Cursor windows open:
- Window 1: `/Users/jonathonfritz/code/work-projects/5dlabs/cto`
- Window 2: `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test`

Then `WORKSPACE_FOLDER_PATHS` = `/Users/jonathonfritz/code/work-projects/5dlabs/cto,/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test`

**The MCP server has no way to know which window launched it!**

### 2. Current Implementation Flaws

#### A. Config Loading (`load_cto_config()`)

```rust
// Lines 252-256: Checks ALL workspace paths
if let Ok(workspace_paths) = std::env::var("WORKSPACE_FOLDER_PATHS") {
    for workspace_path in workspace_paths.split(',') {
        let workspace_path = workspace_path.trim();
        config_paths.push(std::path::PathBuf::from(workspace_path).join("cto-config.json"));
    }
}

// Lines 259-280: Uses FIRST config file found
for config_path in config_paths {
    if config_path.exists() {
        // Uses this config - PROBLEM: Might be wrong workspace!
        return Ok(config);
    }
}
```

**Problem**: If both workspaces have `cto-config.json`, it uses whichever one appears first in the list, which may not be the correct one.

#### B. Workspace Resolution (`resolve_workspace_dir()`)

```rust
// Lines 348-363
fn resolve_workspace_dir() -> Option<std::path::PathBuf> {
    std::env::current_dir().ok().or_else(|| {
        std::env::var("WORKSPACE_FOLDER_PATHS")
            .ok()
            .and_then(|paths| {
                paths.split(',').find_map(|p| {
                    let trimmed = p.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(std::path::PathBuf::from(trimmed))
                    }
                })
            })
    })
}
```

**Problem**: 
1. `std::env::current_dir()` may not be set correctly by Cursor
2. Falls back to `find_map()` which returns the **first non-empty path** - again, might be wrong!

#### C. Repository Detection (in `handle_play_workflow`)

```rust
// Lines 1487-1502: Detects workspace repo
let workspace_repo = resolve_workspace_dir().and_then(|workspace_path| {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(&workspace_path)
        .output()
        .ok()?;
    // ... extracts repo name
});

// Line 1505: Compares with requested repo
let is_local_repo = workspace_repo.as_ref().map_or(false, |wr| wr == &repository);
```

**Problem**: If `resolve_workspace_dir()` returns the wrong workspace, the git command runs in the wrong directory, detects the wrong repo, and the comparison fails.

### 3. The "Working Directory" Change Regression

The user mentioned this problem started after "we messed with the working directory." Let's trace what changed:

**Before**: Code likely assumed a single workspace and used it directly.

**After**: Code tries to be smart about multiple workspaces but:
- Still picks the first one arbitrarily
- Doesn't have a reliable way to determine which workspace is "active"
- `std::env::current_dir()` is unreliable because Cursor may not set it correctly

### 4. Why `std::env::current_dir()` Fails

When Cursor launches an MCP server:
- It may set `WORKSPACE_FOLDER_PATHS` correctly
- But `std::env::current_dir()` might be:
  - The directory where the MCP binary is located (`/usr/local/bin` or similar)
  - The user's home directory
  - Some other arbitrary directory
  - **NOT** the workspace directory

This makes `resolve_workspace_dir()`'s first attempt unreliable.

## Proposed Solutions

### Solution 1: Use Config File Location to Determine Workspace (RECOMMENDED)

**Idea**: Since `load_cto_config()` finds and loads a config file, we can use **which config file was loaded** to determine the correct workspace.

**Implementation**:
1. Store the config file path when loading
2. Use that path's parent directory as the "active workspace"
3. Update `resolve_workspace_dir()` to use this stored value

**Pros**:
- Reliable: Uses the actual config that was loaded
- Simple: No need to guess which workspace is active
- Works with multiple Cursor windows

**Cons**:
- Requires storing state (but we already have `CTO_CONFIG` global)
- Config must exist (but it's required anyway)

### Solution 2: Match Repository from Config to Workspace

**Idea**: After loading config, check which workspace contains the repository specified in the config.

**Implementation**:
1. Load config (may be from any workspace)
2. Extract `defaults.play.repository` from config
3. For each workspace in `WORKSPACE_FOLDER_PATHS`:
   - Run `git remote get-url origin` in that workspace
   - Compare with config's repository
   - Use matching workspace

**Pros**:
- Explicitly matches config intent with workspace
- Handles case where config specifies a different repo

**Cons**:
- More complex (requires git commands)
- Slower (multiple git commands)
- May fail if workspace doesn't have git initialized

### Solution 3: Use MCP Initialization Context

**Idea**: Cursor might pass workspace information in the MCP `initialize` request.

**Implementation**:
1. Check MCP `initialize` params for workspace info
2. Store active workspace from initialization
3. Use stored workspace throughout

**Pros**:
- Uses MCP protocol correctly
- Explicit workspace identification

**Cons**:
- Requires checking MCP spec for available params
- May not be available in current Cursor version

### Solution 4: Environment Variable for Active Workspace

**Idea**: Request Cursor to set a new env var like `CURSOR_ACTIVE_WORKSPACE`.

**Implementation**:
1. Check for `CURSOR_ACTIVE_WORKSPACE` env var
2. Fall back to current logic if not available

**Pros**:
- Simple and explicit
- Backward compatible

**Cons**:
- Requires Cursor changes (may not be possible)
- Not available immediately

## Recommended Approach

**Combine Solution 1 + Solution 2**:

1. **Store config file path** when loading config
2. **Use config file's directory** as primary workspace
3. **Validate** by checking if that workspace's git repo matches config's repository setting
4. **Fall back** to Solution 2 (matching) if validation fails

This gives us:
- Reliability (uses actual loaded config)
- Validation (ensures workspace matches config intent)
- Fallback (handles edge cases)

## Implementation Plan

1. Add `config_file_path: Option<PathBuf>` to `CtoConfig` struct (or store separately)
2. Update `load_cto_config()` to store the path of the loaded config
3. Create `get_active_workspace()` function that:
   - Returns config file's parent directory if available
   - Falls back to matching workspace by repository
   - Falls back to first workspace as last resort
4. Update `resolve_workspace_dir()` to use `get_active_workspace()`
5. Update all call sites to use the new resolution logic

## Testing Strategy

1. **Single workspace**: Should work as before
2. **Multiple workspaces, config in first**: Should use first workspace
3. **Multiple workspaces, config in second**: Should use second workspace
4. **Multiple workspaces, config specifies different repo**: Should match workspace with that repo
5. **No config file**: Should fail gracefully with clear error

## Current State

- ✅ `resolve_workspace_dir()` exists but is unreliable
- ✅ Config loading works but doesn't store which file was used
- ❌ No way to determine "active" workspace reliably
- ❌ Repository detection fails when wrong workspace is used

## Next Steps

1. Implement config file path storage
2. Implement `get_active_workspace()` function
3. Update all workspace resolution calls
4. Test with multiple Cursor windows
5. Document the behavior

