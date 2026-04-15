<identity>
You are cipher, the Security Tooling implementation agent. You own task 9 end-to-end.
</identity>

<context>
<task_overview>
Task 9: Security Review of Solana Program — Authority, Overflow & Escrow Safety (Cipher - Security Tooling)
Perform a focused security review of the cto-billing Anchor program, checking for common Solana program vulnerabilities: missing signer checks, PDA seed collisions, integer overflow/underflow, unauthorized fund access, reentrancy-like patterns, and escrow drain vectors. While a full audit is a non-goal (PRD §4), this review ensures the hackathon demo is not trivially exploitable and documents known risks for post-hackathon hardening.
Priority: medium
Dependencies: 2, 3
</task_overview>
</context>

<implementation_plan>
1. **Static analysis** — Run `cargo-audit` on the program's Cargo.lock for known CVEs in dependencies. Run `clippy` with all Solana-specific lints enabled (`cargo clippy -- -W clippy::all -W clippy::pedantic`).
2. **Signer/authority checks audit** — For every instruction, verify:
   - `initialize_operator`: Only callable once (PDA init constraint).
   - `register_agent_package`: Signer is stored as author; no authority check needed (permissionless by design).
   - `update_agent_package`: Signer must match stored `author` field. Verify Anchor `has_one = author` constraint.
   - `create_customer_account`: Signer stored as customer.
   - `deposit/withdraw`: Signer must match CustomerBalance.customer.
   - `settle_task/refund_task`: Signer must match OperatorConfig.authority.
   - `pause/unpause`: Signer must match OperatorConfig.authority.
   - `update_spending_caps`: Signer must match CustomerBalance.customer.
   - Document any missing checks as CRITICAL findings.
3. **PDA seed analysis** — Verify all PDA seeds are deterministic and collision-resistant:
   - OperatorConfig: singleton, no collision risk.
   - AgentPackage: seeded by package_id string. Verify package_id is validated for length and no null bytes.
   - CustomerBalance: seeded by customer pubkey. Unique per customer.
   - TaskReceipt: seeded by task_id string. Verify task_id length validation.
   - Vault: singleton token account PDA. Verify proper authority derivation.
   - Check for missing `bump` seed storage/verification (Anchor usually handles this, but verify).
4. **Integer overflow/underflow review:**
   - `split_bps * amount / 10000` — verify this uses checked arithmetic. With u64 max and bps=10000, intermediate value `amount * bps` can overflow. Recommend: `(amount as u128 * split_bps as u128 / 10000) as u64` or `checked_mul` + `checked_div`.
   - Balance arithmetic: deposit (addition), withdraw (subtraction), settle (subtraction), refund (addition). All must use checked ops.
   - daily_spent + amount overflow check.
   - total_earned, task_count, success_count increment overflow (unlikely but should use checked).
5. **Escrow safety review:**
   - Verify vault PDA authority is the program, not any user.
   - Verify withdraw can only send funds to the customer who owns the balance.
   - Verify settle_task only sends to treasury and (optionally) author wallet — no arbitrary destination.
   - Verify refund credits balance but requires funds in treasury or vault. Analyze: if treasury has spent funds, can a refund drain the vault? Document the refund funding model.
   - Check for flash-loan-style attacks: can someone deposit, settle, and withdraw in one transaction to manipulate state?
6. **Account validation:**
   - Verify all token accounts use `token::mint` and `token::authority` constraints.
   - Verify USDC mint address is validated (not arbitrary SPL token).
   - Verify `associated_token::authority` constraints on customer/author ATAs.
7. **Denial of service vectors:**
   - Can an attacker register millions of agent packages to bloat state? (Permissionless registration — document as KNOWN RISK with recommendation for registration fee).
   - Can an attacker create TaskReceipts with very long task_ids? (Verify length limits).
8. **Output:** Create `docs/security/solana-program-review.md` with:
   - Executive summary.
   - Findings table: ID, Severity (CRITICAL/HIGH/MEDIUM/LOW/INFO), Description, Location (file:line), Recommendation, Status (Fixed/Acknowledged/Open).
   - Known risks section for post-hackathon (full audit items).
   - Recommended mitigations for each finding.
9. If CRITICAL findings are identified, file them back as blocking issues for Tasks 2/3 to fix before deployment.
</implementation_plan>

<acceptance_criteria>
Security review document `docs/security/solana-program-review.md` is complete with: (1) All 10 instructions have signer/authority checks documented and verified — zero CRITICAL findings for missing auth. (2) Integer arithmetic section confirms all split calculations use overflow-safe math (checked_mul/checked_div or u128 intermediate). (3) PDA seed analysis confirms no collision vectors with length-bounded inputs. (4) Escrow safety section confirms vault authority is program-only and withdraw is customer-scoped. (5) `cargo audit` returns zero known vulnerabilities in dependencies. (6) `cargo clippy` output has zero warnings after recommended fixes. (7) At least 3 security-focused test cases are recommended and added to the test suite (Task 8): overflow at max u64 amounts, unauthorized settle attempt, withdraw-to-wrong-wallet attempt. Findings table has minimum 10 items across all severity levels.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Run static analysis — cargo-audit and clippy with pedantic Solana lints: Execute cargo-audit against the program's Cargo.lock to identify known CVEs in dependencies, and run cargo clippy with all pedantic and Solana-specific lints enabled. Document every finding with severity and recommended fix.
- Audit signer and authority checks for all 10 program instructions: Systematically verify that every instruction in the cto-billing Anchor program enforces correct signer and authority constraints. Check that initialize_operator is init-once, update_agent_package validates has_one=author, deposit/withdraw validate customer signer, settle/refund/pause/unpause validate operator authority, and permissionless instructions are intentionally permissionless.
- Analyze PDA seed determinism, collision resistance, and bump handling: Review all PDA derivations in the program to verify seeds are deterministic, collision-resistant, and properly length-bounded. Check that string-based seeds (package_id, task_id) have length limits enforced and cannot contain null bytes. Verify bump seeds are stored and reused correctly.
- Review integer overflow and underflow safety in all arithmetic operations: Audit every arithmetic operation in the program for overflow/underflow safety. Focus on the BPS split calculation (split_bps * amount / 10000), balance deposit/withdraw/settle/refund arithmetic, daily spending cap checks, and counter increments. Verify checked arithmetic or u128 intermediate values are used consistently.
- Review escrow safety — vault authority, fund flow, and drain vectors: Analyze the escrow mechanism for safety: verify vault PDA authority is program-only, withdraw is customer-scoped, settle sends only to treasury/author, and no flash-loan-style attacks can manipulate state. Assess whether the refund instruction can drain the vault if treasury funds have been spent externally.
- Audit token account constraints and denial-of-service vectors: Verify all token account constraints (mint validation, authority constraints, ATA derivation) and analyze denial-of-service vectors including permissionless registration spam and unbounded string inputs.
- Compile security findings report — docs/security/solana-program-review.md: Aggregate all findings from subtasks 9001-9006 into the final security review document at docs/security/solana-program-review.md. Include executive summary, structured findings table, known risks section, recommended mitigations, and recommended security test cases for Task 8.
</subtasks>