## Acceptance Criteria

- [ ] Security review document `docs/security/solana-program-review.md` is complete with: (1) All 10 instructions have signer/authority checks documented and verified — zero CRITICAL findings for missing auth. (2) Integer arithmetic section confirms all split calculations use overflow-safe math (checked_mul/checked_div or u128 intermediate). (3) PDA seed analysis confirms no collision vectors with length-bounded inputs. (4) Escrow safety section confirms vault authority is program-only and withdraw is customer-scoped. (5) `cargo audit` returns zero known vulnerabilities in dependencies. (6) `cargo clippy` output has zero warnings after recommended fixes. (7) At least 3 security-focused test cases are recommended and added to the test suite (Task 8): overflow at max u64 amounts, unauthorized settle attempt, withdraw-to-wrong-wallet attempt. Findings table has minimum 10 items across all severity levels.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.