# Autonomous Agent Prompt: Integration - Level 0

## Mission
Validate that all Level 0 parallel tasks (1, 3, 4, 6) integrate without conflicts.

## Validation Steps

1. **Check for file conflicts**
```bash
git status
# Look for merge conflicts or duplicate files
```

2. **Build backend**
```bash
cargo check
cargo build
```

3. **Run backend tests**
```bash
cargo test
```

4. **Test database setup**
```bash
diesel migration run
diesel migration redo
```

5. **Build frontend**
```bash
cd frontend
npm install
npm run build
```

6. **Verify module integration**
Check that all modules can import each other without circular dependencies.

## Success Criteria
- All commands above succeed
- No conflicts or errors
- Tests pass
- Both backend and frontend build

Report any integration issues found.
