<identity>
You are cipher working on subtask 7003 of task 7.
</identity>

<context>
<scope>
Systematically review every instruction handler in the Anchor program for access control vulnerabilities: missing or incorrect account validation constraints, absent signer checks on authority-gated operations, missing owner checks on deserialized accounts, and correctness of the pause mechanism (verify only withdraw/refund can bypass pause).
</scope>
</context>

<implementation_plan>
1. **Enumerate all instructions**: List every pub fn in the program's instruction handlers. For each, document the expected authorization model. 2. **Account validation (has_one / constraint)**: For each instruction's Accounts struct, verify: (a) all authority fields use `has_one` or equivalent `constraint` checks, (b) PDA accounts use proper `seeds` and `bump` derivation, (c) no accounts are accepted without validation that could be substituted by an attacker. 3. **Signer verification**: Confirm every instruction that mutates state or transfers funds requires `Signer<'info>` on the appropriate authority account. Flag any instruction where a non-signer can trigger privileged operations. 4. **Owner checks**: Verify that all accounts deserialized via `Account<'info, T>` have implicit owner checks from Anchor, and any raw `AccountInfo` usage includes explicit `owner == program_id` checks. 5. **Pause bypass analysis**: Locate the pause/unpause mechanism. Verify that only the specified instructions (withdraw, refund) can execute when paused. Confirm the pause authority is properly restricted. Check for pause state manipulation in unexpected instructions. 6. Document each finding with file:line reference, vulnerability class, and preliminary severity.
</implementation_plan>

<validation>
Every instruction handler has a documented access control assessment. All has_one/constraint usages are catalogued. Any missing signer checks are flagged with specific file:line. Pause bypass analysis covers all instruction paths with a matrix showing paused vs. unpaused behavior per instruction.
</validation>