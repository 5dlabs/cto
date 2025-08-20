# Cleo Container Issues

This document tracks issues identified with the Cleo code quality agent container.

## Issue #1: Hardcoded Repository-Specific Documentation Logic

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: High  

### Problem
The Cleo container script contains hardcoded documentation enforcement logic that is specific to the 5dlabs/cto repository structure. This appears in two places:

1. **Shell script echo statements** (lines 167-181 in `container-cleo.sh.hbs`):
   ```bash
   echo "CRITICAL MISSION: Before approving any PR, verify that:"
   echo "1. Implementation changes are reflected in documentation"
   echo "2. Or a rationale is provided when docs are not needed"
   echo ""
   echo "Detection logic:"
   echo "- controller/** → engineering and controller references required"
   echo "- infra/** (charts/CRDs) → infra docs and references required"
   echo "- mcp/** → CLI/tooling docs required"
   echo "- API/config changes → README/examples updates required"
   ```

2. **CLAUDE.md memory file** (lines 280-350 in `container-cleo.sh.hbs`):
   ```markdown
   ### Detection Logic
   Map code changes to required documentation:
   - `controller/**` → `docs/engineering/*` and `docs/references/*`
   - `infra/charts/**` → chart docs and `docs/references/*`
   - `workflows/sensors` → `docs/references/argo-events/*`
   - API/config changes → `docs/README.md`, `docs/examples/*`
   ```

### Impact
- The agent is hardcoded to expect specific directory structures (`controller/`, `infra/`, `mcp/`, etc.)
- Documentation paths are hardcoded to match the 5dlabs/cto repository structure
- This makes Cleo unusable for other repositories with different structures
- The logic is embedded in both the startup script and the agent's memory file

### Questions to Investigate
1. Is this logic actually being executed by the agent, or just being printed as informational output?
2. Should this logic be moved to a configurable template or external configuration?
3. Should Cleo be repository-agnostic and receive documentation requirements as part of the task prompt?

### Additional Problem
The hardcoded echo statements are **misleading** - they suggest Cleo is doing documentation enforcement when it should be doing code quality enforcement. This creates confusion about Cleo's actual purpose and capabilities.

### Proposed Solutions
1. **Remove hardcoded logic**: Extract repository-specific documentation requirements to task prompts
2. **Make it configurable**: Add template variables for documentation paths and detection logic
3. **Move to task-specific**: Include documentation requirements in the task prompt rather than hardcoding in the container
4. **Remove misleading output**: Delete the hardcoded echo statements that suggest documentation enforcement
5. **Update agent description**: Change Cleo's description to reflect its actual code quality focus

### Files Affected
- `infra/charts/controller/claude-templates/code/container-cleo.sh.hbs`

---

## Issue #2: Incorrect Focus - Cleo Should Focus on Code Quality, Not Documentation

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: High  

### Problem
The current Cleo container is focused on documentation enforcement, but according to the user requirements, Cleo should be focused on **code quality enforcement**:

**Actual Cleo Requirements:**
- **Primary Focus**: Code quality enforcement
  - Clippy pedantic checks
  - Rust linting
  - Rust tests
- **Secondary Focus**: YAML linting (when YAML changes are detected)
- **Change Detection**: Must detect the nature of changes:
  - Infrastructure changes → YAML files only
  - Code changes → Rust files only
  - Mixed changes → Both Rust and YAML files

### Current vs. Required Behavior
**Current (Incorrect):**
- Focuses on documentation enforcement
- Hardcoded to specific repository structure
- Complex documentation mapping logic

**Required (Correct):**
- Focus on code quality tools (Clippy, rust lint, rust tests)
- YAML linting when YAML files are changed
- Simple change detection logic
- Repository-agnostic approach

### Implementation Requirements
1. **Change Detection Logic**:
   ```bash
   # Detect file types in changes
   RUST_CHANGES=$(git diff --name-only | grep -E '\.(rs|toml)$' || true)
   YAML_CHANGES=$(git diff --name-only | grep -E '\.(yaml|yml)$' || true)
   ```

2. **Quality Checks**:
   - **Rust Changes**: Run `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test`
   - **YAML Changes**: Run YAML linting tools
   - **No Changes**: Skip quality checks gracefully

3. **Error Handling**:
   - Handle cases where no YAML files exist
   - Handle cases where no Rust files exist
   - Provide clear error messages for failed quality checks

### Files Affected
- `infra/charts/controller/claude-templates/code/container-cleo.sh.hbs`
- `infra/charts/controller/claude-templates/code/CLAUDE.md.hbs` (if exists)

---

## Issue #3: Directory Copy Error - Attempting to Copy Directory into Itself

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: Medium  

### Problem
The container script attempts to copy a directory into itself, causing the error:
```
cp: cannot copy a directory, '/workspace/./.', into itself, '/workspace/cto-play-test/.'
```

### Root Cause
The issue occurs in the copy logic around lines 232-236 in `container-cleo.sh.hbs`:

```bash
# Copy repository files to working directory, avoiding recursive copy
if [ "$TARGET_REPO_DIR" != "$(basename "$CLAUDE_WORK_DIR")" ]; then
    cp -r "/workspace/$TARGET_REPO_DIR/." "$CLAUDE_WORK_DIR/"
else
    echo "✓ Working directory is repository root, no copy needed"
fi
```

**The Problem:**
1. `TARGET_REPO_DIR` is set to `"{{#if working_directory}}{{working_directory}}{{else}}{{service}}{{/if}}"`
2. `CLAUDE_WORK_DIR` is set to `/workspace/$REPO_NAME` (when `WORK_DIR` is "." or empty)
3. When `working_directory` is "." (dot), `TARGET_REPO_DIR` becomes "." 
4. The comparison `"$TARGET_REPO_DIR" != "$(basename "$CLAUDE_WORK_DIR")"` becomes `"." != "cto-play-test"`
5. This triggers the copy command: `cp -r "/workspace/./." "/workspace/cto-play-test/"`
6. Since `/workspace/./.` resolves to `/workspace/.` which is the same as `/workspace`, and the destination is `/workspace/cto-play-test/`, it's trying to copy a directory into itself

### Impact
- Container startup fails with copy error
- Prevents Cleo from running properly
- Creates confusing error messages in logs

### Proposed Solutions
1. **Fix the comparison logic**: The current comparison doesn't properly handle the case where `working_directory` is "."
2. **Improve path resolution**: Use `realpath` or similar to resolve relative paths before comparison
3. **Add better error handling**: Check if source and destination are the same before attempting copy

### Files Affected
- `infra/charts/controller/claude-templates/code/container-cleo.sh.hbs`

---

## Issue #4: Cleo Not Updating Feature Branch - Prompt Handling Issue

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: High  

### Problem
Cleo doesn't appear to be updating the feature branch properly. This suggests there may be an issue with how the prompt is handling branch management or git operations.

### Questions to Investigate
1. Is Cleo checking out the correct branch (feature branch vs main)?
2. Is Cleo making commits and pushing changes to the feature branch?
3. Are there git configuration issues preventing branch updates?
4. Is the prompt missing instructions for branch management?

### Impact
- Cleo may not be making the expected changes to the feature branch
- Changes might be getting lost or applied to the wrong branch
- PR reviews may not see the expected updates

---

## Issue #5: Incorrect Prompt Handling - Cleo Should Use Static Prompt, Not Task-Specific

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: High  

### Problem
There's a fundamental misunderstanding in how Cleo's prompt should work compared to Rex:

**Current (Incorrect) Behavior:**
- Cleo is trying to use task-specific prompts (like Rex does)
- Cleo's prompt changes based on the task
- **Evidence**: Log shows "✓ Using task-specific prompt from docs service: task/prompt.md"

**Required (Correct) Behavior:**
- **Cleo should have a static prompt** that doesn't change between tasks
- **Cleo should check out the pull request** and conduct tests accordingly
- **Cleo should have access to all task files for context** (task.md, acceptance-criteria.md, prompt.md, etc.)
- **Cleo should be informed** that the task files contain the prompt given to Rex and the acceptance criteria

### Key Distinction
- **Rex**: Uses task-specific prompts stored in the repo (dynamic)
- **Cleo**: Uses static prompt + task files for context (static + contextual)

### Implementation Requirements
1. **Static Cleo Prompt**: Create a consistent prompt that defines Cleo's role and responsibilities
2. **Task File Access**: Ensure Cleo can read all task files for context
3. **Context Explanation**: Explain to Cleo that task files contain Rex's prompt and acceptance criteria
4. **PR Checkout**: Cleo should checkout the pull request branch for testing

### Files Affected
- `infra/charts/controller/claude-templates/code/container-cleo.sh.hbs`
- `infra/charts/controller/claude-templates/code/CLAUDE.md.hbs` (if exists)
- Task prompt handling logic

---

## Issue #6: Missing Docker/CI Requirements - Cleo Should Ensure Working Dockerfile and GitHub Actions

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: Medium  

### Problem
Cleo is missing requirements to ensure the project has working containerization and CI/CD setup.

### New Requirements for Cleo
1. **Dockerfile Validation**: Ensure there's a working Dockerfile
2. **GitHub Actions Validation**: Ensure there are GitHub Actions that build and test the container
3. **CI/CD Verification**: Verify that the container can be built successfully in CI
4. **GitHub CLI Access**: Cleo should have access to GitHub CLI to manage this independently

### Implementation Notes
- **No Local Docker Building**: Cleo should not attempt to build Docker locally
- **GitHub Actions Only**: All container building should be done through GitHub Actions
- **Future Enhancement**: Docker support will be added later, but not now

### Expected Behavior
1. Cleo checks for presence of Dockerfile
2. Cleo checks for GitHub Actions workflow that builds the container
3. Cleo triggers or verifies the GitHub Actions build
4. Cleo reports on the success/failure of the container build

### Impact
- Ensures projects have proper containerization setup
- Validates that CI/CD pipeline works correctly
- Prevents deployment issues due to missing or broken container builds

---

## Issue #7: Workflow Timeout - Max Duration Limit Exceeded

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: High  

### Problem
The workflow is failing with a timeout error:
```
NAME: test-sonnet4-workflow.update-to-waiting-qa   
ID: test-sonnet4-workflow-2031064373   
TYPE: Retry
PHASE: Failed
MESSAGE: Max duration limit exceeded
START TIME: 8/20/2025, 12:11:12 AM (6m8s ago)
END TIME: 8/20/2025, 12:12:12 AM (5m8s ago)
DURATION: 60s
PROGRESS: 0/3
```

### Root Cause
The workflow has a 60-second timeout limit, but the entire flow can legitimately take **hours or days** to complete.

### Impact
- Workflows fail prematurely due to timeout
- Long-running tasks (like code reviews, testing, CI/CD builds) cannot complete
- Agents cannot perform thorough analysis and quality checks
- Workflow reliability is compromised

### Requirements
- **Remove or extend timeouts**: The entire flow should not have arbitrary time limits
- **Support long-running operations**: Workflows should be able to run for hours or days as needed
- **Handle legitimate delays**: CI/CD builds, code reviews, and thorough testing can take significant time

### Questions to Investigate
1. Where is the 60-second timeout configured?
2. Are there other timeout settings that need to be adjusted?
3. Should there be any timeouts at all, or should they be removed entirely?
4. How can we ensure workflows don't hang indefinitely while still allowing legitimate long-running operations?

### Files Affected
- Workflow templates (likely in Argo Workflows configuration)
- Timeout settings in workflow definitions
- Possibly in Helm chart values or workflow templates

---

## Issue #8: Incorrect Task Selection - Running Task 2 Instead of Task 1

**Status**: Open  
**Date**: 2025-01-27  
**Severity**: Medium  

### Problem
When launching the play workflow, the system is running task 2 instead of task 1. This suggests there may be an issue with task selection, prioritization, or task ordering logic.

### Questions to Investigate
1. **Task Selection Logic**: How is the next task determined when launching a play workflow?
2. **Task Priority**: Are tasks being prioritized by ID, status, dependencies, or some other criteria?
3. **Task Ordering**: Is there a specific order that tasks should follow?
4. **Workflow Configuration**: Is the workflow configured to start with a specific task?
5. **Task Status**: What is the status of task 1 vs task 2? Is task 1 already completed, blocked, or in a different state?

### Potential Root Causes
1. **Task Dependencies**: Task 1 might have unmet dependencies, so task 2 is selected instead
2. **Task Status**: Task 1 might already be completed or in a non-pending state
3. **Selection Algorithm**: The task selection logic might be choosing the wrong task
4. **Configuration Issue**: The workflow might be configured to start with task 2
5. **Data Issue**: There might be incorrect data about task 1's state or requirements

### Impact
- Workflows may not execute tasks in the expected order
- Critical tasks might be skipped or delayed
- Confusion about which task is actually being processed
- Potential for missing important work or dependencies

### Investigation Steps
1. Check the current status of all tasks in the workflow
2. Review task dependencies and their satisfaction status
3. Examine the task selection logic in the workflow configuration
4. Verify the task ordering and prioritization rules
5. Check if there are any configuration overrides or defaults

**Note**: This issue needs investigation to understand why task 2 is being selected instead of task 1.

### Files Affected
- Workflow templates and configuration
- Task selection logic
- Task dependency management
- Possibly workflow state or database records

---

## Issue #9: [To be added]

*Additional issues will be added here as they are identified.*
