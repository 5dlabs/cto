# Cipher Security Scanning Guidelines

## Overview

This document provides comprehensive guidelines for Cipher, the Security Engineer & Code Analysis Specialist, to perform thorough security analysis on Rust and TypeScript codebases.

## Available Security Tools

### 1. Semgrep - Static Application Security Testing (SAST)

**Purpose**: Multi-language static analysis for security vulnerabilities and code quality issues.

**Installation**: Already installed in runtime image via `pip3 install semgrep`

**Basic Usage**:
```bash
# Scan current directory with default rules
semgrep scan --config auto

# Scan with custom rules
semgrep scan --config /workspace/.semgrep.yaml

# Scan specific files
semgrep scan --config auto src/

# Output formats
semgrep scan --config auto --json > semgrep-results.json
semgrep scan --config auto --sarif > semgrep-results.sarif

# Focus on high-severity issues
semgrep scan --config auto --severity ERROR
```

**Custom Rules**: Located at `infra/charts/controller/agent-templates/security/.semgrep.yaml`

**Key Features**:
- Detects unsafe Rust code patterns (unsafe blocks, unwrap/expect usage)
- Identifies hardcoded secrets in both Rust and TypeScript
- Finds SQL injection vulnerabilities
- Detects XSS risks in TypeScript (innerHTML, eval)
- Checks for weak cryptographic algorithms
- Command injection detection

### 2. Rust Security Tools

#### cargo-audit - Dependency Vulnerability Scanner

**Purpose**: Scans Rust dependencies for known security vulnerabilities from the RustSec Advisory Database.

**Installation**: Already installed via `cargo install cargo-audit`

**Usage**:
```bash
# Basic audit
cargo audit

# Audit with detailed output
cargo audit --json

# Audit specific advisory database
cargo audit --db /path/to/advisory-db

# Ignore specific advisories (use sparingly)
cargo audit --ignore RUSTSEC-2024-0001
```

**Exit Codes**:
- 0: No vulnerabilities found
- 1: Vulnerabilities found
- 2: Error occurred

#### cargo-geiger - Unsafe Code Detector

**Purpose**: Detects usage of unsafe code in dependencies and your codebase.

**Installation**: Already installed via `cargo install cargo-geiger`

**Usage**:
```bash
# Scan for unsafe code
cargo geiger

# Output in JSON format
cargo geiger --output-format Json > geiger-report.json

# Include build dependencies
cargo geiger --include-tests
```

**Interpretation**:
- 🔒 = No unsafe code
- ☢️ = Contains unsafe code
- Higher counts indicate more unsafe code usage

#### cargo-deny - Dependency Policy Enforcement

**Purpose**: Lint your dependencies for security, licenses, and sources.

**Installation**: Already installed via `cargo install cargo-deny`

**Usage**:
```bash
# Check all policies
cargo deny check

# Check only advisories
cargo deny check advisories

# Check licenses
cargo deny check licenses

# Check banned dependencies
cargo deny check bans

# Check dependency sources
cargo deny check sources
```

**Configuration**: Create `deny.toml` in project root:
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-3.0"]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

### 3. TypeScript/JavaScript Security Tools

#### npm audit - Dependency Vulnerability Scanner

**Purpose**: Scans npm dependencies for known vulnerabilities.

**Usage**:
```bash
# Basic audit
npm audit

# Audit with JSON output
npm audit --json

# Audit with different severity levels
npm audit --audit-level=moderate

# Attempt automatic fixes
npm audit fix

# Fix only production dependencies
npm audit fix --only=prod
```

#### ESLint with Security Plugins

**Purpose**: Static analysis for TypeScript/JavaScript with security-focused rules.

**Installation**: Already installed via `npm install -g eslint`

**Security Plugins** (install as needed):
```bash
npm install --save-dev eslint-plugin-security
npm install --save-dev eslint-plugin-no-unsanitized
```

**Configuration** (`.eslintrc.json`):
```json
{
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:security/recommended"
  ],
  "plugins": ["security", "no-unsanitized"],
  "rules": {
    "security/detect-object-injection": "error",
    "security/detect-non-literal-regexp": "warn",
    "security/detect-unsafe-regex": "error",
    "no-unsanitized/method": "error",
    "no-unsanitized/property": "error"
  }
}
```

### 4. Secret Scanning Tools

#### Gitleaks - Secret Detection

**Purpose**: Scans for hardcoded secrets, API keys, and credentials.

**Installation**: Already installed in runtime image

**Usage**:
```bash
# Scan current directory
gitleaks detect --source . --verbose

# Scan with report
gitleaks detect --source . --report-path gitleaks-report.json

# Scan git history
gitleaks detect --source . --log-opts="--all"

# Scan specific files
gitleaks detect --source . --no-git
```

**Important**: Always check if detected secrets are in `.gitignore`:
```bash
# Before flagging a secret, verify it's tracked by git
git check-ignore path/to/file
# Exit code 0 = file is ignored (NOT a security issue)
# Exit code 1 = file is tracked (SECURITY ISSUE)
```

#### Trivy - Comprehensive Vulnerability Scanner

**Purpose**: Scans for vulnerabilities in dependencies, containers, and infrastructure.

**Installation**: Already installed in runtime image

**Usage**:
```bash
# Scan filesystem
trivy fs .

# Scan with specific severity
trivy fs --severity HIGH,CRITICAL .

# Output formats
trivy fs --format json --output trivy-report.json .
trivy fs --format sarif --output trivy-report.sarif .

# Scan specific package files
trivy fs --scanners vuln Cargo.toml
trivy fs --scanners vuln package.json
```

### 5. Additional Security Tools

#### Hadolint - Dockerfile Linter

**Purpose**: Lints Dockerfiles for security and best practices.

**Installation**: Already installed in runtime image

**Usage**:
```bash
# Lint Dockerfile
hadolint Dockerfile

# Output in JSON
hadolint --format json Dockerfile
```

#### Qlty - Unified Code Quality

**Purpose**: Unified interface for multiple code quality and security tools.

**Installation**: Already installed in runtime image

**Usage**:
```bash
# Run all configured checks
qlty check

# Initialize configuration
qlty init
```

## Security Scanning Workflow

### Phase 1: Automated Scanning

```bash
#!/bin/bash
# Comprehensive security scan script

echo "🔒 Starting Cipher Security Scan"

# 1. Rust Dependency Vulnerabilities
if [ -f "Cargo.toml" ]; then
    echo "📦 Scanning Rust dependencies..."
    cargo audit --json > cargo-audit.json
    cargo geiger --output-format Json > cargo-geiger.json
    cargo deny check advisories
fi

# 2. TypeScript/JavaScript Dependencies
if [ -f "package.json" ]; then
    echo "📦 Scanning npm dependencies..."
    npm audit --json > npm-audit.json
fi

# 3. Static Code Analysis with Semgrep
echo "🔍 Running Semgrep static analysis..."
semgrep scan --config auto --json --output semgrep-results.json

# 4. Secret Scanning
echo "🔐 Scanning for secrets..."
gitleaks detect --source . --report-path gitleaks-report.json --verbose

# 5. Trivy Vulnerability Scan
echo "🛡️ Running Trivy vulnerability scan..."
trivy fs --severity HIGH,CRITICAL --format json --output trivy-report.json .

# 6. Unsafe Code Detection (Rust)
if [ -f "Cargo.toml" ]; then
    echo "☢️ Checking for unsafe Rust code..."
    cargo geiger
fi

echo "✅ Security scan complete"
```

### Phase 2: Analysis & Reporting

**Severity Classification**:
- **CRITICAL**: Immediate action required, block PR
  - Remote code execution vulnerabilities
  - SQL injection with user input
  - Hardcoded secrets in tracked files
  - Known CVEs with CVSS >= 9.0

- **HIGH**: Must fix before merge
  - XSS vulnerabilities
  - Authentication bypass
  - Unsafe deserialization
  - Known CVEs with CVSS >= 7.0

- **MEDIUM**: Should fix, can be addressed in follow-up
  - Weak cryptographic algorithms
  - Information disclosure
  - Denial of service risks
  - Known CVEs with CVSS >= 4.0

- **LOW**: Informational, nice to have
  - Code quality issues
  - Outdated dependencies (no known CVEs)
  - Minor security improvements

### Phase 3: Remediation Guidance

**For Rust Vulnerabilities**:
```bash
# Update specific dependency
cargo update -p vulnerable-crate

# Update all dependencies
cargo update

# Check for breaking changes
cargo tree -i vulnerable-crate
```

**For TypeScript Vulnerabilities**:
```bash
# Auto-fix vulnerabilities
npm audit fix

# Force update (may cause breaking changes)
npm audit fix --force

# Manual update
npm install package@safe-version
```

**For Code Issues**:
- Provide specific code fixes in PR review comments
- Link to security best practices documentation
- Suggest secure alternatives with code examples

## Best Practices for Cipher

### 1. Always Verify .gitignore

Before flagging secrets:
```bash
# Check if file is ignored
if git check-ignore -q path/to/file; then
    echo "File is in .gitignore - NOT a security issue"
else
    echo "File is tracked - SECURITY ISSUE"
fi
```

### 2. Prioritize Actionable Findings

- Focus on vulnerabilities in production code
- Ignore test files unless they expose real secrets
- Consider false positives (e.g., example code, documentation)

### 3. Provide Clear Remediation Steps

**Bad**:
> "SQL injection vulnerability found"

**Good**:
> "SQL injection vulnerability in `src/db.rs:42`
> 
> **Current code**:
> ```rust
> let query = format!("SELECT * FROM users WHERE id = {}", user_id);
> ```
> 
> **Secure fix**:
> ```rust
> let query = sqlx::query!("SELECT * FROM users WHERE id = ?", user_id);
> ```
> 
> **Why**: Parameterized queries prevent SQL injection by treating user input as data, not executable code."

### 4. Use Multiple Tools

- Semgrep for custom patterns
- cargo-audit/npm audit for known CVEs
- Gitleaks for secrets
- Trivy for comprehensive scanning

### 5. Document Exceptions

When vulnerabilities cannot be immediately fixed:
- Document why (e.g., no patch available)
- Suggest compensating controls
- Create tracking issue for follow-up

## Common Security Patterns

### Rust

**Input Validation**:
```rust
use validator::Validate;

#[derive(Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}
```

**Secure Secret Handling**:
```rust
use secrecy::{Secret, ExposeSecret};

let api_key = Secret::new(
    std::env::var("API_KEY").expect("API_KEY not set")
);

// Use with expose_secret() only when needed
make_request(api_key.expose_secret());
```

**Safe Error Handling**:
```rust
// Bad
let value = option.unwrap();

// Good
let value = option.ok_or(Error::MissingValue)?;
```

### TypeScript

**Input Sanitization**:
```typescript
import sanitizeHtml from 'sanitize-html';

const clean = sanitizeHtml(userInput, {
    allowedTags: ['b', 'i', 'em', 'strong'],
    allowedAttributes: {}
});
```

**Secure Configuration**:
```typescript
// Bad
const apiKey = "sk_live_abc123";

// Good
const apiKey = process.env.API_KEY || 
    throw new Error('API_KEY environment variable not set');
```

**Safe DOM Manipulation**:
```typescript
// Bad
element.innerHTML = userInput;

// Good
element.textContent = userInput;
// Or for HTML:
import DOMPurify from 'dompurify';
element.innerHTML = DOMPurify.sanitize(userInput);
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Cipher Security Scan

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Semgrep
        run: |
          pip install semgrep
          semgrep scan --config auto --sarif > semgrep.sarif
      
      - name: Rust Security Audit
        if: hashFiles('Cargo.toml')
        run: |
          cargo install cargo-audit
          cargo audit
      
      - name: npm Security Audit
        if: hashFiles('package.json')
        run: npm audit --audit-level=moderate
      
      - name: Secret Scanning
        run: |
          docker run -v $(pwd):/path zricethezav/gitleaks:latest \
            detect --source /path --verbose
      
      - name: Upload Results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: semgrep.sarif
```

## References

- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://www.rust-lang.org/policies/security)
- [RustSec Advisory Database](https://rustsec.org/)
- [Semgrep Rules](https://semgrep.dev/explore)
- [npm Security Best Practices](https://docs.npmjs.com/security-best-practices)
- [TypeScript Security](https://www.typescriptlang.org/docs/handbook/security.html)

## Support

For questions or issues with security tooling:
1. Check tool documentation
2. Review this guide
3. Consult with security team
4. Escalate critical findings immediately


