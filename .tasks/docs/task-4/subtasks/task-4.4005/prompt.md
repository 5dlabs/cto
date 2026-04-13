<identity>
You are rex working on subtask 4005 of task 4.
</identity>

<context>
<scope>
Add rustdoc comments to every public instruction handler, every Anchor accounts struct, and every state struct explaining purpose, security model, and design rationale — particularly for security-critical decisions like pause bypass on withdraw/refund and the ledger-only refund model.
</scope>
</context>

<implementation_plan>
Systematically add `///` doc comments across the entire program: (1) Each `#[program]` handler function: document purpose, who can call it (signer requirements), what it does, and any security notes. (2) Each Anchor accounts struct (e.g., `SettleTask`, `Withdraw`): document each account field, why it's needed, and any constraints. (3) State structs (OperatorConfig, CustomerBalance, TaskReceipt): document each field's semantics, units, and invariants. (4) Security-critical annotations: on `withdraw` — comment that pause is intentionally not checked (PRD safety valve); on `refund_task` — comment that pause is not checked and explain ledger-only model; on `settle_task` — document the 5-step validation ordering rationale; on fee computation — document multiply-first-then-divide and D4 resolution. (5) Constants: document SLOTS_PER_DAY = 216_000 derivation (400ms slots × 216000 ≈ 24h). (6) Error enum variants: document when each error is raised. Run `cargo doc --no-deps` to verify all doc comments compile. Run `cargo clippy -- -W clippy::missing_docs_in_private_items` to catch undocumented items.
</implementation_plan>

<validation>
Run `cargo doc --no-deps` — generates without warnings. Spot-check: withdraw handler has doc comment mentioning 'safety valve' and 'pause'. Spot-check: refund_task has doc comment explaining ledger-only model. Spot-check: SLOTS_PER_DAY constant has derivation comment. Run `anchor build` — still compiles cleanly after doc additions.
</validation>