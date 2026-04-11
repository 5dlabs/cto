---
name: codeql
description: CodeQL static analysis for interprocedural data flow and taint tracking across codebases.
---

# CodeQL Static Analysis

## When to Use CodeQL

**Ideal scenarios:**
- Source code access with ability to build (for compiled languages)
- Open-source projects or GitHub Advanced Security license
- Need for interprocedural data flow and taint tracking
- Finding complex vulnerabilities requiring AST/CFG analysis
- Comprehensive security audits where analysis time is not critical

**Consider Semgrep instead when:**
- No build capability for compiled languages
- Licensing constraints
- Need fast, lightweight pattern matching
- Simple, single-file analysis is sufficient

### Why Interprocedural Analysis Matters

Simple grep/pattern tools only see one function at a time. Real vulnerabilities often span multiple functions:

```
HTTP Handler → Input Parser → Business Logic → Database Query
     ↓              ↓              ↓              ↓
   source      transforms       passes        sink (SQL)
```

CodeQL tracks data flow across all these steps. A tainted input in the handler can be traced through 5+ function calls to find where it reaches a dangerous sink.

## Installation

### CodeQL CLI

```bash
# macOS/Linux (Homebrew)
brew install --cask codeql

# Update
brew upgrade codeql
```

### Trail of Bits Queries (Optional)

```bash
# Download ToB query packs
codeql pack download trailofbits/cpp-queries trailofbits/go-queries

# Verify installation
codeql resolve qlpacks | grep trailofbits
```

## Core Workflow

### 1. Create Database

```bash
codeql database create codeql.db --language=<LANG> [--command='<BUILD>'] --source-root=.
```

| Language | `--language=` | Build Required |
|----------|---------------|----------------|
| Python | `python` | No |
| JavaScript/TypeScript | `javascript` | No |
| Go | `go` | No |
| Rust | `rust` | Yes (`--command='cargo build'`) |
| Java/Kotlin | `java` | Yes (`--command='./gradlew build'`) |
| C/C++ | `cpp` | Yes (`--command='make -j8'`) |

### 2. Run Analysis

```bash
# SARIF output (recommended)
codeql database analyze codeql.db \
  --format=sarif-latest \
  --output=results.sarif \
  -- codeql/python-queries:codeql-suites/python-security-extended.qls

# With Trail of Bits queries
codeql database analyze codeql.db \
  --format=sarif-latest \
  --output=results.sarif \
  -- trailofbits/go-queries
```

## Writing Custom Queries

### Basic Template

```ql
/**
 * @name Find SQL injection vulnerabilities
 * @description Identifies potential SQL injection from user input
 * @kind path-problem
 * @problem.severity error
 * @security-severity 9.0
 * @precision high
 * @id py/sql-injection
 * @tags security
 *       external/cwe/cwe-089
 */

import python
import semmle.python.dataflow.new.DataFlow
import semmle.python.dataflow.new.TaintTracking

module SqlInjectionConfig implements DataFlow::ConfigSig {
  predicate isSource(DataFlow::Node source) {
    // Define taint sources (user input)
    exists(source)
  }
  
  predicate isSink(DataFlow::Node sink) {
    // Define dangerous sinks (SQL execution)
    exists(sink)
  }
}

module SqlInjectionFlow = TaintTracking::Global<SqlInjectionConfig>;

from SqlInjectionFlow::PathNode source, SqlInjectionFlow::PathNode sink
where SqlInjectionFlow::flowPath(source, sink)
select sink.getNode(), source, sink, "SQL injection from $@.", source.getNode(), "user input"
```

### Query Metadata

| Field | Description | Values |
|-------|-------------|--------|
| `@kind` | Query type | `problem`, `path-problem` |
| `@problem.severity` | Issue severity | `error`, `warning`, `recommendation` |
| `@security-severity` | CVSS score | `0.0` - `10.0` |
| `@precision` | Confidence | `very-high`, `high`, `medium`, `low` |

## CI/CD Integration (GitHub Actions)

```yaml
name: CodeQL Analysis
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 1'  # Weekly

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      actions: read
      contents: read
      security-events: write
    strategy:
      matrix:
        language: ['python', 'javascript']
    steps:
      - uses: actions/checkout@v4
      - name: Initialize CodeQL
        uses: github/codeql-action/init@v3
        with:
          languages: ${{ matrix.language }}
          queries: security-extended, security-and-quality
      - uses: github/codeql-action/autobuild@v3
      - uses: github/codeql-action/analyze@v3
        with:
          category: "/language:${{ matrix.language }}"
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Database creation fails | Clean build environment, verify build command works independently |
| Slow analysis | Use `--threads`, narrow query scope, check query complexity |
| Missing results | Check file exclusions, verify source files were parsed |
| Out of memory | Set `CODEQL_RAM=48000` environment variable (48GB) |

## Rationalizations to Reject

| Shortcut | Why It's Wrong |
|----------|----------------|
| "No findings means the code is secure" | CodeQL only finds patterns it has queries for |
| "This code path looks safe" | Complex data flow can hide vulnerabilities across 5+ function calls |
| "Small change, low risk" | Small changes can introduce critical bugs; run full analysis |
| "The query didn't flag it" | Default query suites don't cover everything; check custom queries |

## Resources

- Docs: https://codeql.github.com/docs/
- Query Help: https://codeql.github.com/codeql-query-help/
- Security Lab: https://securitylab.github.com/
- Trail of Bits Queries: https://github.com/trailofbits/codeql-queries

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) codeql skill.
