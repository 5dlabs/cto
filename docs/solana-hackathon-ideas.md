# Solana Hackathon — On-Chain Agent Ideas

High-level brainstorm for a Solana hackathon submission that leans on the
CTO platform's existing strengths (18-agent roster, K8s CRD orchestration,
`skills/trader/*` Solana tooling, bare-metal infra). This is an ideation
document, not a spec — the goal is to explore the design space before
committing to one build.

## Framing

The hackathon prize is for an **on-chain agent**, which is a higher bar than
"an agent that writes on-chain code." Judges generally reward things that
actually live, transact, and settle on chain — not off-chain agents that
produce on-chain artifacts as output.

CTO is well-positioned here because the platform already has:

- A specialized agent roster with distinct roles (review, security, test, merge)
- A Kubernetes-native CRD execution loop (`CodeRun`, `BoltRun`) in
  `crates/controller/src/crds/`
- Production Solana skills in `skills/trader/*` (`moltiumv2`, `solana-connect`,
  `solana-transfer`, `solana-sniper-bot`, `solana-copy-trader`, `jup-skill`,
  `fairscale-solana`)
- A planned (not yet built) Solana K8s operator at `docs/solana-operator-spec.md`
- Bare-metal infrastructure (Latitude, Hetzner, PhoenixNAP) that is a genuine
  differentiator vs. cloud-hosted agent platforms

What CTO does **not** yet have:

- Any payment, settlement, or billing layer for agent work
- A public registry / marketplace for agents or skills
- On-chain reputation, attestation, or identity for agents
- Economic accountability (slashing, insurance) for agent quality

Every idea below is an attempt to fill one of those gaps in a way that is
native to Solana and uses the existing CTO building blocks.

## The protocol model (wallet ownership)

Before listing ideas, a shared mental model for "who owns what" wallet:

- **Customer escrow PDA** — program-derived account that holds the customer's
  SPL/SOL deposit for a task. Nobody holds the key; the program is custodian.
- **Author wallet** — registered on-chain against each agent or skill. Whoever
  publishes the agent puts their payout address here. Could be 5dlabs, could
  be a community builder, could be a DAO.
- **Protocol fee wallet** — optional cut to 5dlabs for running the runtime.
  Uniswap-style fee switch.
- **Stake/slash vault** — optional insurance pool funded by author stake, paid
  out to harmed customers when attestations flag failure.

Once framed as a protocol (rather than a 5dlabs product), the registry /
marketplace ideas below stop being awkward and start being the point.

## Round 1 ideas — settlement and attestation

### 1. On-chain bounty settlement for `CodeRun` tasks

Every `CodeRun` CRD gets a matching PDA that holds SPL escrow. When Stitch
merges the PR, Tess attests tests pass, and Cipher attests security clean,
the controller submits proof (commit hash, CI attestations) to a Solana
program that atomically releases payment to the agent author's wallet.

- **Leverages:** `crates/controller`, existing agent wallet infra in
  `solana-connect`, Linear task metadata.
- **Unique angle:** solves a real unsolved problem — how to pay AI agents.
- **Demo loop:** Linear issue → PR merged → SPL hits wallet (filmable in 30s).

### 2. Agent Reputation SBT (extends `fairscale-solana`)

Non-transferable tokens per agent keyed to verifiable outcomes: tests passed,
security findings, review approvals, merge rate. The controller signs and
emits attestations as the `CodeRun` status progresses. The program aggregates
into a rep score. Clients pick agents by on-chain reputation, not vendor trust.

- **Pairs with #1:** reputation gates bounty size.
- **Unique angle:** reputation is earned from verifiable events, not self-report.

### 3. On-chain attestation multi-sig for PR merges

Tess (tests), Cipher (security), and Stitch (review) each hold Solana
keypairs. Their approvals become on-chain signatures, not GitHub labels. A
Solana program gates merging: a PR can only be merged if N-of-M agent
signatures attest success. Replaces Branch Protection with a verifiable
on-chain quorum.

- **Leverages:** existing review/quality/testing loop in the controller.
- **Unique angle:** security-flavored, very demo-able, maps cleanly to
  existing CRD state transitions.

### 4. Anchor audit agent (Cipher variant)

Instead of *writing* programs, Cipher audits them. Feed an Anchor IDL +
source, output findings, sign them, post the hash on-chain as a reusable
attestation that other dapps can consume as a trust signal.

- **Differentiated:** everyone writes; few audit.
- **Leverages:** Cipher's existing role and security posture.

### 5. "Block" — codegen agent with on-chain deploy registry

The original instinct: a Solana/Anchor codegen agent. Made more interesting
by making the **deploy target** the on-chain part: Block takes a Linear
spec, generates Anchor code, deploys to devnet, and self-registers the
program on a Solana registry program so the ecosystem can discover
CTO-built programs.

- **Caveat:** generic "AI writes code" submissions will be a crowded category.
  The registry twist is what makes it on-chain rather than on-chain-adjacent.

## Round 2 ideas — open registry and economics

These emerged from the observation that, once there's a public registry,
5dlabs isn't *the* author anymore — it's the runtime provider for a market
of authors. This is a stronger narrative for a hackathon and resolves the
wallet ownership ambiguity in #1.

### 6. Skill Registry with royalty splits ⭐

Skills (not whole agents) are the unit of reuse — `skills/trader/*` already
proves this. Publish each skill to a Solana program with: author wallet,
version, spec URI (Arweave/IPFS), price per invocation. When an agent runs
a task, the runtime emits a signed "invocation receipt" and the program
streams micro-royalties to every skill author touched. One task = an atomic
royalty split across 3–8 authors.

- **Unique angle:** composition-native. Most "AI marketplaces" pay a single
  recipient. This pays every skill the task touched, proportionally, in one
  Solana transaction. Nobody has built this.
- **Pitch:** "npm + Stripe for agent capabilities, settled on Solana."
- **Model fit:** the `skills/` directory is already the data model.

### 7. Spec-as-NFT bounty marketplace

Flip intake upside down. Customer mints a spec NFT (Linear-style requirements
+ acceptance criteria + escrow). Any registered agent can attempt it. First
submission that passes Tess's on-chain test attestation + Cipher's security
attestation wins the escrow.

- **Unique angle:** Kaggle for Solana dev work, but judging is automated by
  the existing review agents acting as oracles.
- **Demo-able:** mint spec → two agents race → winner wallet gets paid live.

### 8. Verifiable build attestations

Real supply-chain problem on Solana: how do you know deployed program
bytecode matches the source an agent produced? The agent signs its output,
Cipher signs an audit attestation, and a program records the triple
(source hash, build hash, audit hash). Customers verify against on-chain
state before trusting any agent-deployed program.

- **Unique angle:** defensive, timely, plays to Cipher's role. Hits the
  "supply chain security" narrative that is broadly underserved on Solana.

### 9. Agent compute marketplace

CTO's bare-metal footprint (Latitude, Hetzner, PhoenixNAP) is actually a
differentiator. Sell spare cluster capacity to third-party agents via a
Solana program: requester escrows SOL, a CTO cluster picks up the job,
returns signed job result, program releases payment metered by CPU-seconds
or GPU-minutes. Akash/io.net-shaped but agent-native and narrower.

- **Leverages:** `BoltRun` infrastructure and the K8s provisioning stack.

### 10. Stake-to-publish quality signal

To list an agent or skill in the registry, the author must stake SOL. If
their agent causes a task to fail (Tess/Cipher attest failure), a slice of
the stake is slashed and redirected to the harmed customer. Creates
skin-in-the-game quality signal without needing subjective reviews.

- **Unique angle:** reputation becomes economic, not cosmetic.
- **Pairs well with:** #6, #7, #8.

### 11. Agent identity (DID) on Solana

Each agent has a Solana keypair as its canonical identity. Every commit,
review, merge, and attestation is signed by that key and logged on chain.
Reputation becomes portable — an agent could leave CTO and carry its history
to another runtime.

- **Less flashy solo**, but it's the primitive that makes 1–10 possible.
  Probably worth shipping as a foundational layer regardless of which
  headline idea gets chosen.

## Round 3 — additional angles worth considering

Captured here for completeness; some are smaller, some are more speculative.

### 12. Agent-to-agent micropayments

Agents can invoke each other across tasks and settle per-call in SPL. Turns
the existing multi-agent orchestration (Morgan → Rex → Tess → Stitch) into
an economic flow, where each handoff is a paid micropayment. Natural
extension of #6.

### 13. Prompt / spec NFTs with royalties

Reusable prompts, specs, and task templates as NFTs with author royalties.
Every time a CTO task references a prompt NFT, the author earns. Treats
well-engineered prompts as IP.

### 14. Agent SLA with time-locked commitments

Agent commits to an SLA (e.g. "merge within 24h of task dispatch"), stakes
against it, and if the SLA is missed the stake auto-pays the customer. Smart
contract enforced, no dispute resolution needed. Could sit on top of #10.

### 15. Agent service insurance pool

Shared SOL pool that pays out if an agent's output breaks production.
Premiums are paid per task and scale with Cipher audit risk scores. Creates
a market for agent quality without requiring upfront stake from every author.

### 16. Training data provenance

Every merged PR + the agent's full trajectory (prompts, tool calls, diffs)
becomes labeled training data. Hash is recorded on-chain; access to the
dataset is gated by a Solana program. Agents and customers share revenue
from data licensing.

### 17. Agent DAO governance

Platform roadmap, protocol parameters, fee switch, and slashing thresholds
are governed by a token held by agent authors, customers, and 5dlabs. Maps
cleanly to #6's author economy.

### 18. Kubernetes-on-Solana control plane

A Solana program triggers `BoltRun` / `CodeRun` CRDs — the chain becomes the
external control plane for CTO infrastructure. Customers request capacity
on-chain; the controller watches events and reconciles. Reverses the normal
direction of integration and is genuinely novel.

### 19. Cross-chain bridge attestation agent

The `Tap` agent becomes a bridge orchestrator, signing cross-chain transfer
attestations as a specialized role. Pairs with #11 (agent DID).

### 20. Oracle network of specialized agents

Agents publish signed data feeds (price, code quality metrics, CI status,
chain state summaries) to a Solana program. Other programs consume them as
a specialized oracle network. Natural home for Nova (research) and Vex
(debugging/RCA).

### 21. MEV-aware deployment flow

Block-style codegen agent that deploys Anchor programs with MEV protection
baked in (Jito bundles, priority fee tuning). Smaller idea but a concrete
on-chain benefit beyond generic codegen.

### 22. Real-time review marketplace

Stitch posts review offers on-chain; anyone can pay SOL to fast-track a PR
review. Opens up the existing review bottleneck as a paid service and turns
review capacity into a market.

## Recommendation

The strongest shippable-in-a-hackathon combination:

> **Skill Registry with royalty splits (#6) + Stake-to-publish (#10) +
> Agent DID (#11) as the foundational primitive.**

One-sentence pitch:

> *A Solana-native registry where independent builders publish AI agent
> skills, stake to guarantee quality, and earn streaming royalties every
> time a CTO agent invokes their skill on a customer task — all settled
> atomically on-chain.*

### Why this one

- Resolves the wallet ownership question cleanly — authors ≠ 5dlabs.
- The `skills/` directory is already the data model; we'd be formalizing
  what exists rather than inventing.
- Royalty-split math is interesting on-chain content — most marketplaces
  are single-recipient, this is N-way per task.
- Slashing adds a defensive/safety story that is rare in agent projects.
- The demo loop is tight: publish a skill → another agent invokes it →
  watch SOL stream to your wallet in real time.

### What a minimal build looks like

Anchor program with roughly these accounts:

- `SkillRegistry` — global config, fee switch, slashing parameters
- `Skill` — author wallet, version, spec URI, price per invocation
- `AuthorProfile` — aggregate stats, reputation, claimable balance
- `StakeVault` — per-author stake, slashing history
- `TaskReceipt` — per-task record of which skills were invoked and by whom
- `AttestationRecord` — Tess / Cipher / Stitch on-chain signatures

Instructions roughly:

- `register_skill`, `update_skill`, `deprecate_skill`
- `stake`, `unstake_request`, `withdraw_stake`
- `open_task`, `record_invocation`, `settle_task`, `claim_royalties`
- `attest_success`, `attest_failure`, `slash`

Controller integration points to explore:

- Hook in `crates/controller/src/tasks/code/controller.rs` to emit
  invocation receipts when a skill is loaded or a subtask dispatched.
- New CRD status field for `on_chain_receipt_sig` so reconciliation can
  track settlement state alongside the existing task state machine.
- New secret type for agent keypairs (reuse `solana-connect` patterns).

## Open questions

- Which chain environment for the hackathon demo — devnet, mainnet, or a
  local test validator spun up by the existing Solana operator work?
- Does the skill registry need off-chain indexing (Helius webhooks) to make
  the UI snappy, or is on-chain query enough for the demo?
- Should royalty splits be computed on-chain (expensive, transparent) or
  signed off-chain by the runtime and verified on-chain (cheaper, more
  trust in runtime)?
- How does 5dlabs' protocol fee get set — fixed at launch, governance
  controlled, or disabled for the hackathon build?
- What's the minimum viable attestation set for slashing — just Tess
  (tests) and Cipher (security), or also Stitch (review)?

## Not chosen, and why

- **Raw codegen agent ("Block writes Anchor code")**: viable, but the agent
  itself lives off-chain and every hackathon will have this. Worth building
  *only* if combined with a registry/deploy story (#5) that makes the
  on-chain part load-bearing.
- **Trading / DeFi agent**: the existing `skills/trader/*` could support a
  "Block trades on your behalf" submission, but the hackathon prompt is
  about on-chain agents, not trading strategies. Lower ceiling.
- **Full Solana K8s operator**: too large for a hackathon window. Track
  separately under `docs/solana-operator-spec.md`.
