# Path Resolution Problem - Deep Dive Analysis

## Problem Statement

The MCP server is failing to correctly identify and use the workspace directory when running from `cto-parallel-test` repository in a separate Cursor instance. Despite having the repository open in its own Cursor window, the MCP server appears to be using configuration or paths from the wrong repository.

## Current Investigation Findings

### 1. Environment Variable Analysis

**WORKSPACE_FOLDER_PATHS Behavior:**
- Cursor sets this environment variable when launching the MCP server
- When multiple Cursor windows are open, this variable contains ALL open workspace paths, comma-separated
- Our code was taking the FIRST path in the list, regardless of which Cursor instance launched the MCP

**Current Code Issues:**
```rust
// Problem: Always takes first path
let workspace_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
    .map(|paths| {
        let first_path = paths.split(',').next().unwrap_or(&paths).trim();
        first_path.to_string()
    })
```

### 2. Configuration File Resolution

**Expected Behavior:**
- MCP server running from `cto-parallel-test` should read `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test/cto-config.json`

**Actual Behavior:**
- MCP server appears to be reading from `/Users/jonathonfritz/code/work-projects/5dlabs/cto/cto-config.json`
- This happens because the workspace resolution picks the wrong directory

### 3. Working Directory vs Workspace Directory Confusion

**Key Distinction:**
- **Workspace Directory**: Where Cursor is running from (should be cto-parallel-test)
- **Working Directory**: A subdirectory within the workspace for specific operations
- **Docs Directory**: Where tasks.json and documentation live

**The Regression:**
- Recent changes to handle `working_directory` parameter may have broken the workspace resolution
- The system now confuses which repository it should be operating on

### 4. Repository Detection Logic

**Current Implementation:**
1. Checks ConfigMap for existing progress
2. Tries to detect if repository is "local" by comparing with workspace
3. Falls back to requiring explicit task_id for "remote" repositories

**Problem:**
- Step 2 fails because workspace detection is wrong
- System thinks `cto-parallel-test` is "remote" even when running from that repo

### 5. Worktree Complication

**New Discovery:**
- The workspace path shows as `/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`
- This is a Git worktree, not the main repository
- This adds another layer of complexity to path resolution

## Root Causes Identified

### Primary Issue: Multiple Workspace Handling
The MCP server cannot distinguish which Cursor instance launched it when multiple workspaces are open. It defaults to the first workspace in the environment variable list.

### Secondary Issue: Worktree Path Resolution
Git worktrees create alternate working directories that may not resolve correctly when trying to determine the repository.

### Tertiary Issue: Config File Discovery
The config file discovery logic searches multiple locations but may be finding the wrong config due to incorrect workspace resolution.

## Attempted Fixes and Results

### Fix 1: Graceful Handling of Missing tasks.json
- **Change**: Modified error handling to use `.ok().flatten()` instead of `?`
- **Result**: Prevented crashes but didn't solve the core issue
- **Status**: Partial improvement

### Fix 2: Repository Detection
- **Change**: Added logic to detect when repository doesn't match workspace
- **Result**: Made the problem more visible but didn't fix it
- **Status**: Diagnostic improvement only

### Fix 3: Prefer Current Directory
- **Change**: Created `resolve_workspace_dir()` helper to prefer `std::env::current_dir()`
- **Result**: Should have fixed the issue but may not work with worktrees
- **Status**: Needs verification

## Why Zero Progress?

1. **Wrong Assumptions**: We assumed the MCP server could detect its launch context, but Cursor's environment doesn't provide this clearly
2. **Worktree Complexity**: The worktree path (`/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`) adds unexpected complexity
3. **Multiple Moving Parts**: Config resolution, workspace detection, and repository matching all interact in complex ways

## Next Steps for Investigation

### 1. Debug Logging Needed
Add comprehensive logging to understand:
- What `WORKSPACE_FOLDER_PATHS` actually contains
- What `std::env::current_dir()` returns
- Which config file is actually being loaded
- What repository is being detected

### 2. Test Scenarios
Need to test:
- Single Cursor window with cto-parallel-test
- Multiple Cursor windows (cto + cto-parallel-test)
- Worktree vs regular clone behavior

### 3. Potential Solutions to Explore

#### Option A: Explicit Workspace Selection
Add a parameter to explicitly specify which workspace to use:
```rust
cto_play({ workspace: "cto-parallel-test", task_id: 1 })
```

#### Option B: Process ID Matching
Try to match the MCP server process with its parent Cursor process to determine the correct workspace.

#### Option C: Config Path Override
Allow explicit config file path:
```rust
cto_play({ config: "/path/to/cto-parallel-test/cto-config.json" })
```

#### Option D: Repository-Based Resolution
Instead of workspace-based resolution, use the repository parameter to find the correct config.

## Critical Questions to Answer

1. **What is the actual current working directory when MCP runs from cto-parallel-test?**
2. **How does the worktree path affect git operations and config discovery?**
3. **Can we reliably detect which Cursor instance launched the MCP server?**
4. **Should we abandon auto-detection and require explicit configuration?**

## Observations from User Feedback

1. User has two separate Cursor instances open
2. User expects MCP running from cto-parallel-test to use that repo's config
3. The regression happened after working_directory changes
4. The system worked correctly before recent modifications

## Hypothesis

The core issue is that when multiple Cursor windows are open, the MCP server cannot determine which window launched it. The `WORKSPACE_FOLDER_PATHS` environment variable contains all workspaces, and without additional context, we cannot pick the correct one.

The worktree complication (`/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`) suggests the user may be using Cursor's worktree feature, which creates additional complexity in path resolution.

## Actual Workspace Configuration Discovered

### Repository Locations
- **Main CTO Repo**: `/Users/jonathonfritz/code/work-projects/5dlabs/cto` (git@github.com:5dlabs/cto.git)
- **Parallel Test Repo**: `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test` (https://github.com/5dlabs/cto-parallel-test.git)
- **Current Worktree**: `/Users/jonathonfritz/.cursor/worktrees/cto/gXY53` (git@github.com:5dlabs/cto.git)

### Key Discovery: Cursor Worktree Feature
The user is currently in a Cursor worktree (`/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`) which is a feature branch workspace of the main cto repository. This explains why paths are getting confused.

## The Real Problem

1. **User has TWO different setups:**
   - Worktree of `cto` repo at `/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`
   - Separate `cto-parallel-test` repo at `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test`

2. **MCP server in worktree is trying to work with parallel-test repo:**
   - Binary runs from worktree (cto repo)
   - Config points to cto-parallel-test
   - Path resolution gets confused between the two

3. **The resolve_workspace_dir() fix may not be sufficient:**
   - It uses `std::env::current_dir()` which would return the worktree path
   - But the user wants to work with cto-parallel-test repository

## SOLUTION IDENTIFIED

### The Core Issue
The MCP server is running from a Cursor worktree of the `cto` repository, but trying to operate on a completely different repository (`cto-parallel-test`). The path resolution logic cannot handle this cross-repository operation correctly.

### Why It's Failing
1. MCP server binary runs from: `/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`
2. Config in worktree points to: `repository: "5dlabs/cto-parallel-test"`
3. Actual parallel-test repo is at: `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test`
4. MCP server can't find tasks.json because it's looking in the wrong place

### The Fix Needed
We need to add a `repository_path` parameter that explicitly tells the MCP server where the target repository is located on disk:

```rust
// In handle_play_workflow function
let repository_path = arguments
    .get("repository_path")
    .and_then(|v| v.as_str())
    .map(String::from);

// Use this path for finding tasks.json instead of workspace detection
if let Some(repo_path) = repository_path {
    let tasks_path = PathBuf::from(repo_path)
        .join(docs_project_directory)
        .join(".taskmaster/tasks/tasks.json");
    // Use this path for task detection
}
```

### Immediate Workaround
Until we implement the fix, the user should:
1. Open `cto-parallel-test` directly in Cursor (not from a worktree)
2. Run the MCP server from that Cursor instance
3. This will ensure correct path resolution

## Technical Deep Dive: Code Flow Analysis

### 1. Config File Loading Sequence (load_cto_config function)
```rust
// Line 245-290: Config discovery order
1. ./cto-config.json (current directory)
2. ../cto-config.json (parent directory)
3. ${WORKSPACE_FOLDER_PATHS}/cto-config.json (each workspace in env var)
```

**Finding**: The config loader tries multiple locations, taking the FIRST valid config it finds. This means if you're in a worktree with a config, it uses that one, even if you want to use a different repo's config.

### 2. Workspace Resolution (resolve_workspace_dir function - Line 340-351)
```rust
fn resolve_workspace_dir() -> Option<PathBuf> {
    std::env::current_dir().ok().or_else(|| {
        std::env::var("WORKSPACE_FOLDER_PATHS")
            .ok()
            .and_then(|paths| {
                let first_path = paths.split(',').next()?.trim();
                Some(PathBuf::from(first_path))
            })
    })
}
```

**Finding**: This helper prefers `current_dir()` but falls back to first workspace in list. However, when running from Cursor, `current_dir()` might be the Cursor installation directory, not the workspace.

### 3. Task File Discovery (find_tasks_file function - Line 1127-1161)
```rust
// Searches for tasks.json in these locations:
1. ${base_dir}/.taskmaster/tasks/tasks.json
2. ${base_dir}/.taskmaster/tasks.json  
3. ${base_dir}/tasks.json
// Where base_dir is workspace_dir + optional working_dir
```

**Finding**: The task finder is workspace-relative. It cannot find tasks in a different repository unless that repository is the workspace.

### 4. Repository Detection for Auto-Task (Line 1487-1512)
```rust
// Gets workspace repo by:
1. Taking first path from WORKSPACE_FOLDER_PATHS
2. Running 'git remote get-url origin' in that path
3. Extracting org/repo from the URL
4. Comparing with requested repository
```

**Finding**: Always uses FIRST workspace path for git detection, even if MCP was launched from second workspace.

## Critical Code Paths That Need Fixing

### Path 1: Config Loading
- **Problem**: Takes first valid config found, not necessarily the right one
- **Impact**: Wrong configuration loaded when multiple repos have configs

### Path 2: Workspace Resolution  
- **Problem**: Cannot determine which Cursor window launched the MCP server
- **Impact**: Wrong workspace used for all path operations

### Path 3: Task Detection
- **Problem**: Bound to workspace directory, cannot work cross-repository
- **Impact**: Cannot find tasks.json in target repository

### Path 4: Git Repository Detection
- **Problem**: Always checks first workspace, not current context
- **Impact**: Incorrectly identifies repositories as "remote"

## Environment Variable Analysis

### What Cursor Provides
- `WORKSPACE_FOLDER_PATHS`: Comma-separated list of ALL open workspace paths
- No variable to identify WHICH workspace launched the MCP server
- No process hierarchy info to determine parent Cursor instance

### What We Need But Don't Have
- `CURSOR_ACTIVE_WORKSPACE`: The specific workspace that launched this MCP instance
- `CURSOR_INSTANCE_ID`: To differentiate between multiple Cursor windows
- `MCP_LAUNCH_CONTEXT`: Explicit context about where MCP was invoked from

## The Worktree Complication

### Discovery
- User's workspace: `/Users/jonathonfritz/.cursor/worktrees/cto/gXY53`
- This is a Cursor-managed worktree of the main `cto` repository
- Worktrees allow multiple branches to be checked out simultaneously

### Why This Matters
1. Worktree has its own working directory but shares `.git` with main repo
2. Config in worktree points to different repo (`cto-parallel-test`)
3. Path resolution assumes workspace and target repo are the same

## Summary of Findings

### Root Cause
The MCP server cannot differentiate between multiple Cursor workspaces when launched. It defaults to using the first workspace in the environment variable list, causing it to use the wrong repository's configuration and paths.

### Contributing Factors
1. **Worktree Usage**: User is in a Cursor worktree which adds path complexity
2. **Cross-Repository Operation**: Trying to operate on `cto-parallel-test` from `cto` worktree
3. **Config Discovery**: Config loader finds worktree's config first, not target repo's config
4. **No Context Isolation**: MCP server has no way to know which Cursor instance launched it

### Why Previous Fixes Failed
1. **Fix 1 (Error Handling)**: Only prevented crashes, didn't address wrong path usage
2. **Fix 2 (Repository Detection)**: Detected mismatch but couldn't resolve it
3. **Fix 3 (resolve_workspace_dir)**: Still picked wrong workspace from the list

### What Would Actually Fix This
1. **Option A**: Add explicit `repository_path` parameter to bypass auto-detection
2. **Option B**: Use repository name to find correct workspace from the list
3. **Option C**: Require single Cursor workspace when using MCP
4. **Option D**: Create separate MCP config per workspace

## Test Cases Needed

1. **Single Workspace Test**: Close all but `cto-parallel-test` Cursor window
2. **Explicit Path Test**: Add `repository_path` parameter pointing to correct repo
3. **Config Override Test**: Explicitly specify which config file to use
4. **Worktree Isolation Test**: Test from main repo instead of worktree

## Conclusion

The regression was introduced when we changed working directory handling without considering the multi-workspace scenario. The MCP server needs explicit guidance about which repository to operate on when multiple workspaces are open.

---

*Analysis completed: 2024-11-23*
*Status: Root cause fully identified, multiple solution paths available*
*Next Step: Wait for other analyses, then synthesize findings*


## Additional Findings (Agent GPT-5.1 Codex)

1. **Auto-detect still collapses to MCP repo**  
   Even after `resolve_workspace_dir` landed, any run where `current_dir` resolves to the MCP worktree (e.g., `/Users/jonathonfritz/.cursor/worktrees/cto/e5poI`) still searches for `.taskmaster` under that repo. Because the target repo (`cto-parallel-test`) lives at `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test`, TaskMaster lookup cannot succeed without a manual override.

2. **Config coupling remains**  
   MCP always loads the *local* `cto-config.json` (from the worktree it was launched in). The `cto-parallel-test` copy is ignored unless that repo is the Cursor workspace root, so config edits in the parallel repo currently have no effect.

3. **First-run experience is broken**  
   Auto-detection assumes a local `.taskmaster`. For repos that only exist remotely (or in another checkout), the first `cto_play` must either take a `task_id` argument or have logic to clone/fetch tasks before guessing.

4. **Recommended short-term workaround**  
   Launch MCP from inside the target repo (open `cto-parallel-test` in Cursor directly) *or* add a `repository_path` override so TaskMaster discovery points at `/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test/docs`.

5. **Future enhancement**  
   Add telemetry dumps (workspace dir, docs dir, resolved task path) to the play command output so mis-resolutions are obvious without spelunking logs.
