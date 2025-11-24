# PR: Fix Play Workflow Directory Separation

## Summary

This PR fixes the play workflow to properly separate the concerns between `workingDirectory` and `docsProjectDirectory`, enabling proper support for microservices and monorepo architectures.

### Problem

Previously, the play workflow required both `workingDirectory` and `docsProjectDirectory` to point to the same location (where `.taskmaster/` is located). This was incorrect behavior that blocked several important use cases.

**What was broken:**
- The workflow template hardcoded `workingDirectory: "."` in the CodeRun CRD spec
- This meant agents always worked in the repository root, regardless of configuration
- Users had to set both parameters to the same value (where `.taskmaster/` was located)

### Solution

**Changes made:**
1. Added `working-directory` workflow parameter with default value `"."`
2. Updated CodeRun CRD spec to use `{{workflow.parameters.working-directory}}` instead of hardcoded `"."`
3. Added `working-directory` to workflow parameters output for visibility

**What was already correct:**
- Container scripts already properly separated these concerns
- `DOCS_PROJECT_DIRECTORY` is used to locate `.taskmaster/` in docs repository
- `WORK_DIR` (from `{{working_directory}}`) is used to set agent working directory

### Use Cases Now Enabled

#### 1. Single Project with Separate Docs
```json
{
  "play": {
    "workingDirectory": ".",
    "docsProjectDirectory": "docs"
  }
}
```
- Agents work in project root
- Tasks read from `docs/.taskmaster/`

#### 2. Microservices Repository
```json
{
  "play": {
    "workingDirectory": "services/api",
    "docsProjectDirectory": "docs"
  }
}
```
- Agents work in `services/api/`
- Tasks read from shared `docs/.taskmaster/`

#### 3. Monorepo with Packages
```json
{
  "play": {
    "workingDirectory": "packages/core",
    "docsProjectDirectory": "."
  }
}
```
- Agents work in `packages/core/`
- Tasks read from root `.taskmaster/`

### Backward Compatibility

✅ Fully backward compatible - existing configurations where both point to the same directory continue to work.

### Testing

- ✅ YAML structure validated
- ✅ Git diff reviewed for correctness
- ✅ Container scripts verified to already handle separation correctly
- ✅ Rust controller code verified (no changes needed)

### Files Changed

- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`
  - Added `working-directory` parameter
  - Updated CodeRun CRD spec to use parameter

### Related Documentation

See `PLAY_WORKING_DIRECTORY_ISSUE.md` for detailed problem analysis and use cases.

---

## PR Creation

**Branch:** `fix/play-working-directory-separation`

**PR URL:** https://github.com/5dlabs/cto/pull/new/fix/play-working-directory-separation

**Title:** `fix: separate workingDirectory and docsProjectDirectory in play workflow`

Copy the content above for the PR description when creating the pull request via the web interface.
