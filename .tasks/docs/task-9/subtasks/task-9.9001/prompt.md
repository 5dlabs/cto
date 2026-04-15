<identity>
You are cipher working on subtask 9001 of task 9.
</identity>

<context>
<scope>
Execute cargo-audit against the program's Cargo.lock to identify known CVEs in dependencies, and run cargo clippy with all pedantic and Solana-specific lints enabled. Document every finding with severity and recommended fix.
</scope>
</context>

<implementation_plan>
1. Navigate to the Anchor program root directory (e.g., `programs/cto-billing/`).
2. Run `cargo audit` against the workspace Cargo.lock. Capture full output including advisory IDs, affected crates, and CVSS scores.
3. Run `cargo clippy --workspace -- -W clippy::all -W clippy::pedantic -D warnings`. Capture every warning/error with file:line references.
4. For each cargo-audit advisory, determine if it affects runtime code or only dev dependencies. Classify severity (CRITICAL if runtime-exploitable, LOW if dev-only).
5. For each clippy finding, classify as security-relevant (e.g., unchecked arithmetic, unsafe blocks) vs. style. Security-relevant items get MEDIUM+ severity.
6. Produce a structured intermediate findings list (JSON or markdown table) with columns: Tool, Finding ID, Affected Crate/File, Severity, Description, Recommendation.
7. If cargo-audit finds CRITICAL runtime CVEs, flag immediately for Task 2/3 owners.
</implementation_plan>

<validation>
cargo-audit completes without error and output is captured. cargo clippy runs against the full workspace and output is captured. An intermediate findings document exists with at least one entry per tool (or explicit 'no findings' notation). All CRITICAL CVEs (if any) are flagged with crate name, version, and advisory ID.
</validation>