# Project: CTO Pay — On-Chain Usage-Based Payments for the CTO Platform

- **Organization:** 5D Labs
- **Status:** Draft
- **Target:** Solana Agent Hackathon (April 2026)
- **Repo:** https://github.com/5dlabs/cto-pay

## Vision

CTO Pay is the on-chain payment and settlement layer for the CTO platform. It replaces traditional off-chain billing with a Solana program that lets customers pre-pay USDC into an escrow account, have their usage metered per task, and receive verifiable on-chain receipts for every charge. Every payment is transparent, auditable, and composable.

The long-term goal is a protocol-grade billing rail that supports usage-based payments today, and extends to agent registry royalty splits, attestation-gated settlement, and a full agent marketplace tomorrow — without rewriting the core program.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CTO Pay Platform                                │
├─────────────────────────────────────────────────────────────────────────┤
│  Customer                                                               │
│  ┌──────────────┐                                                       │
│  │ Solana Wallet │  (Phantom, Backpack, Solflare)                       │
│  │  USDC deposit │                                                      │
│  └──────┬───────┘                                                       │
│         │                                                               │
├─────────┴───────────────────────────────────────────────────────────────┤
│  Solana Program (Anchor)                                                │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │  OperatorConfig   │  │ CustomerBalance  │  │   TaskReceipt    │      │
│  │  (PDA, singleton) │  │ (PDA per cust.)  │  │  (PDA per task)  │      │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘      │
│           │                     │                      │                │
│  ┌────────┴─────────────────────┴──────────────────────┴────────┐      │
│  │                     Program Vault (USDC)                      │      │
│  │         Holds all customer deposits; program is authority     │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  CTO Controller (Kubernetes)                                            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │  Usage Metering   │  │ Receipt Builder  │  │ Settlement Hook  │      │
│  │  (pod duration,   │  │ (JSON → hash →   │  │ (submit settle/  │      │
│  │   infra tier)     │  │  off-chain store) │  │  refund to chain)│      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Off-Chain Storage                                                      │
│  ┌──────────┐  ┌──────────┐                                             │
│  │  Arweave  │  │   S3     │  (itemized receipt JSON blobs)             │
│  └──────────┘  └──────────┘                                             │
├─────────────────────────────────────────────────────────────────────────┤
│  Future Extensions (not in scope)                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ Agent Registry│  │ Attestations │  │  Marketplace  │                 │
│  │ (royalty      │  │ (Tess/Cipher/│  │  (skill NFTs, │                 │
│  │  splits)      │  │  Stitch sigs)│  │   curation)   │                 │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Services (Workstreams)

### 1. Solana Program — Anchor (Rust)

**Agent**: Rex
**Priority**: Critical
**Language**: Rust (Anchor framework)
**Target**: Solana devnet (mainnet-beta post-hackathon)

The on-chain program that holds customer deposits, enforces spending caps, settles task payments, and writes verifiable receipts.

**Accounts (PDAs)**:

```
OperatorConfig (PDA, singleton)
├── authority: Pubkey          // 5D Labs operator wallet
├── treasury: Pubkey           // 5D Labs revenue wallet
├── protocol_fee_bps: u16     // protocol fee (basis points)
└── paused: bool               // circuit breaker

CustomerBalance (PDA, seeded by customer pubkey)
├── customer: Pubkey
├── balance: u64               // USDC lamports
├── total_deposited: u64
├── total_spent: u64
├── task_count: u64
├── max_per_task: u64          // spending cap per task
├── max_per_day: u64           // daily spending cap
├── daily_spent: u64
├── daily_reset_slot: u64
└── created_at: i64

TaskReceipt (PDA, seeded by task_id)
├── task_id: String
├── customer: Pubkey
├── amount: u64                // USDC charged
├── receipt_hash: [u8; 32]     // SHA-256 of off-chain receipt JSON
├── operator: Pubkey
├── settled_at: i64
└── status: TaskStatus         // Settled | Refunded | Disputed
```

**Instructions**:

```
initialize_operator(authority, treasury, protocol_fee_bps)
  → Creates OperatorConfig PDA. Called once at program deployment.

create_customer_account(max_per_task, max_per_day)
  → Creates CustomerBalance PDA for the signing customer.

deposit(amount)
  → Transfers USDC from customer's token account to the program vault.
     Increments customer balance.

withdraw(amount)
  → Transfers USDC from program vault back to customer.
     Decrements customer balance. Customer-signed.

settle_task(task_id, amount, receipt_hash)
  → Called by operator wallet after CodeRun completion.
     Validates: operator authorized, sufficient balance, within caps.
     Debits customer, credits treasury, writes TaskReceipt.

refund_task(task_id)
  → Called by operator wallet.
     Marks TaskReceipt as Refunded. Credits amount back to customer.

update_spending_caps(max_per_task, max_per_day)
  → Called by customer. Updates their spending limits.

pause / unpause
  → Called by operator. Circuit breaker for emergencies.
```

**Testing**:
- Anchor integration tests (TypeScript, `anchor test`)
- Bankrun for fast local program tests
- Devnet deployment and manual verification

---

### 2. Settlement Hook — CTO Controller Integration (Rust)

**Agent**: Rex
**Priority**: High
**Language**: Rust
**Location**: `crates/controller/` in the main CTO repo (integration points documented here)

The CTO Kubernetes controller submits settlement transactions after each CodeRun reaches a terminal state.

**Integration Points**:

| Point | File (in CTO repo) | Change |
|-------|---------------------|--------|
| Solana config | `crates/controller/src/tasks/config.rs` | Add RPC endpoint + operator keypair path |
| Settlement hook | `crates/controller/src/tasks/code/controller.rs` | After terminal state: compute bill → build receipt → submit `settle_task` |
| Wallet mapping | Customer profile model | Add `solana_pubkey` field |
| Tx recording | CodeRun CRD status | Add `on_chain_settlement_sig` field |

**Settlement Flow**:

1. CodeRun transitions to terminal state (merged / failed / cancelled).
2. Controller computes billable amount from pod duration + infra tier.
3. Builds itemized receipt JSON (CodeRun minutes, compute, AI tokens if managed-key).
4. Uploads receipt to off-chain storage (Arweave or S3).
5. Hashes the receipt (SHA-256).
6. Submits `settle_task` instruction to Solana program (or `refund_task` on failure).
7. Records the transaction signature on the CodeRun status.

**Secrets**: Operator keypair managed via the existing pipeline (1Password → OpenBao → External Secrets Operator → K8s secret → pod env var).

---

### 3. CLI / Demo Script (TypeScript)

**Agent**: Blaze
**Priority**: High
**Language**: TypeScript (Bun runtime)
**Framework**: `@coral-xyz/anchor`, `@solana/web3.js`

A CLI tool and demo script that simulates the full settlement loop end-to-end on devnet. Used for hackathon demo video and local development.

**Commands**:

```
cto-pay init-operator          — Deploy OperatorConfig with treasury wallet
cto-pay create-account         — Create CustomerBalance for a wallet
cto-pay deposit <amount>       — Deposit USDC into balance
cto-pay withdraw <amount>      — Withdraw USDC from balance
cto-pay settle <task_id> <amt> — Submit mock task settlement
cto-pay refund <task_id>       — Refund a settled task
cto-pay balance                — Check customer balance
cto-pay receipts               — List task receipts for a customer
cto-pay demo                   — Run the full demo loop (deposit → settle → verify → withdraw)
```

**Demo Loop** (what gets filmed):

1. Customer deposits 100 USDC into balance PDA.
2. CTO task runs (mocked or real CodeRun).
3. Settlement fires — customer balance decreases, treasury increases.
4. `TaskReceipt` appears on chain with receipt hash.
5. Customer verifies receipt in Solana explorer (Solscan/Explorer link printed).
6. Customer withdraws remaining balance.

---

## Technical Context

| Component | Technology | Agent |
|-----------|------------|-------|
| Solana Program | Rust, Anchor 0.30+ | Rex |
| Program Tests | TypeScript, Anchor test, Bankrun | Tess |
| CLI / Demo | TypeScript, Bun, @coral-xyz/anchor | Blaze |
| Controller Hook | Rust, solana-sdk | Rex |
| Off-Chain Receipts | Arweave / S3 | Bolt |
| Secrets | 1Password → OpenBao → K8s | Bolt |
| CI/CD | GitHub Actions | Bolt |

**Dependencies**:
- Solana CLI 1.18+
- Anchor CLI 0.30+
- Bun 1.1+
- USDC SPL token (devnet mint for testing)

---

## Data Flow Examples

### DF-1: Customer Deposit → Task Settlement → Receipt Verification

```
Customer Wallet (Phantom)
    │
    ▼
deposit(100 USDC)
    │
    ▼
CustomerBalance PDA (+100 USDC)
    │
    │  ... CTO runs a CodeRun (off-chain, K8s) ...
    │
    ▼
Controller: settle_task("TASK-42", 12.50 USDC, receipt_hash)
    │
    ├──► CustomerBalance PDA (−12.50 USDC)
    ├──► Operator Treasury (+12.50 USDC)
    └──► TaskReceipt PDA created
              │
              ├── task_id: "TASK-42"
              ├── amount: 12_500_000  (USDC lamports)
              ├── receipt_hash: 0xabc...
              └── status: Settled

Customer verifies on Solscan: TaskReceipt → receipt_hash → fetch JSON from Arweave/S3
```

### DF-2: Task Failure → Refund

```
Controller detects CodeRun failure
    │
    ▼
Controller: refund_task("TASK-42")
    │
    ├──► TaskReceipt.status → Refunded
    └──► CustomerBalance PDA (+12.50 USDC restored)
```

### DF-3: Spending Cap Enforcement

```
Controller: settle_task("TASK-99", 500 USDC, hash)
    │
    ▼
Program checks:
  - max_per_task: 200 USDC  → 500 > 200 → REJECTED
  - Transaction fails with SpendingCapExceeded error
  - Customer balance unchanged
```

---

## Billing Dimensions

These map directly from the existing CTO monetization model (`docs/business/saas-monetization.md` in the CTO repo):

| Dimension | Description | Metering Source |
|-----------|-------------|-----------------|
| CodeRun execution | Time from pod start to completion | Pod lifecycle events |
| Infrastructure compute | Bare-metal time (standard / high-mem / GPU) | Pod resource requests + node labels |
| AI tokens (managed-key) | Pass-through + margin when using 5D Labs keys | Provider API response metadata |

**Existing tier pricing** (for context — on-chain settlement replaces the payment rail, not the pricing model):

| Tier | Platform Fee | Included CodeRuns | Overage | AI Keys |
|------|-------------|-------------------|---------|---------|
| Free | $0 | 50/month | $3.00/run | BYOK only |
| Team | $199/month | 200/month | $1.50/run | BYOK or managed (+15%) |
| Growth | $499/month | 1,000/month | $0.75/run | BYOK or managed (+10%) |
| Enterprise | Custom | Custom | $0.50/run | Flexible |

For the hackathon, billing is **per task (metered)** — the operator submits the computed cost, and the program settles it. Subscription management stays off-chain.

---

## Open Design Questions

These are genuinely unsolved. The hackathon submission takes a position on each, but they remain open for iteration.

### 1. Success-conditional billing

Should settlement be conditional on task success? Options:
- **Always charge** (metered) — simplest, matches existing model.
- **Charge on success only** — best customer UX but exposes runtime to abuse.
- **Hybrid** — base attempt fee + success bonus.

**Hackathon position**: Always charge (metered). Success-conditional billing is a future extension via attestation gates.

### 2. Trust model for usage reporting

All usage measurements happen off-chain in CTO's K8s cluster. The Solana program trusts the operator wallet.

**Hackathon position**: Trust the operator. Production trust-reduction options (signed pod receipts, review-agent co-signatures, open-source metering, dispute mechanism) are documented but not built.

### 3. Customer payment UX

- **Pre-paid balance** (AWS credits) — best for real customers.
- **Session key delegation** — modern Solana pattern, good for demo.
- **Per-task escrow** — maximum visibility, worst UX.

**Hackathon position**: Pre-paid balance (deposit/withdraw). Most practical and demo-able.

---

## Spending Controls

Enforced on-chain by the program:

| Control | Description | Enforced By |
|---------|-------------|-------------|
| Max per task | Program rejects `settle_task` above this cap | `CustomerBalance.max_per_task` |
| Max per day | Rolling daily spending limit | `CustomerBalance.max_per_day` + `daily_spent` + `daily_reset_slot` |
| Pre-flight estimate | Runtime shows estimated cost before execution | Off-chain UX (not a program instruction) |
| Customer withdrawal | Customer can withdraw unused balance at any time | `withdraw` instruction |
| Circuit breaker | Operator can pause all settlements in emergencies | `OperatorConfig.paused` |

---

## Failure and Refund Handling

| Scenario | Action |
|----------|--------|
| Task fails before any agent work | No settlement submitted; balance unchanged |
| Task fails after partial work | Open question for production; hackathon charges for compute consumed |
| Task succeeds | Full charge per metered usage |
| Customer disputes a charge | Manual resolution initially; on-chain dispute mechanism later |

---

## Quality Assurance & Review Workflow

All code changes go through the CTO automated quality pipeline:

### 1. Automated Code Review (Stitch)
- **Agent**: Stitch — Automated Code Reviewer
- **Trigger**: On every pull request
- **Scope**: Style, correctness, architecture alignment
- **Tools**: GitHub PR integration via GitHub App

### 2. Code Quality Enforcement (Cleo)
- **Agent**: Cleo — Quality Guardian
- **Trigger**: CI/CD pipeline
- **Focus**: Maintainability, refactor opportunities, code smells
- **Tools**: Clippy (pedantic), Rustfmt, Biome (TypeScript)

### 3. Comprehensive Testing (Tess)
- **Agent**: Tess — Testing Genius
- **Trigger**: CI/CD pipeline after review approval
- **Coverage**: Anchor program tests, CLI integration tests, settlement flow tests
- **Tools**: `anchor test`, Bankrun, Bun test
- **Enforcement**: All program instructions must have positive and negative test cases

### 4. Security Scanning (Cipher)
- **Agent**: Cipher — Security Sentinel
- **Trigger**: CI/CD pipeline
- **Focus**: Solana program vulnerabilities, account validation, signer checks, overflow/underflow
- **Tools**: Anchor verify, Soteria (if available), manual audit checklist
- **Blocker**: Critical/high severity issues block merge

### 5. Merge Gate (Atlas)
- **Agent**: Atlas — Integration Master
- **Policy**: Required approvals + passing CI + passing QA
- **Tools**: GitHub merge automation

### 6. Deployment & Operations (Bolt)
- **Agent**: Bolt — DevOps Engineer
- **Workflow**: `anchor build` → `anchor deploy` to devnet
- **Monitoring**: Transaction logs, program account state verification

---

## Future Extensions (Post-Hackathon, Not In Scope)

These are documented for context and to show the program's extensibility. They are explicitly **not part of the hackathon build**.

### Agent Registry with Royalty Splits
Full agent packages (skills, tools, Soul, user context) registered on-chain. Each agent has an author wallet and a price. When a task invokes multiple agents/skills, the settlement program splits payment across all authors atomically.

### Authorship NFTs
Each registered agent represented by a transferable NFT. Holder receives the royalty stream. Transfer the NFT = sell a revenue-generating AI agent as an asset.

### Attestation-Based Settlement
Tess, Cipher, and Stitch hold Solana keypairs. Their pass/fail signals become on-chain attestations. The program gates payment release on a quorum of attestation signatures.

### cNFT Invocation Receipts
Compressed NFTs minted per task settlement as the on-chain audit trail. Billions feasible at ~$0.00001 per mint.

### Agent Reputation
Cumulative earnings, attestation pass rates, and slash history per agent, queryable on chain.

### Agent Compute Marketplace
Sell spare CTO bare-metal cluster capacity to third-party agent runs, metered and settled via the same program.

---

## Non-Goals

- Full agent marketplace / skill registry (future extension)
- NFT-based authorship tokens or receipt tokens (future extension)
- Subscription tier management on chain (subscriptions stay off-chain)
- Managed-key LLM token pass-through billing (BYOK customers pay their LLM provider directly)
- Production-grade security audit of the Solana program (post-hackathon)
- Frontend / web dashboard (CLI only for hackathon)
- Multi-chain support (Solana only)

---

## Hackathon Deliverables

1. **Anchor program** deployed to Solana devnet implementing: `initialize_operator`, `create_customer_account`, `deposit`, `withdraw`, `settle_task`, `refund_task`, `update_spending_caps`, `pause/unpause`.
2. **CLI / demo script** that runs the full settlement loop: create customer → deposit USDC → submit mock task → settle → verify receipt on chain → withdraw.
3. **Demo video** showing:
   - Customer deposits USDC into balance PDA.
   - CTO task runs (mocked or real CodeRun).
   - Settlement fires — customer balance decreases, operator treasury increases, `TaskReceipt` appears on chain with receipt hash.
   - Customer verifies receipt in Solana explorer.
   - Customer withdraws remaining balance.
4. **This PRD** as supporting documentation.

---

## Success Criteria

1. A judge can watch the demo video and understand: a customer paid for AI agent work, settled on Solana, with a verifiable on-chain receipt.
2. The program compiles, deploys to devnet, and passes integration tests (deposit, settle, refund, withdraw, cap enforcement).
3. Spending caps are enforced on-chain — oversized settlements are rejected.
4. Refund flow works — failed tasks return funds to the customer balance.
5. The design is extensible — adding agent registry splits or attestation gates later does not require rewriting the core program.
6. End-to-end flow works: deposit USDC → task settles → receipt on chain → balance verifiable → withdrawal succeeds.

---

## References

- `docs/business/saas-monetization.md` (CTO repo) — existing pricing model
- `docs/solana-hackathon-ideas.md` (CTO repo) — full ideation brainstorm
- `docs/solana-hackathon-prd.md` (CTO repo) — original detailed PRD
- `crates/controller/src/crds/coderun.rs` (CTO repo) — CodeRun CRD definition
- `crates/controller/src/tasks/code/controller.rs` (CTO repo) — task reconciliation loop
- `crates/controller/src/tasks/code/resources.rs` (CTO repo) — pod resource construction
- `crates/controller/src/cli/types.rs` (CTO repo) — provider/key resolution
- `docs/secrets-management.md` (CTO repo) — secrets pipeline (1Password → OpenBao → K8s)
- `AGENTS.md` (CTO repo) — agent roster (Tess, Cipher, Stitch attestation roles)
