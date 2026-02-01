---
name: coverage-analysis
description: Fuzzing coverage analysis - identify uncovered code, magic values, and track campaign effectiveness.
---

# Coverage Analysis

Coverage analysis for understanding which parts of code are exercised during fuzzing.

## Overview

Code coverage during fuzzing serves two critical purposes:

1. **Assessing harness effectiveness**: Which parts are actually executed by fuzzing harnesses
2. **Tracking fuzzing progress**: How coverage changes when updating harnesses or fuzzers

## When to Apply

**Apply this technique when:**
- Starting a new fuzzing campaign to establish a baseline
- Fuzzer appears to plateau without finding new paths
- After harness modifications to verify improvements
- When migrating between different fuzzers
- Identifying areas requiring dictionary entries or seed inputs
- Debugging why certain code paths aren't reached

**Skip this technique when:**
- Fuzzing campaign is actively finding crashes
- Coverage infrastructure isn't set up yet
- Fuzzer's internal coverage metrics are sufficient

## Quick Reference

| Task | Command/Pattern |
|------|-----------------|
| LLVM coverage (C/C++) | `-fprofile-instr-generate -fcoverage-mapping` |
| GCC coverage | `-ftest-coverage -fprofile-arcs` |
| cargo-fuzz coverage (Rust) | `cargo +nightly fuzz coverage <target>` |
| Generate LLVM profile | `llvm-profdata merge -sparse file.profraw -o file.profdata` |
| LLVM coverage report | `llvm-cov report ./binary -instr-profile=file.profdata` |
| LLVM HTML report | `llvm-cov show ./binary -instr-profile=file.profdata -format=html -output-dir html/` |
| gcovr HTML report | `gcovr --html-details -o coverage.html` |

## Ideal Coverage Workflow

```
[Fuzzing Campaign]
        |
        v
[Generate Corpus]
        |
        v
[Coverage Analysis]
        |
        +---> Coverage Increased? --> Continue fuzzing
        |
        +---> Coverage Decreased? --> Fix harness or investigate changes
        |
        +---> Coverage Plateaued? --> Add dictionary entries or seed inputs
```

## Rust: cargo-fuzz Coverage

```bash
# Install prerequisites
rustup toolchain install nightly --component llvm-tools-preview
cargo install cargo-binutils rustfilt

# Generate coverage data
cargo +nightly fuzz coverage fuzz_target_1

# Create HTML report script
cat <<'EOF' > ./generate_html
#!/bin/sh
FUZZ_TARGET="$1"
shift
SRC_FILTER="$@"
TARGET=$(rustc -vV | sed -n 's|host: ||p')
cargo +nightly cov -- show -Xdemangler=rustfilt \
  "target/$TARGET/coverage/$TARGET/release/$FUZZ_TARGET" \
  -instr-profile="fuzz/coverage/$FUZZ_TARGET/coverage.profdata" \
  -show-line-counts-or-regions -show-instantiations \
  -format=html -o fuzz_html/ $SRC_FILTER
EOF
chmod +x ./generate_html

# Generate report
./generate_html fuzz_target_1 src/lib.rs
```

## C/C++: LLVM Coverage

```bash
# Build with coverage instrumentation
clang++ -fprofile-instr-generate -fcoverage-mapping \
  -O2 -DNO_MAIN \
  main.cc harness.cc execute-rt.cc -o fuzz_exec

# Execute on corpus
LLVM_PROFILE_FILE=fuzz.profraw ./fuzz_exec corpus/

# Process and generate report
llvm-profdata merge -sparse fuzz.profraw -o fuzz.profdata

llvm-cov show ./fuzz_exec \
  -instr-profile=fuzz.profdata \
  -ignore-filename-regex='harness.cc|execute-rt.cc' \
  -format=html -output-dir fuzz_html/
```

## Common Patterns

### Identifying Magic Values

**Coverage reveals:**
```c
// Coverage shows this block is never executed
if (buf == 0x7F454C46) {  // ELF magic number
    // start parsing buf
}
```

**Solution**: Add magic values to dictionary file:
```
# magic.dict
"\x7F\x45\x4C\x46"
```

## Anti-Patterns

| Anti-Pattern | Problem | Correct Approach |
|--------------|---------|------------------|
| Using fuzzer-reported coverage for comparisons | Different fuzzers calculate coverage differently | Use dedicated coverage tools |
| Generating coverage with -O3 | Optimizations eliminate code | Use -O2 or -O0 |
| Not filtering harness code | Harness inflates numbers | Use `-ignore-filename-regex` |
| Ignoring crashing inputs | Crashes prevent coverage generation | Fix crashes first or use process forking |

## Tips

| Tip | Why It Helps |
|-----|--------------|
| Use LLVM 18+ with `-show-directory-coverage` | Organizes large reports by directory |
| Export to lcov format for better HTML | `llvm-cov export -format=lcov` + `genhtml` |
| Compare coverage across campaigns | Store `.profdata` files with timestamps |
| Filter harness code from reports | Focus on SUT coverage only |
| Automate coverage in CI/CD | Generate reports after scheduled fuzzing runs |

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) coverage-analysis skill - 45+ installs.
