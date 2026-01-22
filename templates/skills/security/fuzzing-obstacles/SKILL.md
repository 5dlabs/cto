# Overcoming Fuzzing Obstacles

Codebases often contain anti-fuzzing patterns that prevent effective coverage. Checksums, global state (like time-seeded PRNGs), and validation checks can block the fuzzer from exploring deeper code paths.

## Overview

Many real-world programs were not designed with fuzzing in mind. They may:

- Verify checksums or cryptographic hashes before processing input
- Rely on global state (e.g., system time, environment variables)
- Use non-deterministic random number generators
- Perform complex validation that makes it difficult for the fuzzer to generate valid inputs

The solution is conditional compilation: modify code behavior during fuzzing builds while keeping production code unchanged.

## When to Apply

**Apply this technique when:**

- The fuzzer gets stuck at checksum or hash verification
- Coverage reports show large blocks of unreachable code behind validation
- Code uses time-based seeds or other non-deterministic global state
- Complex validation makes it nearly impossible to generate valid inputs

**Skip this technique when:**

- The obstacle can be overcome with a good seed corpus or dictionary
- The validation is simple enough for the fuzzer to learn (e.g., magic bytes)
- Skipping the check would introduce too many false positives

## Quick Reference

| Task | C/C++ | Rust |
| --- | --- | --- |
| Check if fuzzing build | `#ifdef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION` | `cfg!(fuzzing)` |
| Skip check during fuzzing | `#ifndef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION return -1; #endif` | `if !cfg!(fuzzing) { return Err(...) }` |
| Common obstacles | Checksums, PRNGs, time-based logic | Checksums, PRNGs, time-based logic |
| Supported fuzzers | libFuzzer, AFL++, LibAFL, honggfuzz | cargo-fuzz, libFuzzer |

## Step-by-Step

### Step 1: Identify the Obstacle

Run the fuzzer and analyze coverage to find code that's unreachable. Common patterns:

1. Look for checksum/hash verification before deeper processing
2. Check for calls to `rand()`, `time()`, or `srand()` with system seeds
3. Find validation functions that reject most inputs
4. Identify global state initialization that differs across runs

### Step 2: Add Conditional Compilation

**C/C++ Example:**

```c++
// Before: Hard obstacle
if (checksum != expected_hash) {
    return -1;  // Fuzzer never gets past here
}

// After: Conditional bypass
if (checksum != expected_hash) {
#ifndef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION
    return -1;  // Only enforced in production
#endif
}
```

**Rust Example:**

```rust
// Before: Hard obstacle
if checksum != expected_hash {
    return Err(MyError::Hash);
}

// After: Conditional bypass
if checksum != expected_hash {
    if !cfg!(fuzzing) {
        return Err(MyError::Hash);
    }
}
```

### Step 3: Verify Coverage Improvement

After patching:

1. Rebuild with fuzzing instrumentation
2. Run the fuzzer for a short time
3. Compare coverage to the unpatched version
4. Confirm new code paths are being explored

## Common Patterns

### Pattern: Bypass Checksum Validation

```c++
uint32_t computed = hash_function(data, size);
if (computed != expected_checksum) {
#ifndef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION
    return ERROR_INVALID_HASH;
#endif
}
process_data(data, size);
```

### Pattern: Deterministic PRNG Seeding

```c++
void initialize() {
#ifdef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION
    srand(12345);  // Fixed seed for fuzzing
#else
    srand(time(NULL));
#endif
}
```

### Pattern: Careful Validation Skip

```c++
#ifndef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION
if (!validate_config(&config)) {
    return -1;
}
#else
// During fuzzing, use safe defaults for failed validation
if (!validate_config(&config)) {
    config.x = 1;  // Prevent division by zero
    config.y = 1;
}
#endif
```

## Tool-Specific Guidance

### libFuzzer

libFuzzer automatically defines `FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION` during compilation.

```bash
clang++ -g -fsanitize=fuzzer,address harness.cc target.cc -o fuzzer
```

### AFL++

AFL++ also defines `FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION` when using its compiler wrappers.

```bash
afl-clang-fast++ -g -fsanitize=address target.cc harness.cc -o fuzzer
```

### cargo-fuzz (Rust)

cargo-fuzz automatically sets the `fuzzing` cfg option during builds.

```bash
cargo fuzz build fuzz_target_name
cargo fuzz run fuzz_target_name
```

## Anti-Patterns

| Anti-Pattern | Problem | Correct Approach |
| --- | --- | --- |
| Skip all validation wholesale | Creates false positives | Skip only specific obstacles |
| No risk assessment | False positives waste time | Analyze downstream code for assumptions |
| Forget to document patches | Future maintainers don't understand | Add comments explaining why patch is safe |
| Patch without measuring | Don't know if it helped | Compare coverage before and after |
