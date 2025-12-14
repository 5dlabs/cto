# E2E Test Outputs

This directory contains the results of running actual agents against real CLIs.

## Directory Structure

After running tests, each agent/CLI combination will have:

```
outputs/
├── rex-claude/
│   ├── run-info.json      # Metadata (timestamps, status, etc.)
│   ├── code/              # Generated source code
│   │   ├── src/
│   │   │   └── http/
│   │   │       └── client.rs
│   │   ├── Cargo.toml
│   │   └── ...
│   ├── code.patch         # Git diff of all changes
│   └── logs/
│       └── stdout.log     # Full container output
├── blaze-claude/
│   ├── code/
│   │   └── src/
│   │       └── components/
│   │           └── DataTable/
│   └── ...
└── ...
```

## Running Tests

```bash
# Run a single agent/CLI combination
../scripts/run-single-task.sh rex claude

# Run all combinations (takes hours, costs $$$)
../scripts/run-matrix.sh
```

## What Gets Captured

1. **Generated Code** (`code/`) - All files the agent created/modified
2. **Git Diff** (`code.patch`) - Unified diff of all changes
3. **Logs** (`logs/stdout.log`) - Full container output including:
   - Environment setup
   - CLI invocation
   - Agent thinking/reasoning
   - Tool calls
   - Final commit

## Note

These outputs are gitignored because they're large and auto-generated.
Re-run the tests to regenerate them.

