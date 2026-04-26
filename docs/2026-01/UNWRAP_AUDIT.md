# unwrap() Usage Audit

Generated: 2026-01-19  
Total unwrap() calls: ~700

## Summary

The majority of `unwrap()` calls are in acceptable contexts:
- Test code (should panic on failure)
- Compile-time regex patterns (programming error if invalid)
- Static initialization (should fail fast at startup)

## Categories

### 1. Test Code - Acceptable (400+)
Files with high unwrap() count are mostly tests:

| File | Count | Context |
|------|-------|---------|
| healer/tests/play_integration.rs | 69 | Test file |
| intake/src/storage/file.rs | 25 | Test module |
| pm/src/handlers/intake.rs | 31 | Test module |
| intake/tests/intake_tests.rs | 13 | Test file |

In tests, `unwrap()` is idiomatic - test failures should panic.

### 2. Regex::new().unwrap() - Acceptable (67)
Static regex patterns in `LazyLock`:

| File | Count | Pattern |
|------|-------|---------|
| healer/src/ci/router.rs | 45 | Static pattern matching |
| healer/src/scanner.rs | 16 | Log pattern detection |
| Others | 6 | Various static patterns |

These are compile-time constant patterns. If invalid, they should panic at startup (programming error).

### 3. JSON Parsing in Tests - Acceptable
```rust
let tasks: TasksJson = serde_json::from_str(json).unwrap();
```
These parse known-good JSON literals in test code.

### 4. CLI Adapters - Low Risk (varies)
Files like `adapters/claude.rs`, `adapters/gemini.rs` have unwrap() in:
- Regex patterns for parsing output
- Optional field access with fallbacks

These are low-risk as they:
- Parse known CLI output formats
- Have fallback handling nearby
- Are not user-input parsing

## Recommendations

### Phase 1 (Complete) - Documentation
Document the current state and acceptable patterns.

### Phase 2 (Future) - Consider Converting
Lower priority items that could be improved:

1. **Regex patterns**: Convert to `.expect("regex is valid")` for better error messages
2. **Option access**: Review `.unwrap()` on Option types outside tests
3. **Error chain**: Consider `anyhow` context in error-heavy paths

### Not Recommended
- Converting test code unwrap() - idiomatic in tests
- Removing compile-time regex unwrap() - correct behavior
- Adding error handling where panic is appropriate

## Conclusion

**Status: ACCEPTABLE - Low risk**

The codebase follows Rust conventions:
- Tests use unwrap() for clarity
- Compile-time patterns use unwrap() (correct behavior)
- Production error paths use proper error handling (Result types)

No high-risk unwrap() patterns found in user-facing code paths.
