# Cipher Security Templates

This directory contains security scanning configurations and guidelines for the Cipher security agent.

## Files

### `.semgrep.yaml`
Semgrep configuration file with custom security rules for Rust and TypeScript/JavaScript projects.

**Features**:
- Rust-specific security patterns (unsafe blocks, unwrap usage, SQL injection)
- TypeScript/JavaScript security patterns (XSS, eval usage, command injection)
- Hardcoded secret detection for both languages
- Weak cryptography detection
- Comprehensive exclusion patterns for test files

**Usage**:
```bash
# Scan with custom rules
semgrep scan --config .semgrep.yaml

# Scan with default rules
semgrep scan --config auto

# Scan with specific severity
semgrep scan --config auto --severity ERROR
```

### `CIPHER_SECURITY_GUIDELINES.md`
Comprehensive security scanning guidelines for Cipher agents.

**Contents**:
- Complete tool reference (Semgrep, cargo-audit, cargo-geiger, gitleaks, trivy)
- Step-by-step security scanning workflow
- Severity classification matrix
- Remediation guidance with code examples
- Best practices for Rust and TypeScript
- CI/CD integration examples

**Target Audience**: Cipher security agents performing automated security scans

## Integration

These templates are automatically mounted into Cipher agent containers via the controller Helm chart. The files are available at:

- `/workspace/.semgrep.yaml` - Semgrep configuration
- `/workspace/CIPHER_SECURITY_GUIDELINES.md` - Complete security guidelines

## Updating Rules

To add new security patterns:

1. Edit `.semgrep.yaml` to add new rules
2. Test rules locally: `semgrep scan --config .semgrep.yaml --test`
3. Update `CIPHER_SECURITY_GUIDELINES.md` with usage examples
4. Rebuild runtime image if new tools are needed
5. Deploy updated Helm chart

## References

- [Semgrep Rule Syntax](https://semgrep.dev/docs/writing-rules/rule-syntax/)
- [Semgrep Registry](https://semgrep.dev/explore)
- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [RustSec Advisory Database](https://rustsec.org/)


