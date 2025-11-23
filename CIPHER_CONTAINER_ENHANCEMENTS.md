# Cipher Container Template Enhancements

## Overview

Enhanced the Cipher container template (`container-cipher.sh.hbs`) with comprehensive security tool integration, helper functions, and detailed instructions for utilizing all security scanning capabilities.

## Changes Made

### 1. Security Tools Verification Section (Lines ~171-244)

Added comprehensive tool verification at container startup:

```bash
════════════════════════════════════════════════════════════════
║              CIPHER SECURITY TOOLS VERIFICATION               ║
════════════════════════════════════════════════════════════════
```

**Verifies availability of:**
- ✅ Semgrep (multi-language SAST)
- ✅ cargo-audit (Rust CVE scanner)
- ✅ cargo-geiger (unsafe code detector)
- ✅ cargo-deny (policy enforcement)
- ✅ gitleaks (secret scanner)
- ✅ trivy (vulnerability scanner)
- ✅ hadolint (Dockerfile linter)

**Checks for documentation:**
- ✅ /workspace/CIPHER_SECURITY_GUIDELINES.md
- ✅ /workspace/CIPHER_QUICK_REFERENCE.md
- ✅ /workspace/.semgrep.yaml

### 2. Comprehensive Security Scan Helper Function (Lines ~536-732)

Added `run_cipher_security_scan()` function that Cipher can call with a single command:

```bash
run_cipher_security_scan . /workspace/security-reports
```

**Function Features:**

#### Automated Scanning Sequence
1. **Semgrep Static Analysis** - Multi-language SAST with custom rules
2. **Rust Security** (if Cargo.toml exists):
   - cargo audit (CVE scanning)
   - cargo geiger (unsafe code detection)
   - cargo deny (policy enforcement)
3. **TypeScript Security** (if package.json exists):
   - npm audit (CVE scanning)
4. **Secret Scanning** - gitleaks with .gitignore awareness
5. **Vulnerability Scanning** - trivy for comprehensive checks
6. **GitHub Code Scanning** - API integration for PR-specific alerts

#### Report Generation
All reports saved to `/workspace/security-reports/`:
- `semgrep-results.json`
- `cargo-audit.json`
- `cargo-geiger.json`
- `cargo-deny.log`
- `npm-audit.json`
- `gitleaks-report.json`
- `trivy-report.json`
- `github-code-scanning.json`

#### Return Codes
- **0**: No CRITICAL/HIGH issues found (pass)
- **1**: HIGH severity issues found (must fix)
- **2**: CRITICAL issues found (block PR)

#### Severity Detection
Automatically counts and reports:
- CRITICAL vulnerabilities (immediate action required)
- HIGH vulnerabilities (must fix before merge)
- Provides clear summary of findings

### 3. Enhanced Language-Specific Security Instructions

#### Rust Projects (Lines ~1972-2020)

**Added comprehensive scan commands:**
```bash
# 1. Semgrep static analysis (CRITICAL - run first)
semgrep scan --config auto --severity ERROR --json > semgrep-results.json

# 2. Dependency vulnerabilities (CRITICAL)
cargo audit --json > cargo-audit.json

# 3. Unsafe code detection (HIGH)
cargo geiger --output-format Json > cargo-geiger.json

# 4. Policy enforcement (HIGH)
cargo deny check advisories
cargo deny check licenses

# 5. Secret scanning (CRITICAL - check .gitignore first)
gitleaks detect --source . --report-path gitleaks-report.json --verbose

# 6. Vulnerability scanning (HIGH)
trivy fs --severity HIGH,CRITICAL --format json --output trivy-report.json .
```

**Security Requirements:**
- Semgrep: No CRITICAL/HIGH issues
- cargo audit: Zero CRITICAL/HIGH CVEs
- cargo geiger: Review unsafe blocks
- Input validation verification
- Cryptography best practices
- Secret detection with .gitignore awareness

**Helper Function Reference:**
```bash
run_cipher_security_scan . /workspace/security-reports
```

**Documentation Links:**
- Complete guide: `/workspace/CIPHER_SECURITY_GUIDELINES.md`
- Quick reference: `/workspace/CIPHER_QUICK_REFERENCE.md`
- Custom rules: `/workspace/.semgrep.yaml`

#### TypeScript/JavaScript Projects (Lines ~1932-1980)

**Added comprehensive scan commands:**
```bash
# 1. Semgrep static analysis (CRITICAL - run first)
semgrep scan --config auto --severity ERROR --json > semgrep-results.json

# 2. Dependency vulnerabilities (CRITICAL)
npm audit --json > npm-audit.json

# 3. Secret scanning (CRITICAL - check .gitignore first)
gitleaks detect --source . --report-path gitleaks-report.json --verbose

# 4. Vulnerability scanning (HIGH)
trivy fs --severity HIGH,CRITICAL --format json --output trivy-report.json .

# 5. ESLint with security plugins (if configured)
eslint --ext .ts,.tsx . --config .eslintrc.json
```

**Security Requirements:**
- Semgrep: No CRITICAL/HIGH issues (XSS, eval, innerHTML, SQL injection)
- npm audit: Zero CRITICAL/HIGH CVEs
- Package security verification
- Secret detection
- XSS protection
- Input sanitization

**Helper Function Reference:**
```bash
run_cipher_security_scan . /workspace/security-reports
```

**Documentation Links:**
- Complete guide: `/workspace/CIPHER_SECURITY_GUIDELINES.md`
- Quick reference: `/workspace/CIPHER_QUICK_REFERENCE.md`
- Custom rules: `/workspace/.semgrep.yaml`

## Usage Examples

### Example 1: Run Comprehensive Scan

Cipher can now run a single command to execute all security scans:

```bash
# Run comprehensive security scan
run_cipher_security_scan . /workspace/security-reports

# Check return code
if [ $? -eq 0 ]; then
    echo "✅ Security scan passed - no CRITICAL/HIGH issues"
    # Proceed with code quality checks
elif [ $? -eq 1 ]; then
    echo "⚠️ HIGH severity issues found - must fix before merge"
    # Review reports and fix issues
elif [ $? -eq 2 ]; then
    echo "❌ CRITICAL issues found - block PR immediately"
    # Review reports and fix critical issues
fi
```

### Example 2: Access Security Reports

All reports are saved in a structured format:

```bash
# View Semgrep findings
cat /workspace/security-reports/semgrep-results.json | jq '.results[]'

# View cargo audit vulnerabilities
cat /workspace/security-reports/cargo-audit.json | jq '.vulnerabilities.list[]'

# View gitleaks secrets (verify .gitignore first!)
cat /workspace/security-reports/gitleaks-report.json | jq '.[]'

# View trivy vulnerabilities
cat /workspace/security-reports/trivy-report.json | jq '.Results[]'
```

### Example 3: Check Individual Tools

Cipher can also run individual tools as needed:

```bash
# Run only Semgrep
semgrep scan --config /workspace/.semgrep.yaml --severity ERROR

# Run only cargo audit
cargo audit --json

# Run only secret scanning
gitleaks detect --source . --verbose

# Check if file is in .gitignore before flagging secret
git check-ignore path/to/suspicious/file
# Exit 0 = ignored (NOT a security issue)
# Exit 1 = tracked (SECURITY ISSUE)
```

## Benefits

### 1. Simplified Workflow
- **Before**: Cipher had to remember and run 6+ different commands
- **After**: Single `run_cipher_security_scan` command runs everything

### 2. Consistent Reporting
- All reports saved to standard location
- JSON format for easy parsing
- Structured logs for debugging

### 3. Automatic Severity Detection
- Function automatically counts CRITICAL/HIGH issues
- Returns appropriate exit codes
- Provides clear summary

### 4. Comprehensive Coverage
- Multi-language support (Rust, TypeScript, Python)
- Multiple scanning approaches (SAST, CVE, secrets, vulnerabilities)
- GitHub integration for PR-specific alerts

### 5. Clear Documentation
- Inline instructions in container script
- References to comprehensive guides
- Helper function usage examples

## Integration with Existing Workflow

The container script enhancements integrate seamlessly with the existing Cipher workflow:

1. **Container Startup**: Tools verification runs automatically
2. **Cipher Prompt**: Includes helper function documentation
3. **Security Scanning**: Cipher can use helper function or individual tools
4. **Report Generation**: All reports saved to standard location
5. **Exit Handling**: Container script can use return codes for PR reviews

## Testing

To test the enhancements:

1. **Build updated runtime image** (with Semgrep and Rust tools)
2. **Deploy updated Helm chart** (with security templates)
3. **Run Cipher on a test PR**:
   ```bash
   # In Cipher container
   run_cipher_security_scan . /workspace/security-reports
   
   # Check reports
   ls -la /workspace/security-reports/
   ```

## Files Modified

- `infra/charts/controller/agent-templates/code/claude/container-cipher.sh.hbs`
  - Added security tools verification (lines ~171-244)
  - Added `run_cipher_security_scan()` helper function (lines ~536-732)
  - Enhanced Rust security instructions (lines ~1972-2020)
  - Enhanced TypeScript security instructions (lines ~1932-1980)

## Related Files

These container enhancements work with:
- `infra/images/runtime/Dockerfile` - Contains installed security tools
- `infra/charts/controller/agent-templates/security/.semgrep.yaml` - Custom rules
- `infra/charts/controller/agent-templates/security/CIPHER_SECURITY_GUIDELINES.md` - Complete guide
- `infra/charts/controller/agent-templates/security/CIPHER_QUICK_REFERENCE.md` - Quick reference
- `infra/charts/controller/values.yaml` - Cipher system prompt

## Next Steps

1. **Mount Security Templates**: Ensure security templates are mounted as ConfigMaps
2. **Test Helper Function**: Verify `run_cipher_security_scan` works in container
3. **Validate Reports**: Check that all report files are generated correctly
4. **Update Other Agents**: Consider similar enhancements for other agent containers

---

**Summary**: Cipher now has comprehensive, easy-to-use security scanning capabilities built directly into the container template. The helper function simplifies the scanning process while maintaining full flexibility for individual tool usage. All security tools, documentation, and custom rules are verified at startup and readily available throughout the security scanning workflow.


