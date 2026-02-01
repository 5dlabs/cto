---
name: sarif-parsing
description: Parse and analyze SARIF files from static analysis tools - aggregation, deduplication, filtering.
---

# SARIF Parsing Best Practices

Parse, analyze, and process SARIF files from static analysis tools like CodeQL, Semgrep, and others.

## When to Use

- Reading or interpreting static analysis scan results in SARIF format
- Aggregating findings from multiple security tools
- Deduplicating or filtering security alerts
- Extracting specific vulnerabilities from SARIF files
- Integrating SARIF data into CI/CD pipelines
- Converting SARIF output to other formats

## SARIF Structure Overview

SARIF 2.1.0 is the current OASIS standard:

```
sarifLog
├── version: "2.1.0"
└── runs[] (array of analysis runs)
    ├── tool
    │   ├── driver
    │   │   ├── name (required)
    │   │   ├── version
    │   │   └── rules[] (rule definitions)
    │   └── extensions[] (plugins)
    ├── results[] (findings)
    │   ├── ruleId
    │   ├── level (error/warning/note)
    │   ├── message.text
    │   ├── locations[]
    │   │   └── physicalLocation
    │   │       ├── artifactLocation.uri
    │   │       └── region (startLine, startColumn, etc.)
    │   ├── fingerprints{}
    │   └── partialFingerprints{}
    └── artifacts[] (scanned files metadata)
```

### Why Fingerprinting Matters

Without stable fingerprints, you can't track findings across runs:
- **Baseline comparison**: "Is this a new finding or did we see it before?"
- **Regression detection**: "Did this PR introduce new vulnerabilities?"
- **Suppression**: "Ignore this known false positive in future runs"

## Tool Selection Guide

| Use Case | Tool | Installation |
|----------|------|--------------|
| Quick CLI queries | jq | `brew install jq` / `apt install jq` |
| Python scripting (simple) | pysarif | `pip install pysarif` |
| Python scripting (advanced) | sarif-tools | `pip install sarif-tools` |
| .NET applications | SARIF SDK | NuGet package |
| JavaScript/Node.js | sarif-js | npm package |

## Quick Analysis with jq

```bash
# Pretty print the file
jq '.' results.sarif

# Count total findings
jq '[.runs[].results[]] | length' results.sarif

# List all rule IDs triggered
jq '[.runs[].results[].ruleId] | unique' results.sarif

# Extract errors only
jq '.runs[].results[] | select(.level == "error")' results.sarif

# Get findings with file locations
jq '.runs[].results[] | {
  rule: .ruleId,
  message: .message.text,
  file: .locations[0].physicalLocation.artifactLocation.uri,
  line: .locations[0].physicalLocation.region.startLine
}' results.sarif

# Filter by severity and get count per rule
jq '[.runs[].results[] | select(.level == "error")] | 
    group_by(.ruleId) | 
    map({rule: .[0].ruleId, count: length})' results.sarif
```

## Python with sarif-tools

```python
from sarif import loader

# Load single file
sarif_data = loader.load_sarif_file("results.sarif")

# Or load multiple files
sarif_set = loader.load_sarif_files(["tool1.sarif", "tool2.sarif"])

# Get summary report
report = sarif_data.get_report()

# Get histogram by severity
errors = report.get_issue_type_histogram_for_severity("error")
warnings = report.get_issue_type_histogram_for_severity("warning")

# Filter results
high_severity = [r for r in sarif_data.get_results() 
                 if r.get("level") == "error"]
```

**sarif-tools CLI commands:**

```bash
# Summary of findings
sarif summary results.sarif

# List all results with details
sarif ls results.sarif

# Get results by severity
sarif ls --level error results.sarif

# Diff two SARIF files (find new/fixed issues)
sarif diff baseline.sarif current.sarif

# Convert to other formats
sarif csv results.sarif > results.csv
sarif html results.sarif > report.html
```

## Aggregating Multiple SARIF Files

```python
import json
from pathlib import Path

def aggregate_sarif_files(sarif_paths: list[str]) -> dict:
    """Combine multiple SARIF files into one."""
    aggregated = {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": []
    }
    
    for path in sarif_paths:
        with open(path) as f:
            sarif = json.load(f)
            aggregated["runs"].extend(sarif.get("runs", []))
    
    return aggregated

def deduplicate_results(sarif: dict) -> dict:
    """Remove duplicate findings based on fingerprints."""
    seen_fingerprints = set()
    
    for run in sarif["runs"]:
        unique_results = []
        for result in run.get("results", []):
            # Use partialFingerprints or create key from location
            fp = None
            if result.get("partialFingerprints"):
                fp = tuple(sorted(result["partialFingerprints"].items()))
            elif result.get("fingerprints"):
                fp = tuple(sorted(result["fingerprints"].items()))
            else:
                # Fallback: create fingerprint from rule + location
                loc = result.get("locations", [{}])[0]
                phys = loc.get("physicalLocation", {})
                fp = (
                    result.get("ruleId"),
                    phys.get("artifactLocation", {}).get("uri"),
                    phys.get("region", {}).get("startLine")
                )
            
            if fp not in seen_fingerprints:
                seen_fingerprints.add(fp)
                unique_results.append(result)
        
        run["results"] = unique_results
    
    return sarif
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif

- name: Check for high severity
  run: |
    HIGH_COUNT=$(jq '[.runs[].results[] | select(.level == "error")] | length' results.sarif)
    if [ "$HIGH_COUNT" -gt 0 ]; then
      echo "Found $HIGH_COUNT high severity issues"
      exit 1
    fi
```

### Fail on New Issues

```python
from sarif import loader

def check_for_regressions(baseline: str, current: str) -> int:
    """Return count of new issues not in baseline."""
    baseline_data = loader.load_sarif_file(baseline)
    current_data = loader.load_sarif_file(current)
    
    baseline_fps = {get_fingerprint(r) for r in baseline_data.get_results()}
    new_issues = [r for r in current_data.get_results() 
                  if get_fingerprint(r) not in baseline_fps]
    
    return len(new_issues)
```

## Common Pitfalls and Solutions

### Path Normalization Issues

```python
from urllib.parse import unquote
from pathlib import Path

def normalize_path(uri: str, base_path: str = "") -> str:
    """Normalize SARIF artifact URI to consistent path."""
    # Remove file:// prefix if present
    if uri.startswith("file://"):
        uri = uri[7:]
    
    # URL decode
    uri = unquote(uri)
    
    # Handle relative paths
    if not Path(uri).is_absolute() and base_path:
        uri = str(Path(base_path) / uri)
    
    return str(Path(uri))
```

### Safe Data Access

```python
def safe_get_location(result: dict) -> tuple[str, int]:
    """Safely extract file and line from result."""
    try:
        loc = result.get("locations", [{}])[0]
        phys = loc.get("physicalLocation", {})
        file_path = phys.get("artifactLocation", {}).get("uri", "unknown")
        line = phys.get("region", {}).get("startLine", 0)
        return file_path, line
    except (IndexError, KeyError, TypeError):
        return "unknown", 0
```

## Key Principles

1. **Validate first**: Check SARIF structure before processing
2. **Handle optionals**: Many fields are optional; use defensive access
3. **Normalize paths**: Tools report paths differently; normalize early
4. **Fingerprint wisely**: Combine multiple strategies for stable deduplication
5. **Stream large files**: Use ijson or similar for 100MB+ files

## Resources

- [OASIS SARIF 2.1.0 Specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
- [GitHub SARIF Support](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning)
- [SARIF Validator](https://sarifweb.azurewebsites.net/)

## Attribution

Based on [trailofbits/skills](https://github.com/trailofbits/skills) sarif-parsing skill.
