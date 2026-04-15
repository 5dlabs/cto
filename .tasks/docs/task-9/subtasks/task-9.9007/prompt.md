<identity>
You are cipher working on subtask 9007 of task 9.
</identity>

<context>
<scope>
Aggregate all findings from subtasks 9001-9006 into the final security review document at docs/security/solana-program-review.md. Include executive summary, structured findings table, known risks section, recommended mitigations, and recommended security test cases for Task 8.
</scope>
</context>

<implementation_plan>
1. Create `docs/security/` directory if it doesn't exist.
2. Write `docs/security/solana-program-review.md` with the following structure:
   - **Header**: Review date, reviewer, program version/commit hash, scope statement.
   - **Executive Summary**: 2-3 paragraph overview — total findings by severity, overall risk posture, key recommendations. State whether the program is safe for hackathon demo use.
   - **Findings Table**: Markdown table with columns: ID (SEC-001, SEC-002...), Severity (CRITICAL/HIGH/MEDIUM/LOW/INFO), Title, Description, Location (file:line), Recommendation, Status (Open/Fixed/Acknowledged). Minimum 10 findings across all severity levels.
   - **Detailed Findings**: One subsection per finding with full analysis, proof-of-concept description (if applicable), and remediation code snippet.
   - **Known Risks for Post-Hackathon**: Items that are acknowledged but not fixed for the demo — full audit scope, formal verification candidates, economic attack modeling.
   - **Recommended Security Test Cases**: List at least 3 test cases for Task 8:
     a. Overflow test: settle_task with u64::MAX-adjacent amounts.
     b. Unauthorized settle: non-operator tries to call settle_task.
     c. Wrong wallet withdraw: customer A tries to withdraw to customer B's ATA.
     d. (Optional) Pause bypass: call deposit while paused.
   - **Appendix**: Raw tool output (cargo-audit, clippy) or links to CI artifacts.
3. Cross-reference all findings from subtasks 9001-9006. Ensure no finding is dropped.
4. Assign final severity to each finding based on exploitability in the hackathon demo context.
5. If any CRITICAL findings remain unresolved, create a clear blocking notice at the top of the document and note the dependency on Tasks 2/3 for fixes.
6. Review the document for completeness against the test_strategy criteria of the parent task.
</implementation_plan>

<validation>
docs/security/solana-program-review.md exists and contains: (1) Executive summary with risk posture assessment. (2) Findings table with minimum 10 entries spanning at least 3 severity levels. (3) Every finding has ID, severity, description, location, recommendation, and status. (4) Known risks section with at least 3 post-hackathon items. (5) At least 3 recommended security test cases with descriptions suitable for Task 8 implementation. (6) All findings from subtasks 9001-9006 are represented (no dropped findings). (7) Document renders correctly as markdown.
</validation>