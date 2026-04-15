<identity>
You are cipher working on subtask 9005 of task 9.
</identity>

<context>
<scope>
Analyze the escrow mechanism for safety: verify vault PDA authority is program-only, withdraw is customer-scoped, settle sends only to treasury/author, and no flash-loan-style attacks can manipulate state. Assess whether the refund instruction can drain the vault if treasury funds have been spent externally.
</scope>
</context>

<implementation_plan>
1. **Vault authority**: Locate the vault token account PDA. Verify its `authority` is set to the program's PDA (using `seeds` and `bump`), NOT any user pubkey. If authority is a user key, this is CRITICAL.
2. **Withdraw scoping**: In the withdraw instruction, verify:
   - The signer is the customer who owns the CustomerBalance.
   - The destination token account is the customer's ATA (constrained via `associated_token::authority = customer`).
   - The amount cannot exceed `balance.amount`.
   - Funds transfer uses `token::transfer` with PDA signer seeds (program signs on behalf of vault).
3. **Settle fund flow**: In settle_task, trace the fund flow:
   - Source: vault PDA.
   - Destination 1: treasury token account. Verify treasury is constrained (e.g., stored in OperatorConfig).
   - Destination 2: author ATA (optional, for author split). Verify it's derived from AgentPackage.author.
   - No third destination should be possible.
4. **Refund funding model**: In refund_task, determine:
   - Does refund transfer tokens back to customer from vault? If so, are the tokens still in the vault after settle sent them to treasury?
   - Or does refund only credit the customer's `balance.amount` field (bookkeeping only, no token transfer)?
   - If tokens are transferred: vault must have sufficient balance. After settle moves funds to treasury, a refund could fail (insufficient vault balance) or drain other customers' funds. Document as HIGH if fund commingling exists.
5. **Flash-loan-style attack**: Can a single transaction include: deposit → settle_task → withdraw? In Solana, a transaction can include multiple instructions. Check:
   - Does settle_task require a valid TaskReceipt? Can an attacker create a receipt and settle in the same tx?
   - Is there a time-lock or confirmation requirement between deposit and settle?
   - Document feasibility and severity.
6. **Cross-customer fund isolation**: If multiple customers deposit into the same vault, can settle_task for customer A use customer B's funds? Check whether settle verifies `task_cost <= customerA.balance.amount` before transferring from vault.
7. Produce findings with severity classifications for each vector analyzed.
</implementation_plan>

<validation>
Vault authority is confirmed as program-owned PDA with documented seeds. Withdraw is confirmed customer-scoped with ATA constraint. Settle fund flow is traced with both destinations (treasury, author) verified as constrained. Refund funding model is documented with explicit analysis of whether vault can be drained. Flash-loan attack feasibility is assessed. Cross-customer isolation is verified. At least 2 findings are documented (minimum INFO-level for design observations).
</validation>