# Cleo PR Context Missing - Critical Issues



## Issue Summary
Cleo cannot perform proper code quality review because it lacks awareness of which pull request to review and has repository management problems.

## Key Problems Identified

### 1. **No PR Context Available**
- **Issue**: Cleo has no way to know which PR number to checkout and review
- **Evidence**: No PR URL, number, or branch information in logs
- **Impact**: Cannot checkout specific PR branch for quality review

### 2. **Repository Management Issues**
- **Issue**: Multiple git repository errors and directory copy failures
- **Evidence**:







```
  fatal: not a git repository (or any parent up to mount point /)
  cp: cannot copy a directory, '/workspace/./.', into itself, '/workspace/cto-play-test/.'







```
- **Impact**: Cannot access code changes to review

### 3. **Rex→Cleo Handoff Gap**
- **Issue**: No mechanism to pass PR information from Rex to Cleo
- **Impact**: Cleo operates in isolation without context of what Rex created

## Root Cause Analysis

The current workflow assumes:


1. Rex creates a PR


2. Cleo magically knows which PR to review


3. Repository is properly initialized

**Reality**: There's no bridge between Rex completion and Cleo activation with PR context.

## Proposed Solutions

### **Option 1: Workflow Parameter Passing**




```yaml
# In play-workflow-template.yaml
- name: cleo-quality-check
  when: "{{workflow.outputs.parameters.pr-created}}"
  arguments:
    parameters:
      - name: pr-number
        value: "{{workflow.outputs.parameters.pr-number}}"
      - name: pr-url
        value: "{{workflow.outputs.parameters.pr-url}}"








```

### **Option 2: GitHub API Discovery**




```bash
# In Cleo container script
PR_NUMBER=$(gh pr list --repo $REPO --label "task-$TASK_ID" --json number --jq '.[0].number')
gh pr checkout $PR_NUMBER








```

### **Option 3: Environment Variable Injection**




```yaml
# Via Argo Events sensor when PR created
env:
  - name: TARGET_PR_NUMBER
    value: "{{.Input.body.pull_request.number}}"
  - name: TARGET_PR_URL
    value: "{{.Input.body.pull_request.html_url}}"








```

### **Option 4: Deterministic Branch Names**




```bash
# Rex uses predictable branch naming
BRANCH_NAME="task-${TASK_ID}-implementation"
# Cleo can checkout known branch
git checkout "$BRANCH_NAME"








```

### **Option 5: Marker File Approach**




```bash


# Rex leaves a marker file
echo "$PR_NUMBER" > /workspace/.pr-context
echo "$PR_URL" >> /workspace/.pr-context


# Cleo reads marker file
PR_NUMBER=$(head -1 /workspace/.pr-context)








```

## Recommended Approach

**Hybrid Solution**: Combine Options 1, 2, and 4:

1. **Deterministic Branch Naming**: Rex uses `task-{id}-implementation` branches
2. **Workflow Parameters**: Pass PR context through workflow outputs
3. **GitHub API Fallback**: Query by task labels if parameters missing
4. **Fresh Repository Clone**: Always clone clean and checkout target branch

## Implementation Priority
- **Critical**: Fix repository initialization and PR context passing
- **High**: Implement GitHub API discovery as fallback
- **Medium**: Add deterministic branch naming convention

## Current Status
- **Blocking**: Multi-agent workflow progression
- **Impact**: Rex→Cleo→Tess pipeline broken at Cleo stage
- **Needs**: Immediate fix for continued testing



## Date
2025-08-20
