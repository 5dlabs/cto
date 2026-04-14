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
> |-----------|-------------|-----------------:|
> | CodeRun execution | Time from pod start to completion | Pod lifecycle events |
> | Infrastructure compute | Bare-metal time (standard / high-mem / GPU) | Pod resource requests + node labels |
> | AI tokens (managed-key) | Pass-through + margin when using 5D Labs keys | Provider API response metadata |
>
> **Existing tier pricing** (for context — on-chain settlement replaces the payment rail, not the pricing model):
>
> | Tier | Platform Fee | Included CodeRuns | Overage | AI Keys |
> |------|-------------|-------------------|---------|---------:|
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

The initial task decomposition identified **8 tasks** spanning 4 agents and 3 technology stacks. The project is a hackathon-scoped, self-contained system with one critical integration point into the existing CTO controller.

### Tasks Identified

| ID | Title | Agent | Stack | Dependencies |
|----|-------|-------|-------|-------------|
| 1 | Dev Infra Bootstrap — Repo Scaffold, Secrets, Receipt Storage | Bolt | Anchor CLI, SeaweedFS/S3, External Secrets Operator | None |
| 2 | Anchor Program — Account Structures, Errors, and `initialize_operator` | Rex | Rust, Anchor 0.30+, SPL Token | Task 1 |
| 3 | Anchor Program — Customer Balance Operations (create, deposit, withdraw, update caps) | Rex | Rust, Anchor 0.30+, SPL Token | Task 2 |
| 4 | Anchor Program — Settlement Engine, Refunds, and Circuit Breaker | Rex | Rust, Anchor 0.30+, SPL Token | Task 3 |
| 5 | Anchor Program Tests — Happy-Path Integration Suite | Tess | TypeScript, Anchor Test, Bankrun | Task 4 |
| 6 | Anchor Program Tests — Edge Cases, Error Paths, and Security Scenarios | Tess | TypeScript, Anchor Test, Bankrun | Task 4 |
| 7 | Solana Program Security Audit | Cipher | Anchor Verify, Soteria, Manual Audit | Task 4 |
| 8 | Controller CRD Extension and Solana Client Configuration | Rex | Rust, kube-rs, solana-sdk, solana-client | Task 4 |

**Note:** The initial decomposition did not include a standalone CLI task (Task for Blaze agent). The CLI/demo script (PRD Service 3) is referenced in decision points but not yet decomposed into a numbered task. This should be added during task generation as Task 9 or equivalent.

### Key Services and Components

1. **Solana Anchor Program** (`programs/cto-pay/`) — Core on-chain logic with 8 instructions, 3 PDA account types
2. **CTO Controller Integration** (`crates/controller/`) — Settlement hook added to existing production Kubernetes controller
3. **CLI / Demo Script** — TypeScript/Bun tool for hackathon demo video
4. **Infrastructure** — SeaweedFS receipt bucket, External Secrets for operator keypair, CI/CD via GitHub Actions

### Agent Assignments

- **Rex** (Rust): Anchor program (Tasks 2–4), Controller integration (Task 8)
- **Tess** (TypeScript): Test suites (Tasks 5–6)
- **Cipher** (Security): Audit (Task 7)
- **Bolt** (DevOps): Infrastructure bootstrap (Task 1)
- **Blaze** (TypeScript): CLI / Demo script (not yet decomposed)

### Cross-Cutting Concerns

- **11 decision points** identified across tasks, covering storage backends, RPC endpoints, data models, API design, and security
- **Production controller modification** (Task 8) carries regression risk — requires feature flag
- **Secrets management** spans the full 1Password → OpenBao → External Secrets → K8s pipeline
- **SPL token strategy** affects all tasks (program, tests, CLI, controller)

---

## 3. Resolved Decisions

The deliberation session completed in 6 minutes with 2 debate turns. Both personas (Optimist and Pessimist) reached explicit agreement on 6 of 11 decision points. The remaining 5 had substantive disagreement requiring synthesis. Since the session completed without formal votes (no `decision_points` array in the result), resolutions below are derived from the debate arguments — where both parties agreed, the decision is marked Accepted; where they disagreed, the strongest position is selected with rationale.

---

### [D1] Should off-chain receipt storage use SeaweedFS, Arweave, or both?

**Status:** Accepted

**Task Context:** Task 1 (Infra Bootstrap), Task 8 (Controller Integration)

**Context:** The Optimist proposed SeaweedFS as primary with best-effort Arweave uploads. The Pessimist argued that "both" adds two failure modes, two code paths, and an external network dependency with propagation delay, while the on-chain `receipt_hash` already provides verifiability regardless of storage location.

**Decision:** SeaweedFS only. No Arweave integration for the hackathon.

**Consensus:** Pessimist's position is stronger. The on-chain SHA-256 hash IS the verifiability anchor. SeaweedFS is deployed, in-cluster, zero-latency. Arweave adds external dependency, new SDK, wallet funding, and upload delays — all risk during a live demo.

**Consequences:**
- **Positive:** Single storage backend, single failure mode. Simpler Task 1 (bucket provisioning) and Task 8 (upload logic). No external network dependency during demo recording.
- **Negative:** Loses the "decentralized storage" narrative for judges.
- **Caveat:** Arweave should be mentioned in `README.md` as a documented future extension. The architecture supports it — the `receipt_hash` on-chain means storage location is pluggable.

---

### [D2] Should the CLI live in the cto-pay repo or the existing monorepo?

**Status:** Accepted

**Task Context:** Task 1 (Repo Scaffold)

**Context:** Both personas agreed immediately. Self-contained repo for hackathon submission is unambiguous.

**Decision:** CLI lives in the `cto-pay` repo alongside the Anchor program.

**Consensus:** Unanimous agreement.

**Consequences:**
- **Positive:** Judges clone one repo, run one command. Anchor IDL generated in-repo and imported directly by CLI without cross-repo publishing.
- **Negative:** Post-hackathon migration to monorepo may be needed if CLI evolves into production tooling.

---

### [D3] Which Solana RPC endpoint for the controller?

**Status:** Accepted

**Task Context:** Task 1 (Infra Bootstrap), Task 8 (Controller Integration)

**Context:** The Optimist proposed spinning a dedicated devnet agave-rpc instance in-cluster, citing public devnet rate limits and unreliability. The Pessimist argued this is not a simple "config change" — it requires stateful deployment, genesis config, snapshot sync, and persistent storage, and increases blast radius near the production mainnet node.

**Decision:** Use Helius free-tier devnet RPC. Do not spin up a new agave-rpc instance.

**Consensus:** Pessimist's position is stronger for hackathon scope. Helius free tier provides 100K requests/day on devnet with websocket support — more than sufficient. One environment variable, zero infrastructure provisioning. If Helius has issues during recording, re-record; debugging a self-hosted devnet node takes hours.

**Consequences:**
- **Positive:** Zero infrastructure provisioning. Task 1 provisions a config secret with an RPC URL; Task 8 consumes it. No risk of misconfiguring namespace boundaries near production mainnet node.
- **Negative:** External dependency on Helius availability. Rate-limited compared to self-hosted.
- **Caveat:** Store the RPC URL in a ConfigMap/Secret so it can be swapped to self-hosted, public, or another provider without code changes. If Helius free tier proves insufficient during development, upgrade to paid tier before spinning infrastructure.

---

### [D4] How should the protocol fee be handled in settle_task?

**Status:** Accepted

**Task Context:** Task 2 (Account Structures), Task 4 (Settlement Engine), Tasks 5–6 (Tests)

**Context:** Both personas agreed. Recording fee_amount on TaskReceipt is one `u64` field (~8 bytes) with zero runtime complexity and maximum future optionality for royalty splits.

**Decision:** Debit full amount to treasury (operator IS the service provider for hackathon). Record `fee_amount` as a computed field on `TaskReceipt` derived from `protocol_fee_bps` for future split compatibility.

**Consensus:** Unanimous agreement.

**Consequences:**
- **Positive:** Zero-cost future compatibility. When agent registry royalty splits arrive, the program already records what the protocol's share was.
- **Negative:** `fee_amount` is informational-only for the hackathon — no actual fee split logic executes.
- **Implementation note:** Add `fee_amount: u64` to `TaskReceipt` struct. Compute as `amount * protocol_fee_bps / 10_000` using checked arithmetic.

---

### [D5] Should settlement transaction submission be synchronous, async-spawned, or a separate reconciliation loop?

**Status:** Accepted

**Task Context:** Task 8 (Controller Integration)

**Context:** The Optimist proposed `tokio::spawn` fire-and-forget with CRD status update on confirmation. The Pessimist argued that spawned tasks have no backpressure, no retry budget, and silent failure modes; kube-rs reconcilers are designed for retry, and hackathon scale has single-digit concurrent CodeRuns.

**Decision:** Synchronous submission in the reconciler with a short timeout (5 seconds). On failure, return error — kube-rs requeues with backoff automatically.

**Consensus:** Pessimist's position is stronger for hackathon scope. Synchronous is dramatically simpler: submit transaction, await confirmation with timeout, update CRD, return. No orphaned tasks, no silent failures, no extra state management. Reconciler stalling is not a real risk at hackathon scale.

**Consequences:**
- **Positive:** Task 8 implementation is significantly simpler. kube-rs retry semantics handle failure cases natively. No orphaned background tasks.
- **Negative:** If 10+ CodeRuns hit terminal state simultaneously in the future, reconciler could stall for up to 50 seconds. This is a post-hackathon concern.
- **Caveat:** The 5-second timeout should be configurable via `SolanaConfig`. If the transaction doesn't confirm in 5s, fail and let kube-rs requeue. Log the transaction signature for manual verification.

---

### [D6] How should daily spending cap reset work on-chain?

**Status:** Accepted

**Task Context:** Task 2 (Account Structures), Task 4 (Settlement Engine), Tasks 5–6 (Tests)

**Context:** The Optimist argued for Clock sysvar `unix_timestamp` citing accuracy (Solana slot times fluctuate ±10%). The Pessimist argued slot numbers are monotonically increasing, deterministic, and require zero trust in validator clocks, and that ±10% variance on a spending cap is acceptable.

**Decision:** Use slot-based reset as the PRD specifies. Store `daily_reset_slot` and reset `daily_spent` when `current_slot >= daily_reset_slot + SLOTS_PER_DAY`.

**Consensus:** Pessimist's position aligns with the PRD's original design. Slot numbers are monotonically increasing and deterministic. Clock sysvar timestamps can jump forward or backward based on validator vote medians. For a spending *cap* (not a billing period), ±10% variance on "one day" is acceptable — customers set conservative caps anyway.

**Consequences:**
- **Positive:** Matches PRD field name `daily_reset_slot`. Simpler testing with Bankrun's `warp_to_slot()`. No trust dependency on validator clock accuracy.
- **Negative:** A "day" is approximately 23–25 hours depending on slot rate variance. Not suitable for precise billing periods.
- **Implementation note:** Define `SLOTS_PER_DAY: u64 = 216_000` as a constant in `constants.rs` with a doc comment explaining the approximation. Keep the constant adjustable for future tuning.

---

### [D7] How should the operator keypair be loaded by the controller?

**Status:** Accepted

**Task Context:** Task 1 (Infra Bootstrap), Task 8 (Controller Integration)

**Context:** Both personas agreed on file-mounted secret volume. Solana CLI conventions use file-based keypairs. File mounts don't appear in `/proc/PID/environ` or `kubectl describe pod`.

**Decision:** File-mounted secret volume with the keypair as a JSON file, referenced by path via environment variable `SOLANA_OPERATOR_KEYPAIR_PATH`.

**Consensus:** Unanimous agreement.

**Consequences:**
- **Positive:** `solana_sdk::signer::keypair::read_keypair_file()` is a one-liner. Marginally more secure than env var for a financial signing key. Matches Solana ecosystem conventions.
- **Negative:** None identified.
- **Implementation note:** External Secrets Operator creates K8s secret; pod spec mounts it as a volume at `/secrets/operator-keypair.json`. The env var `SOLANA_OPERATOR_KEYPAIR_PATH` points to this path.

---

### [D8] Should the CLI use devnet USDC, a custom SPL token, or wrapped SOL?

**Status:** Accepted

**Task Context:** Task 1 (Infra Bootstrap), Tasks 2–6 (Program + Tests), CLI (not yet decomposed)

**Context:** Both personas agreed on custom SPL mint. The Pessimist added a hard constraint: the `initialize_operator` instruction MUST accept `mint: Pubkey` as a parameter, not hardcode it.

**Decision:** Create a custom SPL token mint (6 decimals, controlled by operator wallet) labeled as "USDC" in the demo. Document that mainnet migration is a single mint address change in the README.

**Consensus:** Unanimous agreement, with the Pessimist's sharpened constraint adopted.

**Consequences:**
- **Positive:** Unlimited supply, full control. No dependency on Circle's devnet USDC faucet (known for availability issues). Demo video shows "100.00 USDC deposited" with correct 6-decimal display.
- **Negative:** Not actual USDC — judges may notice. Mitigated by transparent documentation.
- **Hard constraint (from Pessimist):** The program MUST accept `mint: Pubkey` as a parameter in `initialize_operator` and store it in `OperatorConfig`. Every subsequent instruction validates the token account's mint against this stored value. If the mint is hardcoded anywhere — in PDA seeds, constants, or test fixtures — mainnet migration requires a program redeploy. Tasks 2–6 must enforce this pattern.

---

### [D9] Single shared USDC vault or per-customer token accounts?

**Status:** Accepted

**Task Context:** Tasks 2–6 (All program tasks)

**Context:** Both personas agreed. This is a hard constraint from the PRD architecture diagram.

**Decision:** Single program-owned USDC vault (PDA-derived ATA) that holds all customer deposits. Balances tracked in CustomerBalance PDAs.

**Consensus:** Unanimous agreement. PRD hard constraint.

**Consequences:**
- **Positive:** Cheaper (one ATA vs N ATAs), simpler PDA derivation. Program's internal `CustomerBalance.balance` is source of truth for balance isolation.
- **Negative:** Single vault means program logic must perfectly enforce balance isolation — a bug could allow one customer's funds to be spent by another's account. Mitigated by security audit (Task 7).

---

### [D10] How should task_id be represented on-chain in TaskReceipt PDA?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 5, 6, 8

**Context:** The Optimist proposed `[u8; 32]` SHA-256 hash as PDA seed plus the original String stored in account data. The Pessimist argued that storing the String on-chain adds variable-length serialization complexity and inflated account size for zero value — the off-chain receipt JSON already contains the human-readable task_id.

**Decision:** Use `[u8; 32]` SHA-256 hash of the task_id string as the sole PDA seed and on-chain representation. Do NOT store the original task_id string in account data.

**Consensus:** Pessimist's position is stronger. The receipt JSON contains the human-readable task_id. The PDA is deterministically derivable by anyone who knows the task_id (compute SHA-256, derive PDA). Storing the string adds variable-length serialization complexity in Anchor and inflates rent cost for a demo convenience that Solana explorers can't render usefully anyway.

**Consequences:**
- **Positive:** Fixed-size account. Simpler serialization. Deterministic PDA derivation for any consumer who knows the task_id.
- **Negative:** On-chain TaskReceipt does not contain the human-readable task_id — must be looked up from off-chain receipt or CLI local state.
- **Implementation note:** The controller (Task 8) computes `SHA256(task_id)` for PDA derivation. The CLI prints the task_id from its local state. The `settle_task` instruction accepts `task_id_hash: [u8; 32]` directly, not a String. Update the PRD's `TaskReceipt` struct to use `task_id_hash: [u8; 32]` instead of `task_id: String`.

---

### [D11] Should receipt building and upload happen in the controller or a separate service?

**Status:** Accepted

**Task Context:** Task 8 (Controller Integration)

**Context:** Both personas agreed immediately. A separate microservice for a <1KB JSON blob at hackathon scale is unjustifiable complexity.

**Decision:** Build receipts and upload directly in the controller's settlement hook. No separate service.

**Consensus:** Unanimous agreement.

**Consequences:**
- **Positive:** Single-service simplicity. If SeaweedFS upload fails, still submit `settle_task` with the locally-computed hash. Receipt can be re-uploaded on next reconciliation cycle.
- **Negative:** If upload latency grows (e.g., future Arweave integration), the settlement path gets slower. Extractable to async worker post-hackathon.

---

## 4. Escalated Decisions

No decision points were formally escalated. All 11 decision points have resolved positions from the deliberation.

However, one **structural concern** raised by the Pessimist warrants special attention:

### [ADVISORY] Controller Feature Flag for Settlement Hook

**Status:** Strong recommendation (not a formal decision point, but raised as a critical concern)

**Task Context:** Task 8 (Controller Integration)

**Context:** The Pessimist raised that Task 8 modifies the production controller (`crates/controller/`) which manages ALL CodeRun lifecycles. Adding `solana-sdk` and `solana-client` pulls in ~200 transitive dependencies and increases compile time. If the settlement code panics, every CodeRun reconciliation is affected.

**Recommendation:** Task 8 MUST implement a `settlement_enabled: bool` (or `CTO_PAY_ENABLED`) configuration flag, defaulting to `false`. All Solana settlement code must be gated behind this flag. When disabled, the reconciler operates exactly as before — no Solana dependencies are loaded, no settlement is attempted. This is a safety requirement for production controller modifications.

**Implementing agents must treat this as a hard constraint for Task 8.**

---

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version |
|-------|------------|---------|
| On-chain program | Rust, Anchor framework | Anchor 0.30+, Solana 1.18+ |
| Program tests | TypeScript, Bankrun, Anchor test | Bun 1.1+ |
| CLI / Demo | TypeScript, Bun, @coral-xyz/anchor, @solana/web3.js | Bun 1.1+ |
| Controller hook | Rust, kube-rs, solana-sdk, solana-client | kube-rs 0.98, solana-sdk 1.18 |
| Off-chain receipts | SeaweedFS (S3-compatible, in-cluster) | Existing deployment |
| Secrets | 1Password → OpenBao → External Secrets → K8s | Existing pipeline |
| RPC | Helius free-tier devnet | External service |
| CI/CD | GitHub Actions | Existing |

### Service Architecture

```
┌──────────────────────────────────────────────────┐
│  cto-pay repo (self-contained hackathon project) │
│                                                  │
│  programs/cto-pay/     ← Anchor program (Rust)   │
│  tests/                ← Bankrun tests (TS)      │
│  cli/                  ← Demo CLI (TS/Bun)       │
│  scripts/              ← Devnet setup scripts    │
│  config/               ← Token config, devnet    │
└──────────────────────────────────────────────────┘
         │ IDL (generated)
         ▼
┌──────────────────────────────────────────────────┐
│  CTO Monorepo (production, existing)             │
│                                                  │
│  crates/controller/    ← Settlement hook added   │
│    └─ Feature-flagged behind CTO_PAY_ENABLED     │
│    └─ Syncs via Solana RPC (Helius devnet)       │
│    └─ Uploads receipts to SeaweedFS              │
│    └─ Reads operator keypair from mounted secret │
└──────────────────────────────────────────────────┘
```

### Communication Patterns

1. **CLI → Solana Devnet**: Direct RPC calls via `@solana/web3.js` to Helius devnet endpoint
2. **Controller → Solana Devnet**: `solana-client` RPC calls via Helius devnet endpoint, synchronous within reconciler with 5s timeout
3. **Controller → SeaweedFS**: HTTP PUT for receipt JSON uploads via S3-compatible API
4. **Controller → K8s API**: CRD status patches via kube-rs to record settlement signatures

### Key Patterns

- **Single vault model**: All customer USDC held in one PDA-derived ATA; balance isolation enforced by program logic
- **Operator-trusted metering**: Controller computes billable amounts off-chain; program trusts operator signature
- **Feature-flagged integration**: Settlement hook in production controller gated by `CTO_PAY_ENABLED` (default `false`)
- **Configurable mint**: `initialize_operator` accepts `mint: Pubkey` parameter — never hardcoded
- **Slot-based daily reset**: Approximate daily spending cap using `SLOTS_PER_DAY = 216,000`
- **Synchronous settlement**: Transaction submission is synchronous in reconciler; kube-rs requeue handles retries

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Arweave storage | External dependency, propagation delay, additional SDK — not needed when on-chain hash provides verifiability |
| Self-hosted devnet agave-rpc | Too much operational overhead for hackathon; Helius free tier sufficient |
| Async-spawned settlement tasks | Fire-and-forget `tokio::spawn` from reconciler is a footgun — no backpressure, silent failures |
| Clock sysvar for daily reset | Validator timestamp drift; slots are monotonic and deterministic |
| String task_id on-chain | Variable-length serialization complexity for no incremental value over off-chain receipt |
| Per-customer escrow accounts | PRD hard constraint specifies single vault; per-customer adds rent costs with no benefit |
| Separate receipt-builder service | Over-engineered for <1KB JSON at hackathon scale |
| HSM/remote signer | Significantly more complex than file-mounted secret for hackathon scope |

---

## 6. Implementation Constraints

These are hard constraints that every implementing agent must respect.

### Security Requirements

1. **Operator keypair**: Loaded from file-mounted K8s secret at `SOLANA_OPERATOR_KEYPAIR_PATH`. Never stored in environment variables, never logged, never included in crash dumps.
2. **All arithmetic**: Must use checked operations (`checked_add`, `checked_sub`, `checked_mul`). Zero raw arithmetic on `u64` values.
3. **Account validation**: Every instruction must validate all account constraints via Anchor's `has_one`, `constraint`, `seeds`, `bump` attributes. No instruction may allow arbitrary account substitution.
4. **Signer checks**: Operator-only instructions verify `operator.key() == operator_config.authority`. Customer-only instructions verify `customer_balance.customer == customer.key()`.
5. **Mint validation**: All token account instructions must validate the token's mint matches `OperatorConfig`'s stored mint. Mint must never be hardcoded.
6. **No `init_if_needed`**: Known dangerous pattern — all account init uses standard `init` constraint.
7. **Feature flag**: Controller settlement code gated behind `CTO_PAY_ENABLED` (default `false`). When disabled, zero Solana code executes in reconciliation path.

### Performance Targets

1. **Settlement timeout**: 5-second timeout for Solana transaction confirmation in the controller reconciler.
2. **Bankrun tests**: Full integration suite must complete in under 30 seconds.
3. **Program size**: Must fit within Solana's BPF program size limits (~1.4MB). Anchor programs typically well under this.

### Operational Requirements

1. **Receipt storage**: SeaweedFS bucket `cto-pay-receipts` in the `cto` namespace. Endpoint stored in `cto-pay-infra-endpoints` ConfigMap.
2. **RPC endpoint**: Helius free-tier devnet. URL stored in ConfigMap/Secret, swappable without code changes.
3. **Secrets pipeline**: 1Password → OpenBao → External Secrets Operator → K8s Secret (file mount). No shortcuts.
4. **CRD backward compatibility**: All CodeRun CRD schema changes must be additive (new optional fields only). Existing CodeRuns must not break.

### Service Dependencies

| Dependency | Type | Critical? |
|------------|------|-----------|
| Helius devnet RPC | External SaaS | Yes (for settlement and CLI) |
| SeaweedFS (in-cluster) | Internal | Yes (for receipt storage) |
| External Secrets Operator | Internal | Yes (for keypair provisioning) |
| OpenBao | Internal | Yes (for secrets) |
| 1Password | External SaaS | Yes (for secrets source-of-truth) |

### Organizational Preferences

- Prefer in-cluster services when already deployed (SeaweedFS over external S3)
- Follow existing secrets management pipeline — no ad hoc secret creation
- All code changes go through automated quality pipeline (Stitch, Cleo, Tess, Cipher, Atlas)
- Critical/high security findings block merge

---

## 7. Design Intake Summary

**`hasFrontend`:** `false`

**`frontendTargets`:** None

**Supplied design artifacts:** None

**Reference URLs:** None

**Provider status:**
- Stitch (design generation): Failed — `design_intake_failed`
- Framer: Skipped — `not_requested`

**Component-library artifacts:** None

### Implications for Implementation

This is a **backend-only + CLI project** for the hackathon. The PRD explicitly lists "Frontend / web dashboard (CLI only for hackathon)" as a non-goal. No frontend implementation tasks exist. The CLI is a command-line TypeScript tool using `@coral-xyz/anchor` — it has no UI components, design system, or visual layer.

Design intake is not applicable to this project's current scope.

---

## 8. Open Questions

These are non-blocking items where implementing agents should use their best judgment.

1. **Refund token flow mechanics**: Task 4 identified complexity around refund_task — when `settle_task` transfers USDC from vault to treasury, `refund_task` needs to transfer back. The simplest approach for hackathon: credit `customer_balance.balance` and mark TaskReceipt as Refunded. Whether tokens physically move from treasury back to vault depends on whether the operator's treasury wallet co-signs. Implementing agent should choose the simplest correct approach and document the accounting model.

2. **Pause behavior for refunds**: Should `refund_task` work when the program is paused? The PRD says withdrawals work during pause (customers can always exit). Refunds are operator-initiated and return funds to customers — arguably they should also work during pause. Tests (Task 6) should verify and document the chosen behavior.

3. **CLI task decomposition**: The CLI/demo script (PRD Service 3, Blaze agent) is referenced throughout but not yet decomposed into a numbered task. Task generation should create this task with dependencies on Task 4 (program complete) and Task 1 (repo scaffold).

4. **`fee_amount` precision**: When computing `fee_amount = amount * protocol_fee_bps / 10_000`, integer division may lose precision. Implementing agent should decide ordering (multiply first, then divide) and document rounding behavior. Use checked arithmetic.

5. **Customer profile `solana_pubkey` field**: The PRD mentions adding a `solana_pubkey` field to the customer profile model in the CTO monorepo. This is a data model change in the existing system that isn't fully scoped. Task 8 should document the integration point but may defer the actual customer profile change.

6. **Demo video production**: Not scoped as a task. Someone needs to record the demo video showing the full settlement loop. This is a hackathon deliverable but not a code task.

7. **Anchor version pinning**: PRD says "Anchor 0.30+" — implementing agents should pin to a specific minor version (e.g., 0.30.1) and document it in Anchor.toml to prevent build reproducibility issues.

8. **TaskReceipt PDA seed update**: With the resolved decision to use `[u8; 32]` SHA-256 hash instead of String for task_id, the PDA seeds change from `[b"task_receipt", task_id.as_bytes()]` to `[b"task_receipt", &task_id_hash]`. All task descriptions referencing the original String-based seed pattern should be updated during implementation.

