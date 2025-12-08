# Acceptance Criteria: Calculator Module

## File Structure
- [ ] `src/calculator.rs` exists
- [ ] Module registered in `src/lib.rs` or `src/main.rs`

## Function Implementation
- [ ] `add(a: i32, b: i32) -> i32` implemented
- [ ] `subtract(a: i32, b: i32) -> i32` implemented
- [ ] `multiply(a: i32, b: i32) -> i32` implemented
- [ ] `divide(a: i32, b: i32) -> Result<i32, &'static str>` implemented

## Error Handling
- [ ] Division by zero returns `Err("division by zero")`

## Testing
- [ ] Unit tests exist in `#[cfg(test)]` module
- [ ] `cargo test` passes with all tests green

## Code Quality
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No compiler warnings







