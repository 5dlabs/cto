# E2E Environment Reset Scripts

This directory contains scripts for resetting the end-to-end testing environment for the CTO platform. These scripts clean up Kubernetes resources and manage the test repository lifecycle.

## Available Scripts

### 1. `quick-e2e-reset.sh` (Fastest Option)
**Purpose:** Rapid reset for quick iteration during development  
**Usage:** 
```bash
# Basic Kubernetes cleanup only
./scripts/quick-e2e-reset.sh

# Full reset including GitHub repository
./scripts/quick-e2e-reset.sh --github
```
**Features:**
- Minimal output for speed
- Deletes all workflows, pods, test ConfigMaps, and PVCs
- Optionally recreates GitHub repository with minimal structure
- ~10 seconds execution time

### 2. `reset-e2e-environment.sh` (Standard Option)
**Purpose:** Complete environment reset with template preservation  
**Usage:**
```bash
./scripts/reset-e2e-environment.sh
```
**Features:**
- Comprehensive cleanup with detailed progress
- Saves existing repository as template on first run
- Restores from template on subsequent runs
- Interactive confirmations for safety
- Detailed summary of actions taken
- ~30 seconds execution time

### 3. `reset-e2e-advanced.sh` (Advanced Option)
**Purpose:** Flexible reset with multiple repository strategies  
**Usage:**
```bash
# Use template strategy (default)
./scripts/reset-e2e-advanced.sh

# Use Git submodule for template
./scripts/reset-e2e-advanced.sh --strategy submodule

# Create minimal structure
./scripts/reset-e2e-advanced.sh --strategy minimal

# Skip confirmations
./scripts/reset-e2e-advanced.sh --force

# Only clean Kubernetes (skip GitHub)
./scripts/reset-e2e-advanced.sh --skip-github

# Only reset GitHub (skip Kubernetes)
./scripts/reset-e2e-advanced.sh --skip-k8s
```
**Features:**
- Multiple repository initialization strategies
- Configuration file support (`e2e-reset-config.yaml`)
- Submodule support for maintainable templates
- Selective component reset
- Comprehensive PRD generation

## Repository Strategies

### Template Strategy (Default)
- **Pros:** Fast, preserves exact state, no network dependencies
- **Cons:** Template can become outdated, requires local storage
- **Use when:** You want consistent test environments across runs

### Submodule Strategy
- **Pros:** Centrally managed template, always up-to-date, version controlled
- **Cons:** Requires network access, slightly slower
- **Use when:** Multiple team members need consistent templates

### Minimal Strategy
- **Pros:** Smallest footprint, always fresh, no dependencies
- **Cons:** No complex test scenarios, requires more setup
- **Use when:** Testing basic functionality or debugging

## What Gets Reset

### Kubernetes Resources (namespace: `cto`)
- ✅ All Argo Workflows
- ✅ All Pods
- ✅ Test-related ConfigMaps (play-*, test-*, coderun-*, docsrun-*)
- ✅ Test-related PVCs (workspace-play-*, workspace-test-*)
- ❌ System ConfigMaps (preserved)
- ❌ System Secrets (preserved)
- ❌ Workflow Templates (preserved)

### GitHub Repository
- ✅ Complete repository deletion and recreation
- ✅ Fresh git history
- ✅ Clean main branch
- ✅ Proper remote configuration

### Local Repository
- ✅ Complete directory removal and recreation
- ✅ Fresh git initialization
- ✅ Template restoration or minimal setup

## Prerequisites

All scripts require:
- `kubectl` configured with cluster access
- `gh` (GitHub CLI) authenticated
- Appropriate permissions for:
  - Kubernetes namespace operations
  - GitHub repository management
  - Local filesystem operations

## Configuration

The advanced script uses `e2e-reset-config.yaml` for configuration:

```yaml
kubernetes:
  namespace: cto
  cleanup:
    workflows: true
    pods: true
    configmaps: true
    pvcs: true

github:
  org: 5dlabs
  repo: cto-parallel-test
  visibility: private

repo_strategy: template  # template, submodule, or minimal
```

## Template Repository Structure

When using template strategy, the following structure is preserved:

```
cto-parallel-test-template/
├── cto-config.json           # CTO configuration
├── docs/
│   └── .taskmaster/
│       └── docs/
│           └── test-prd.txt   # Test PRD document
├── src/                       # Source code (if any)
├── tests/                     # Test files (if any)
└── .gitignore                # Git ignore rules
```

## Typical Workflow

1. **Before Testing:**
   ```bash
   # Quick reset for clean slate
   ./scripts/quick-e2e-reset.sh --github
   ```

2. **Run E2E Test:**
   ```bash
   # Trigger CTO workflow
   cto play --task-id 1
   ```

3. **Monitor Progress:**
   ```bash
   # Watch workflow logs
   kubectl logs -f -l workflow -n cto
   
   # Check workflow status
   kubectl get workflows -n cto
   ```

4. **After Test Failure:**
   ```bash
   # Quick cleanup without GitHub reset
   ./scripts/quick-e2e-reset.sh
   
   # Fix issues and rerun
   ```

5. **Complete Reset:**
   ```bash
   # Full reset with template preservation
   ./scripts/reset-e2e-environment.sh
   ```

## Troubleshooting

### "Repository already exists" Error
- The script will prompt to delete existing repository
- Use `--force` flag to skip confirmation
- Or manually delete: `gh repo delete 5dlabs/cto-parallel-test --yes`

### "Permission denied" Errors
- Ensure GitHub CLI is authenticated: `gh auth status`
- Check Kubernetes permissions: `kubectl auth can-i delete pods -n cto`

### Template Not Found
- First run creates template automatically
- Or manually create: `cp -r /path/to/repo /path/to/repo-template`
- Script falls back to minimal strategy if template missing

### Submodule Issues
- Ensure submodule repository exists and is accessible
- Update submodule: `git submodule update --remote --merge`
- Check `.gitmodules` file for correct configuration

## Performance Considerations

| Script | Kubernetes Cleanup | GitHub Reset | Total Time |
|--------|-------------------|--------------|------------|
| quick-e2e-reset.sh | ~5s | ~5s | ~10s |
| reset-e2e-environment.sh | ~10s | ~20s | ~30s |
| reset-e2e-advanced.sh (template) | ~10s | ~20s | ~30s |
| reset-e2e-advanced.sh (submodule) | ~10s | ~30s | ~40s |

## Safety Features

1. **Confirmation Prompts:** All destructive operations require confirmation
2. **Template Preservation:** Original state saved before first deletion
3. **Namespace Isolation:** Only affects `cto` namespace
4. **Pattern Matching:** Only deletes resources matching specific patterns
5. **Force Grace Period:** Ensures immediate resource cleanup

## Best Practices

1. **Use Quick Reset During Development:** For rapid iteration when debugging
2. **Use Standard Reset Between Test Runs:** For consistent test environments
3. **Use Advanced Reset for Team Collaboration:** When using shared templates
4. **Always Check Status First:** Run `kubectl get all -n cto` before reset
5. **Preserve Important Data:** Export any needed logs or configs before reset
6. **Version Control Templates:** Keep test templates in separate repository for team sharing

## Integration with CI/CD

These scripts can be integrated into CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Reset E2E Environment
  run: ./scripts/quick-e2e-reset.sh --github
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    
- name: Run E2E Tests
  run: cto play --task-id 1

- name: Cleanup
  if: always()
  run: ./scripts/quick-e2e-reset.sh
```

## Contributing

When modifying these scripts:
1. Test changes in non-production environment first
2. Update this README with any new features or options
3. Ensure backwards compatibility
4. Add error handling for new failure modes
5. Update configuration examples if needed

## Related Documentation

- [CTO Platform Documentation](../docs/README.md)
- [Agent Workflow Guide](../docs/architecture.md)
- [Kubernetes Operations](../infra/README.md)
- [GitHub Integration](../docs/github-integration.md)
