---
name: cargo-fuzz
description: Fuzzing Rust projects with cargo-fuzz and libFuzzer for finding bugs through automated input generation.
---

# cargo-fuzz

cargo-fuzz is the de facto choice for fuzzing Rust projects using Cargo. It uses libFuzzer as the backend and provides automatic sanitizer support including AddressSanitizer.

## When to Use

**Choose cargo-fuzz when:**
- Your project uses Cargo (required)
- You want simple, quick setup with minimal configuration
- You need integrated sanitizer support
- You're fuzzing Rust code with or without unsafe blocks

| Fuzzer | Best For | Complexity |
|--------|----------|------------|
| cargo-fuzz | Cargo-based Rust projects, quick setup | Low |
| AFL++ | Multi-core fuzzing, non-Cargo projects | Medium |
| LibAFL | Custom fuzzers, research, advanced use cases | High |

## Installation

cargo-fuzz requires the nightly Rust toolchain.

```bash
# Install nightly toolchain
rustup install nightly

# Install cargo-fuzz
cargo install cargo-fuzz

# Verify
cargo +nightly fuzz --version
```

## Quick Start

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fn harness(data: &[u8]) {
    your_project::check_buf(data);
}

fuzz_target!(|data: &[u8]| {
    harness(data);
});
```

Initialize and run:

```bash
cargo fuzz init
# Edit fuzz/fuzz_targets/fuzz_target_1.rs with your harness
cargo +nightly fuzz run fuzz_target_1
```

## Writing a Harness

### Project Structure

cargo-fuzz works best when your code is structured as a library crate:

```
src/
  main.rs    # Entry point (main function)
  lib.rs     # Code to fuzz (public functions)
Cargo.toml
fuzz/
  Cargo.toml
  fuzz_targets/
    fuzz_target_1.rs
```

### Harness Structure

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fn harness(data: &[u8]) {
    // 1. Validate input size if needed
    if data.is_empty() {
        return;
    }
    
    // 2. Call target function with fuzz data
    your_project::target_function(data);
}

fuzz_target!(|data: &[u8]| {
    harness(data);
});
```

### Harness Rules

| Do | Don't |
|----|-------|
| Structure code as library crate | Keep everything in main.rs |
| Use `fuzz_target!` macro | Write custom main function |
| Handle `Result::Err` gracefully | Panic on expected errors |
| Keep harness deterministic | Use random number generators |

## Structure-Aware Fuzzing

Use the `arbitrary` crate for structure-aware fuzzing:

```rust
// In your library crate
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
pub struct Config {
    pub timeout: u32,
    pub retries: u8,
    pub data: Vec<u8>,
}
```

```rust
// In your fuzz target
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|config: your_project::Config| {
    your_project::process_config(config);
});
```

Add to your library's `Cargo.toml`:

```toml
[dependencies]
arbitrary = { version = "1", features = ["derive"] }
```

## Running Campaigns

### Basic Run

```bash
cargo +nightly fuzz run fuzz_target_1
```

### Without Sanitizers (Safe Rust)

If your project doesn't use unsafe Rust, disable sanitizers for 2x performance:

```bash
cargo +nightly fuzz run --sanitizer none fuzz_target_1
```

Check if your project uses unsafe code:

```bash
cargo install cargo-geiger
cargo geiger
```

### Using Dictionaries

```bash
cargo +nightly fuzz run fuzz_target_1 -- -dict=./dict.dict
```

### libFuzzer Options

```bash
# See all options
cargo +nightly fuzz run fuzz_target_1 -- -help=1

# Set timeout per run
cargo +nightly fuzz run fuzz_target_1 -- -timeout=10

# Limit maximum input size
cargo +nightly fuzz run fuzz_target_1 -- -max_len=1024
```

### Interpreting Output

| Output | Meaning |
|--------|---------|
| `NEW` | New coverage-increasing input discovered |
| `pulse` | Periodic status update |
| `INITED` | Fuzzer initialized successfully |
| Crash with stack trace | Bug found, saved to `fuzz/artifacts/` |

- Corpus location: `fuzz/corpus/fuzz_target_1/`
- Crashes location: `fuzz/artifacts/fuzz_target_1/`

## Coverage Analysis

### Prerequisites

```bash
rustup toolchain install nightly --component llvm-tools-preview
cargo install cargo-binutils
cargo install rustfilt
```

### Generating Coverage Reports

```bash
# Generate coverage data from corpus
cargo +nightly fuzz coverage fuzz_target_1
```

## Tips and Tricks

| Tip | Why It Helps |
|-----|--------------|
| Start with a seed corpus | Dramatically speeds up initial coverage discovery |
| Use `--sanitizer none` for safe Rust | 2x performance improvement |
| Check coverage regularly | Identifies gaps in harness or seed corpus |
| Use dictionaries for parsers | Helps overcome magic value checks |
| Structure code as library | Required for cargo-fuzz integration |

## Troubleshooting

| Problem | Cause | Solution |
|---------|-------|----------|
| "requires nightly" error | Using stable toolchain | Use `cargo +nightly fuzz` |
| Slow fuzzing performance | ASan enabled for safe Rust | Add `--sanitizer none` flag |
| "cannot find binary" | No library crate | Move code from `main.rs` to `lib.rs` |
| Low coverage | Missing seed corpus | Add sample inputs to `fuzz/corpus/` |
| Magic value not found | No dictionary | Create dictionary file with magic values |

## Example: Parsing Library

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let mut reader = Cursor::new(data.to_vec());
    
    // Parse the data - don't panic on expected errors
    if let Ok(parsed) = my_parser::parse(&mut reader) {
        // Optionally re-serialize to check roundtrip
        let _ = my_parser::serialize(&parsed);
    }
});
```

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) cargo-fuzz skill.
