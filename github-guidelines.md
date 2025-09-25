# GitHub Workflow Guidelines

## ⛔ CRITICAL: YOUR TASK WILL FAIL WITHOUT A PULL REQUEST ⛔

**THE CONTAINER WILL AUTOMATICALLY FAIL IF YOU DON'T CREATE A PR**

This is not a suggestion - it's enforced. The container checks for PR creation and exits with failure if none exists.

You MUST run `gh pr create` before your work is complete. Just pushing commits is NOT enough.

## �� **MANDATORY BRANCH AND PR REQUIREMENTS** 🚨

**YOU MUST COMMIT REGULARLY AND SUBMIT A PR WHEN IMPLEMENTATION IS COMPLETE**

### **Critical Requirements:**

- ⭐ **COMMIT AND PUSH FREQUENTLY** - Ideally after every significant change or turn
- ⭐ **SUBMIT A PULL REQUEST** when implementation meets all acceptance criteria
- ⭐ **NEVER PUSH TO MAIN BRANCH** - Always work on your feature branch only
- ⭐ **USE GITHUB APP AUTHENTICATION** - All git operations use GitHub App tokens (already configured)

## Git Workflow

### Your Current Context
- **Repository**: 
- **Feature Branch**: feature/task--implementation
- **Target Branch**: main (never push directly to this)
- **Authentication**: GitHub App (5DLabs-Cleo - pre-configured)

### **Required Git Pattern:**

```bash
# After making changes, always commit and push to feature branch:
git add .
git commit -m "feat: implement [specific change made]"
git push origin feature/task--implementation
```

### **When to Commit & Push:**
- ✅ After implementing a significant feature or fix
- ✅ After completing a subtask or milestone
- ✅ When you've made meaningful progress (ideally every turn)
- ✅ Before running tests or verification steps
- ✅ When switching between different areas of the codebase

### **Commit Message Format:**
```
<type>: <brief description of what was implemented>

Examples:
feat: add user authentication endpoint
fix: resolve database connection timeout
refactor: extract validation logic to helpers
test: add unit tests for payment processing
```

## 🔄 **Merge Conflict Prevention & Resolution**

### **Prevention (Automated in Container Script):**
The container automatically syncs with main before you start work:
```bash
# This happens automatically for you:
git fetch origin main
git merge origin/main --no-edit  # Auto-merge if possible
```

### **⚠️ Manual Resolution Required (If Auto-Merge Fails):**

**If you see merge conflict warnings during startup or at any time:**

1. **Check conflict status:**
   ```bash
   git status
   # Look for "Unmerged paths" or files marked with "UU", "AA", or "DD"
   ```

2. **Identify conflicted files:**
   ```bash
   # Files with merge conflicts will show:
   # - <<<<<<< HEAD (your changes)
   # - ======= (separator)
   # - >>>>>>> origin/main (main branch changes)
   ```

3. **Resolve conflicts manually:**
   - Edit each conflicted file
   - Remove conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
   - Keep the correct combination of changes
   - Save the file

4. **Complete the merge:**
   ```bash
   git add .                           # Stage resolved files
   git commit -m "Resolve merge conflicts with main"
   git push origin feature/task--implementation          # Push resolution
   ```

### **Best Practices:**
- ✅ **Always resolve conflicts immediately** - Don't ignore them
- ✅ **Test after resolving** - Ensure your changes still work
- ✅ **Ask for clarification** if unsure which changes to keep
- ✅ **Sync frequently** - Smaller conflicts are easier to resolve

### **If Stuck on Conflicts:**
Comment in your PR: "Need help resolving merge conflicts in [file names]" and describe what you're unsure about.

## **🚨 PULL REQUEST SUBMISSION - MANDATORY FOR TASK COMPLETION 🚨**

**⛔ CRITICAL: THE TASK IS NOT COMPLETE UNTIL YOU CREATE A PULL REQUEST. NO EXCEPTIONS. ⛔**

### **FAILURE CONDITIONS - If any of these occur, YOU HAVE FAILED:**
- ❌ Exiting without running `gh pr create`
- ❌ Getting error from `gh pr create` and not fixing it
- ❌ Assuming PR creation is optional
- ❌ Thinking pushing commits is enough (IT IS NOT)
- ❌ Stopping work after implementation without PR

When you have completed implementation and met all acceptance criteria, and ONLY after all pre-PR quality gates are green locally:

### ✅ Pre-PR Quality Gates (must pass locally)
```bash
# Formatting
cargo fmt --all -- --check

# Clippy with pedantic and deny warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic

# Tests and coverage (aim for ≥95%, target ~100% on critical paths)
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 95 || \
  cargo tarpaulin --all --fail-under 95
```

### **✅ MANDATORY: Submit a Pull Request Using GitHub CLI:**

**⚠️ PR CREATION TROUBLESHOOTING:**
If `gh pr create` fails, here's how to fix common issues:

```bash
# Issue: "no commits between main and feature branch"
# Fix: Make sure you've committed and pushed changes
git status  # Check for uncommitted changes
git add .
git commit -m "feat: your changes"
git push origin feature/task--implementation

# Issue: "pull request already exists"
# Fix: Update the existing PR instead
gh pr list --head feature/task--implementation
gh pr view [PR_NUMBER]  # View existing PR

# Issue: "branch not found on remote"
# Fix: Push the branch first
git push -u origin feature/task--implementation

# Issue: "authentication failed"
# Fix: The container should have auth configured, but verify:
gh auth status
```

**NOW CREATE THE PR:**
```bash
# This command is REQUIRED - the task is not done without it
gh pr create --title "feat: [brief summary of implementation]" \
             --body "## Implementation Summary
[Brief description of what was implemented]

## Changes Made
- [List key changes]
- [New features added]
- [Bug fixes implemented]

## Testing Performed
- [Tests written/updated]
- [Manual testing completed]
- [Verification steps]

## Notes
- [Any important technical decisions]
- [Performance/security considerations]"

# VERIFY THE PR WAS CREATED:
gh pr list --head feature/task--implementation
# You should see your PR listed - if not, YOU HAVE FAILED
```

### **✅ PR Requirements:**
- Create PR from your feature branch (feature/task--implementation) to main
- Use descriptive title starting with feat:, fix:, etc.
- Include comprehensive PR description with all sections above
- **CRITICAL**: You MUST run the `gh pr create` command - just pushing is not enough

### **❌ NEVER Push to Main:**
- ❌ **DO NOT** push directly to main branch
- ❌ **DO NOT** merge your own PR
- ✅ **ONLY** work on feature branch feature/task--implementation

## Authentication

### GitHub App Configuration
- GitHub App authentication is pre-configured in the container
- All git operations use GitHub App tokens automatically
- Repository access: ``
- GitHub App: `5DLabs-Cleo`

### Git Commands (GitHub App-based)
```bash
# Check current status
git status

# Stage changes
git add .

# Commit with message
git commit -m "feat: describe your change"

# Push to feature branch (GitHub App authentication automatic)
git push origin feature/task--implementation

# Create pull request (when implementation complete)
gh pr create --title "feat: [summary]" --body "[detailed description]"

# Check git log
git log --oneline -10
```

### **Gitignore Requirements**
- ⭐ **ALWAYS add hooks to .gitignore** - Never commit hook files
- Add these patterns to your .gitignore:
  ```
  # Hook files - never commit
  hooks/
  .hooks/
  **/hooks/
  ```

## Progress Tracking Philosophy

**The goal is continuous visibility and proper PR submission:**

1. **Frequent commits** help track your thought process
2. **Regular pushes** keep the team informed of progress
3. **Clear commit messages** document your implementation decisions
4. **PR submission** provides proper code review process

## **🚨 TASK COMPLETION CHECKLIST - ALL STEPS MANDATORY 🚨**

**A task is ONLY complete when ALL these steps are done:**

1. ✅ Implementation meets all acceptance criteria
2. ✅ Final commit with all changes: `git add . && git commit -m "..."`
3. ✅ Push to feature branch: `git push origin feature/task--implementation`
4. 🚨 **MANDATORY**: Create pull request: `gh pr create --title "..." --body "..."`
5. ❌ **NEVER** push to main branch

**WITHOUT STEP 4, THE TASK IS INCOMPLETE - NO EXCEPTIONS**

### **PR Description Template:**
```markdown
## Implementation Summary
Brief description of what was implemented and why.

## Changes Made
- List of significant changes
- New features added
- Bug fixes implemented
- Refactoring completed

## Testing Performed
- Unit tests written/updated
- Integration testing completed
- Manual testing performed
- Edge cases verified

## Implementation Notes
- Any important technical decisions
- Performance considerations
- Security implications
- Breaking changes (if any)
```

---

**Remember: Your feature branch (feature/task--implementation) is your workspace. Keep it updated with regular commits, then submit a comprehensive PR when implementation is complete!**
