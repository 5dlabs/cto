# Test Task: Create a Simple Calculator Module

## Mission
You are a Rust developer. Create a simple calculator module with basic arithmetic operations.

## Goal
Create a Rust module `src/calculator.rs` with functions for add, subtract, multiply, and divide.

## Requirements

### 1. Create Calculator Module
Create `src/calculator.rs` with these functions:

```rust
pub fn add(a: i32, b: i32) -> i32
pub fn subtract(a: i32, b: i32) -> i32
pub fn multiply(a: i32, b: i32) -> i32
pub fn divide(a: i32, b: i32) -> Result<i32, &'static str>
```

### 2. Handle Division by Zero
The `divide` function must return `Err("division by zero")` when `b` is 0.

### 3. Add Tests
Create unit tests in the same file using `#[cfg(test)]` module.

### 4. Register Module
Add `pub mod calculator;` to `src/lib.rs` or `src/main.rs`.

## Success Criteria
- [ ] `src/calculator.rs` exists
- [ ] All 4 functions implemented
- [ ] Division by zero handled with Result
- [ ] Unit tests included
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Validation Commands
```bash
cargo test
cargo clippy -- -D warnings
```

## Deliverables
- `src/calculator.rs`
- Updated `src/lib.rs` or `src/main.rs`






