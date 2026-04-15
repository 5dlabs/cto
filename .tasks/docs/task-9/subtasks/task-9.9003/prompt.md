<identity>
You are cipher working on subtask 9003 of task 9.
</identity>

<context>
<scope>
Review all PDA derivations in the program to verify seeds are deterministic, collision-resistant, and properly length-bounded. Check that string-based seeds (package_id, task_id) have length limits enforced and cannot contain null bytes. Verify bump seeds are stored and reused correctly.
</scope>
</context>

<implementation_plan>
1. Enumerate all PDA accounts: OperatorConfig, AgentPackage, CustomerBalance, TaskReceipt, Vault (token account PDA).
2. For OperatorConfig (singleton): Verify seeds are a fixed string (e.g., `b"operator_config"`). Confirm no collision risk since it's a singleton.
3. For AgentPackage: Verify seeds include `b"agent_package"` + `package_id`. Check that `package_id` is a String field — verify max length constraint in the account struct (e.g., `#[max_len(64)]` or equivalent). Check for null byte handling: does Anchor's `seeds` macro handle null bytes in strings, or could `package_id = "abc\0def"` collide with `package_id = "abc"`? Document finding.
4. For CustomerBalance: Verify seeds include `b"customer_balance"` + `customer.key()`. Pubkey is fixed 32 bytes — no collision risk.
5. For TaskReceipt: Verify seeds include task_id string. Check max length constraint. Same null byte analysis as AgentPackage.
6. For Vault: Verify it's a token account PDA with proper `token::authority` set to the program's PDA (not a user). Check authority derivation seeds.
7. For all PDAs: Verify that `bump` is stored in the account struct and reused in subsequent instructions via `bump = account.bump`. If Anchor's `init` handles this automatically, confirm the pattern.
8. Check if any PDA uses user-controlled seeds without length bounds — this is a MEDIUM finding (potential rent exhaustion via large accounts).
9. Produce findings: seed scheme per PDA, length bounds, null byte risk, bump handling status.
</implementation_plan>

<validation>
All 5 PDA types have documented seed schemes with explicit collision analysis. String-seeded PDAs (AgentPackage, TaskReceipt) have verified max length constraints. Null byte collision risk is assessed and documented. Bump seed storage/reuse pattern is confirmed for all PDAs. Vault authority derivation is confirmed as program-owned.
</validation>