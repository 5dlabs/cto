# Enhanced PRD

## 1. Original Requirements

> # PRD: On-Chain Usage-Based Billing for CTO
>
> **Author:** 5D Labs
> **Status:** Draft
> **Target:** Solana Agent Hackathon (April 2026)
> **Branch:** `claude/solana-agent-hackathon-oY0EF`
>
> ---
>
> ## 1. Problem statement
>
> CTO's existing monetization model (defined in `docs/business/saas-monetization.md`)
> relies on traditional SaaS billing: platform subscription fees, CodeRun overage
> charges, and optional managed-key token pass-through, all settled off-chain via
> conventional payment processors.
>
> This creates three gaps:
>
> 1. **No verifiable billing.** Customers cannot independently verify that their
>    usage charges are accurate. They trust 5D Labs' billing system.
> 2. **No composable payment rail.** If CTO ever opens to third-party skill
>    authors or an agent marketplace, there is no programmatic way to split
>    payments across multiple recipients atomically.
> 3. **No on-chain presence.** CTO's value proposition is autonomous AI agents
>    running production workloads, but none of that economic activity is visible
>    or settleable on chain — a missed opportunity in the Solana ecosystem.
>
> ## 2. Proposed solution
>
> Build a **Solana program** that settles CTO usage-based billing on chain,
> with an integrated quality-gated marketplace for third-party skills.
>
> Customers pre-pay into an escrow account. The CTO runtime reports usage per
> task. The program debits the customer's balance and writes an itemized
> on-chain receipt. When a task uses a **public agent package** published by a
> third-party author, the program conditionally splits payment between 5D Labs
> and the agent author — but only if the task meets a quality threshold
> (attestations from CTO's review agents). If quality isn't met, the customer
> doesn't pay.
>
> The marketplace is not a separate system — it's a conditional split inside
> the same settlement instruction. Since we're already measuring quality for
> billing purposes, routing a portion of successful payments to agent authors
> is one extra branch, not a new project.
>
> This is not a hackathon toy — it is intended to become a real billing rail
> that runs alongside (and eventually replaces) traditional payment processing
> for usage-based charges.
>
> ## 3. Goals
>
> - Demonstrate end-to-end on-chain settlement of a CTO task at the hackathon.
> - Demonstrate quality-gated marketplace splits: a public agent author gets
>   paid only when the task meets quality thresholds.
> - Produce a Solana program that can be extended post-hackathon into a
>   production billing layer with a growing ecosystem of third-party skills.
>
> ## 4. Non-goals (for the hackathon)
>
> - NFT-based authorship tokens or receipt tokens.
> - Subscription tier management on chain (subscriptions stay off-chain).
> - Managed-key LLM token pass-through billing (BYOK customers pay their
>   LLM provider directly; that cost is out of scope for on-chain settlement).
> - Full skill discovery UI / leaderboard / search (the on-chain primitive
>   exists; the frontend comes later).
> - Production-grade security audit of the Solana program.
>
> ## 5. Background: existing billing model
>
> From `docs/business/saas-monetization.md`, the chosen model is **hybrid
> pricing** (platform subscription + usage-based components):
>
> | Tier | Platform fee | Included CodeRuns | Overage | AI keys |
> |---|---|---|---|---|
> | Free | $0 | 50/month | $3.00/run | BYOK only |
> | Team | $199/month | 200/month | $1.50/run | BYOK or managed (+15%) |
> | Growth | $499/month | 1,000/month | $0.75/run | BYOK or managed (+10%) |
> | Enterprise | Custom | Custom | $0.50/run | Flexible |
>
> **Billing dimensions today (all off-chain):**
>
> 1. **CodeRun execution** — time from pod start to completion.
> 2. **AI tokens** — pass-through + margin when using 5D Labs managed keys.
> 3. **Infrastructure compute** — bare-metal time (standard / high-mem / GPU).
>
> **Key architectural facts:**
>
> - Provider model supports Anthropic, OpenAI, Google, Cursor, Factory,
>   Moonshot with a 5-level resolution precedence (CRD field → legacy
>   settings → operator config → model inference → Fireworks fallback).
>   See `crates/controller/src/tasks/code/resources.rs`.
> - Secrets flow: 1Password → OpenBao → External Secrets Operator → K8s
>   Secrets → pod env vars. See `docs/secrets-management.md`.
> - API keys per provider via `Provider::secret_key()` in
>   `crates/controller/src/cli/types.rs`.
>
> ## 6. What goes on chain
>
> ### 6.1. Customer balance (escrow PDA)
>
> - Customer tops up a **balance PDA** with USDC (SPL token).
> - The program is the sole authority over the PDA; no external party holds
>   the key.
> - Balance is debitable only by the CTO runtime's authorized signer (the
>   **operator wallet**, controlled by 5D Labs).
> - Customer can withdraw unused balance at any time.
>
> ### 6.2. Agent package registration (public marketplace)
>
> Third-party authors can register **agent packages** on chain. An agent
> package is the full bundle needed to run an agent on CTO's runtime:
>
> - **Skills** — reusable capabilities from `skills/`
> - **Tools** — MCP server configs, GitHub repo references
> - **SOUL.md** — agent personality, identity, and behavioral guidelines
> - **USER.md** — user context and preferences
> - **AGENT.md** — agent specification and role definition
> - **Any other config** relevant to OpenClaw or Hermes (CTO's agent
>   runtimes)
>
> An `AgentPackage` PDA stores the author's wallet, a split percentage
> (basis points), and a reference to the package source (repo URL, Arweave
> URI, compressed archive, etc.). Registration is permissionless — anyone
> can publish.
>
> When a customer's task uses a public agent package, the `settle_task`
> instruction includes the package's PDA. The program uses it to route a
> portion of payment to the author — but only if quality is met (see 6.3).
>
> Default agents (5D Labs' own roster: Rex, Blaze, Tess, etc.) are
> registered the same way, with 5D Labs as the author wallet. **One code
> path for everything.** The only difference between a "default" and a
> "public" agent is whose wallet the author field points to.
>
> ### 6.3. Task settlement (with quality-gated marketplace split)
>
> When a `CodeRun` completes, the CTO controller submits a **settle_task**
> instruction to the program with:
>
> - `task_id` — Linear issue ID or internal identifier.
> - `customer` — customer's pubkey (linked via CTO profile).
> - `amount` — the billable amount in USDC.
> - `receipt_hash` — hash of a JSON receipt blob stored off-chain (Arweave
>   or S3) containing the itemized breakdown.
> - `skill_author` — (optional) pubkey of the public agent author, if a
>   third-party skill was used.
> - `author_split_bps` — (optional) basis points for the author's share
>   (e.g. 3000 = 30%).
> - `quality_met` — boolean indicating whether the task passed the quality
>   threshold (Tess/Cipher/Stitch attestations).
>
> The program evaluates three cases:
>
> **Case 1: No public agent package (default agents only)**
> 1. Verifies operator wallet signature.
> 2. Debits `amount` from customer balance.
> 3. Credits 100% to operator treasury (5D Labs).
> 4. Writes `TaskReceipt`.
>
> **Case 2: Public skill used, quality threshold MET**
> 1. Verifies operator wallet signature.
> 2. Debits `amount` from customer balance.
> 3. Splits payment: `author_split_bps` to agent author wallet, remainder
>    to operator treasury.
> 4. Writes `TaskReceipt` with `author_earned > 0` and `quality_met = true`.
>
> **Case 3: Public skill used, quality threshold NOT MET**
> 1. Customer is **not charged** (no debit). The task effectively costs
>    nothing.
> 2. Skill author earns nothing.
> 3. Writes `TaskReceipt` with `amount = 0`, `author_earned = 0`,
>    `quality_met = false`.
>
> This creates aligned incentives:
> - **Skill authors** are incentivized to publish high-quality skills —
>   they only earn when their skill contributes to a successful task.
> - **Customers** are protected — they don't pay for failed work when
>   using public agent packages, which makes them willing to experiment.
> - **5D Labs** gets a marketplace for free inside the billing rail,
>   taking a cut on every successful public-skill settlement.
>
> > **Note:** Case 3 (customer pays nothing on failure) is the boldest
> > version and strongest for the hackathon pitch. For production, this
> > may be refined to charge a base compute fee while zeroing the author's
> > portion — see section 7.6.
>
> ### 6.4. On-chain receipt
>
> Each settled task produces a `TaskReceipt` account (or PDA keyed by
> task ID) that the customer can look up in any Solana explorer to verify:
>
> - Which task was billed.
> - How much was charged.
> - Whether a public agent package was used and how much the author earned.
> - Whether the quality threshold was met.
> - When settlement occurred.
> - The receipt hash, which they can resolve off-chain to see the full
>   itemized breakdown (CodeRun minutes, infra compute, etc.).
>
> ## 7. Open design questions
>
> These are the genuinely unsolved problems. The hackathon submission should
> take a position on each, but they remain open for iteration.
>
> ### 7.1. What is the billing unit?
>
> Candidates, from coarsest to most granular:
>
> | Unit | Description | Tradeoff |
> |---|---|---|
> | Per task (flat) | Fixed price per CodeRun regardless of duration | Simple but doesn't reflect actual cost |
> | Per task (metered) | Price based on CodeRun duration + infra tier | Matches existing billing dimensions |
> | Per successful task | Only charge when attestations confirm success | Best customer UX, but exposes runtime to abuse |
> | Hybrid | Base attempt fee + success bonus | Balances abuse protection and customer trust |
>
> The existing monetization doc prices by **CodeRun count + duration**, so
> "per task (metered)" is the natural starting point. Whether to condition
> payment on task success is the key open question.
>
> ### 7.2. How is success measured?
>
> CTO already has review agents that produce pass/fail signals:
>
> - **Tess** — tests pass.
> - **Cipher** — security audit clean.
> - **Stitch** — code review approved, PR merged.
>
> These signals could be lifted on chain as **attestations** that the billing
> program reads to decide whether to release or refund payment. This is not
> required for the hackathon MVP but is the natural extension.
>
> Design sub-questions if we go this route:
>
> - Which agents count as success-signers? Tess alone? Tess + Cipher? All
>   three?
> - What's the quorum — 2-of-3, all-of-3, weighted?
> - Does a failed attestation trigger full refund, partial refund, or just
>   a reduced charge (compute cost only)?
> - How is "success" defined per task type? (A bug fix vs. a greenfield
>   feature vs. a refactor have different success criteria.)
>
> ### 7.3. Trust model for usage reporting
>
> All usage measurements (CodeRun duration, infra time, skill invocation
> counts) happen **off-chain** in CTO's Kubernetes cluster. The Solana
> program accepts them as truth from the operator wallet. This means the
> customer trusts 5D Labs to report honestly.
>
> For the hackathon, this is acceptable. For production, trust-reduction
> options include:
>
> - **Signed receipts per agent pod.** Each pod holds a per-task keypair;
>   usage is signed by the pod, not a central aggregator.
> - **Review-agent co-signatures.** Tess/Cipher attest to usage totals as
>   part of their task attestation, making them complicit if numbers are
>   wrong.
> - **Open-source runtime.** Customers can inspect metering logic (relevant
>   now that the platform is going open source).
> - **Dispute mechanism.** Customer can challenge a task's usage; if proven
>   fraudulent, operator stake gets slashed.
>
> ### 7.4. Customer payment UX
>
> Three patterns, ranked by real-world usability:
>
> 1. **Pre-paid balance (AWS credits)** — customer tops up once, runtime
>    debits per task silently. Best for real customers. Requires a
>    `deposit` and `withdraw` instruction on the program.
> 2. **Session key delegation** — customer signs once, delegates spending
>    authority to CTO for a capped amount over a time window. Modern
>    Solana pattern. Good for demo.
> 3. **Per-task escrow** — customer signs per task. Maximum on-chain
>    visibility but worst UX at scale.
>
> Hackathon demo can use #2 or #3. Production should converge on #1.
>
> ### 7.5. Spending controls
>
> Customers need guardrails:
>
> - **Max per task** — program rejects settlement above this cap.
> - **Max per day / per month** — rolling spending limit enforced by the
>   program.
> - **Pre-flight estimate** — runtime shows estimated cost before task
>   execution, customer approves (off-chain UX, not a program instruction).
>
> ### 7.6. Failure and refund handling
>
> - Task fails before any agent work → full refund (no settlement submitted).
> - Task fails after partial work (e.g., implementation done, tests fail) →
>   open question: charge for compute consumed, or refund fully?
> - Task succeeds per attestations → full charge per usage.
> - Customer disputes a charge → manual resolution initially, on-chain
>   dispute mechanism later.
>
> ## 8. Anchor program sketch
>
> ### Accounts
>
> ```
> OperatorConfig (PDA, singleton)
> ├── authority: Pubkey          // 5D Labs operator wallet
> ├── treasury: Pubkey           // 5D Labs revenue wallet
> ├── protocol_fee_bps: u16     // optional protocol fee (basis points)
> └── paused: bool               // circuit breaker
>
> AgentPackage (PDA, seeded by package_id)
> ├── package_id: String         // unique identifier
> ├── author: Pubkey             // author's wallet (receives royalties)
> ├── split_bps: u16             // author's share in basis points (e.g. 3000 = 30%)
> ├── source_uri: String         // repo URL, Arweave URI, or compressed archive ref
> ├── total_earned: u64          // cumulative USDC earned (quality signal)
> ├── task_count: u64            // total tasks run
> ├── success_count: u64         // tasks that met quality threshold
> ├── registered_at: i64
> └── active: bool
>
> CustomerBalance (PDA, seeded by customer pubkey)
> ├── customer: Pubkey
> ├── balance: u64               // USDC lamports
> ├── total_deposited: u64
> ├── total_spent: u64
> ├── task_count: u64
> ├── max_per_task: u64          // spending cap
> ├── max_per_day: u64           // daily spending cap
> ├── daily_spent: u64
> ├── daily_reset_slot: u64
> └── created_at: i64
>
> TaskReceipt (PDA, seeded by task_id)
> ├── task_id: String
> ├── customer: Pubkey
> ├── amount: u64                // total USDC charged to customer
> ├── author_earned: u64         // portion paid to agent author (0 if no public agent or quality failed)
> ├── quality_met: bool          // whether quality threshold was met
> ├── agent_package: Option<Pubkey>  // AgentPackage PDA, if a public agent was used
> ├── receipt_hash: [u8; 32]     // SHA-256 of off-chain receipt JSON
> ├── operator: Pubkey
> ├── settled_at: i64
> └── status: TaskStatus         // Settled | Refunded | Disputed
> ```
>
> ### Instructions
>
> ```
> initialize_operator(authority, treasury, protocol_fee_bps)
>   → Creates OperatorConfig PDA.
>
> register_agent_package(package_id, split_bps, source_uri)
>   → Called by the author. Creates AgentPackage PDA with the signer as
>      author. Permissionless — anyone can publish.
>
> update_agent_package(source_uri, split_bps, active)
>   → Called by the author. Updates their package metadata.
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
>      Decrements customer balance.
>
> settle_task(task_id, amount, receipt_hash, agent_package?, quality_met)
>   → Called by operator wallet.
>      Validates: operator is authorized, customer has sufficient balance,
>      amount is within per-task and daily caps.
>      If agent_package is provided and quality_met is true:
>        Splits payment — author_split_bps to author wallet, rest to treasury.
>        Increments AgentPackage.total_earned, task_count, success_count.
>      If agent_package is provided and quality_met is false:
>        Customer is not charged (amount = 0). Author earns nothing.
>        Increments AgentPackage.task_count only.
>      If no agent_package:
>        Debits full amount, credits operator treasury.
>      Writes TaskReceipt in all cases.
>
> refund_task(task_id)
>   → Called by operator wallet.
>      Marks TaskReceipt as Refunded. Credits amount back to customer balance.
>
> update_spending_caps(max_per_task, max_per_day)
>   → Called by customer. Updates their caps.
>
> pause / unpause
>   → Called by operator. Circuit breaker for emergencies.
> ```
>
> ## 9. Controller integration points
>
> The CTO Kubernetes controller needs minimal changes to submit settlement
> transactions:
>
> ### 9.1. New config
>
> Add Solana RPC endpoint and operator keypair path to the controller config
> (`crates/controller/src/tasks/config.rs`). The operator keypair should be
> managed via the existing OpenBao secrets pipeline.
>
> ### 9.2. Settlement hook
>
> In `crates/controller/src/tasks/code/controller.rs`, after a `CodeRun`
> transitions to a terminal state (merged / failed / cancelled), the
> controller:
>
> 1. Computes the billable amount from pod duration + infra tier.
> 2. Builds the itemized receipt JSON and uploads to off-chain storage.
> 3. Hashes the receipt.
> 4. Submits a `settle_task` (or `refund_task` on failure) instruction to
>    the Solana program.
> 5. Records the transaction signature on the `CodeRun` status for
>    traceability.
>
> ### 9.3. Customer wallet mapping
>
> The customer's Solana pubkey needs to be associated with their CTO
> account. For the hackathon, this can be a simple field in the customer
> profile. For production, wallet linking via Solana wallet-adapter
> (`@solana/wallet-adapter`) with signature verification.
>
> ## 10. Future extensions (post-hackathon, not in scope)
>
> These are documented for context but are explicitly **not part of the
> hackathon build**:
>
> ### 10.1. Authorship NFTs
>
> Each registered agent package is represented by a transferable NFT.
> Whoever holds the NFT receives the royalty stream. Transfer the NFT =
> sell the revenue-generating agent as an asset on secondary markets.
>
> ### 10.2. On-chain attestations (trustless quality)
>
> Tess, Cipher, and Stitch hold Solana keypairs. Their pass/fail signals
> become on-chain attestations. The billing program gates payment release
> on a quorum of attestation signatures, removing trust in the operator
> for quality determination. (In the hackathon build, the operator reports
> `quality_met` — this extension makes it trustless.)
>
> ### 10.3. Agent reputation and leaderboards
>
> The `AgentPackage` account already tracks `total_earned`, `task_count`,
> and `success_count` on chain. Future work: build indexed leaderboards
> with time-decay, per-category rankings, and discovery UIs so customers
> can find top-performing public agents.
>
> ### 10.4. Agent compute marketplace
>
> Sell spare CTO bare-metal cluster capacity to third-party agent runs,
> metered and settled via the same program. Extends the billing primitive
> from internal to external.
>
> ### 10.5. Multi-agent splits per task
>
> The hackathon build supports one agent package per task. Future: a task
> invokes multiple agent packages, and the program splits payment across
> all of them atomically (N-way split in one Solana transaction).
>
> ## 11. Hackathon deliverables
>
> 1. **Anchor program** deployed to Solana devnet implementing:
>    `initialize_operator`, `register_agent_package`, `update_agent_package`,
>    `create_customer_account`, `deposit`, `withdraw`, `settle_task`,
>    `refund_task`, `update_spending_caps`, `pause/unpause`.
> 2. **CLI or script** that simulates the full flow: register an agent
>    package → create customer → deposit USDC → run a task with the public
>    agent → settle with quality met (author gets paid) → run another task
>    → settle with quality NOT met (customer not charged, author earns
>    nothing) → verify receipts on chain.
> 3. **Demo video** showing:
>    - Author registers an agent package on chain.
>    - Customer deposits USDC into balance PDA.
>    - CTO task runs using the public agent (can be mocked or real CodeRun).
>    - Quality threshold met → settlement fires, payment splits between
>      5D Labs treasury and agent author wallet. `TaskReceipt` shows
>      `quality_met = true` and `author_earned > 0`.
>    - Second task with quality NOT met → customer balance unchanged, author
>      earns nothing. `TaskReceipt` shows `quality_met = false`.
>    - Customer verifies both receipts in Solana explorer.
>    - Customer withdraws remaining balance.
> 4. **This PRD** and the companion ideas doc (`docs/solana-hackathon-ideas.md`)
>    as supporting documentation.
>
> ## 12. Success criteria
>
> - A judge can watch the demo video and understand: a customer paid for
>   AI agent work, a third-party agent author got a quality-gated royalty
>   split, and the full flow settled on Solana with verifiable receipts.
> - The program compiles, deploys to devnet, and passes basic integration
>   tests (deposit, settle with split, settle with quality failure, refund,
>   withdraw, cap enforcement, agent package registration).
> - The marketplace is not a separate system — it's a conditional branch
>   in `settle_task`, demonstrating that billing + marketplace are one
>   program.
>
> ## 13. References
>
> - `docs/business/saas-monetization.md` — existing pricing model
> - `docs/solana-hackathon-ideas.md` — full ideation brainstorm
> - `crates/controller/src/crds/coderun.rs` — CodeRun CRD definition
> - `crates/controller/src/tasks/code/controller.rs` — task reconciliation loop
> - `crates/controller/src/tasks/code/resources.rs` — pod resource construction
> - `crates/controller/src/cli/types.rs` — provider/key resolution
> - `docs/secrets-management.md` — secrets pipeline (1Password → OpenBao → K8s)
> - `AGENTS.md` — agent roster (Tess, Cipher, Stitch attestation roles)

## 2. Project Scope

The initial task decomposition identified **7 tasks** spanning the on-chain program, controller integration, demo tooling, web dashboard, and CI/CD infrastructure.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|-------------|
| 1 | Anchor Program Core Accounts & Foundation Instructions | Rex | Rust/Anchor/Solana | High | None |
| 2 | Settlement Engine & Quality-Gated Marketplace Splits | Rex | Rust/Anchor/Solana | High | Task 1 |
| 3 | Comprehensive Anchor Program Integration Test Suite | Tess | Anchor/TypeScript/Mocha | High | Tasks 1, 2 |
| 4 | Integrate Solana Settlement Hook into CTO Controller | Rex | Rust/Axum/solana-sdk | Medium | Tasks 1, 2 |
| 5 | End-to-End CLI Demo Script | Stitch | TypeScript/Solana-Web3.js/Anchor | High | Tasks 1, 2 |
| 6 | Hackathon Demo Web Dashboard | Blaze | React/Next.js/TypeScript | Medium | Tasks 1, 2 |
| 7 | Devnet Deployment Pipeline & Demo Environment | Atlas | GitHub Actions/Anchor CLI/Solana CLI | Medium | Tasks 1, 2, 3 |

### Key Services & Components

- **On-chain program** (`cto-billing`): Anchor-based Solana program with 4 PDA account types (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt) and 10 instructions
- **CTO controller integration** (`crates/controller/`): Rust/Axum Kubernetes operator extended with Solana settlement hooks
- **CLI demo script** (`demo/`): TypeScript end-to-end flow runner for hackathon video
- **Web dashboard** (`demo/web/`): Next.js 14 React app with wallet adapter for visual demo
- **CI/CD pipeline** (`.github/workflows/`): Build, test, deploy, and provision pipeline

### Cross-Cutting Concerns

- **Secrets management**: Operator keypair via OpenBao → External Secrets Operator pipeline
- **Token handling**: Mint-agnostic program with configurable USDC mint
- **PDA derivation**: Consistent SHA-256 hash-based seeding across all client code (Rust, TypeScript)
- **Network strategy**: Localnet for CI/testing, devnet for demo artifacts
- **Feature gating**: Solana billing behind a `solana-billing` feature flag in the controller

## 3. Resolved Decisions

### [D1] Should the on-chain program use Anchor framework or raw solana-program?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5 — all on-chain program development and client interaction

**Context:** This was a hard constraint from the PRD (Section 8: "Anchor program sketch"). Both debaters agreed immediately with no contention.

**Decision:** Use **Anchor framework v0.30+** with derive macros, IDL generation, and TypeScript client codegen.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Account validation macros eliminate the most common class of Solana program vulnerabilities (missing owner/signer checks)
- (+) IDL generation directly enables TypeScript client generation for Tasks 3, 5, and 6
- (+) Ecosystem standard — judges and reviewers expect Anchor
- (−) None raised — this was a PRD constraint

---

### [D2] How should TaskReceipt PDAs be seeded — string task_id or hash-based?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5 — PDA derivation across Rust program and TypeScript clients

**Context:** Both debaters agreed that SHA-256 hashing normalizes variable-length identifiers to fixed-size seeds. The Pessimist added the nuance that the hash should also be stored as an account field for client-side derivation verification.

**Decision:** Seed by **SHA-256 hash of the string task_id** (first 32 bytes). Store the human-readable task_id as a String field inside the account for explorer readability.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Fixed-size seeds eliminate variable-length PDA derivation bugs
- (+) Works uniformly for short Linear IDs ("CTO-1234") and longer internal identifiers
- (+) Clients hash the task_id to derive the PDA address — trivial in both Rust and TypeScript
- (−) Clients must perform a hash operation for PDA derivation (negligible cost)
- Caveat: Same pattern applies to AgentPackage PDAs with `package_id`

---

### [D3] Which Solana cluster for development and CI testing?

**Status:** Accepted

**Task Context:** Tasks 3, 5, 6, 7 — testing, demo, dashboard, and CI pipeline

**Context:** Both debaters agreed immediately. Devnet airdrop rate limits and network instability make it unsuitable for CI.

**Decision:** **Hybrid** — localnet (`solana-test-validator`) for Anchor integration tests (Task 3) and CI (Task 7); devnet for CLI demo (Task 5) and dashboard (Task 6).

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Deterministic, sub-second feedback for Task 3 test suite
- (+) Task 7 CI pipeline is not dependent on external network availability
- (+) Devnet demo provides real-world Solana Explorer links for judges
- (−) Must maintain two deployment targets (localnet config and devnet config)

---

### [D4] Real devnet USDC or custom mock SPL token?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5, 6, 7 — every task that touches token handling

**Context:** Both debaters agreed that devnet USDC faucets are unreliable. The mint-agnostic approach is standard practice in Solana DeFi.

**Decision:** **Mint-agnostic program** with a configurable mint `Pubkey` in `OperatorConfig`. Use a custom-minted mock "USDC" token for the hackathon. Switch to real USDC for production by updating the config field.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Full control over token minting for demo provisioning
- (+) One `Pubkey` field addition to OperatorConfig — trivial cost
- (+) Clean production migration path (just change the mint address)
- (−) Demo uses a "fake" USDC — judges must understand it represents real USDC (mitigated by clear labeling)

---

### [D5] How should the controller submit Solana transactions?

**Status:** Accepted

**Task Context:** Task 4 — controller-Solana integration; Task 7 — CI pipeline

**Context:** This was the most contested decision. The Optimist argued for direct `solana-sdk` integration in the controller with async spawned tasks, citing simplicity and the "one HTTP POST" nature of RPC calls. The Pessimist argued forcefully for a thin sidecar reading from Redis, citing blast radius isolation — specifically, Solana RPC failures (BlockhashNotFound, congestion, rate limits) could cause spawned tasks to pile up, consume memory, and starve the tokio runtime in a critical-path Kubernetes operator. The Pessimist's question about what happens to CodeRun reconciliation during a 10-minute Solana outage with 50 queued settlements was decisive.

**Decision:** **Thin sidecar service** reading settlement events from Redis (already deployed as `gitlab/gitlab-redis-master` in the cluster), decoupling the Solana SDK from the controller binary entirely.

**Consensus:** 2/2 (100%) — The Optimist did not rebut in the second turn; the Pessimist's blast-radius argument stood.

**Consequences:**
- (+) Controller's reconciliation loop is completely isolated from Solana RPC failures
- (+) Redis stream provides natural backpressure and event persistence
- (+) Sidecar can retry independently with its own backoff logic
- (+) Controller's dependency tree stays clean (no solana-sdk, no 50+ new crates)
- (+) Clear operational separation: Kubernetes on-call vs Solana on-call
- (−) Two binaries to deploy and monitor instead of one
- (−) Slight additional latency (Redis hop) for settlement — irrelevant at billing granularity
- Caveat: For the hackathon, the sidecar is a small Rust binary in the same repo, not a complex microservice. Post-hackathon extraction into a proper service is natural.

---

### [D6] How should the operator wallet keypair be managed?

**Status:** Accepted

**Task Context:** Task 4 — controller/sidecar integration; Task 7 — CI pipeline

**Context:** Both debaters agreed immediately. The infrastructure already has External Secrets CRDs deployed, and the secrets pipeline is documented in `docs/secrets-management.md`.

**Decision:** Store the operator keypair in **OpenBao**, delivered to the sidecar pod via **External Secrets Operator** as a mounted file. For CI (Task 7), use a **GitHub Actions secret** for the devnet deployer keypair (devnet keys are disposable).

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Consistent with existing secrets architecture — no new patterns to learn or maintain
- (+) CI keypair management is separate from production keypair management
- (−) None raised

---

### [D7] What storage backend for off-chain receipt JSON?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 5, 6 — receipt storage in settlement, controller, demo, and dashboard

**Context:** The Optimist argued for Arweave via Irys, citing decentralization narrative appeal for Solana hackathon judges. The Pessimist pushed back strongly: Arweave/Irys adds a runtime dependency to the settlement critical path. If the upload fails, settlement either fails or proceeds without a resolvable receipt — both are unacceptable for what's intended to become a production billing rail. The verification story (judges check hash matches content) works identically regardless of storage backend.

**Decision:** **S3-compatible storage** (or serve from `cto-web`) for the hackathon. The receipt hash stored on-chain provides the verification primitive. Arweave is a post-hackathon enhancement.

**Consensus:** 2/2 (100%) — The Optimist did not rebut; the operational risk argument was accepted.

**Consequences:**
- (+) No runtime dependency on an external upload service in the settlement path
- (+) S3 is already operationally understood by the team
- (+) Verification story is identical — hash on-chain, content resolvable off-chain
- (−) Loses the "decentralized storage" narrative for hackathon judges — mitigated by the strong on-chain hash verification story
- Caveat: Arweave integration should be added post-hackathon as an optional storage backend, not a replacement for S3

---

### [D8] Should task_id be stored as String or fixed-size on-chain?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5, 6 — on-chain data model and explorer readability

**Context:** Both debaters agreed. Explorer readability directly serves PRD Section 12 success criteria. The cost is negligible.

**Decision:** Store `task_id` as a **bounded String (max 64 bytes)** using Anchor's `#[max_len(64)]` for display. Use its **SHA-256 hash for PDA derivation** (complementary to D2).

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Judges can read "CTO-1234" directly in Solana Explorer
- (+) ~68 bytes rent (~0.0005 SOL) is negligible
- (+) Same approach applies to `package_id` and `source_uri`
- (−) Variable-size account data — mitigated by Anchor's bounded string handling

---

### [D9] How should daily spending caps reset?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3 — on-chain cap enforcement logic

**Context:** Both debaters agreed immediately. Clock sysvar drift is seconds; billing granularity is days. Slot-based mapping to calendar days is imprecise.

**Decision:** **Unix timestamp-based reset** using `Clock` sysvar, comparing against UTC day boundaries via integer division (`unix_timestamp / 86400`).

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Clean "day number" that resets deterministically
- (+) Five lines of code in `settle_task`
- (+) More intuitive than slot-based mapping
- (−) Minor Clock drift (seconds) — irrelevant at daily billing granularity
- Note: The `daily_reset_slot` field in the PRD sketch should be renamed to `daily_reset_day` (storing the day number, not slot)

---

### [D10] Wallet adapter framework vs minimal custom implementation?

**Status:** Accepted

**Task Context:** Task 6 — web dashboard

**Context:** Both debaters agreed immediately. The standard wallet adapter is less code than a custom implementation for better results.

**Decision:** Use **`@solana/wallet-adapter-react`** with Phantom and Solflare wallet providers.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Drop-in with ~5 lines of provider setup
- (+) Interactive deposit/withdraw flows judges can try themselves
- (+) Standard pattern expected by Solana ecosystem judges
- (−) None raised

---

### [D11] Should the program auto-create ATAs or require pre-existing accounts?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5 — token account handling in program and demo

**Context:** The Optimist proposed a hybrid: require pre-existing ATAs for customers, use `init_if_needed` for author ATAs in `settle_task`. The Pessimist pushed back citing the Cashio $48M exploit involving re-initialization attack vectors. The Pessimist proposed moving ATA creation to `register_agent_package` (a one-time operation per author), eliminating `init_if_needed` from the high-frequency `settle_task` path entirely.

**Decision:** **Require pre-existing ATAs for all accounts.** Create author ATAs during `register_agent_package`, not during `settle_task`. The demo script (Task 5) handles ATA creation during onboarding phases.

**Consensus:** 2/2 (100%) — The Optimist did not rebut; the security argument was accepted.

**Consequences:**
- (+) Eliminates `init_if_needed` from the payment-critical `settle_task` instruction — zero re-initialization attack surface
- (+) Simpler instruction account lists (no system_program, associated_token_program, rent in settle_task)
- (+) Smaller transactions, cheaper to execute
- (−) Authors must have an ATA before receiving their first payment — solved by creating the ATA during `register_agent_package`
- (−) If an author's ATA is closed between registration and settlement, `settle_task` will fail — acceptable at hackathon scale, resolvable post-hackathon

---

### [D12] What should the dashboard's primary view display?

**Status:** Accepted

**Task Context:** Task 6 — web dashboard design

**Context:** Both debaters agreed. The PRD success criteria require demonstrating billing, splits, and verification — a split-panel covers both the settlement flow and the customer balance story.

**Decision:** **Split-panel view** — real-time settlement feed (left) showing quality-gated splits + customer balance overview (right) with deposit/withdraw.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Two panels on one screen tell the complete billing + marketplace story
- (+) Covers all PRD Section 12 success criteria in a single view
- (−) More complex layout than a single-purpose view — mitigated by being one page, not a complex app

---

### [D13] What visual style for the demo dashboard?

**Status:** Accepted

**Task Context:** Task 6 — web dashboard visual identity

**Context:** Both debaters agreed. Hackathon context makes ecosystem aesthetics important for judges.

**Decision:** **Solana-branded dark theme** with purple/green accents, incorporating the CTO/5D Labs logo — hybrid that signals ecosystem membership while maintaining brand identity.

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Signals "this team understands the Solana ecosystem" to judges
- (+) CTO logo maintains brand identity
- (−) No production brand implications — this is a demo aesthetic

---

### [D14] Which component library for the demo dashboard?

**Status:** Accepted

**Task Context:** Task 6 — web dashboard implementation

**Context:** Both debaters agreed immediately.

**Decision:** **shadcn/ui with Tailwind CSS.**

**Consensus:** 2/2 (100%)

**Consequences:**
- (+) Copy-paste components (not npm-installed), fully customizable, tree-shakeable
- (+) Tailwind is common in Solana DApp scaffold templates
- (+) Data table component handles settlement feed; card component handles balance display
- (−) None raised

## 4. Escalated Decisions

No decisions were escalated. All 14 decision points reached consensus (100% agreement).

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| On-chain program | Anchor framework | v0.30+ |
| Program language | Rust | stable (1.18.x Solana SDK compat) |
| Test framework | Anchor test suite | TypeScript/Mocha |
| Controller sidecar | Rust + solana-sdk + solana-client | 1.18.x |
| Message queue | Redis Streams | (existing in-cluster: `gitlab/gitlab-redis-master`) |
| CLI demo | TypeScript + @coral-xyz/anchor + @solana/web3.js | Latest |
| Web dashboard | Next.js 14 + React + TypeScript | App Router |
| UI components | shadcn/ui + Tailwind CSS | Latest |
| Wallet integration | @solana/wallet-adapter-react | Latest |
| CI/CD | GitHub Actions + Anchor CLI + Solana CLI | — |
| Secrets | OpenBao → External Secrets Operator → K8s Secrets | Existing pipeline |

### Service Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Solana Devnet                        │
│  ┌──────────────────────────────────────────────┐   │
│  │          cto-billing Program (Anchor)         │   │
│  │  OperatorConfig | CustomerBalance |           │   │
│  │  AgentPackage   | TaskReceipt                 │   │
│  └──────────────────────────────────────────────┘   │
└───────────────────────┬─────────────────────────────┘
                        │ RPC
           ┌────────────┼────────────┐
           │            │            │
  ┌────────▼──┐  ┌──────▼───┐  ┌────▼────────┐
  │ Settlement │  │ CLI Demo │  │ Web Dashboard│
  │ Sidecar    │  │ Script   │  │ (Next.js)    │
  │ (Rust)     │  │ (TS)     │  │              │
  └────────▲──┘  └──────────┘  └──────────────┘
           │ Redis Stream
  ┌────────┴──────────┐
  │  CTO Controller   │
  │  (Rust/Axum K8s)  │
  │  Reconcile Loop   │
  └───────────────────┘
```

**Communication Pattern:**
1. CTO Controller writes settlement events to a Redis stream on CodeRun terminal state transitions
2. Settlement Sidecar reads events from Redis, constructs and submits Solana transactions
3. Sidecar handles retry logic, blockhash management, and Solana-specific failure modes independently
4. Controller's reconciliation loop is never blocked by Solana RPC issues

### Key Patterns

- **PDA derivation**: All PDAs use SHA-256 hashing of human-readable identifiers for seed consistency
- **Mint-agnostic tokens**: Program accepts any SPL mint via `OperatorConfig.mint` field
- **Pre-existing ATAs**: All token accounts must be pre-created; no `init_if_needed` in any instruction
- **Daily cap reset**: UTC day boundaries via `Clock::unix_timestamp / 86400` integer division
- **Circuit breaker**: `OperatorConfig.paused` flag checked by all settlement instructions
- **Feature gating**: Controller's Solana billing behind `solana-billing` Cargo feature flag

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Direct solana-sdk in controller binary | Blast radius — Solana RPC failures must not affect CodeRun reconciliation (D5) |
| Arweave/Irys for receipt storage at hackathon | Runtime dependency on external upload service in critical billing path (D7) |
| `init_if_needed` in settle_task | Re-initialization attack surface in high-frequency payment instruction (D11) |
| Real devnet USDC | Faucet unreliability; mock token with configurable mint is standard practice (D4) |
| Slot-based daily cap reset | Slots don't map cleanly to calendar days; timestamp / 86400 is precise enough (D9) |
| Custom wallet implementation | Standard wallet-adapter-react is less code for better results (D10) |

## 6. Implementation Constraints

### Security Requirements

- **Operator keypair**: Stored in OpenBao, delivered via External Secrets Operator as a mounted file. Never committed to source control. CI uses a separate, disposable devnet keypair stored as a GitHub Actions secret.
- **No `init_if_needed`**: All ATAs must be pre-created. The `settle_task` instruction must never auto-create token accounts.
- **Signer validation**: Every operator-signed instruction must verify `operator.key() == operator_config.authority`.
- **Circuit breaker**: All settlement and refund instructions must check `!operator_config.paused`.
- **Spending caps**: `settle_task` must enforce `max_per_task` and `max_per_day` on every invocation, with daily reset logic.

### Performance Targets

- **Test suite**: All Anchor integration tests must run against local validator and complete in under 60 seconds
- **Demo script**: Full 10-phase CLI demo must complete in under 120 seconds on localnet
- **Dashboard**: Page load under 3 seconds on devnet; Lighthouse accessibility score ≥ 80
- **CI pipeline**: Build + test job should complete within a reasonable time; devnet deployment is additive

### Operational Requirements

- **Feature flag**: Controller's Solana billing integration must be behind a `solana-billing` Cargo feature flag, defaulting to `false`
- **Settlement isolation**: The settlement sidecar must be an independent binary. Controller reconciliation must continue operating if the sidecar is down or Solana is unreachable.
- **Idempotent settlement**: `settle_task` with a duplicate `task_id` must fail with `DuplicateTaskId` — the PDA seed ensures this at the protocol level
- **Graceful degradation**: If Solana settlement fails after sidecar retries, the event remains in Redis for manual investigation. The CodeRun lifecycle is not blocked.

### Service Dependencies and Integration Points

- **Redis**: Settlement sidecar depends on the in-cluster Redis instance (`gitlab/gitlab-redis-master`) for event streaming
- **OpenBao**: Operator keypair provisioning depends on the existing External Secrets pipeline
- **Solana devnet RPC**: Demo artifacts (Tasks 5, 6, 7 deployment) depend on devnet availability; CI tests do not
- **Anchor IDL**: Tasks 3, 5, and 6 consume the generated IDL at `target/idl/cto_billing.json` — Task 1 must produce this artifact

### Organizational Preferences

- Prefer existing infrastructure (Redis, OpenBao, S3) over new external dependencies
- Follow existing secrets pipeline patterns documented in `docs/secrets-management.md`
- Maintain consistency with the CTO controller's Rust/Axum architecture

## 7. Design Intake Summary

### Frontend Targets

- **`hasFrontend`**: true
- **`frontendTargets`**: web
- **Provider mode**: Stitch (design generation via Stitch provider)
- **Stitch status**: generated
- **Framer status**: skipped (not requested)

### Implications for Web Implementation (Task 6)

The dashboard is a web-only target built with Next.js 14 / React / TypeScript. Design decisions resolved in the deliberation provide clear direction:

- **Visual identity** (D13): Solana-branded dark theme with purple/green accents + CTO logo
- **Component library** (D14): shadcn/ui + Tailwind CSS — copy-paste components, tree-shakeable
- **Layout pattern** (D12): Split-panel — settlement feed (left) + customer balance (right)
- **Wallet UX** (D10): @solana/wallet-adapter-react with Phantom/Solflare

These decisions collectively mean the dashboard should use:
- Tailwind's `dark` mode with custom Solana-purple (`#9945FF`) and Solana-green (`#14F195`) accent colors
- shadcn/ui `DataTable` for the settlement feed, `Card` for balance display, `Badge` for quality_met status indicators
- Color-coded rows: green for `quality_met=true`, red/gray for `quality_met=false`, yellow for Refunded
- Responsive design optimized for 1920×1080 screen recording

No design selections or design deliberation results were provided; the technical deliberation resolved all visual/UX decisions directly.

## 8. Open Questions

The following items were not debated or fully resolved and should be handled by implementing agents using their best judgment:

1. **Vault PDA authority pattern**: The exact PDA seed structure for the vault authority (e.g., `['vault_authority']` vs `['vault', mint.key()]`) should be chosen by the implementing agent for Task 1. Both approaches work; pick whichever keeps the seed namespace cleanest.

2. **Anchor event emission**: The specific `emit!()` event structures for `TaskSettled`, `Deposited`, `Withdrawn`, and `AgentPackageRegistered` are outlined in Task 2 details but the exact field lists should be finalized during implementation.

3. **Refund treasury mechanics**: For the hackathon, refunds come from the treasury (5D Labs absorbs the cost if an author was already paid). The implementing agent should document this simplification clearly in code comments for post-hackathon refinement.

4. **Receipt JSON schema**: The exact fields and structure of the off-chain receipt JSON blob are left to the implementing agents for Tasks 4 and 5. It should include at minimum: task_id, customer, duration, infra tier, provider, agent used, itemized cost breakdown, and timestamp.

5. **Dashboard real-time updates**: Whether to use Solana WebSocket subscriptions or polling (every 5 seconds as suggested in Task 6) is an implementation choice. Polling is simpler and sufficient for the demo.

6. **Web dashboard hosting**: Whether to deploy the Next.js dashboard to Vercel/Netlify for a live demo URL is optional but recommended. The CI pipeline (Task 7, Job 4) can handle this.

7. **Error recovery UX**: How the dashboard handles RPC errors, disconnected wallets, or missing PDAs during navigation should follow standard patterns — show error states gracefully, don't crash.

8. **Billing unit for demo**: The demo script (Task 5) uses fixed amounts ($1.50, $3.00) per task. Whether to show metered billing calculation (duration × tier rate) in the demo narrative is a presentation choice — the program accepts any amount from the operator.

