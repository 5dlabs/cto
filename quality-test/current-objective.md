# Objective: CQ-005 - Address Cast Safety Issues

Address the 51 instances of cast safety Clippy allows in the codebase.

## Target Files
- `crates/healer/src/*.rs` - Most cast issues
- `crates/controller/src/*.rs`
- `crates/pm/src/*.rs`

## Strategy
1. Review each `#[allow(clippy::cast_possible_truncation)]` etc.
2. For safe casts: Add reason comment explaining why it's safe
3. For unsafe casts: Use `try_from()` or add bounds checking
4. For API-required casts: Document the constraint

## Lint Types to Address
- `cast_possible_truncation` - Casting to smaller type
- `cast_precision_loss` - Casting float to int
- `cast_possible_wrap` - Signed/unsigned conversions
- `cast_sign_loss` - Signed to unsigned

## Verification Commands
```bash
# Count remaining cast allows
rg '#\[allow\(clippy::cast_' crates --count-matches

# Verify Clippy passes
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --all --lib
```

## Gates
- [ ] All cast allows have reason comments OR are fixed
- [ ] cargo clippy --all-targets -- -D warnings passes
- [ ] cargo test --all passes

## Evidence
- Record changes in quality-test/progress.txt
- Commit and push after completion
