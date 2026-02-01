---
name: semgrep
description: Static analysis with Semgrep - pattern-based security scanning and custom rule creation.
---

# Semgrep Static Analysis

## When to Use Semgrep

**Ideal scenarios:**
- Quick security scans (minutes, not hours)
- Pattern-based bug detection
- Enforcing coding standards and best practices
- Finding known vulnerability patterns
- Single-file analysis without complex data flow
- First-pass analysis before deeper tools

**Consider CodeQL instead when:**
- Need interprocedural taint tracking across files
- Complex data flow analysis required
- Analyzing custom proprietary frameworks

## Installation

```bash
# pip
pip install semgrep

# Homebrew
brew install semgrep

# Docker
docker run --rm -v "${PWD}:/src" returntocorp/semgrep semgrep --config auto /src
```

## Core Workflow

### 1. Quick Scan

```bash
semgrep --config auto .                    # Auto-detect rules
semgrep --config auto --metrics=off .      # Disable telemetry
```

### 2. Use Rulesets

```bash
semgrep --config p/<RULESET> .             # Single ruleset
semgrep --config p/security-audit --config p/trailofbits .  # Multiple
```

| Ruleset | Description |
|---------|-------------|
| `p/default` | General security and code quality |
| `p/security-audit` | Comprehensive security rules |
| `p/owasp-top-ten` | OWASP Top 10 vulnerabilities |
| `p/cwe-top-25` | CWE Top 25 vulnerabilities |
| `p/trailofbits` | Trail of Bits security rules |
| `p/python` | Python-specific |
| `p/javascript` | JavaScript-specific |
| `p/rust` | Rust-specific |

### 3. Output Formats

```bash
semgrep --config p/security-audit --sarif -o results.sarif .  # SARIF
semgrep --config p/security-audit --json -o results.json .    # JSON
semgrep --config p/security-audit --dataflow-traces .         # Show data flow
```

## Writing Custom Rules

### Basic Structure

```yaml
rules:
  - id: hardcoded-password
    languages: [python]
    message: "Hardcoded password detected: $PASSWORD"
    severity: ERROR
    pattern: password = "$PASSWORD"
```

### Pattern Syntax

| Syntax | Description | Example |
|--------|-------------|---------|
| `...` | Match anything | `func(...)` |
| `$VAR` | Capture metavariable | `$FUNC($INPUT)` |
| `<... ...>` | Deep expression match | `<... user_input ...>` |

### Pattern Operators

| Operator | Description |
|----------|-------------|
| `pattern` | Match exact pattern |
| `patterns` | All must match (AND) |
| `pattern-either` | Any matches (OR) |
| `pattern-not` | Exclude matches |
| `pattern-inside` | Match only inside context |
| `pattern-not-inside` | Match only outside context |
| `metavariable-regex` | Regex on captured value |

### Combining Patterns

```yaml
rules:
  - id: sql-injection
    languages: [python]
    message: "Potential SQL injection"
    severity: ERROR
    patterns:
      - pattern-either:
          - pattern: cursor.execute($QUERY)
          - pattern: db.execute($QUERY)
      - pattern-not:
          - pattern: cursor.execute("...", (...))
      - metavariable-regex:
          metavariable: $QUERY
          regex: .*\+.*|.*\.format\(.*|.*%.*
```

### Taint Mode (Data Flow)

Taint mode tracks data through assignments and transformations:

```yaml
rules:
  - id: command-injection
    languages: [python]
    message: "User input flows to command execution"
    severity: ERROR
    mode: taint
    pattern-sources:
      - pattern: request.args.get(...)
      - pattern: request.form[...]
      - pattern: request.json
    pattern-sinks:
      - pattern: os.system($SINK)
      - pattern: subprocess.call($SINK, shell=True)
      - pattern: subprocess.run($SINK, shell=True, ...)
    pattern-sanitizers:
      - pattern: shlex.quote(...)
      - pattern: int(...)
```

## CI/CD Integration (GitHub Actions)

```yaml
name: Semgrep
on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 0 1 * *'  # Monthly

jobs:
  semgrep:
    runs-on: ubuntu-latest
    container:
      image: returntocorp/semgrep
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Run Semgrep
        run: |
          if [ "${{ github.event_name }}" = "pull_request" ]; then
            semgrep ci --baseline-commit ${{ github.event.pull_request.base.sha }}
          else
            semgrep ci
          fi
        env:
          SEMGREP_RULES: >-
            p/security-audit
            p/owasp-top-ten
            p/trailofbits
```

## Configuration

### .semgrepignore

```
tests/
fixtures/
**/testdata/
generated/
vendor/
node_modules/
```

### Suppress False Positives

```python
password = get_from_vault()  # nosemgrep: hardcoded-password
dangerous_but_safe()         # nosemgrep
```

## Rationalizations to Reject

| Shortcut | Why It's Wrong |
|----------|----------------|
| "Semgrep found nothing, code is clean" | Semgrep is pattern-based; can't track complex data flow |
| "I wrote a rule, so we're covered" | Rules need testing; false negatives are silent |
| "Too many findings = noisy tool" | High finding count often means real problems |

## Resources

- Registry: https://semgrep.dev/explore
- Playground: https://semgrep.dev/playground
- Docs: https://semgrep.dev/docs/
- Trail of Bits Rules: https://github.com/trailofbits/semgrep-rules

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) semgrep skill.
