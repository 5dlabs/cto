# Expected Behaviors: Tess (Testing Agent)

## Success Patterns
```
✅ test result: ok
✅ \d+ passed
✅ 0 failed
✅ All tests passed
✅ cargo test
✅ npm test
✅ pytest
✅ Tests complete
```

## Failure Indicators
```
❌ test result: FAILED
❌ FAILED
❌ \d+ failed
❌ panicked at
❌ thread .* panicked
❌ assertion failed
❌ left: .* right:
❌ error\[E
❌ cannot find
```

## What to Verify
1. Did all tests pass?
2. Were there any panics or crashes?
3. Did Tess actually run the test suite?
4. Are there test compilation errors?
5. Did Tess approve despite failures? (BUG!)

