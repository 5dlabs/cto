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
> |------|-------------|-------------------|---------|---------||
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

The initial task decomposition identified **6 tasks** (with additional tasks for testing, CLI, CI/CD, and controller integration implied by the PRD but not yet decomposed into the parsed set). Below is the scope as discovered:

### Tasks Identified

| ID | Title | Agent | Stack | Dependencies | Priority |
|----|-------|-------|-------|--------------|----------|
| 1 | Dev Infra Bootstrap — Anchor Project, Devnet Config, Receipt Storage, Secrets | Bolt | Kubernetes/Helm | None | High |
| 2 | Solana Program Foundations — Account Structures, Errors, Events & initialize_operator | Rex | Rust/Anchor | Task 1 | High |
| 3 | Customer Account Management — create_customer_account & update_spending_caps | Rex | Rust/Anchor | Task 2 | High |
| 4 | Deposit & Withdraw — USDC Token Transfer Instructions | Rex | Rust/Anchor | Task 3 | High |
| 5 | Settle Task — On-Chain Settlement with Spending Cap Enforcement | Rex | Rust/Anchor | Task 4 | High |
| 6 | Refund Task — On-Chain Refund with Status Transition | Rex | Rust/Anchor | Task 5 | High |

**Note:** The PRD also specifies a CLI/Demo tool (Blaze, TypeScript/Bun), a program test suite (Tess, TypeScript/Bankrun), a controller settlement hook (Rex, Rust), CI/CD pipelines (Bolt), and secret management — these are implied as additional tasks (referenced as Task 7+ in the debate) but were not fully decomposed in the initial parse.

### Key Services and Components

1. **Solana Anchor Program** (`programs/cto-pay/`) — Core on-chain logic: 8 instructions, 3 PDA account types, 1 vault token account
2. **Off-Chain Receipt Storage** — S3/MinIO bucket for receipt JSON blobs, referenced by on-chain SHA-256 hash
3. **CTO Controller Integration** (`crates/controller/`) — Settlement hook in the existing Kubernetes operator
4. **CLI/Demo Tool** (`cli/`) — TypeScript/Bun CLI for hackathon demo and local development
5. **Infrastructure** — Devnet deployment targets, secrets pipeline (1Password → OpenBao → K8s), ConfigMaps

### Agent Assignments

- **Rex** (Rust/Anchor): Solana program (Tasks 2–6), controller settlement hook
- **Bolt** (Kubernetes/Helm): Infrastructure bootstrap (Task 1), CI/CD, secrets, receipt storage
- **Blaze** (TypeScript/Bun): CLI/demo tool
- **Tess** (TypeScript/Bankrun): Program and integration test suites
- **Cipher**: Security scanning
- **Stitch/Cleo**: Code review and quality enforcement
- **Atlas**: Merge gate automation

### Cross-Cutting Concerns

- **Checked arithmetic** required on all balance mutations (overflow/underflow prevention)
- **Pause/unpause circuit breaker** checked by every customer-facing instruction
- **PDA derivation consistency** across program, CLI, and controller
- **USDC mint validation** on every token instruction
- **Event emission** for off-chain indexing and CLI tooling
- **Secrets management** for operator keypair across local dev, CI, and K8s

---

## 3. Resolved Decisions

### [D1] Should off-chain receipt blobs go to S3/MinIO or Arweave?

**Status:** Accepted

**Task Context:** Task 1 (Dev Infra Bootstrap)

**Context:** Both debaters agreed that MinIO is already deployed in-cluster (`gitlab/gitlab-minio-svc`) and Arweave adds latency (2–4s), a bundler dependency, and token costs that hurt demo speed. The on-chain SHA-256 hash provides verifiability independent of storage backend.

**Decision:** S3-compatible storage (in-cluster MinIO) for the hackathon. Upload behind a simple function (not a trait/interface — YAGNI per Pessimist). Document Arweave as the production path. The receipt hash on-chain ensures verifiability regardless of storage backend.

**Consensus:** 2/2 (100%) — both speakers agreed on MinIO.

**Consequences:**
- **Positive:** Zero new dependencies, sub-100ms upload latency, leverages existing infrastructure
- **Positive:** On-chain receipt hash preserves verifiability narrative for judges
- **Negative:** Receipts are not permanently immutable (MinIO data can be deleted/modified); mitigated by on-chain hash serving as integrity check
- **Caveat (Pessimist):** Do not build a `ReceiptStore` trait abstraction for a single backend — a plain function is sufficient. Swap the implementation when a second backend actually exists.

---

### [D2] Should the CLI/demo tool live in cto-pay or the cto monorepo?

**Status:** Accepted

**Task Context:** Task 1 (Dev Infra Bootstrap), future CLI task (Task 7)

**Context:** Both debaters agreed immediately. The PRD specifies `https://github.com/5dlabs/cto-pay` as the repo, and hackathon judges need a self-contained, independently runnable submission.

**Decision:** CLI lives in the `cto-pay` repo alongside the Anchor program. Self-contained hackathon submission: judges clone one repo, run one command.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Single repo clone for judges, natural co-location with Anchor IDL
- **Positive:** No monorepo build dependencies or context required
- **Negative:** CLI code is not co-located with other CTO TypeScript apps; acceptable for hackathon scope

---

### [D3] Single global vault PDA or per-customer vault token accounts?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 5, 6 (all program instructions involving token transfers)

**Context:** The Optimist argued that the PRD explicitly specifies a single vault architecture and that per-customer vaults add rent costs, CPI complexity, and account proliferation. The Pessimist agreed but explicitly named the blast radius risk: a bug in balance arithmetic can drain all customers simultaneously.

**Decision:** Single global vault token account with a program PDA as authority. `CustomerBalance` PDAs track individual balances as a software ledger.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Simpler instruction contexts (one vault account, one PDA authority derivation)
- **Positive:** No per-customer ATA creation costs (~0.002 SOL each)
- **Negative:** A balance arithmetic bug has total-pool blast radius — all customer funds at risk simultaneously
- **Caveat (Pessimist):** This decision is **conditional on**: (1) every balance mutation uses `checked_add`/`checked_sub`/`checked_mul`, and (2) the test suite (Tess) includes explicit overflow/underflow test cases for all arithmetic in Tasks 4, 5, and 6. Not production-ready without a formal audit.

---

### [D4] Slot-based or Clock sysvar timestamp for daily cap reset?

**Status:** Accepted

**Task Context:** Tasks 3, 5 (create_customer_account, settle_task)

**Context:** Both debaters agreed. Slot-based timing drifts ±15 minutes over 24 hours due to variable slot times (380–450ms on mainnet). The Clock sysvar provides validator-consensus-backed `unix_timestamp` accurate to ~1–2 seconds, which is more than sufficient for daily spending caps.

**Decision:** Use `Clock::get().unix_timestamp` for daily cap resets. Store `daily_reset_ts` as `i64` on `CustomerBalance`. Compare against Clock sysvar timestamp to determine if a 24-hour window has elapsed. Bankrun's `set_clock()` enables test manipulation.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Accurate to ~1–2 seconds (vs ±15 minutes for slot-based)
- **Positive:** Testable via Bankrun `set_clock()`
- **Positive:** More intuitive for developers reading the code
- **Negative:** None identified
- **Implementation note:** The PRD's original `daily_reset_slot: u64` field must be renamed to `daily_reset_ts: i64` on the `CustomerBalance` account structure.

---

### [D5] How should task_id be stored: fixed byte array or variable String?

**Status:** Accepted

**Task Context:** Tasks 2, 5, 6, and CLI (Task 7)

**Context:** The Optimist proposed hashing task_id to `[u8; 32]` via SHA-256 for deterministic sizing and clean PDA seeds. The Pessimist argued this destroys debuggability — you cannot look up a TaskReceipt by human-readable task_id without an off-chain index or hash computation. The Pessimist proposed direct `[u8; 32]` zero-padded encoding with a length enforcement error, noting CTO CodeRun names are short Kubernetes resource names (well under 32 bytes).

**Decision:** Fixed `[u8; 32]` zero-padded with direct byte encoding, used as PDA seed. Task IDs exceeding 32 bytes are rejected with a `TaskIdTooLong` error. The raw bytes are stored on `TaskReceipt` for human-readable Solana explorer inspection. No hashing.

**Consensus:** 2/2 (100%) — the Pessimist's counter-proposal was adopted. The Optimist's hash approach was rejected due to debuggability concerns.

**Consequences:**
- **Positive:** Human-readable task IDs visible in Solana explorer without off-chain tooling
- **Positive:** No hash computation required on program or client side
- **Positive:** Deterministic account sizing (fixed 32 bytes)
- **Positive:** Simpler PDA derivation — raw bytes as seed
- **Negative:** 32-byte limit on task_id length; IDs exceeding this are rejected
- **Caveat:** CTO CodeRun names are Kubernetes resource names, typically well under 32 bytes (e.g., `coderun-abc123`). If the naming scheme ever changes to produce longer IDs, this constraint must be revisited.
- **Implementation note:** The PRD's original `task_id: String` field on TaskReceipt changes to `task_id: [u8; 32]`. The `TaskIdTooLong` error code already exists in the error enum. The CLI must zero-pad short task IDs and enforce the 32-byte limit client-side.

---

### [D6] Implement protocol fee split in settle_task now or defer?

**Status:** Accepted

**Task Context:** Tasks 2, 5 (OperatorConfig, settle_task)

**Context:** Both debaters agreed immediately. Implementing the split adds a second CPI, a separate fee destination ATA, and rounding edge cases for a feature unused at hackathon. The PRD's future royalty splits will redesign this anyway.

**Decision:** Store `protocol_fee_bps` on `OperatorConfig` (set during `initialize_operator`) but do **not** implement fee split logic in `settle_task`. The full settlement amount goes to the operator treasury. Split logic is deferred to post-hackathon.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Simpler `settle_task` instruction (one CPI, one destination)
- **Positive:** `protocol_fee_bps` field signals extensibility intent to hackathon judges at zero cost
- **Negative:** No actual fee collection until post-hackathon
- **Negative:** Future implementation requires a program upgrade (new instruction logic + potentially new accounts)

---

### [D7] Emit Anchor events or rely on transaction log parsing?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 5, 6, and CLI (Task 7)

**Context:** The Optimist proposed emitting events on all state-changing instructions. The Pessimist agreed on the high-value instructions but argued that events on `initialize_operator` and `create_customer_account` are waste — these are called rarely and are trivially queryable via account reads.

**Decision:** Emit Anchor events (`emit!`) on **settle_task, refund_task, deposit, and withdraw**. Skip events on `initialize_operator` and `create_customer_account` (called rarely, queryable via direct account reads).

**Consensus:** 2/2 (100%) — the Pessimist's scoped approach was adopted.

**Consequences:**
- **Positive:** CLI `receipts` command can parse events from transaction logs instead of expensive `getProgramAccounts` scans
- **Positive:** ~200 CU per event — negligible against 200k CU budget
- **Positive:** Enables real-time event streaming for demo video
- **Negative:** `initialize_operator` and `create_customer_account` are not event-indexed; must use account reads to query these
- **Implementation note:** The event structs defined in Task 2 (`OperatorInitialized`, `CustomerAccountCreated`) should still be defined for completeness, but the `emit!` calls are omitted from those instructions. Events `DepositMade`, `WithdrawalMade`, `TaskSettled`, `TaskRefunded` are emitted.

---

### [D8] Which USDC mint for testing vs demo?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 4, 5 (infrastructure, OperatorConfig, token instructions)

**Context:** Both debaters agreed. Bankrun has no real USDC — a custom test mint is mandatory. For the deployed demo, the official USDC devnet mint looks authentic to judges. The mint address is configured on `OperatorConfig` during `initialize_operator` (dovetails with D9).

**Decision:** Test-controlled mint for Bankrun/anchor-test; official USDC devnet mint (`4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`) for the deployed demo. Mint address is set on `OperatorConfig` during `initialize_operator`.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Tests have full control over mint authority and supply
- **Positive:** Demo looks authentic with real USDC branding
- **Positive:** No code changes between test and demo environments — just different `initialize_operator` parameters
- **Negative:** Test mint is not "real" USDC; test assertions must not depend on external mint state

---

### [D9] Hardcode USDC mint, store on OperatorConfig, or accept any mint?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 5 (OperatorConfig, deposit, withdraw, settle_task)

**Context:** Both debaters agreed. Hardcoding breaks across environments (test/devnet/mainnet have different mints). Accepting any mint is a security hole. Storing the accepted mint on `OperatorConfig` is both secure and test-friendly.

**Decision:** Store `accepted_mint: Pubkey` on `OperatorConfig`, set once during `initialize_operator`, validated on every token instruction (`deposit`, `withdraw`, `settle_task`) with a custom `InvalidMint` error.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Single source of truth for accepted mint, set at initialization
- **Positive:** Works across all environments without code changes
- **Positive:** Prevents deposits of wrong tokens with a clear error
- **Negative:** Changing the accepted mint requires reinitialization (or a new instruction); acceptable since mint changes are extraordinary
- **Implementation note:** Add `accepted_mint: Pubkey` to `OperatorConfig` struct. Add `InvalidMint` to the error enum. Every instruction that touches token accounts must validate `mint == operator_config.accepted_mint`.

---

### [D10] Use solana-sdk Rust crates or shell out to Solana CLI?

**Status:** Accepted

**Task Context:** Controller integration (not explicitly numbered in initial parse, but referenced in PRD §2 and debate)

**Context:** Both debaters agreed on using native Rust SDK. The Pessimist raised a critical operational concern: if Solana RPC calls are made inline in the Kubernetes reconciliation loop, devnet RPC slowness (no SLA) will block K8s operator reconciliation.

**Decision:** Use `solana-sdk` and `solana-client` Rust crates directly for native async integration with the controller's tokio runtime. Settlement submission **must be async and decoupled from the reconciliation loop**: record `settlement_pending` status on the CRD, spawn the Solana submission as a separate async task, update the CRD status on transaction confirmation.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Native `Result` error handling, type safety, async compatibility
- **Positive:** No `solana` CLI binary dependency in the container image
- **Positive:** Decoupled submission prevents reconciliation loop stalls
- **Negative:** `solana-sdk` adds ~30s to initial compile time; incremental builds are fast
- **Negative:** More complex state machine in the controller (pending → confirmed → failed states)
- **Caveat (Pessimist):** The fire-and-forget + async retry pattern is mandatory, not optional. Inline RPC calls in the reconciliation loop is a known failure mode in K8s operators making synchronous external calls.

---

## 4. Escalated Decisions

No decisions were escalated. All 10 decision points reached consensus (2/2 agreement).

---

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| On-chain program | Rust, Anchor framework | Anchor 0.30+ |
| Program testing | TypeScript, Bankrun, Anchor test | Solana CLI 1.18+ |
| CLI/Demo | TypeScript, Bun runtime | Bun 1.1+ |
| Controller integration | Rust, solana-sdk, solana-client, kube-rs | tokio async runtime |
| Off-chain receipt storage | S3-compatible (MinIO, in-cluster) | Existing `gitlab/gitlab-minio-svc` |
| Secrets | 1Password → OpenBao → External Secrets Operator → K8s | Existing pipeline |
| Target chain | Solana devnet (mainnet-beta post-hackathon) | — |

### Service Architecture

```
┌───────────────────────────────────────────────────────────────┐
│  cto-pay repo (self-contained hackathon submission)           │
│                                                               │
│  programs/cto-pay/         Anchor program (Rust)              │
│  ├── OperatorConfig PDA    accepted_mint, treasury, fee_bps   │
│  ├── CustomerBalance PDA   per-customer ledger, spending caps │
│  ├── TaskReceipt PDA       [u8;32] task_id, receipt_hash      │
│  └── Vault Token Account   single global vault, PDA authority │
│                                                               │
│  cli/                      TypeScript/Bun CLI                 │
│  tests/                    Anchor integration tests           │
└───────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────┐
│  cto repo (existing monorepo)                                 │
│                                                               │
│  crates/controller/        K8s operator (Rust/kube-rs)        │
│  ├── Settlement hook       async, decoupled from reconcile    │
│  ├── Receipt builder       JSON → SHA-256 → MinIO upload      │
│  └── CRD status            settlement_pending → confirmed     │
└───────────────────────────────────────────────────────────────┘

          ┌─────────────────┐
          │  MinIO (S3)     │  receipt JSON blobs
          │  (in-cluster)   │  
          └─────────────────┘
          
          ┌─────────────────┐
          │  Solana devnet   │  program deployment + RPC
          └─────────────────┘
```

### Communication Patterns

1. **Customer → Program:** Direct Solana transactions (deposit, withdraw, create_customer_account, update_spending_caps)
2. **Controller → Program:** Async Solana transactions via solana-client (settle_task, refund_task), decoupled from K8s reconciliation loop
3. **Controller → MinIO:** S3 PutObject for receipt JSON uploads
4. **CLI → Program:** Solana transactions via @coral-xyz/anchor SDK for demo/testing

### Key Patterns

- **Single vault with software ledger:** All USDC pools in one vault token account; `CustomerBalance` PDAs track per-customer balances
- **Configurable mint:** `OperatorConfig.accepted_mint` set at initialization, validated on every token instruction
- **Clock-based daily resets:** `daily_reset_ts: i64` compared against `Clock::get().unix_timestamp` for 24-hour spending windows
- **Fixed-size task IDs:** `[u8; 32]` zero-padded, used directly as PDA seeds — no hashing
- **Deferred fee splits:** `protocol_fee_bps` stored but not used in settlement logic
- **Scoped event emission:** Events on deposit, withdraw, settle, refund; not on setup instructions
- **Async settlement:** Controller records `settlement_pending` on CRD, spawns async Solana submission, updates on confirmation

### Explicitly Ruled Out

| Option | Reason |
|--------|--------|
| Per-customer vault token accounts | Unnecessary rent costs and CPI complexity for a pre-paid credit system |
| Arweave for hackathon receipts | Latency (2–4s), bundler dependency, token costs; deferred to production |
| CLI in the cto monorepo | Judges need self-contained repo; monorepo adds unnecessary build dependencies |
| Slot-based daily cap resets | ±15 minute drift over 24 hours; Clock sysvar is more accurate |
| SHA-256 hashed task_id | Destroys debuggability; CTO task IDs are short enough for direct encoding |
| Implemented fee split in settlement | Second CPI, fee ATA, rounding edge cases — all dead weight for hackathon |
| Events on all instructions | Setup instructions (initialize_operator, create_customer_account) are rarely called and trivially queryable |
| Accept any SPL token mint | Security hole; configurable-but-validated is the correct middle ground |
| Shelling out to Solana CLI | Fragile, loses type safety and error handling, adds binary dependency |
| Inline RPC calls in reconciliation loop | Blocks K8s operator reconciliation when devnet RPC is slow |
| ReceiptStore trait/interface | Premature abstraction for a single backend; a plain function is sufficient |

---

## 6. Implementation Constraints

### Security Requirements

- **Checked arithmetic everywhere:** Every balance mutation in the program must use `checked_add`, `checked_sub`, `checked_mul`. Return a custom error on overflow/underflow. This is a **hard requirement** given the single-vault architecture (total-pool blast radius on arithmetic bugs).
- **Mint validation on every token instruction:** `deposit`, `withdraw`, and `settle_task` must validate the token mint against `OperatorConfig.accepted_mint` and reject with `InvalidMint` error.
- **Signer verification:** Operator-only instructions (`settle_task`, `refund_task`, `pause`, `unpause`) must verify `signer == operator_config.authority`. Customer-only instructions must verify PDA ownership.
- **Pause check:** All customer-facing instructions must check `!operator_config.paused`.
- **Double-refund prevention:** `refund_task` must verify `task_receipt.status == TaskStatus::Settled` before processing.
- **Task ID length enforcement:** Reject task IDs exceeding 32 bytes with `TaskIdTooLong` at the instruction boundary.

### Performance Targets

- **Compute budget:** All instructions must fit within Solana's 200k CU default budget. Event emission adds ~200 CU per event — negligible.
- **Demo loop speed:** The full demo (deposit → settle → verify → withdraw) should complete in under 30 seconds on devnet. MinIO receipt upload must be sub-100ms.

### Operational Requirements

- **Async settlement in controller:** Settlement submission must be fire-and-forget with async retry, NOT inline in the K8s reconciliation loop. CRD status field tracks settlement state (`settlement_pending` → `settlement_confirmed` → `settlement_failed`).
- **Secrets pipeline:** Operator keypair follows existing path: 1Password → OpenBao → External Secrets Operator → K8s secret → pod env var. Local dev uses file-based keypair in `.gitignored` `keypairs/` directory.
- **Devnet deployment:** All testing and demo against Solana devnet. No mainnet interaction during hackathon.

### Service Dependencies

- **MinIO (S3):** `gitlab/gitlab-minio-svc` in-cluster — receipt blob storage
- **Solana devnet RPC:** `https://api.devnet.solana.com` — no SLA; settlement must tolerate unavailability
- **1Password → OpenBao → K8s:** Existing secrets pipeline for operator keypair
- **Anchor CLI 0.30+:** Build and deploy toolchain
- **Bun 1.1+:** CLI runtime

### Organizational Preferences

- Prefer in-cluster/self-hosted services when available (MinIO over external S3, existing secrets pipeline over new approaches)
- Hackathon scope: optimize for demo impact and developer velocity, not abstract future-proofing
- All code goes through the established QA pipeline (Stitch review, Cleo quality, Tess testing, Cipher security, Atlas merge gate)

---

## 7. Design Intake Summary

- **`hasFrontend`:** `false`
- **`frontendTargets`:** None
- **Supplied design artifacts:** None
- **Reference URLs:** None
- **Provider status:** Stitch design intake failed; Framer skipped (not requested)
- **Component-library artifacts:** None

**Implications:** This is a backend-only hackathon project (Solana program + Rust controller integration + TypeScript CLI). There is no web/mobile/desktop frontend. The CLI is the sole user interface. No design system, visual identity, or UI component work is required.

The PRD explicitly lists "Frontend / web dashboard (CLI only for hackathon)" as a **Non-Goal**.

---

## 8. Open Questions

The following items are non-blocking. Implementing agents should use their best judgment:

1. **Refund fund flow mechanics:** When `refund_task` is called, should funds transfer from treasury back to vault (requiring operator/treasury authority to sign the reverse transfer), or should the vault retain sufficient funds to cover potential refunds? The debate did not resolve this edge case. **Recommendation:** For hackathon simplicity, have the vault retain funds — `settle_task` transfers from vault to treasury, and `refund_task` credits the customer's `CustomerBalance` ledger without a reverse token transfer. This means the vault must always hold enough USDC to cover outstanding un-refunded settlements. Document this as a simplification.

2. **SpendingCapsUpdated and ProgramPaused/Unpaused events:** The scoped event decision (D7) explicitly covers deposit, withdraw, settle, refund. Whether `update_spending_caps`, `pause`, and `unpause` should also emit events was not debated. These are low-frequency operations. **Recommendation:** Emit events on `pause`/`unpause` (security-relevant state changes worth indexing); skip on `update_spending_caps` (queryable via account reads).

3. **OperatorConfig.accepted_mint and vault initialization ordering:** The vault token account must use the accepted mint. If `accepted_mint` is set during `initialize_operator`, the vault token account should be initialized in the same instruction. Implementing agents should ensure this is atomic.

4. **Controller retry strategy for failed Solana submissions:** The async/decoupled pattern is decided (D10), but the specific retry policy (exponential backoff, max retries, dead-letter handling) is left to implementing agents. **Recommendation:** Exponential backoff with jitter, 5 max retries, record `settlement_failed` on CRD status after exhaustion.

5. **Receipt JSON schema:** The PRD mentions "itemized receipt JSON (CodeRun minutes, compute, AI tokens if managed-key)" but does not specify the exact schema. Implementing agents should define a reasonable JSON schema and document it in the repo.

6. **Task ID encoding in CLI:** The CLI must zero-pad short task IDs to `[u8; 32]` before PDA derivation. The encoding scheme (UTF-8 bytes, zero-padded right) should be documented and consistent between the CLI, the controller, and any future clients.

