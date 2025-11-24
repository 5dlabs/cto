# E2E Test Template Workflow

## Overview

This document describes how E2E test templates work in the CTO repository. The template directory is **ephemeral** and **NOT a git submodule** to avoid crosstalk issues.

## The Problem We Solved

Previously, we used git submodules for test templates, which caused "crosstalk" where test repository code would accidentally get mixed into the main CTO repository. This created confusion and required manual cleanup.

## The Solution: Ephemeral Template Directory

The template directory (`testing/cto-parallel-test/`) is treated as **regular files** that are temporary and get deleted after each use.

### Workflow

1. **Template files exist** at `testing/cto-parallel-test/` (NO `.git` directory, just regular files)
2. **Script initializes git** in that directory
3. **Script commits and pushes** to GitHub (`5dlabs/cto-parallel-test`)
4. **Script deletes** `testing/cto-parallel-test/` entirely (ephemeral cleanup)
5. **Script clones** from GitHub to the actual test location (e.g., `/Users/.../cto-parallel-test`)

### Key Points

- ✅ Template directory is **NOT** a git repository or submodule
- ✅ Template directory is **ephemeral** - deleted after push
- ✅ Fresh clone from GitHub ensures clean test environment
- ✅ No crosstalk between test code and main CTO repository
- ✅ `.gitignore` excludes `testing/cto-parallel-test/.git` as a safety measure

## Scripts

### quick-e2e-reset.sh

Simple, fast reset script for iteration:

```bash
# Reset Kubernetes only
./scripts/quick-e2e-reset.sh

# Reset Kubernetes AND GitHub repo
./scripts/quick-e2e-reset.sh --github
```

### reset-e2e-advanced.sh

Advanced script with multiple strategies:

```bash
# Default: use template strategy
./scripts/reset-e2e-advanced.sh

# Use minimal strategy (no template)
./scripts/reset-e2e-advanced.sh --strategy minimal

# Skip Kubernetes cleanup
./scripts/reset-e2e-advanced.sh --skip-k8s

# Force without confirmation
./scripts/reset-e2e-advanced.sh --force
```

## Setting Up Templates

To create or update template files:

```bash
# Create template directory
mkdir -p testing/cto-parallel-test

# Add your template files
cd testing/cto-parallel-test
# Add cto-config.json, docs/, src/, etc.

# DO NOT run 'git init' here!
# The script will handle git initialization
```

### Template Structure Example

```
testing/cto-parallel-test/
├── cto-config.json
├── docs/
│   └── .taskmaster/
│       └── docs/
│           └── prd.txt
├── src/
│   └── main.rs
└── tests/
    └── integration_test.rs
```

## Verification

To verify the template is set up correctly:

```bash
# Should return nothing (no .git directory)
ls -la testing/cto-parallel-test/.git 2>/dev/null

# Should show your template files
ls testing/cto-parallel-test/
```

## Troubleshooting

### If `.git` directory exists in template

```bash
rm -rf testing/cto-parallel-test/.git
```

### If template gets accidentally committed

The template directory should be recreated fresh each time. If it's missing, the scripts will fall back to minimal setup.

### If you see crosstalk issues

1. Verify no `.gitmodules` file exists in project root
2. Verify `testing/cto-parallel-test/` has no `.git` directory
3. Check that template directory is deleted after push (ephemeral)

## Migration Notes

**Old approach (DEPRECATED):**
- Used git submodules
- Template was a persistent git repository
- Caused crosstalk issues

**New approach (CURRENT):**
- Regular files in `testing/cto-parallel-test/`
- Template is ephemeral (deleted after push)
- Clean separation between main repo and test code

