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
> ## 3a. Design principles
> 
> - **Decentralized-first.** When choosing between a centralized and a
>   decentralized approach for any component, prefer the decentralized option
>   unless it makes the hackathon demo technically infeasible. This applies to
>   storage (Arweave / IPFS over S3 / SeaweedFS), identity (wallet-based over
>   OAuth), data availability (on-chain or permanent storage over ephemeral
>   databases), and coordination (program-enforced splits over off-chain
>   accounting). A Solana hackathon rewards on-chain-native thinking; leaning
>   into decentralized infrastructure earns credibility with judges and aligns
>   with the long-term architecture.
> - **Verifiable by default.** Every economic event should produce an
>   independently verifiable artifact — an on-chain transaction, a content-
>   addressed receipt, or a signed attestation.
> - **Composable over monolithic.** Prefer small, well-defined instructions that
>   external programs and clients can compose, over large instructions that
>   internalize workflow logic.
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
>   via Irys/Bundlr — preferred per §3a decentralized-first principle)
>   containing the itemized breakdown.
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
> 2. Builds the itemized receipt JSON and uploads to Arweave via Irys
>    (content-addressed, permanent, decentralized — per §3a).
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
> 

## 2. Project Scope

- Total tasks identified: 10
- Task 1: Bootstrap Solana Dev Infrastructure & Anchor Workspace (Bolt - Kubernetes/Helm) — Set up the Solana development toolchain, Anchor workspace scaffolding, devnet configuration, Irys/Bundlr devnet credentials, and supporting infrastructure (local validator option, USDC devnet mint). This is the foundational layer that all subsequent Solana-related tasks depend on.
- Task 2: Implement Anchor Program Core — Operator, Customer & Escrow Instructions (Rex - Rust/Anchor) — Implement the foundational Anchor program accounts (OperatorConfig, CustomerBalance) and instructions (initialize_operator, create_customer_account, deposit, withdraw, update_spending_caps, pause/unpause). This establishes the escrow-based billing primitive that all settlement logic depends on.
- Task 3: Implement Anchor Program Marketplace — Agent Packages, Settlement & Receipts (Rex - Rust/Anchor) — Implement the marketplace and settlement layer: AgentPackage registration, the settle_task instruction with quality-gated payment splitting (all three cases from PRD §6.3), refund_task, and TaskReceipt on-chain accounts. This completes the Anchor program per the hackathon deliverables.
- Task 4: Build Arweave/Irys Receipt Upload Module (Nova - Bun/TypeScript) — Create a TypeScript module that takes an itemized receipt JSON object, uploads it to Arweave via Irys (formerly Bundlr) for permanent decentralized storage per §3a design principles, and returns the content-addressed transaction ID and SHA-256 hash for on-chain reference. This module is consumed by both the controller integration (Task 5) and the CLI demo (Task 6).
- Task 5: Integrate Solana Settlement Hook into CTO Controller (Rex - Rust/Axum) — Add the settlement hook to the existing CTO Kubernetes controller so that when a CodeRun reaches a terminal state, the controller computes the billable amount, uploads the receipt to Arweave, and submits settle_task or refund_task to the Solana program. This bridges the off-chain runtime to the on-chain billing rail per PRD §9.
- Task 6: Build Hackathon Demo CLI Script — Full E2E Flow (Nova - Bun/TypeScript) — Create a TypeScript CLI script that demonstrates the complete hackathon flow end-to-end: register an agent package, create customer, deposit USDC, settle with quality met (author gets paid), settle with quality NOT met (customer not charged), verify receipts, and withdraw remaining balance. This is hackathon deliverable #2 and the basis for the demo video.
- Task 7: Build Web Dashboard for Receipts, Balances & Agent Packages (Blaze - React/Next.js) — Create a web dashboard that lets customers view their on-chain balance, browse task receipts with verification links, and explore registered agent packages with stats. This provides the visual verification layer that makes the hackathon demo compelling — a customer can see their billing history directly from the blockchain.
- Task 8: Comprehensive Integration & E2E Test Suite (Tess - Test Frameworks) — Build a comprehensive test suite that validates the entire on-chain billing system end-to-end: Anchor program integration tests covering all instruction paths, edge cases, error conditions, and the full hackathon demo flow as an automated test. This ensures the program is robust for the hackathon demo and establishes the test foundation for production hardening.
- Task 9: Security Review of Solana Program — Authority, Overflow & Escrow Safety (Cipher - Security Tooling) — Perform a focused security review of the cto-billing Anchor program, checking for common Solana program vulnerabilities: missing signer checks, PDA seed collisions, integer overflow/underflow, unauthorized fund access, reentrancy-like patterns, and escrow drain vectors. While a full audit is a non-goal (PRD §4), this review ensures the hackathon demo is not trivially exploitable and documents known risks for post-hackathon hardening.
- Task 10: CI/CD Pipeline & Devnet Deployment Automation (Atlas - CI/CD Platforms) — Build the CI/CD pipeline that builds, tests, and deploys the Anchor program to Solana devnet, runs the full integration test suite, builds the web dashboard, and packages the demo CLI. This ensures reproducible builds and automated deployment for the hackathon submission.

## 3. Resolved Decisions

### [D1] Decision 1

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D2] Decision 2

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D3] Decision 3

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D4] Decision 4

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D5] Decision 5

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D6] Decision 6

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D7] Decision 7

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output

### [D8] Decision 8

- Status: Accepted
- Decision: Not specified
- Consensus: Recorded in deliberation output


## 4. Architecture Overview

- Deliberation status: completed
- Debate turns: 2
- Elapsed minutes: 26
- Decision points considered: 15
- Decisions resolved: 8

## 5. Design Intake Summary

mode=stitch; provider_mode=stitch; has_frontend=true; targets=web; stitch_status=generated; stitch_reason=; framer_status=skipped; framer_reason=not_requested

## 6. Open Questions

- Use the deliberation record and task decomposition to resolve any remaining implementation details not captured above.


