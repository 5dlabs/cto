# Blocking I/O Audit

Generated: 2026-01-19  
Total std::fs calls: 141

## Summary

Blocking I/O (`std::fs`) usage has been audited. Most usages are in appropriate contexts.

## Risk Assessment

### Low Risk - Sync Code (No Async)
These files have no async functions, so blocking I/O is appropriate:

| File | std::fs calls | Risk |
|------|---------------|------|
| tools/config.rs | 12 | ✅ Low - sync config loading |
| tools/context.rs | 8 | ✅ Low - sync context management |
| controller/bin/test_templates.rs | 8 | ✅ Low - test binary |
| intake/bin/cli_comparison_test.rs | 7 | ✅ Low - test binary |
| metal/bin/metal.rs | 4 | ✅ Low - CLI binary |

### Medium Risk - Mixed Async/Sync
These files have async functions and std::fs calls:

| File | std::fs | async fn | Assessment |
|------|---------|----------|------------|
| healer/main.rs | 30 | 24 | Medium - CLI with async runtime |
| mcp/main.rs | 11 | 1 | Low - mostly sync handlers |

### Low Risk - Tests
Test files using blocking I/O:
- intake/tests/intake_tests.rs
- tools/tests/integration/real_servers/npx_servers.rs

## Detailed Analysis: healer/main.rs

The 30 `std::fs` calls in healer are used for:
1. **Config loading** (read_to_string) - Called at startup before async runtime
2. **Log writing** - Writes to local files for debugging
3. **Temp directory management** - Short-lived file operations
4. **Prompt file reading** - Template loading at startup

These operations are:
- Short-duration (single file reads/writes)
- Called infrequently (startup, on error, on completion)
- In a CLI tool where blocking is acceptable

## Recommendations

### Phase 1 (Not Required Now)
The current blocking I/O usage is acceptable for these reasons:
1. CLI tools can tolerate brief blocking
2. Operations are short-duration
3. Config loading happens before async work begins

### Phase 2 (Future - If Needed)
If healer becomes a long-running service handling many concurrent requests:
1. Replace `std::fs::read_to_string` with `tokio::fs::read_to_string`
2. Use `tokio::fs::write` for log files
3. Consider async file handles for long-running operations

### Not Recommended
- Converting test files to async (unnecessary complexity)
- Converting sync-only modules (tools/config.rs, tools/context.rs)

## Conclusion

**Status: ACCEPTABLE - No immediate action required**

The blocking I/O is used appropriately:
- Sync code uses std::fs correctly
- CLI binaries can tolerate blocking
- Long-running async operations don't hold fs locks

Future optimization should be driven by actual performance issues, not preemptive refactoring.
