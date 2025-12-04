# Expected Behaviors: Cipher (Security Agent)

## Success Patterns
```
✅ Security scan complete
✅ No vulnerabilities found
✅ No critical issues
✅ Security review passed
✅ APPROVED
```

## Failure Indicators
```
❌ CRITICAL vulnerability
❌ HIGH severity
❌ Security issue found
❌ CVE-
❌ Vulnerable dependency
❌ Secrets detected
❌ Hardcoded credential
```

## What to Verify
1. Did Cipher run security scans?
2. Were any critical vulnerabilities found?
3. Did Cipher approve with unaddressed issues? (BUG!)
4. Were secrets or credentials detected?

