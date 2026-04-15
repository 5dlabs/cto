<identity>
You are cipher working on subtask 9002 of task 9.
</identity>

<context>
<scope>
Systematically verify that every instruction in the cto-billing Anchor program enforces correct signer and authority constraints. Check that initialize_operator is init-once, update_agent_package validates has_one=author, deposit/withdraw validate customer signer, settle/refund/pause/unpause validate operator authority, and permissionless instructions are intentionally permissionless.
</scope>
</context>

<implementation_plan>
1. Enumerate all 10 instructions from the program: initialize_operator, register_agent_package, update_agent_package, create_customer_account, deposit, withdraw, settle_task, refund_task, pause, unpause, update_spending_caps.
2. For each instruction, open the corresponding Accounts struct and instruction handler.
3. Check initialize_operator: Verify `init` constraint on OperatorConfig PDA (ensures one-time init). Verify the signer is stored as `authority`.
4. Check register_agent_package: Confirm permissionless by design — signer stored as `author`, no authority gate. Document as INFO (intentional design).
5. Check update_agent_package: Verify `has_one = author` constraint on the AgentPackage account. Confirm the `author` field matches the transaction signer.
6. Check create_customer_account: Verify `init` constraint. Confirm signer stored as `customer`.
7. Check deposit: Verify signer matches `CustomerBalance.customer`. Check that the source token account belongs to the signer.
8. Check withdraw: Verify signer matches `CustomerBalance.customer`. Verify destination token account belongs to the customer.
9. Check settle_task: Verify signer matches `OperatorConfig.authority`. Verify destination accounts (treasury, optionally author ATA) are constrained.
10. Check refund_task: Verify signer matches `OperatorConfig.authority`.
11. Check pause/unpause: Verify signer matches `OperatorConfig.authority`.
12. Check update_spending_caps: Verify signer matches `CustomerBalance.customer`.
13. For each instruction, record: instruction name, required signers, Anchor constraint used, pass/fail, notes.
14. Any missing signer check is a CRITICAL finding. Any weak check (e.g., unconstrained `Signer` without has_one) is HIGH.
</implementation_plan>

<validation>
A completed checklist covers all 10 instructions with explicit pass/fail for signer validation. Zero CRITICAL findings for missing authority checks. Each instruction entry includes the specific Anchor constraint (e.g., has_one, init, Signer<>) and the file:line reference where it's enforced.
</validation>