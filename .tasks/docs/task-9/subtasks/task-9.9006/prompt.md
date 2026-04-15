<identity>
You are cipher working on subtask 9006 of task 9.
</identity>

<context>
<scope>
Verify all token account constraints (mint validation, authority constraints, ATA derivation) and analyze denial-of-service vectors including permissionless registration spam and unbounded string inputs.
</scope>
</context>

<implementation_plan>
1. **Token account constraints**: For every instruction that involves token accounts (deposit, withdraw, settle_task, refund_task):
   - Verify `token::mint = usdc_mint` constraint exists — preventing arbitrary SPL token deposits.
   - Verify USDC mint address is either hardcoded or stored in OperatorConfig and validated.
   - Verify `token::authority` constraints match expected owners (customer for source, program PDA for vault).
   - Verify `associated_token::authority` on customer ATAs and author ATAs.
   - Check if the vault accepts only USDC or any SPL token. If any token, this is a HIGH finding.
2. **USDC mint validation**: Is the USDC mint address (e.g., devnet USDC: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` or mainnet) validated? Check if it's a constant in the program, a field in OperatorConfig set at init, or unchecked. Unchecked is MEDIUM.
3. **Permissionless registration DoS**: `register_agent_package` is permissionless. An attacker could:
   - Register millions of packages, bloating on-chain state.
   - Each registration costs rent (~0.002 SOL for typical account size) — attacker pays rent.
   - Document as KNOWN RISK / LOW severity (attacker bears cost).
   - Recommend: add optional registration fee or rate limit for post-hackathon.
4. **String length DoS**: Check `package_id` and `task_id` max lengths:
   - If unbounded, an attacker could create accounts with very long strings, increasing rent cost for the program or consuming excessive blockspace.
   - Verify Anchor `#[max_len()]` attributes on String fields.
   - Document max lengths found and whether they're reasonable.
5. **Account close / rent reclamation**: Can closed accounts (e.g., zeroed TaskReceipts) be reclaimed? Check if any `close = destination` constraints exist. If accounts are never closed, document as INFO (state bloat over time).
6. **Pause mechanism effectiveness**: When paused, verify ALL fund-moving instructions (deposit, withdraw, settle, refund) check the pause flag. If any instruction bypasses the pause check, document as MEDIUM.
7. Produce categorized findings for each vector.
</implementation_plan>

<validation>
All token account instructions have verified mint constraints documented. USDC mint validation approach is identified and assessed. Permissionless registration DoS is analyzed with cost-to-attack estimate. String fields have documented max lengths (or lack thereof is flagged). Pause mechanism coverage is verified across all fund-moving instructions. At least 4 findings are documented across the vectors analyzed.
</validation>