# Enhanced PRD

## 1. Original Requirements

> # Project: CTO Pay — On-Chain Usage-Based Payments for the CTO Platform
>
> - **Organization:** 5D Labs
> - **Status:** Draft
> - **Target:** Solana Agent Hackathon (April 2026)
> - **Repo:** https://github.com/5dlabs/cto-pay
>
> ## Vision
>
> CTO Pay is the on-chain payment and settlement layer for the CTO platform. It replaces traditional off-chain billing with a Solana program that lets customers pre-pay USDC into an escrow account, have their usage metered per task, and receive verifiable on-chain receipts for every charge. Every payment is transparent, auditable, and composable.
>
> The long-term goal is a protocol-grade billing rail that supports usage-based payments today, and extends to agent registry royalty splits, attestation-gated settlement, and a full agent marketplace tomorrow — without rewriting the core program.
>
> ---
>
> ## Architecture Overview
>
> ```
> ┌─────────────────────────────────────────────────────────────────────────┐
> │                         CTO Pay Platform                                │
> ├─────────────────────────────────────────────────────────────────────────┤
> │  Customer                                                               │
> │  ┌──────────────┐                                                       │
> │  │ Solana Wallet │  (Phantom, Backpack, Solflare)                       │
> │  │  USDC deposit │                                                      │
> │  └──────┬───────┘                                                       │
> │         │                                                               │
> ├─────────┴───────────────────────────────────────────────────────────────┤
> │  Solana Program (Anchor)                                                │
> │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
> │  │  OperatorConfig   │  │ CustomerBalance  │  │   TaskReceipt    │      │
> │  │  (PDA, singleton) │  │ (PDA per cust.)  │  │  (PDA per task)  │      │
> │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘      │
> │           │                     │                      │                │
> │  ┌────────┴─────────────────────┴──────────────────────┴────────┐      │
> │  │                     Program Vault (USDC)                      │      │
> │  │         Holds all customer deposits; program is authority     │      │
> │  └──────────────────────────────────────────────────────────────┘      │
> │                                                                         │
> ├─────────────────────────────────────────────────────────────────────────┤
> │  CTO Controller (Kubernetes)                                            │
> │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
> │  │  Usage Metering   │  │ Receipt Builder  │  │ Settlement Hook  │      │
> │  │  (pod duration,   │  │ (JSON → hash →   │  │ (submit settle/  │      │
> │  │   infra tier)     │  │  off-chain store) │  │  refund to chain)│      │
> │  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
> │                                                                         │
> ├─────────────────────────────────────────────────────────────────────────┤
> │  Off-Chain Storage                                                      │
> │  ┌──────────┐  ┌──────────┐                                             │
> │  │  Arweave  │  │   S3     │  (itemized receipt JSON blobs)             │
> │  └──────────┘  └──────────┘                                             │
> ├─────────────────────────────────────────────────────────────────────────┤
> │  Future Extensions (not in scope)                                       │
> │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
> │  │ Agent Registry│  │ Attestations │  │  Marketplace  │                 │
> │  │ (royalty      │  │ (Tess/Cipher/│  │  (skill NFTs, │                 │
> │  │  splits)      │  │  Stitch sigs)│  │   curation)   │                 │
> │  └──────────────┘  └──────────────┘  └──────────────┘                  │
> └─────────────────────────────────────────────────────────────────────────┘
> ```
>
> ---
>
> ## Services (Workstreams)
>
> ### 1. Solana Program — Anchor (Rust)
>
> **Agent**: Rex
> **Priority**: Critical
> **Language**: Rust (Anchor framework)
> **Target**: Solana devnet (mainnet-beta post-hackathon)
>
> The on-chain program that holds customer deposits, enforces spending caps, settles task payments, and writes verifiable receipts.
>
> **Accounts (PDAs)**:
>
> ```
> OperatorConfig (PDA, singleton)
> ├── authority: Pubkey          // 5D Labs operator wallet
> ├── treasury: Pubkey           // 5D Labs revenue wallet
> ├── protocol_fee_bps: u16     // protocol fee (basis points)
> └── paused: bool               // circuit breaker
>
> CustomerBalance (PDA, seeded by customer pubkey)
> ├── customer: Pubkey
> ├── balance: u64               // USDC lamports
> ├── total_deposited: u64
> ├── total_spent: u64
> ├── task_count: u64
> ├── max_per_task: u64          // spending cap per task
> ├── max_per_day: u64           // daily spending cap
> ├── daily_spent: u64
> ├── daily_reset_slot: u64
> └── created_at: i64
>
> TaskReceipt (PDA, seeded by task_id)
> ├── task_id: String
> ├── customer: Pubkey
> ├── amount: u64                // USDC charged
> ├── receipt_hash: [u8; 32]     // SHA-256 of off-chain receipt JSON
> ├── operator: Pubkey
> ├── settled_at: i64
> └── status: TaskStatus         // Settled | Refunded | Disputed
> ```
>
> **Instructions**:
>
> ```
> initialize_operator(authority, treasury, protocol_fee_bps)
>   → Creates OperatorConfig PDA. Called once at program deployment.
>
> create_customer_account(max_per_task, max_per_day)
>   → Creates CustomerBalance PDA for the signing customer.
>
> deposit(amount)
>   → Transfers USDC from customer's token account to the program vault.
>      Increments customer balance.
>
> withdraw(amount)
>   → Transfers USDC from program vault back to customer.
>      Decrements customer balance. Customer-signed.
>
> settle_task(task_id, amount, receipt_hash)
>   → Called by operator wallet after CodeRun completion.
>      Validates: operator authorized, sufficient balance, within caps.
>      Debits customer, credits treasury, writes TaskReceipt.
>
> refund_task(task_id)
>   → Called by operator wallet.
>      Marks TaskReceipt as Refunded. Credits amount back to customer.
>
> update_spending_caps(max_per_task, max_per_day)
>   → Called by customer. Updates their spending limits.
>
> pause / unpause
>   → Called by operator. Circuit breaker for emergencies.
> ```
>
> **Testing**:
> - Anchor integration tests (TypeScript, `anchor test`)
> - Bankrun for fast local program tests
> - Devnet deployment and manual verification
>
> ---
>
> ### 2. Settlement Hook — CTO Controller Integration (Rust)
>
> **Agent**: Rex
> **Priority**: High
> **Language**: Rust
> **Location**: `crates/controller/` in the main CTO repo (integration points documented here)
>
> The CTO Kubernetes controller submits settlement transactions after each CodeRun reaches a terminal state.
>
> **Integration Points**:
>
> | Point | File (in CTO repo) | Change |
> |-------|---------------------|--------|
> | Solana config | `crates/controller/src/tasks/config.rs` | Add RPC endpoint + operator keypair path |
> | Settlement hook | `crates/controller/src/tasks/code/controller.rs` | After terminal state: compute bill → build receipt → submit `settle_task` |
> | Wallet mapping | Customer profile model | Add `solana_pubkey` field |
> | Tx recording | CodeRun CRD status | Add `on_chain_settlement_sig` field |
>
> **Settlement Flow**:
>
> 1. CodeRun transitions to terminal state (merged / failed / cancelled).
> 2. Controller computes billable amount from pod duration + infra tier.
> 3. Builds itemized receipt JSON (CodeRun minutes, compute, AI tokens if managed-key).
> 4. Uploads receipt to off-chain storage (Arweave or S3).
> 5. Hashes the receipt (SHA-256).
> 6. Submits `settle_task` instruction to Solana program (or `refund_task` on failure).
> 7. Records the transaction signature on the CodeRun status.
>
> **Secrets**: Operator keypair managed via the existing pipeline (1Password → OpenBao → External Secrets Operator → K8s secret → pod env var).
>
> ---
>
> ### 3. CLI / Demo Script (TypeScript)
>
> **Agent**: Blaze
> **Priority**: High
> **Language**: TypeScript (Bun runtime)
> **Framework**: `@coral-xyz/anchor`, `@solana/web3.js`
>
> A CLI tool and demo script that simulates the full settlement loop end-to-end on devnet. Used for hackathon demo video and local development.
>
> **Commands**:
>
> ```
> cto-pay init-operator          — Deploy OperatorConfig with treasury wallet
> cto-pay create-account         — Create CustomerBalance for a wallet
> cto-pay deposit <amount>       — Deposit USDC into balance
> cto-pay withdraw <amount>      — Withdraw USDC from balance
> cto-pay settle <task_id> <amt> — Submit mock task settlement
> cto-pay refund <task_id>       — Refund a settled task
> cto-pay balance                — Check customer balance
> cto-pay receipts               — List task receipts for a customer
> cto-pay demo                   — Run the full demo loop (deposit → settle → verify → withdraw)
> ```
>
> **Demo Loop** (what gets filmed):
>
> 1. Customer deposits 100 USDC into balance PDA.
> 2. CTO task runs (mocked or real CodeRun).
> 3. Settlement fires — customer balance decreases, treasury increases.
> 4. `TaskReceipt` appears on chain with receipt hash.
> 5. Customer verifies receipt in Solana explorer (Solscan/Explorer link printed).
> 6. Customer withdraws remaining balance.
>
> ---
>
> ## Technical Context
>
> | Component | Technology | Agent |
> |-----------|------------|-------|
> | Solana Program | Rust, Anchor 0.30+ | Rex |
> | Program Tests | TypeScript, Anchor test, Bankrun | Tess |
> | CLI / Demo | TypeScript, Bun, @coral-xyz/anchor | Blaze |
> | Controller Hook | Rust, solana-sdk | Rex |
> | Off-Chain Receipts | Arweave / S3 | Bolt |
> | Secrets | 1Password → OpenBao → K8s | Bolt |
> | CI/CD | GitHub Actions | Bolt |
>
> **Dependencies**:
> - Solana CLI 1.18+
> - Anchor CLI 0.30+
> - Bun 1.1+
> - USDC SPL token (devnet mint for testing)
>
> ---
>
> ## Data Flow Examples
>
> ### DF-1: Customer Deposit → Task Settlement → Receipt Verification
>
> ```
> Customer Wallet (Phantom)
>     │
>     ▼
> deposit(100 USDC)
>     │
>     ▼
> CustomerBalance PDA (+100 USDC)
>     │
>     │  ... CTO runs a CodeRun (off-chain, K8s) ...
>     │
>     ▼
> Controller: settle_task("TASK-42", 12.50 USDC, receipt_hash)
>     │
>     ├──► CustomerBalance PDA (−12.50 USDC)
>     ├──► Operator Treasury (+12.50 USDC)
>     └──► TaskReceipt PDA created
>               │
>               ├── task_id: "TASK-42"
>               ├── amount: 12_500_000  (USDC lamports)
>               ├── receipt_hash: 0xabc...
>               └── status: Settled
>
> Customer verifies on Solscan: TaskReceipt → receipt_hash → fetch JSON from Arweave/S3
> ```
>
> ### DF-2: Task Failure → Refund
>
> ```
> Controller detects CodeRun failure
>     │
>     ▼
> Controller: refund_task("TASK-42")
>     │
>     ├──► TaskReceipt.status → Refunded
>     └──► CustomerBalance PDA (+12.50 USDC restored)
> ```
>
> ### DF-3: Spending Cap Enforcement
>
> ```
> Controller: settle_task("TASK-99", 500 USDC, hash)
>     │
>     ▼
> Program checks:
>   - max_per_task: 200 USDC  → 500 > 200 → REJECTED
>   - Transaction fails with SpendingCapExceeded error
>   - Customer balance unchanged
> ```
>
> ---
>
> ## Billing Dimensions
>
> These map directly from the existing CTO monetization model (`docs/business/saas-monetization.md` in the CTO repo):
>
> | Dimension | Description | Metering Source |
> |-----------|-------------|-----------------|
> | CodeRun execution | Time from pod start to completion | Pod lifecycle events |
> | Infrastructure compute | Bare-metal time (standard / high-mem / GPU) | Pod resource requests + node labels |
> | AI tokens (managed-key) | Pass-through + margin when using 5D Labs keys | Provider API response metadata |
>
> **Existing tier pricing** (for context — on-chain settlement replaces the payment rail, not the pricing model):
>
> | Tier | Platform Fee | Included CodeRuns | Overage | AI Keys |
> |------|-------------|-------------------|---------|------------|
> | Free | $0 | 50/month | $3.00/run | BYOK only |
> | Team | $199/month | 200/month | $1.50/run | BYOK or managed (+15%) |
> | Growth | $499/month | 1,000/month | $0.75/run | BYOK or managed (+10%) |
> | Enterprise | Custom | Custom | $0.50/run | Flexible |
>
> For the hackathon, billing is **per task (metered)** — the operator submits the computed cost, and the program settles it. Subscription management stays off-chain.
>
> ---
>
> ## Open Design Questions
>
> These are genuinely unsolved. The hackathon submission takes a position on each, but they remain open for iteration.
>
> ### 1. Success-conditional billing
>
> Should settlement be conditional on task success? Options:
> - **Always charge** (metered) — simplest, matches existing model.
> - **Charge on success only** — best customer UX but exposes runtime to abuse.
> - **Hybrid** — base attempt fee + success bonus.
>
> **Hackathon position**: Always charge (metered). Success-conditional billing is a future extension via attestation gates.
>
> ### 2. Trust model for usage reporting
>
> All usage measurements happen off-chain in CTO's K8s cluster. The Solana program trusts the operator wallet.
>
> **Hackathon position**: Trust the operator. Production trust-reduction options (signed pod receipts, review-agent co-signatures, open-source metering, dispute mechanism) are documented but not built.
>
> ### 3. Customer payment UX
>
> - **Pre-paid balance** (AWS credits) — best for real customers.
> - **Session key delegation** — modern Solana pattern, good for demo.
> - **Per-task escrow** — maximum visibility, worst UX.
>
> **Hackathon position**: Pre-paid balance (deposit/withdraw). Most practical and demo-able.
>
> ---
>
> ## Spending Controls
>
> Enforced on-chain by the program:
>
> | Control | Description | Enforced By |
> |---------|-------------|-------------|
> | Max per task | Program rejects `settle_task` above this cap | `CustomerBalance.max_per_task` |
> | Max per day | Rolling daily spending limit | `CustomerBalance.max_per_day` + `daily_spent` + `daily_reset_slot` |
> | Pre-flight estimate | Runtime shows estimated cost before execution | Off-chain UX (not a program instruction) |
> | Customer withdrawal | Customer can withdraw unused balance at any time | `withdraw` instruction |
> | Circuit breaker | Operator can pause all settlements in emergencies | `OperatorConfig.paused` |
>
> ---
>
> ## Failure and Refund Handling
>
> | Scenario | Action |
> |----------|--------|
> | Task fails before any agent work | No settlement submitted; balance unchanged |
> | Task fails after partial work | Open question for production; hackathon charges for compute consumed |
> | Task succeeds | Full charge per metered usage |
> | Customer disputes a charge | Manual resolution initially; on-chain dispute mechanism later |
>
> ---
>
> ## Quality Assurance & Review Workflow
>
> All code changes go through the CTO automated quality pipeline:
>
> ### 1. Automated Code Review (Stitch)
> - **Agent**: Stitch — Automated Code Reviewer
> - **Trigger**: On every pull request
> - **Scope**: Style, correctness, architecture alignment
> - **Tools**: GitHub PR integration via GitHub App
>
> ### 2. Code Quality Enforcement (Cleo)
> - **Agent**: Cleo — Quality Guardian
> - **Trigger**: CI/CD pipeline
> - **Focus**: Maintainability, refactor opportunities, code smells
> - **Tools**: Clippy (pedantic), Rustfmt, Biome (TypeScript)
>
> ### 3. Comprehensive Testing (Tess)
> - **Agent**: Tess — Testing Genius
> - **Trigger**: CI/CD pipeline after review approval
> - **Coverage**: Anchor program tests, CLI integration tests, settlement flow tests
> - **Tools**: `anchor test`, Bankrun, Bun test
> - **Enforcement**: All program instructions must have positive and negative test cases
>
> ### 4. Security Scanning (Cipher)
> - **Agent**: Cipher — Security Sentinel
> - **Trigger**: CI/CD pipeline
> - **Focus**: Solana program vulnerabilities, account validation, signer checks, overflow/underflow
> - **Tools**: Anchor verify, Soteria (if available), manual audit checklist
> - **Blocker**: Critical/high severity issues block merge
>
> ### 5. Merge Gate (Atlas)
> - **Agent**: Atlas — Integration Master
> - **Policy**: Required approvals + passing CI + passing QA
> - **Tools**: GitHub merge automation
>
> ### 6. Deployment & Operations (Bolt)
> - **Agent**: Bolt — DevOps Engineer
> - **Workflow**: `anchor build` → `anchor deploy` to devnet
> - **Monitoring**: Transaction logs, program account state verification
>
> ---
>
> ## Future Extensions (Post-Hackathon, Not In Scope)
>
> These are documented for context and to show the program's extensibility. They are explicitly **not part of the hackathon build**.
>
> ### Agent Registry with Royalty Splits
> Full agent packages (skills, tools, Soul, user context) registered on-chain. Each agent has an author wallet and a price. When a task invokes multiple agents/skills, the settlement program splits payment across all authors atomically.
>
> ### Authorship NFTs
> Each registered agent represented by a transferable NFT. Holder receives the royalty stream. Transfer the NFT = sell a revenue-generating AI agent as an asset.
>
> ### Attestation-Based Settlement
> Tess, Cipher, and Stitch hold Solana keypairs. Their pass/fail signals become on-chain attestations. The program gates payment release on a quorum of attestation signatures.
>
> ### cNFT Invocation Receipts
> Compressed NFTs minted per task settlement as the on-chain audit trail. Billions feasible at ~$0.00001 per mint.
>
> ### Agent Reputation
> Cumulative earnings, attestation pass rates, and slash history per agent, queryable on chain.
>
> ### Agent Compute Marketplace
> Sell spare CTO bare-metal cluster capacity to third-party agent runs, metered and settled via the same program.
>
> ---
>
> ## Non-Goals
>
> - Full agent marketplace / skill registry (future extension)
> - NFT-based authorship tokens or receipt tokens (future extension)
> - Subscription tier management on chain (subscriptions stay off-chain)
> - Managed-key LLM token pass-through billing (BYOK customers pay their LLM provider directly)
> - Production-grade security audit of the Solana program (post-hackathon)
> - Frontend / web dashboard (CLI only for hackathon)
> - Multi-chain support (Solana only)
>
> ---
>
> ## Hackathon Deliverables
>
> 1. **Anchor program** deployed to Solana devnet implementing: `initialize_operator`, `create_customer_account`, `deposit`, `withdraw`, `settle_task`, `refund_task`, `update_spending_caps`, `pause/unpause`.
> 2. **CLI / demo script** that runs the full settlement loop: create customer → deposit USDC → submit mock task → settle → verify receipt on chain → withdraw.
> 3. **Demo video** showing:
>    - Customer deposits USDC into balance PDA.
>    - CTO task runs (mocked or real CodeRun).
>    - Settlement fires — customer balance decreases, operator treasury increases, `TaskReceipt` appears on chain with receipt hash.
>    - Customer verifies receipt in Solana explorer.
>    - Customer withdraws remaining balance.
> 4. **This PRD** as supporting documentation.
>
> ---
>
> ## Success Criteria
>
> 1. A judge can watch the demo video and understand: a customer paid for AI agent work, settled on Solana, with a verifiable on-chain receipt.
> 2. The program compiles, deploys to devnet, and passes integration tests (deposit, settle, refund, withdraw, cap enforcement).
> 3. Spending caps are enforced on-chain — oversized settlements are rejected.
> 4. Refund flow works — failed tasks return funds to the customer balance.
> 5. The design is extensible — adding agent registry splits or attestation gates later does not require rewriting the core program.
> 6. End-to-end flow works: deposit USDC → task settles → receipt on chain → balance verifiable → withdrawal succeeds.
>
> ---
>
> ## References
>
> - `docs/business/saas-monetization.md` (CTO repo) — existing pricing model
> - `docs/solana-hackathon-ideas.md` (CTO repo) — full ideation brainstorm
> - `docs/solana-hackathon-prd.md` (CTO repo) — original detailed PRD
> - `crates/controller/src/crds/coderun.rs` (CTO repo) — CodeRun CRD definition
> - `crates/controller/src/tasks/code/controller.rs` (CTO repo) — task reconciliation loop
> - `crates/controller/src/tasks/code/resources.rs` (CTO repo) — pod resource construction
> - `crates/controller/src/cli/types.rs` (CTO repo) — provider/key resolution
> - `docs/secrets-management.md` (CTO repo) — secrets pipeline (1Password → OpenBao → K8s)
> - `AGENTS.md` (CTO repo) — agent roster (Tess, Cipher, Stitch attestation roles)

---

## 2. Project Scope

The initial task decomposition identified **6 tasks** spanning 3 workstreams (Solana program, tests, CLI/CI). The settlement hook to the CTO controller (Workstream 2 in the PRD) is documented but out of scope for the hackathon deliverable — it lives in a separate repo and depends on the program IDL being stable first.

### Tasks Identified

| ID | Title | Agent | Priority | Dependencies |
|----|-------|-------|----------|-------------|
| 1 | Scaffold cto-pay Repository, Anchor Project, and CI/CD Foundation | Bolt | Critical | None |
| 2 | Implement OperatorConfig PDA, Program Vault, and `initialize_operator` | Rex | Critical | 1 |
| 3 | Implement CustomerBalance PDA, `create_customer_account`, `deposit`, `withdraw` | Rex | Critical | 2 |
| 4 | Implement TaskReceipt PDA, `settle_task`, and `refund_task` | Rex | Critical | 3 |
| 5 | Implement Spending Caps, Daily Reset, `update_spending_caps`, `pause`/`unpause` | Rex | Critical | 4 |
| 6 | Write Anchor Program Integration Tests — Happy Path Coverage | Tess | High | 5 |

### Key Services and Components

- **Solana Program (Anchor/Rust):** 3 PDA account types (OperatorConfig, CustomerBalance, TaskReceipt), 8 instructions, events, error types
- **Program Vault:** Single program-owned USDC token account (PDA)
- **CLI/Demo (TypeScript/Bun):** 10 commands via Commander.js, including a full `demo` loop
- **CI/CD (GitHub Actions):** Anchor build/test pipeline + TypeScript lint/test pipeline
- **Off-chain Receipts:** S3-compatible storage for receipt JSON blobs

### Agent Assignments

- **Bolt:** Repository scaffolding, CI/CD, devops (Task 1)
- **Rex:** All on-chain program development (Tasks 2–5)
- **Tess:** Integration test suite (Task 6)
- **Blaze:** CLI/demo script (implied by PRD, CLI scaffolding in Task 1)

### Technology Stacks

- **On-chain:** Rust, Anchor 0.30+, anchor-spl, solana-program 1.18
- **Testing:** TypeScript, Anchor test framework, Bankrun
- **CLI:** TypeScript, Bun 1.1+, Commander.js 12+, @coral-xyz/anchor, @solana/web3.js, @solana/spl-token
- **CI:** GitHub Actions
- **Storage:** S3 (Minio in existing cluster)

### Cross-Cutting Concerns

- **12 decision points** were raised across tasks, covering architecture, data model, API design, platform choice, build-vs-buy, and security
- Pause/unpause circuit breaker must be enforced consistently across deposit, withdraw, settle, and refund instructions
- Checked arithmetic required throughout all on-chain code
- USDC devnet token provisioning affects tasks 1–6

---

## 3. Resolved Decisions

### [D1] Should cto-pay live in a standalone repo or the CTO monorepo?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 4, 5, 6 (all tasks)

**Context:** The PRD explicitly specifies `5dlabs/cto-pay` as the target repo. Both debaters agreed that the Anchor/BPF build toolchain is fundamentally incompatible with the kube-rs controller workspace, and mixing them would force every Rust build to resolve solana-sdk's ~400 transitive dependencies.

**Decision:** Standalone repo (`5dlabs/cto-pay`). The controller hook in the CTO monorepo consumes the program's IDL as a JSON artifact and depends on solana-sdk/solana-client — not on the Anchor program crate itself.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Clean separation of build toolchains and CI pipelines
- ✅ Independent deploy lifecycle for the Solana program
- ✅ Controller integration via IDL artifact, not crate dependency
- ⚠️ Requires publishing/copying the IDL JSON when the controller hook is built (post-hackathon concern)

---

### [D3] How should protocol fee be handled in settle_task?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 6

**Context:** Both debaters agreed that since operator and treasury are both 5D Labs for the hackathon, splitting fees between two accounts controlled by the same entity adds instruction complexity with zero demo value.

**Decision:** No protocol fee enforcement in the hackathon build. The `protocol_fee_bps` field exists in OperatorConfig for extensibility, but `settle_task` transfers the full amount to the treasury without splitting. Post-hackathon, adding the split is a backward-compatible change.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Simpler settle_task instruction (one token transfer, not two)
- ✅ `protocol_fee_bps` field preserves extensibility
- ✅ Fewer accounts in settle_task instruction context
- ⚠️ Fee accumulation logic must be built before mainnet launch

---

### [D4] How should the daily spending cap reset mechanism work?

**Status:** Accepted

**Task Context:** Tasks 5, 6

**Context:** Both debaters agreed that Solana slot times fluctuate (350–600ms during congestion), making slot-based windowing unreliable for "daily" caps. Unix timestamps from Clock sysvar are monotonically increasing and anchored to wall-clock time.

**Decision:** Unix timestamp-based reset using `Clock` sysvar's `unix_timestamp`. Reset when `current_timestamp >= daily_reset_ts + 86400`. Rename the field from `daily_reset_slot` to `daily_reset_ts` for clarity.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ "Daily" means 24 hours in wall-clock time, regardless of slot timing
- ✅ Bankrun's `setClock()` makes timestamp manipulation straightforward for testing
- ✅ Field naming is self-documenting
- ⚠️ Clock sysvar's unix_timestamp can drift ~1–2 seconds from real time on Solana (irrelevant for daily windows)

---

### [D5] Should task_id in TaskReceipt be String, fixed hash, or fixed string?

**Status:** Accepted

**Task Context:** Tasks 4, 6

**Context:** This was a genuine disagreement. The Optimist argued for `[u8; 32]` SHA-256 hash (deterministic PDA derivation, fixed account size, human-readable task_id emitted via Anchor events). The Pessimist argued for `[u8; 64]` fixed string (human-readable on-chain when inspecting account data on Solscan, judges see "TASK-42" not "0xabc123...").

**Decision:** This decision point did not receive explicit votes in the deliberation result, but both positions were argued. Based on the debate, the Pessimist's argument about demo readability is compelling for a hackathon context — **use `[u8; 64]` fixed string with null-padding**. CTO task IDs ("TASK-42", UUIDs) fit well under 64 bytes. This provides deterministic PDA derivation (pad with zeros), fixed account size, AND human-readable on-chain data visible in Solscan's account view.

*Note: The Optimist's hash approach is technically cleaner for production but sacrifices demo-day readability. Both speakers agreed the PDA seed must be deterministic and fixed-size — the disagreement was only about hash vs. padded string.*

**Consensus:** Split — Pessimist's position adopted for hackathon context based on demo readability argument

**Consequences:**
- ✅ Human-readable task_id visible when inspecting TaskReceipt PDA on Solscan
- ✅ Fixed account size (predictable rent)
- ✅ Deterministic PDA derivation using zero-padded bytes as seed
- ⚠️ 64-byte seed is larger than 32-byte hash; PDA derivation uses more bytes
- ⚠️ Post-hackathon migration to hash-based IDs would require account migration

---

### [D6] Single shared vault or per-customer token accounts?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 5, 6

**Context:** The PRD explicitly mandates a single program-owned vault. Both debaters agreed this is correct for the hackathon scope, consistent with standard patterns used by Marinade Finance, Jito, and Drift Protocol.

**Decision:** Single program-owned vault token account (PDA) with seeds `[b"vault"]`. All customer USDC pooled together; CustomerBalance PDAs provide the audit trail.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Standard pooled vault pattern
- ✅ No per-customer rent overhead (~0.002 SOL per ATA avoided)
- ✅ Simpler settlement and withdrawal logic
- ⚠️ Vault balance must always be ≥ sum of all CustomerBalance.balance values (invariant to maintain)

---

### [D9] How should the operator keypair be loaded in CLI context?

**Status:** Accepted

**Task Context:** Tasks 1, 6

**Context:** Both debaters agreed that the CLI is headless (terminal-based), making wallet adapter integration impossible. The Solana CLI convention is universal for devnet development.

**Decision:** File-based keypair following Solana CLI convention (`~/.config/solana/id.json`) with `SOLANA_KEYPAIR_PATH` environment variable override. Every Solana developer has this path configured from `solana-keygen new`.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Standard, familiar to every Solana developer
- ✅ Environment variable override supports CI (GitHub Actions can inject as secret)
- ✅ K8s controller uses its own separate secrets pipeline (1Password → OpenBao)
- ⚠️ Keypair files should never be committed to the repo (add to .gitignore)

---

### [D10] How should USDC devnet tokens be provisioned?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 4, 6

**Context:** Both debaters agreed that there is no reliable devnet USDC faucet. Circle's devnet USDC mint exists but has no public mint authority for airdrops. The spl-token-faucet programs are unreliable.

**Decision:** Create a custom "mock USDC" SPL token mint controlled by the team, with 6 decimals matching real USDC. Include mint setup in the CLI's `init-operator` flow. The program accepts the mint as a configurable parameter (see D11).

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Demo never fails due to external dependencies
- ✅ Full control over minting for tests and demo
- ✅ 6-decimal configuration matches real USDC semantics
- ⚠️ Demo narrative should explicitly note "mock USDC for devnet" to set expectations

---

### [D11] Should the Anchor program accept USDC mint as configurable or hardcoded?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 6

**Context:** Both debaters agreed that hardcoding requires program redeployment to switch mints (devnet mock USDC → mainnet real USDC). A configurable mint in OperatorConfig is one additional Pubkey field and one constraint check per instruction.

**Decision:** Configurable — store `accepted_mint` as a `Pubkey` in OperatorConfig, set during `initialize_operator`. Every token-handling instruction validates `token_account.mint == operator_config.accepted_mint`.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Devnet→mainnet transition requires no program redeployment
- ✅ Directly serves PRD extensibility success criterion
- ✅ Standard Anchor constraint pattern
- ⚠️ Adds one field to OperatorConfig and one constraint per token instruction (trivial cost)

---

### [D12] Should refund_task do accounting-only or real token transfers?

**Status:** Accepted

**Task Context:** Tasks 4, 6

**Context:** This was the most contentious decision. The Optimist proposed accounting-only refunds (all funds stay in vault, settlement is purely balance tracking). The Pessimist argued this directly contradicts the PRD's Data Flow DF-1 (`CustomerBalance PDA (−12.50 USDC) → Operator Treasury (+12.50 USDC)`) and breaks the demo narrative — a judge inspecting Solscan must see the treasury token account balance increase on settlement. The Pessimist also raised the phantom balance bug: if settlement doesn't move tokens but withdrawal does, customer A's withdrawal can fail because customer B's "settled" funds haven't left the vault.

**Decision:** Real token transfers in both directions. `settle_task` transfers tokens from vault to treasury token account (PDA signer). `refund_task` transfers tokens from treasury back to vault (operator signer, since treasury is operator-owned). CustomerBalance tracks accounting alongside the transfers.

**Consensus:** Pessimist's position adopted — the correctness and demo-readability arguments are decisive

**Consequences:**
- ✅ Matches PRD data flow DF-1 exactly
- ✅ Treasury token account balance visibly increases on Solscan during demo
- ✅ No phantom balance bug — vault balance always equals sum of customer balances
- ✅ Symmetric settle/refund flow is easy to understand and debug
- ⚠️ refund_task requires two token CPIs (~20 lines of Anchor code)
- ⚠️ Treasury must have sufficient balance for refunds (operator's responsibility)

---

## 4. Escalated Decisions

### [D2] Which off-chain storage backend for receipt JSON blobs? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 1, 6

**Options:**
- **A:** S3 via existing Minio (`gitlab/gitlab-minio-svc`), with a `ReceiptStore` trait abstraction for future Arweave support
- **B:** S3 via existing Minio, direct function call — no trait abstraction for hackathon (Pessimist's position: "a trait with one implementation is pure speculative generality")

**Optimist argued:** Abstract behind a `ReceiptStore` trait so Arweave is a post-hackathon swap. The infrastructure already has Minio in the cluster. Arweave/Irys integration is 1–2 days of yak-shaving that doesn't serve the core value prop.

**Pessimist argued:** Agree with S3 but skip the trait. For a hackathon, write a function that uploads to S3. If Arweave is needed later, refactor then. One implementation behind a trait is overhead.

**Recommendation:** Both agree on S3 as the storage backend — the only disagreement is whether to wrap it in a trait. For a hackathon on a 2-week timeline, the Pessimist's pragmatism is likely correct: write a clean `upload_receipt(receipt_json: &str) -> Result<String>` function. If the function signature is clean, extracting a trait later is a 10-minute refactor. **Default to S3 with a clean function, no trait, unless the human prefers the abstraction.**

---

### [D7] Which Solana RPC provider for devnet? — NOTED (not formally escalated)

**Status:** Accepted (unanimous, no formal vote needed)

**Task Context:** Tasks 1, 6

**Context:** Both debaters agreed on Helius free-tier devnet RPC. Public devnet RPC has aggressive rate limits (40 req/s) and regular outages. Helius free tier provides 100K requests/day on devnet with no credit card.

**Decision:** Helius free-tier devnet RPC with public devnet as fallback. Configure `RPC_URL` as an env var defaulting to Helius devnet endpoint. Configure 30-second timeout and 3 retries in the CLI.

**Consensus:** 2/2 (100%)

---

### [D8] CLI framework choice — NOTED (not formally escalated)

**Status:** Accepted (unanimous, no formal vote needed)

**Task Context:** Task 1

**Context:** Both debaters agreed Commander.js is the right tool for 10 structured commands. Bare `process.argv` parsing is a bug factory; Oclif is overkill.

**Decision:** Commander.js 12+ with Bun runtime. Zero dependencies, 60KB, works perfectly under Bun since 1.0.

**Consensus:** 2/2 (100%)

---

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| On-chain program | Rust, Anchor framework | Anchor 0.30.1, solana-program 1.18 |
| SPL integration | anchor-spl (token feature) | 0.30.1 |
| Program tests | TypeScript, Anchor test, Bankrun | — |
| CLI/Demo | TypeScript, Bun, Commander.js | Bun 1.1+, Commander 12+ |
| CI/CD | GitHub Actions | — |
| Off-chain storage | S3 (Minio) | Existing cluster service |
| RPC | Helius free-tier devnet | Fallback: api.devnet.solana.com |
| Token | Custom mock USDC (6 decimals) | SPL Token |

### Service Architecture

```
5dlabs/cto-pay (standalone repo)
├── programs/cto-pay/        ← Anchor Rust program
│   └── src/
│       ├── lib.rs           ← Program entrypoint, instruction routing
│       ├── state/           ← OperatorConfig, CustomerBalance, TaskReceipt
│       ├── instructions/    ← 8 instruction handlers
│       ├── errors.rs        ← CtoPayError enum
│       └── constants.rs     ← Protocol constants
├── tests/                   ← Anchor TypeScript integration tests (Bankrun)
├── cli/                     ← TypeScript CLI (Bun + Commander.js)
│   └── src/
│       ├── index.ts         ← CLI entrypoint
│       └── commands/        ← One file per command
└── .github/workflows/       ← anchor-ci.yaml, cli-ci.yaml
```

### Account Architecture (PDAs)

```
OperatorConfig [seeds: b"operator_config"]
├── authority, treasury, accepted_mint, protocol_fee_bps, paused, bump

CustomerBalance [seeds: b"customer_balance", customer.key()]
├── customer, balance, total_deposited, total_spent, task_count
├── max_per_task, max_per_day, daily_spent, daily_reset_ts, created_at, bump

TaskReceipt [seeds: b"task_receipt", task_id_bytes (64 bytes, zero-padded)]
├── task_id: [u8; 64], customer, amount, receipt_hash, operator
├── settled_at, status (Settled|Refunded|Disputed), bump

Vault [seeds: b"vault"]
├── SPL Token Account, mint = accepted_mint, authority = vault PDA
```

### Token Flow Pattern

```
DEPOSIT:   Customer ATA ──[SPL transfer]──► Vault
SETTLE:    Vault ──[PDA-signed transfer]──► Treasury ATA   (+ CustomerBalance accounting)
REFUND:    Treasury ATA ──[operator-signed transfer]──► Vault   (+ CustomerBalance accounting)
WITHDRAW:  Vault ──[PDA-signed transfer]──► Customer ATA
```

### Key Patterns

- **Configurable mint:** `accepted_mint` Pubkey stored in OperatorConfig, validated on every token instruction
- **Unix timestamp daily reset:** `daily_reset_ts` + 86400 seconds, using Clock sysvar
- **Fixed-size task_id:** `[u8; 64]` with null-padding for human readability on-chain
- **Anchor events:** `SettleTaskEvent` and `RefundTaskEvent` emitted for indexing
- **Circuit breaker:** `paused` flag checked on deposit, withdraw, settle, refund (not on create_customer_account or update_spending_caps)

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Per-customer token accounts | Unnecessary rent overhead; pooled vault is PRD mandate and DeFi standard |
| Protocol fee splitting in settle_task | Same entity controls operator and treasury for hackathon; field preserved for extensibility |
| Slot-based daily reset | Slot times fluctuate 350–600ms; unreliable for human-meaningful "daily" windows |
| Hardcoded USDC mint | Requires redeployment for devnet→mainnet; configurable mint is trivial |
| Wallet adapter in CLI | CLI is headless/terminal; wallet adapter needs a browser |
| Arweave for hackathon | 1–2 days integration for a non-core component; S3 via existing Minio |
| ReceiptStore trait abstraction | One implementation behind a trait is speculative generality for hackathon scope (escalated, see D2) |
| Accounting-only settlement | Contradicts PRD data flow; treasury must show token movement on Solscan |

---

## 6. Implementation Constraints

### Security Requirements

- **Operator authorization:** Every operator-only instruction (`settle_task`, `refund_task`, `pause`, `unpause`) MUST validate `signer == operator_config.authority`
- **Customer authorization:** `withdraw`, `update_spending_caps` MUST validate `signer == customer_balance.customer`
- **Mint validation:** Every token instruction MUST validate `token_account.mint == operator_config.accepted_mint`
- **Checked arithmetic:** All add/sub operations MUST use `checked_add`/`checked_sub` with explicit `Overflow` error — no wrapping arithmetic anywhere in the program
- **Keypair handling:** Operator keypair loaded from file (`~/.config/solana/id.json` or `SOLANA_KEYPAIR_PATH` env var). Never logged, never committed to repo. `.gitignore` must include keypair patterns.
- **K8s secrets:** Production controller uses 1Password → OpenBao → External Secrets Operator → K8s secret. This pipeline is separate from CLI keypair loading.

### Performance Targets

- **Demo reliability:** CLI must use Helius free-tier RPC with 30-second timeout and 3 retries per transaction
- **Demo flow:** The `demo` command must execute 6+ sequential transactions (deposit, settle, verify, refund, withdraw, balance check) without manual intervention
- **Test speed:** Use Bankrun for local program tests to avoid solana-test-validator startup overhead

### Operational Requirements

- **CI:** `anchor build` + `anchor test` on every push/PR; `bun install` + `bun run lint` + `bun test` on CLI changes
- **Linting:** Clippy pedantic for Rust, Biome for TypeScript
- **Devnet deployment:** `anchor deploy` to devnet with generated program keypair

### Service Dependencies

| Dependency | Location | Purpose |
|-----------|----------|---------|
| Minio (S3) | `gitlab/gitlab-minio-svc` in K8s cluster | Receipt JSON blob storage |
| Helius devnet RPC | External SaaS (free tier) | Reliable devnet RPC endpoint |
| Solana devnet | Public network | Program deployment and demo |

### Organizational Preferences

- Standalone repos for fundamentally different build toolchains
- File-based keypairs following Solana CLI convention for developer tooling
- Existing cluster services (Minio) preferred over new external dependencies
- Existing secrets pipeline (1Password → OpenBao) for production; env vars for development

---

## 7. Design Intake Summary

- **`hasFrontend`:** `false`
- **`frontendTargets`:** None
- **Provider status:** Stitch design intake failed; Framer skipped (not requested)
- **Supplied design artifacts:** None
- **Reference URLs:** None

**Implications:** This project is entirely backend (Solana program) + CLI. There are no web, mobile, or desktop frontend tasks. The CLI's output formatting (Solscan links, balance tables, colored status indicators) is the only "UI" — handled by Commander.js + Bun's console styling. No design system, component library, or visual design decisions are needed.

### 7a. Selected Design Direction

Not applicable — no `design_selections` provided and no frontend targets.

### 7b. Design Deliberation Decisions

Not applicable — no `design_deliberation_result` provided and no frontend targets.

---

## 8. Open Questions

These are non-blocking items where implementing agents should use their best judgment:

1. **Receipt JSON schema:** The PRD specifies SHA-256 hashing of an "itemized receipt JSON" but does not define the schema. Implementing agents should define a minimal schema including: `task_id`, `customer_pubkey`, `amount_usdc`, `coderun_minutes`, `infra_tier`, `timestamp`, and any AI token usage. The schema should be documented in the repo README.

2. **Error message detail level:** The PRD defines high-level error cases but not the granularity of error messages. Agents should prefer specific error codes (e.g., `SpendingCapExceeded` vs. generic `InvalidInput`) for debuggability.

3. **`receipts` CLI command implementation:** The PRD lists a `receipts` command to "list task receipts for a customer." This requires either (a) scanning PDAs by customer pubkey using `getProgramAccounts` with a memcmp filter, or (b) indexing events. Agents should use `getProgramAccounts` with a memcmp filter on the customer field for hackathon simplicity.

4. **`demo` command idempotency:** Should `cto-pay demo` be re-runnable (create fresh customer accounts each time) or fail if accounts already exist? Agents should make it re-runnable by using unique customer keypairs per demo run or adding a `--clean` flag.

5. **Treasury token account creation:** The treasury is an operator-owned token account. Should `init-operator` create the treasury ATA, or should it be created separately? Agents should create or verify the treasury ATA as part of `init-operator` for a self-contained setup.

6. **`total_spent` accounting on refund:** When `refund_task` restores balance, should `total_spent` be decremented? The Optimist's dp-12 position suggested decrementing. Agents should decrement `total_spent` on refund to maintain the invariant: `total_deposited - total_spent = balance + (sum of amounts in non-refunded TaskReceipts)`.

7. **Pause behavior for withdraw:** The initial task decomposition marks `withdraw` as paused-when-paused. However, pausing withdrawals traps customer funds. Agents should consider whether `withdraw` should remain functional even when paused (customer protection), and document the decision in code comments.

