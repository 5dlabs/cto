<identity>
You are cipher working on subtask 7002 of task 7.
</identity>

<context>
<scope>
Execute all available automated static analysis tools against the Anchor program source to identify known vulnerability patterns, dependency CVEs, unsafe Rust usage, unchecked arithmetic, and Solana-specific anti-patterns before manual review begins.
</scope>
</context>

<implementation_plan>
1. **Soteria**: Run `soteria -analyzeAll programs/<program_name>` if available. Capture all findings to `audit/soteria-report.json`. If Soteria is unavailable, document the gap and proceed with alternatives. 2. **cargo-audit**: Run `cargo audit` in the program workspace to check for known CVEs in dependencies. Record all advisories. 3. **Clippy security lints**: Run `cargo clippy --all-targets -- -W clippy::arithmetic_side_effects -W clippy::cast_possible_truncation -W clippy::cast_sign_loss -W clippy::integer_arithmetic` and capture warnings. 4. **Pattern grep for raw arithmetic**: Search for non-checked arithmetic on u64/u128 types: `grep -rn '[^_]\(+\|-\|\*\|/\)[^=]' programs/ --include='*.rs'` — cross-reference with checked_add/checked_sub/checked_mul/checked_div usage. 5. **Unsafe block scan**: `grep -rn 'unsafe' programs/ --include='*.rs'` — flag any unsafe blocks for manual review. 6. **init_if_needed scan**: `grep -rn 'init_if_needed' programs/ --include='*.rs'` — flag as high-severity if found (reinitialization attack vector). 7. Consolidate all automated findings into `audit/automated-findings.md` with tool, file:line, and raw output.
</implementation_plan>

<validation>
All specified tools have been executed (or unavailability documented). Consolidated `audit/automated-findings.md` exists with structured entries per tool. Zero init_if_needed usages confirmed. All unsafe blocks (if any) are catalogued with justification assessment.
</validation>