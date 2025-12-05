# Cipher Security Scanning Quick Reference

## 🚀 Quick Start

```bash
# Run comprehensive security scan
semgrep scan --config auto --severity ERROR
cargo audit  # If Rust project
npm audit    # If TypeScript project
gitleaks detect --source . --verbose
```

## 🔧 Available Tools

| Tool | Command | Purpose |
|------|---------|---------|
| **Semgrep** | `semgrep scan --config auto` | Multi-language SAST |
| **cargo-audit** | `cargo audit` | Rust dependency CVEs |
| **cargo-geiger** | `cargo geiger` | Rust unsafe code detection |
| **cargo-deny** | `cargo deny check` | Rust policy enforcement |
| **npm audit** | `npm audit` | npm dependency CVEs |
| **gitleaks** | `gitleaks detect --source .` | Secret scanning |
| **trivy** | `trivy fs .` | Vulnerability scanning |

## 📊 Severity Levels

- **CRITICAL** (CVSS >= 9.0): Block PR immediately
- **HIGH** (CVSS >= 7.0): Must fix before merge
- **MEDIUM** (CVSS >= 4.0): Should fix, can follow up
- **LOW**: Informational only

## ✅ Security Checklist

### Rust Projects
```bash
☐ cargo audit --json > audit.json
☐ cargo geiger --output-format Json > geiger.json
☐ cargo deny check advisories
☐ semgrep scan --config auto --severity ERROR
☐ gitleaks detect --source . --verbose
☐ Check: Cargo.lock committed
☐ Check: No unsafe blocks without justification
```

### TypeScript Projects
```bash
☐ npm audit --json > audit.json
☐ semgrep scan --config auto --severity ERROR
☐ gitleaks detect --source . --verbose
☐ trivy fs --severity HIGH,CRITICAL .
☐ Check: package-lock.json committed
☐ Check: No eval() or innerHTML usage
```

## 🔍 Common Vulnerabilities

### Rust
```rust
// ❌ BAD: Hardcoded secret
let api_key = "sk_live_abc123";

// ✅ GOOD: Environment variable
let api_key = std::env::var("API_KEY")?;

// ❌ BAD: SQL injection
format!("SELECT * FROM users WHERE id = {}", id)

// ✅ GOOD: Parameterized query
sqlx::query!("SELECT * FROM users WHERE id = ?", id)

// ❌ BAD: Panic-prone
option.unwrap()

// ✅ GOOD: Proper error handling
option.ok_or(Error::Missing)?
```

### TypeScript
```typescript
// ❌ BAD: Hardcoded secret
const apiKey = "sk_live_abc123";

// ✅ GOOD: Environment variable
const apiKey = process.env.API_KEY || throw new Error('Missing API_KEY');

// ❌ BAD: XSS vulnerability
element.innerHTML = userInput;

// ✅ GOOD: Safe text content
element.textContent = userInput;

// ❌ BAD: SQL injection
`SELECT * FROM users WHERE id = ${id}`

// ✅ GOOD: Parameterized query
client.query('SELECT * FROM users WHERE id = $1', [id])
```

## 🎯 Decision Matrix

| Finding | Action |
|---------|--------|
| CRITICAL/HIGH | REQUEST_CHANGES + detailed fixes |
| MEDIUM only | COMMENT with recommendations |
| LOW/none | COMMENT + status (no approval) |

## 📝 PR Review Template

### For Issues Found
```markdown
### 🔴 Security Issues Found

## Critical Vulnerabilities
- [CVE-XXXX] Description
  - **Severity**: CRITICAL (CVSS 9.8)
  - **Fix**: Update to version X.Y.Z
  - **Command**: `cargo update -p package`

## Summary
- ❌ CRITICAL: X issues
- ❌ HIGH: X issues
- ✅ MEDIUM: 0 issues

**Action Required**: Fix all CRITICAL and HIGH issues before merge.
```

### For Clean Scan
```markdown
### ✅ Security Analysis Complete

## Scan Results
- ✅ No CRITICAL vulnerabilities
- ✅ No HIGH severity issues
- ✅ No MEDIUM severity issues

## Scans Performed
- ✅ Semgrep static analysis
- ✅ Dependency vulnerability scan
- ✅ Secret scanning
- ✅ Supply chain security check

**Status**: Security checks passed — Tess will provide PR approval.
```

## 🛠️ Useful Commands

### Semgrep
```bash
# Scan with default rules
semgrep scan --config auto

# Scan with custom rules
semgrep scan --config .semgrep.yaml

# High severity only
semgrep scan --config auto --severity ERROR

# JSON output
semgrep scan --config auto --json > results.json

# SARIF output (for GitHub)
semgrep scan --config auto --sarif > results.sarif
```

### Rust Security
```bash
# Audit dependencies
cargo audit --json > audit.json

# Check unsafe code
cargo geiger --output-format Json > geiger.json

# Policy enforcement
cargo deny check advisories
cargo deny check licenses
cargo deny check bans

# Update vulnerable dependency
cargo update -p package-name
```

### TypeScript Security
```bash
# Audit dependencies
npm audit --json > audit.json

# Auto-fix vulnerabilities
npm audit fix

# Moderate+ severity only
npm audit --audit-level=moderate
```

### Secret Scanning
```bash
# Scan for secrets
gitleaks detect --source . --verbose

# With report
gitleaks detect --source . --report-path gitleaks.json

# Check if file is in .gitignore (before flagging)
git check-ignore path/to/file
```

### Vulnerability Scanning
```bash
# Scan filesystem
trivy fs .

# High/Critical only
trivy fs --severity HIGH,CRITICAL .

# JSON output
trivy fs --format json --output trivy.json .
```

## 🚨 Critical Reminders

1. **Always check .gitignore** before flagging secrets:
   ```bash
   git check-ignore path/to/file
   # Exit 0 = ignored (NOT a security issue)
   # Exit 1 = tracked (SECURITY ISSUE)
   ```

2. **Never approve PRs** - Only Tess approves after QA testing

3. **Block on CRITICAL/HIGH** - Zero tolerance for severe issues

4. **Provide remediation** - Always include fix commands and examples

5. **Use multiple tools** - Don't rely on a single scanner

## 📚 Full Documentation

- **Complete Guide**: `/workspace/CIPHER_SECURITY_GUIDELINES.md`
- **Semgrep Rules**: `/workspace/.semgrep.yaml`
- **System Prompt**: See Helm values.yaml

## 🔗 External References

- [Semgrep Docs](https://semgrep.dev/docs/)
- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [RustSec Database](https://rustsec.org/)
- [npm Security](https://docs.npmjs.com/security-best-practices)



