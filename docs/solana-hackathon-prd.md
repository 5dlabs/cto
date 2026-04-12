# PRD: On-Chain Usage-Based Billing for CTO

**Author:** 5D Labs
**Status:** Draft
**Target:** Solana Agent Hackathon (April 2026)
**Branch:** `claude/solana-agent-hackathon-oY0EF`

---

## 1. Problem statement

CTO's existing monetization model (defined in `docs/business/saas-monetization.md`)
relies on traditional SaaS billing: platform subscription fees, CodeRun overage
charges, and optional managed-key token pass-through, all settled off-chain via
conventional payment processors.

This creates three gaps:

1. **No verifiable billing.** Customers cannot independently verify that their
   usage charges are accurate. They trust 5D Labs' billing system.
2. **No composable payment rail.** If CTO ever opens to third-party skill
   authors or an agent marketplace, there is no programmatic way to split
   payments across multiple recipients atomically.
3. **No on-chain presence.** CTO's value proposition is autonomous AI agents
   running production workloads, but none of that economic activity is visible
   or settleable on chain — a missed opportunity in the Solana ecosystem.

## 2. Proposed solution

Build a **Solana program** that settles CTO usage-based billing on chain.
Customers pre-pay into an escrow account, the CTO runtime reports usage per
task, and the program debits the customer's balance and writes an itemized
on-chain receipt.

This is not a hackathon toy — it is intended to become a real billing rail
that runs alongside (and eventually replaces) traditional payment processing
for usage-based charges.

## 3. Goals

- Demonstrate end-to-end on-chain settlement of a CTO task at the hackathon.
- Produce a Solana program that can be extended post-hackathon into a
  production billing layer.
- Lay groundwork for future features (agent marketplace, skill registry,
  attestation-based success verification) without building them now.

## 4. Non-goals (for the hackathon)

- Full marketplace / skill registry with third-party authors.
- NFT-based authorship tokens or receipt tokens.
- Subscription tier management on chain (subscriptions stay off-chain).
- Managed-key LLM token pass-through billing (BYOK customers pay their
  LLM provider directly; that cost is out of scope for on-chain settlement).
- Production-grade security audit of the Solana program.

## 5. Background: existing billing model

From `docs/business/saas-monetization.md`, the chosen model is **hybrid
pricing** (platform subscription + usage-based components):

| Tier | Platform fee | Included CodeRuns | Overage | AI keys |
|---|---|---|---|---|
| Free | $0 | 50/month | $3.00/run | BYOK only |
| Team | $199/month | 200/month | $1.50/run | BYOK or managed (+15%) |
| Growth | $499/month | 1,000/month | $0.75/run | BYOK or managed (+10%) |
| Enterprise | Custom | Custom | $0.50/run | Flexible |

**Billing dimensions today (all off-chain):**

1. **CodeRun execution** — time from pod start to completion.
2. **AI tokens** — pass-through + margin when using 5D Labs managed keys.
3. **Infrastructure compute** — bare-metal time (standard / high-mem / GPU).

**Key architectural facts:**

- Provider model supports Anthropic, OpenAI, Google, Cursor, Factory,
  Moonshot with a 5-level resolution precedence (CRD field → legacy
  settings → operator config → model inference → Fireworks fallback).
  See `crates/controller/src/tasks/code/resources.rs`.
- Secrets flow: 1Password → OpenBao → External Secrets Operator → K8s
  Secrets → pod env vars. See `docs/secrets-management.md`.
- API keys per provider via `Provider::secret_key()` in
  `crates/controller/src/cli/types.rs`.

## 6. What goes on chain

### 6.1. Customer balance (escrow PDA)

- Customer tops up a **balance PDA** with USDC (SPL token).
- The program is the sole authority over the PDA; no external party holds
  the key.
- Balance is debitable only by the CTO runtime's authorized signer (the
  **operator wallet**, controlled by 5D Labs).
- Customer can withdraw unused balance at any time.

### 6.2. Task settlement

When a `CodeRun` completes, the CTO controller submits a **settle_task**
instruction to the program with:

- `task_id` — Linear issue ID or internal identifier.
- `customer` — customer's pubkey (linked via CTO profile).
- `usage` — the billable amount in USDC.
- `receipt_hash` — hash of a JSON receipt blob stored off-chain (Arweave
  or S3) containing the itemized breakdown.

The program:

1. Verifies the operator wallet signature.
2. Debits `usage` from the customer's balance PDA.
3. Credits the operator wallet (5D Labs).
4. Writes a `TaskReceipt` record on chain (task ID, customer, amount,
   receipt hash, timestamp).

### 6.3. On-chain receipt

Each settled task produces a `TaskReceipt` account (or PDA keyed by
task ID) that the customer can look up in any Solana explorer to verify:

- Which task was billed.
- How much was charged.
- When settlement occurred.
- The receipt hash, which they can resolve off-chain to see the full
  itemized breakdown (CodeRun minutes, infra compute, etc.).

## 7. Open design questions

These are the genuinely unsolved problems. The hackathon submission should
take a position on each, but they remain open for iteration.

### 7.1. What is the billing unit?

Candidates, from coarsest to most granular:

| Unit | Description | Tradeoff |
|---|---|---|
| Per task (flat) | Fixed price per CodeRun regardless of duration | Simple but doesn't reflect actual cost |
| Per task (metered) | Price based on CodeRun duration + infra tier | Matches existing billing dimensions |
| Per successful task | Only charge when attestations confirm success | Best customer UX, but exposes runtime to abuse |
| Hybrid | Base attempt fee + success bonus | Balances abuse protection and customer trust |

The existing monetization doc prices by **CodeRun count + duration**, so
"per task (metered)" is the natural starting point. Whether to condition
payment on task success is the key open question.

### 7.2. How is success measured?

CTO already has review agents that produce pass/fail signals:

- **Tess** — tests pass.
- **Cipher** — security audit clean.
- **Stitch** — code review approved, PR merged.

These signals could be lifted on chain as **attestations** that the billing
program reads to decide whether to release or refund payment. This is not
required for the hackathon MVP but is the natural extension.

Design sub-questions if we go this route:

- Which agents count as success-signers? Tess alone? Tess + Cipher? All
  three?
- What's the quorum — 2-of-3, all-of-3, weighted?
- Does a failed attestation trigger full refund, partial refund, or just
  a reduced charge (compute cost only)?
- How is "success" defined per task type? (A bug fix vs. a greenfield
  feature vs. a refactor have different success criteria.)

### 7.3. Trust model for usage reporting

All usage measurements (CodeRun duration, infra time, skill invocation
counts) happen **off-chain** in CTO's Kubernetes cluster. The Solana
program accepts them as truth from the operator wallet. This means the
customer trusts 5D Labs to report honestly.

For the hackathon, this is acceptable. For production, trust-reduction
options include:

- **Signed receipts per agent pod.** Each pod holds a per-task keypair;
  usage is signed by the pod, not a central aggregator.
- **Review-agent co-signatures.** Tess/Cipher attest to usage totals as
  part of their task attestation, making them complicit if numbers are
  wrong.
- **Open-source runtime.** Customers can inspect metering logic (relevant
  now that the platform is going open source).
- **Dispute mechanism.** Customer can challenge a task's usage; if proven
  fraudulent, operator stake gets slashed.

### 7.4. Customer payment UX

Three patterns, ranked by real-world usability:

1. **Pre-paid balance (AWS credits)** — customer tops up once, runtime
   debits per task silently. Best for real customers. Requires a
   `deposit` and `withdraw` instruction on the program.
2. **Session key delegation** — customer signs once, delegates spending
   authority to CTO for a capped amount over a time window. Modern
   Solana pattern. Good for demo.
3. **Per-task escrow** — customer signs per task. Maximum on-chain
   visibility but worst UX at scale.

Hackathon demo can use #2 or #3. Production should converge on #1.

### 7.5. Spending controls

Customers need guardrails:

- **Max per task** — program rejects settlement above this cap.
- **Max per day / per month** — rolling spending limit enforced by the
  program.
- **Pre-flight estimate** — runtime shows estimated cost before task
  execution, customer approves (off-chain UX, not a program instruction).

### 7.6. Failure and refund handling

- Task fails before any agent work → full refund (no settlement submitted).
- Task fails after partial work (e.g., implementation done, tests fail) →
  open question: charge for compute consumed, or refund fully?
- Task succeeds per attestations → full charge per usage.
- Customer disputes a charge → manual resolution initially, on-chain
  dispute mechanism later.

## 8. Anchor program sketch

### Accounts

```
OperatorConfig (PDA, singleton)
├── authority: Pubkey          // 5D Labs operator wallet
├── treasury: Pubkey           // 5D Labs revenue wallet
├── protocol_fee_bps: u16     // optional protocol fee (basis points)
└── paused: bool               // circuit breaker

CustomerBalance (PDA, seeded by customer pubkey)
├── customer: Pubkey
├── balance: u64               // USDC lamports
├── total_deposited: u64
├── total_spent: u64
├── task_count: u64
├── max_per_task: u64          // spending cap
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

### Instructions

```
initialize_operator(authority, treasury, protocol_fee_bps)
  → Creates OperatorConfig PDA.

create_customer_account(max_per_task, max_per_day)
  → Creates CustomerBalance PDA for the signing customer.

deposit(amount)
  → Transfers USDC from customer's token account to the program vault.
     Increments customer balance.

withdraw(amount)
  → Transfers USDC from program vault back to customer.
     Decrements customer balance.

settle_task(task_id, amount, receipt_hash)
  → Called by operator wallet.
     Validates: operator is authorized, customer has sufficient balance,
     amount is within per-task and daily caps.
     Debits customer balance, credits operator treasury, writes TaskReceipt.

refund_task(task_id)
  → Called by operator wallet.
     Marks TaskReceipt as Refunded. Credits amount back to customer balance.

update_spending_caps(max_per_task, max_per_day)
  → Called by customer. Updates their caps.

pause / unpause
  → Called by operator. Circuit breaker for emergencies.
```

## 9. Controller integration points

The CTO Kubernetes controller needs minimal changes to submit settlement
transactions:

### 9.1. New config

Add Solana RPC endpoint and operator keypair path to the controller config
(`crates/controller/src/tasks/config.rs`). The operator keypair should be
managed via the existing OpenBao secrets pipeline.

### 9.2. Settlement hook

In `crates/controller/src/tasks/code/controller.rs`, after a `CodeRun`
transitions to a terminal state (merged / failed / cancelled), the
controller:

1. Computes the billable amount from pod duration + infra tier.
2. Builds the itemized receipt JSON and uploads to off-chain storage.
3. Hashes the receipt.
4. Submits a `settle_task` (or `refund_task` on failure) instruction to
   the Solana program.
5. Records the transaction signature on the `CodeRun` status for
   traceability.

### 9.3. Customer wallet mapping

The customer's Solana pubkey needs to be associated with their CTO
account. For the hackathon, this can be a simple field in the customer
profile. For production, wallet linking via Solana wallet-adapter
(`@solana/wallet-adapter`) with signature verification.

## 10. Future extensions (post-hackathon, not in scope)

These are documented for context but are explicitly **not part of the
hackathon build**:

### 10.1. Skill registry with royalty splits

Third-party authors publish reusable skills to an on-chain registry.
Each skill has an author wallet and a price per invocation. When a task
invokes multiple skills, the settlement program splits payment across
all skill authors atomically. Authors earn royalties per use.

### 10.2. Authorship NFTs

Each registered skill is represented by a transferable NFT. Whoever
holds the NFT receives the royalty stream. Transfer the NFT = sell the
revenue-generating skill as an asset on secondary markets.

### 10.3. Attestation-based settlement

Tess, Cipher, and Stitch hold Solana keypairs. Their pass/fail signals
become on-chain attestations. The billing program gates payment release
on a quorum of attestation signatures, removing trust in the operator
for success determination.

### 10.4. Agent reputation

Cumulative earnings, attestation pass rates, and slash history per agent
or skill, queryable on chain. Serves as a market-validated quality signal
for discovery and ranking.

### 10.5. Agent compute marketplace

Sell spare CTO bare-metal cluster capacity to third-party agent runs,
metered and settled via the same program. Extends the billing primitive
from internal to external.

## 11. Hackathon deliverables

1. **Anchor program** deployed to Solana devnet implementing:
   `initialize_operator`, `create_customer_account`, `deposit`, `withdraw`,
   `settle_task`, `refund_task`, `update_spending_caps`, `pause/unpause`.
2. **CLI or script** that simulates the controller settlement flow: create
   customer → deposit USDC → submit a mock task → settle → verify receipt
   on chain.
3. **Demo video** showing:
   - Customer deposits USDC into balance PDA.
   - CTO task runs (can be mocked or real CodeRun if time permits).
   - Settlement fires — customer balance decreases, operator treasury
     increases, `TaskReceipt` appears on chain with receipt hash.
   - Customer verifies receipt in Solana explorer.
   - Customer withdraws remaining balance.
4. **This PRD** and the companion ideas doc (`docs/solana-hackathon-ideas.md`)
   as supporting documentation.

## 12. Success criteria

- A judge can watch the demo video and understand: a customer paid for
  AI agent work, settled on Solana, with a verifiable on-chain receipt.
- The program compiles, deploys to devnet, and passes basic integration
  tests (deposit, settle, refund, withdraw, cap enforcement).
- The design is extensible — adding skill registry splits or attestation
  gates later does not require rewriting the core program.

## 13. References

- `docs/business/saas-monetization.md` — existing pricing model
- `docs/solana-hackathon-ideas.md` — full ideation brainstorm
- `crates/controller/src/crds/coderun.rs` — CodeRun CRD definition
- `crates/controller/src/tasks/code/controller.rs` — task reconciliation loop
- `crates/controller/src/tasks/code/resources.rs` — pod resource construction
- `crates/controller/src/cli/types.rs` — provider/key resolution
- `docs/secrets-management.md` — secrets pipeline (1Password → OpenBao → K8s)
- `AGENTS.md` — agent roster (Tess, Cipher, Stitch attestation roles)
