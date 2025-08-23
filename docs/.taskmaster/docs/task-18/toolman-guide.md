# Task 18: Test Coverage Requirements - Tool Usage Guide

## Overview
This guide covers the comprehensive toolset required for implementing automated test coverage enforcement with the Tess agent. The implementation spans Rust coverage analysis, GitHub API integration, automated test generation, and comprehensive reporting systems.

## Required Tools

### 1. Rust Coverage Analysis Stack
**Primary Tools**: `cargo llvm-cov`, `rustup`, `llvm-tools-preview`

```bash
# Install LLVM coverage tools
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.5.36

# Verify installation
rustup component list | grep llvm-tools
cargo llvm-cov --version

# Basic coverage workflow
cargo llvm-cov clean
cargo llvm-cov --html --open  # Interactive development
cargo llvm-cov --lcov --output-path coverage.lcov
cargo llvm-cov --cobertura --output-path coverage.xml
cargo llvm-cov --json --output-path coverage.json
```

**Advanced Coverage Usage**:
```bash
# Workspace coverage analysis
cargo llvm-cov --workspace --html --output-dir coverage-report

# Include/exclude specific paths
cargo llvm-cov --ignore-filename-regex "tests/.*"

# Show missing lines detail
cargo llvm-cov --show-missing-lines

# Coverage with specific test selection
cargo llvm-cov --test integration_tests
```

### 2. GitHub API Integration Tools
**Primary Tools**: `curl`, `gh`, `jq`, `python3`

```bash
# GitHub CLI setup
gh auth login
gh config get -h github.com

# API testing and development
curl -H "Authorization: token $GITHUB_TOKEN" \
     -H "Accept: application/vnd.github.v3+json" \
     https://api.github.com/repos/owner/repo/pulls/123

# PR review operations
gh pr review 123 --approve --body "Coverage requirements met"
gh pr review 123 --request-changes --body "Insufficient coverage"

# Advanced GitHub operations
gh api repos/owner/repo/pulls/123/reviews --method POST \
  --field event=APPROVE \
  --field body="Automated approval by Tess"
```

**GitHub Integration Development**:
```bash
# Test GitHub API connectivity
curl -I https://api.github.com/user -H "Authorization: token $GITHUB_TOKEN"

# Monitor API rate limits
gh api rate_limit

# Test PR operations on development repo
gh repo create test-coverage-repo --private
gh pr create --title "Test Coverage PR" --body "Testing coverage workflow"
```

### 3. Test Generation and Analysis
**Primary Tools**: `python3`, `ast-grep`, `tree-sitter`

```python
# Coverage data processing
import json
import subprocess

def analyze_coverage_json(coverage_file):
    with open(coverage_file, 'r') as f:
        data = json.load(f)

    total_lines = data['data'][0]['totals']['lines']['count']
    covered_lines = data['data'][0]['totals']['lines']['covered']
    coverage_pct = (covered_lines / total_lines) * 100

    return {
        'total': total_lines,
        'covered': covered_lines,
        'percentage': coverage_pct
    }

# Uncovered code identification
def find_uncovered_functions(source_file, coverage_data):
    # Parse Rust source and identify uncovered functions
    # Generate appropriate test templates
    pass
```

**Test Generation Pipeline**:
```bash
# Extract function signatures from Rust files
grep -n "^pub fn\|^fn " src/**/*.rs

# Generate test templates
python3 scripts/generate_tests.py --input src/lib.rs --output tests/generated/

# Validate generated tests compile
cargo check --tests

# Run generated tests and measure coverage improvement
cargo llvm-cov --include-tests test
```

### 4. Report Generation and Visualization
**Primary Tools**: `cargo llvm-cov`, `python3`, `pandoc`

```bash
# Generate multiple report formats
cargo llvm-cov --workspace --html --output-dir /tmp/coverage-html
cargo llvm-cov --workspace --lcov --output-path /tmp/coverage.lcov
cargo llvm-cov --workspace --cobertura --output-path /tmp/coverage.xml
cargo llvm-cov --workspace --json --output-path /tmp/coverage.json

# Serve coverage reports locally
python3 -m http.server 8080 --directory /tmp/coverage-html

# Generate summary reports
python3 -c "
import json
with open('/tmp/coverage.json', 'r') as f:
    data = json.load(f)
    coverage = data['data'][0]['totals']['lines']
    print(f'Coverage: {coverage[\"covered\"]}/{coverage[\"count\"]} ({coverage[\"percent\"]:.2f}%)')
"
```

**Report Processing Scripts**:
```python
# coverage_processor.py
import json
from datetime import datetime

class CoverageReportProcessor:
    def __init__(self, coverage_file):
        self.data = self.load_coverage_data(coverage_file)

    def load_coverage_data(self, file_path):
        with open(file_path, 'r') as f:
            return json.load(f)

    def generate_summary(self):
        totals = self.data['data'][0]['totals']
        return {
            'timestamp': datetime.now().isoformat(),
            'lines': totals['lines'],
            'functions': totals['functions'],
            'regions': totals['regions']
        }

    def find_uncovered_files(self):
        files = self.data['data'][0]['files']
        return [f for f in files if f['summary']['lines']['percent'] < 100.0]
```

### 5. Container Environment Management
**Primary Tools**: `docker`, `bash`, `envsubst`

```bash
# Test container environment locally
docker run -it --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  rust:1.70 \
  bash

# Inside container - setup coverage tools
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# Test script execution in container
docker run --rm \
  -v $(pwd)/templates:/templates \
  -e GITHUB_TOKEN="test-token" \
  -e REPO_URL="https://github.com/test/repo" \
  -e PR_NUMBER="123" \
  rust:1.70 \
  bash /templates/container-tess.sh.hbs
```

## Development Workflow

### Phase 1: Coverage Analysis Development
```bash
# 1. Setup development environment
mkdir -p test-project/src test-project/tests
cd test-project/
cargo init

# 2. Create sample code with varying coverage
cat > src/lib.rs << 'EOF'
pub fn covered_function(x: i32) -> i32 {
    x * 2
}

pub fn partially_covered_function(x: i32) -> i32 {
    if x > 0 {
        x * 3  // This branch might not be covered
    } else {
        x * -1  // This branch might not be covered
    }
}

pub fn uncovered_function() -> String {
    "never called".to_string()
}
EOF

# 3. Create basic tests
cat > tests/basic_test.rs << 'EOF'
use test_project::covered_function;

#[test]
fn test_covered_function() {
    assert_eq!(covered_function(5), 10);
}
EOF

# 4. Analyze coverage iteratively
cargo llvm-cov --html --open
```

### Phase 2: Test Generation System
```bash
# 1. Analyze uncovered code
cargo llvm-cov --show-missing-lines test > uncovered.txt

# 2. Develop test generation script
python3 << 'EOF'
import re
import ast

def parse_uncovered_lines(uncovered_file):
    with open(uncovered_file, 'r') as f:
        content = f.read()

    # Extract uncovered line information
    files_lines = {}
    current_file = None

    for line in content.split('\n'):
        if line.endswith('.rs'):
            current_file = line.strip()
            files_lines[current_file] = []
        elif 'Missing:' in line and current_file:
            # Extract line numbers
            numbers = re.findall(r'\d+(?:-\d+)?', line)
            files_lines[current_file].extend(numbers)

    return files_lines

def generate_test_template(source_file, uncovered_lines):
    template = f'''
//! Generated tests for {source_file}

use super::*;

#[cfg(test)]
mod generated_coverage_tests {{
    use super::*;

    // Tests for uncovered lines: {uncovered_lines}

    #[test]
    fn test_uncovered_branches() {{
        // TODO: Add specific tests for uncovered code paths
    }}

    #[test]
    fn test_error_conditions() {{
        // TODO: Add tests for error handling paths
    }}
}}
'''

    test_file = source_file.replace('src/', 'tests/generated_').replace('.rs', '_test.rs')
    with open(test_file, 'w') as f:
        f.write(template)

    print(f"Generated test template: {test_file}")

# Process uncovered code
uncovered = parse_uncovered_lines('uncovered.txt')
for source_file, lines in uncovered.items():
    if lines:
        generate_test_template(source_file, lines)
EOF

# 3. Test generated code
cargo check --tests
cargo test
```

### Phase 3: GitHub Integration Development
```bash
# 1. Setup GitHub API testing
export GITHUB_TOKEN="your-dev-token"
export TEST_REPO="owner/test-repo"
export TEST_PR="123"

# 2. Test API connectivity
python3 << 'EOF'
import requests
import os
import json

token = os.environ['GITHUB_TOKEN']
repo = os.environ['TEST_REPO']
pr_number = os.environ['TEST_PR']

headers = {
    'Authorization': f'token {token}',
    'Accept': 'application/vnd.github.v3+json'
}

# Test API access
response = requests.get('https://api.github.com/user', headers=headers)
print(f"API Access: {response.status_code}")

# Get PR information
pr_url = f'https://api.github.com/repos/{repo}/pulls/{pr_number}'
response = requests.get(pr_url, headers=headers)
print(f"PR Access: {response.status_code}")

# Test review creation (dry run)
review_data = {
    'event': 'COMMENT',
    'body': 'Test comment from coverage analysis'
}
print(f"Would submit review: {json.dumps(review_data, indent=2)}")
EOF

# 3. Develop complete GitHub integration
cat > github_integration.py << 'EOF'
import requests
import json
import os

class GitHubCoverageIntegration:
    def __init__(self, token):
        self.token = token
        self.headers = {
            'Authorization': f'token {token}',
            'Accept': 'application/vnd.github.v3+json'
        }

    def submit_coverage_review(self, repo, pr_number, coverage_report):
        meets_threshold = coverage_report['meets_threshold']

        if meets_threshold:
            event = 'APPROVE'
            body = f"✅ Coverage requirements met ({coverage_report['percentage']:.2f}%)"
        else:
            event = 'REQUEST_CHANGES'
            body = f"❌ Coverage below threshold ({coverage_report['percentage']:.2f}%)"

        review_data = {'event': event, 'body': body}

        url = f'https://api.github.com/repos/{repo}/pulls/{pr_number}/reviews'
        response = requests.post(url, headers=self.headers, json=review_data)

        return response.status_code == 200

# Usage example
if __name__ == '__main__':
    integration = GitHubCoverageIntegration(os.environ['GITHUB_TOKEN'])

    # Mock coverage report
    report = {
        'percentage': 97.5,
        'meets_threshold': True,
        'improvement': 5.2
    }

    success = integration.submit_coverage_review(
        os.environ['TEST_REPO'],
        int(os.environ['TEST_PR']),
        report
    )

    print(f"Review submission: {'✅ Success' if success else '❌ Failed'}")
EOF
```

### Phase 4: Complete Workflow Integration
```bash
# 1. Create comprehensive container script
cat > container-tess-complete.sh << 'EOF'
#!/bin/bash
set -euo pipefail

# Configuration
COVERAGE_THRESHOLD_EXISTING=${COVERAGE_THRESHOLD_EXISTING:-95}
COVERAGE_THRESHOLD_NEW=${COVERAGE_THRESHOLD_NEW:-100}

echo "=== Tess Coverage Workflow ==="

# Stage 1: Tool setup
echo "Installing coverage tools..."
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov || echo "cargo-llvm-cov already installed"

# Stage 2: Initial coverage
echo "Running initial coverage analysis..."
cargo llvm-cov clean
cargo llvm-cov --json --output-path /tmp/initial-coverage.json test

# Stage 3: Generate missing tests
echo "Analyzing uncovered code..."
cargo llvm-cov --show-missing-lines test > /tmp/uncovered.txt
python3 /scripts/generate_tests.py /tmp/uncovered.txt

# Stage 4: Final coverage
echo "Running final coverage analysis..."
cargo llvm-cov clean
cargo llvm-cov --html --output-dir /tmp/coverage-html test
cargo llvm-cov --json --output-path /tmp/final-coverage.json test

# Stage 5: GitHub integration
echo "Processing GitHub integration..."
python3 /scripts/github_integration.py \
  --coverage-file /tmp/final-coverage.json \
  --repo "$REPO_URL" \
  --pr "$PR_NUMBER" \
  --threshold-existing "$COVERAGE_THRESHOLD_EXISTING" \
  --threshold-new "$COVERAGE_THRESHOLD_NEW"

echo "✅ Tess workflow completed"
EOF

chmod +x container-tess-complete.sh

# 2. Test complete workflow
./container-tess-complete.sh
```

## Common Issues and Solutions

### Issue 1: LLVM Tools Installation Failures
**Symptoms**: `cargo llvm-cov` command not found, installation errors

**Diagnosis**:
```bash
# Check Rust toolchain
rustup show
rustup component list | grep llvm

# Verify cargo installation directory
echo $CARGO_HOME
ls -la ~/.cargo/bin/

# Check for conflicting installations
which cargo-llvm-cov
cargo --list | grep llvm-cov
```

**Solutions**:
- Ensure correct Rust toolchain version (stable channel)
- Install llvm-tools-preview component first
- Clear cargo cache and reinstall: `rm -rf ~/.cargo/registry/index`
- Use specific version: `cargo install cargo-llvm-cov --version 0.5.36`

### Issue 2: Coverage Data Inconsistencies
**Symptoms**: Coverage percentages vary between runs, missing coverage data

**Diagnosis**:
```bash
# Clean build artifacts thoroughly
cargo llvm-cov clean
cargo clean
rm -rf target/

# Verify test execution
cargo test --verbose
cargo test --test integration_tests

# Check coverage instrumentation
CARGO_LOG=cargo::util::rustc=debug cargo llvm-cov test
```

**Solutions**:
- Always run `cargo llvm-cov clean` before analysis
- Ensure all tests are included: `cargo llvm-cov --workspace test`
- Use consistent build flags across runs
- Check for excluded files: `cargo llvm-cov --ignore-filename-regex "tests/.*"`

### Issue 3: GitHub API Integration Failures
**Symptoms**: API calls fail, authentication errors, rate limiting

**Diagnosis**:
```bash
# Test GitHub token validity
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user

# Check API rate limits
gh api rate_limit

# Verify repository access
gh repo view owner/repo

# Test PR access
gh pr view 123 --repo owner/repo
```

**Solutions**:
- Ensure token has correct scopes (repo, pull_request)
- Implement retry logic with exponential backoff
- Handle rate limiting with appropriate delays
- Use GitHub App installation tokens for higher limits

### Issue 4: Test Generation Quality Issues
**Symptoms**: Generated tests don't compile, don't improve coverage

**Diagnosis**:
```bash
# Check generated test compilation
cargo check --tests
cargo test generated_coverage_tests

# Analyze coverage improvement
cargo llvm-cov --html test
# Compare before/after coverage reports

# Validate test content
grep -n "TODO" tests/generated_*
grep -n "assert" tests/generated_*
```

**Solutions**:
- Improve test generation templates with actual assertions
- Add semantic analysis to understand code context
- Generate tests based on function signatures and types
- Validate generated tests before integration

## Best Practices

### Coverage Analysis Workflow
```bash
# Standard coverage analysis sequence
cargo llvm-cov clean                          # Clean previous data
cargo llvm-cov --workspace test               # Run with coverage
cargo llvm-cov --workspace --html report      # Generate HTML report
cargo llvm-cov --workspace --json report      # Generate JSON data

# Comprehensive reporting
cargo llvm-cov --workspace --lcov --output-path coverage.lcov report
cargo llvm-cov --workspace --cobertura --output-path coverage.xml report
```

### Test Generation Strategy
```python
# Quality test generation approach
class TestGenerator:
    def generate_function_test(self, function_name, parameters):
        return f'''
    #[test]
    fn test_{function_name}() {{
        // Test normal operation
        let result = {function_name}({self.generate_valid_params(parameters)});
        assert!(result.is_ok() || result.is_some());

        // Test edge cases
        {self.generate_edge_case_tests(function_name, parameters)}
    }}
'''

    def generate_edge_case_tests(self, function_name, parameters):
        # Generate boundary condition tests
        # Generate error condition tests
        # Generate null/empty input tests
        pass
```

### GitHub Integration Patterns
```python
# Robust GitHub API integration
import time
import requests
from functools import wraps

def retry_on_failure(max_retries=3):
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_retries):
                try:
                    return func(*args, **kwargs)
                except requests.exceptions.RequestException as e:
                    if attempt == max_retries - 1:
                        raise
                    time.sleep(2 ** attempt)  # Exponential backoff
            return None
        return wrapper
    return decorator

class GitHubAPI:
    @retry_on_failure()
    def submit_review(self, repo, pr_number, review_data):
        # Implementation with retry logic
        pass
```

## Performance Optimization

### Coverage Analysis Performance
```bash
# Optimize coverage collection
export CARGO_LLVM_COV_TARGET_DIR=/tmp/coverage  # Use fast storage
export CARGO_INCREMENTAL=0                      # Disable incremental compilation
export CARGO_PROFILE_TEST_DEBUG=0               # Reduce debug info

# Parallel test execution
cargo llvm-cov --workspace test -- --test-threads=4

# Selective coverage for large projects
cargo llvm-cov --package core_lib test
```

### Memory Usage Optimization
```bash
# Monitor memory usage during coverage
/usr/bin/time -v cargo llvm-cov test

# Limit parallel test execution
cargo llvm-cov test -- --test-threads=2

# Use streaming for large reports
cargo llvm-cov report | gzip > coverage-report.json.gz
```

## Monitoring and Debugging

### Coverage Analysis Debugging
```bash
# Enable verbose coverage logging
CARGO_LOG=cargo::util::rustc=debug cargo llvm-cov test

# Check coverage data files
ls -la target/llvm-cov-target/
llvm-profdata show --all-functions target/llvm-cov-target/default.profdata

# Validate coverage instrumentation
cargo llvm-cov --show-instantiations test
```

### GitHub Integration Monitoring
```python
# Log all GitHub API interactions
import logging

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

class GitHubAPI:
    def make_request(self, method, url, **kwargs):
        logger.debug(f"GitHub API: {method} {url}")
        response = requests.request(method, url, **kwargs)
        logger.debug(f"Response: {response.status_code} {response.reason}")
        return response
```

## Troubleshooting Checklist

### Pre-Development Setup
- [ ] Rust toolchain installed with stable channel
- [ ] LLVM tools component added to toolchain
- [ ] cargo-llvm-cov installed and accessible
- [ ] GitHub token configured with appropriate scopes
- [ ] Test repository setup for development

### Development Phase
- [ ] Coverage analysis produces consistent results
- [ ] Generated tests compile without errors
- [ ] GitHub API integration responds correctly
- [ ] Report generation completes successfully
- [ ] Container script executes without failures

### Integration Testing
- [ ] Complete workflow executes end-to-end
- [ ] Coverage thresholds enforced correctly
- [ ] GitHub reviews submitted successfully
- [ ] Reports accessible and properly formatted
- [ ] Performance meets requirements under load

### Production Deployment
- [ ] All tools available in production container
- [ ] Environment variables configured correctly
- [ ] GitHub API access working with production tokens
- [ ] Monitoring and alerting functional
- [ ] Rollback procedures tested and ready
