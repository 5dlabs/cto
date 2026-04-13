<identity>
You are cipher working on subtask 7005 of task 7.
</identity>

<context>
<scope>
Aggregate all findings from automated scanning (7002), access control review (7003), and numerical/state safety review (7004) into a structured security audit report with severity ratings (Critical/High/Medium/Low/Informational), specific file:line references, reproduction steps, and actionable remediation guidance for each finding.
</scope>
</context>

<implementation_plan>
1. **Collect all findings** from subtasks 7002, 7003, and 7004 into a master findings list. Deduplicate overlapping findings from automated and manual passes. 2. **Severity classification**: Rate each finding using: Critical (direct fund loss, authority bypass), High (conditional fund loss, privilege escalation), Medium (DoS, state corruption without direct fund loss), Low (gas inefficiency, minor logic issues), Informational (best practice deviations, code quality). 3. **For each finding, document**: (a) Finding ID and title, (b) Severity rating with justification, (c) Vulnerability class (account validation / signer / arithmetic / PDA / rent / reentrancy / pause), (d) Affected file(s) and line number(s), (e) Description of the vulnerability, (f) Reproduction steps or proof-of-concept scenario, (g) Recommended remediation with code snippet where applicable. 4. **Executive summary**: Total finding counts by severity, overall risk assessment, and top-priority remediation items. 5. **Audit metadata**: Commit SHA audited, toolchain versions, tools used, date range, auditor identifiers. 6. Output as `audit/security-audit-report.md` with a companion `audit/findings.json` for programmatic consumption.
</implementation_plan>

<validation>
Report file exists at audit/security-audit-report.md. Every finding from subtasks 7002-7004 is either included or documented as deduplicated. Each finding has all required fields (ID, severity, class, file:line, description, reproduction, remediation). Executive summary tallies match individual finding counts. Companion findings.json is valid JSON and parseable. No finding is left without a severity rating or remediation suggestion.
</validation>