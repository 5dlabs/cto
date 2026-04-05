Implement subtask 10010: Create CI/CD security scanning pipeline definitions (Semgrep, Snyk, CodeQL)

## Objective
Define CI/CD pipeline step configurations for static analysis (Semgrep), dependency scanning (Snyk/Dependabot), and code scanning (CodeQL) across Rust, Go, and TypeScript codebases, with merge blockers for critical/high findings.

## Steps
1. Semgrep configuration:
   - Create `.semgrep.yml` or `semgrep-rules/` directory with rulesets for:
     - Rust: `p/rust`, custom rules for unsafe blocks, SQL injection patterns
     - Go: `p/golang`, `p/owasp-top-ten`
     - TypeScript: `p/typescript`, `p/jwt`, `p/owasp-top-ten`
   - Create a CI workflow step (GitHub Actions or equivalent) that runs Semgrep on PRs.
   - Configure to fail on severity >= HIGH.
2. Snyk/Dependabot configuration:
   - Create `snyk.yml` or `.github/dependabot.yml` for dependency scanning.
   - Cover: Cargo.lock (Rust), go.sum (Go), package.json/bun.lockb (TypeScript/Bun).
   - Configure automatic PR creation for security updates.
   - Set merge blocker for critical vulnerabilities.
3. CodeQL configuration:
   - Create `.github/workflows/codeql-analysis.yml` for Go and TypeScript.
   - Note: CodeQL does not natively support Rust; Semgrep covers Rust static analysis.
   - Configure to run on PRs and weekly scheduled scans.
   - Set merge blocker for high/critical findings.
4. Store all configuration files in the repository's CI directory.
5. Document the scanning pipeline in the ops runbook.

## Validation
Verify Semgrep config parses without errors by running `semgrep --validate`. Verify Dependabot config is valid YAML with correct ecosystem entries. Verify CodeQL workflow YAML is valid. Run each scanner locally against the codebase and confirm it produces output (even if no findings). Confirm merge blocker rules are configured for critical/high severity.