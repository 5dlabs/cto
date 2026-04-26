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

---

# Follow-up brainstorm: payment flow, UX, ranking, and NFT integration

The sections below capture follow-up discussion after the initial ideation,
working through four questions that came up while chewing on the lead idea
(Skill Registry with royalty splits):

1. Where does the money actually come from?
2. What does the customer-facing UX look like?
3. How do we rank agents/skills truthfully?
4. Can NFTs play a role here, and where?

Nothing in this section changes the plan — it's a brain dump to preserve the
thinking.

## 1. Where the money comes from (economics walkthrough)

The skill registry doesn't print anything. It's a **payment-routing and
accountability layer on top of real customer demand**. The same money that
today flows to 5dlabs as consulting / platform fees gets routed differently
— a slice lands in skill author wallets instead of 100% in 5dlabs' pocket.

### Concrete scenario with numbers

1. **Customer "Acme Corp"** wants a feature built. They'd pay a dev agency
   $5k for it today. Instead they pay CTO $5k (in USDC, deposited into a
   task escrow PDA).
2. **CTO runtime dispatches the task.** Morgan decomposes, Rex implements,
   Tess tests, Cipher audits, Stitch merges. Along the way the agents
   invoke six skills: `solana-connect`, `moltiumv2`, an Anchor codegen
   skill, a test-harness skill, a security-audit skill, a linter skill.
3. **Tess + Cipher + Stitch attest success on-chain.** The Solana program
   releases the $5k escrow and splits it per the registry rules, e.g.:
   - 60% ($3,000) to 5dlabs (runtime operator — servers, orchestration,
     customer relationship)
   - 30% ($1,500) split across the 6 skill authors, weighted by some policy
     (flat split, invocation count, gas-style complexity weight, etc.)
   - 5% ($250) to a protocol fee / treasury
   - 5% ($250) to a slashing / insurance pool
4. **Skill authors wake up to USDC in their wallets.** Whoever wrote
   `moltiumv2` earns every time any customer's task touches it, forever,
   without any further work from them.

That's the entire flow. The customer was always paying for software work.
The registry just makes sure the people who wrote the reusable pieces get
a cut, on-chain, automatically.

### Analogies that make it click

- **Spotify.** Listener (customer) pays $10/month. Spotify keeps ~30%,
  distributes ~70% to artists weighted by play share. Skill authors are
  the artists; CTO is Spotify; customers are listeners.
- **App Store.** User buys an app for $10. Apple takes 30%, dev gets 70%.
  One transaction, split at the point of sale.
- **Uniswap LPs.** Liquidity providers earn fees from swaps. Skill authors
  earn fees from invocations. Fees come from the actor with real demand.

The novel bit vs. all three: instead of one recipient per transaction, CTO
splits across *every skill touched in the task*, atomically, on chain, in
one Solana transaction.

### Two plausible customer-facing payment models

1. **Per-task escrow** (preferred for the hackathon demo). Customer deposits
   per task. Fully on-chain, fully transparent. Very Solana-native.
2. **Subscription + metered pool** (preferred for real enterprise customers).
   Customer pays CTO a flat monthly fee. 5dlabs routes a fixed % of revenue
   into an on-chain royalty pool. The program distributes to skill authors
   weighted by invocation share over the month. More Spotify-shaped.

Real product eventually offers both. Hackathon demo uses #1 because it's
more visibly on-chain in the video.

### Cold-start options

Before real customers exist, nobody gets paid. Options to bootstrap:

- **5dlabs seeds it with existing customers.** CTO already has paying
  customers. Route their payments through the registry from day one —
  authors earn from real revenue immediately.
- **Hackathon prize / grant funds a public-goods bounty pool.** Put $X into
  the escrow as bounties that pay out to skill authors when their skills
  get used on open-source tasks.
- **Stake-to-publish as self-selection.** Authors who believe in the
  platform stake SOL to list, write skills speculatively, earn when volume
  arrives. Same dynamic as early Uniswap LPs.

For the hackathon itself, cold start is not a problem — seed with
5dlabs-controlled USDC and demonstrate the loop live on devnet.

### The bigger bet (worth naming in the pitch)

Today, open-source maintainers get nothing. If you wrote a library that
every AI agent depends on, you earn zero. This model is the first time
open-source skill authors **get paid per use, automatically**, without
invoices or contracts. The hackathon submission is a prototype of the
economic layer that makes open-source AI tooling sustainable.

## 2. Customer-facing UX (wallet linking and payment flow)

### Wallet linking is standard and dynamic

No technical blocker. Solana wallet-adapter libraries (Phantom, Backpack,
Solflare, etc.) handle this. The UX:

1. User logs into CTO with normal auth.
2. Settings page has a "Connect Wallet" button → Phantom pops up → user
   approves → CTO backend gets their pubkey.
3. Pubkey is stored on the user's CTO profile. CTO **never** touches the
   private key — it stays in the wallet extension.
4. User can unlink, relink, or swap wallets anytime.

### Default agents vs. public agents (one consistent code path)

- **Default agents** (curated by 5dlabs, known-good) — registered in the
  registry with 5dlabs as the author wallet. Every task flows through
  settlement regardless of whether the user is on default or public.
- **Public agents** (community-submitted) — opt-in for customers who want
  to experiment, earn their keep by getting chosen, discoverable via the
  leaderboard.

Important: **there is only one code path.** Everything goes through the
registry. The only difference between "default" and "public" is whose
wallet the authorship field points to. This avoids branching logic in the
runtime and keeps the economic story consistent.

### Three payment UX patterns (escrow mechanics)

Real customers will hate signing a Phantom transaction for every task.
Three patterns, ranked by fit:

1. **Pre-paid balance (AWS credits model)** — customer tops up a balance
   PDA with, say, 50 USDC. Runtime deducts per task, no wallet prompt for
   each task. UI nudges top-ups when balance runs low. **Best for real
   customers.**
2. **Session key delegation** — customer signs once to delegate "CTO can
   spend up to 50 USDC over the next 30 days from my wallet." Modern
   Solana pattern. Middle ground between pure and practical. **Slick for
   demo, workable for real.**
3. **Per-task escrow signature** — customer signs a new transaction per
   task. Maximally visible on chain, clunky UX. **Fine for hackathon demo,
   not for real customers.**

Hackathon demo should use #2 or #3 because they're more visibly on-chain
in the video. Real product converges on #1.

## 3. Performance, settlement, and ranking

### Binary settlement beats graded performance

The temptation is to pay agents proportional to "how well they performed,"
but "performance" is subjective and every author will argue about the
rubric. The cleaner version:

- **Each skill has a fixed price** set by its author (per invocation, or
  per task tier).
- **Task cost is deterministic** — sum of skill prices invoked + runtime
  fee. Customer knows the bill upfront.
- **Settlement is binary.** Did the task pass the attestation bar (Tess
  green, Cipher clean, Stitch merged)? If yes → escrow pays out per the
  split. If no → escrow refunds to customer and the failing skill
  authors' stake gets slashed.
- **Performance is expressed through *selection*, not *discount*.** Bad
  skills don't get chosen. Good skills get picked more. Payment per
  invocation is fixed, but total earnings reflect quality because volume
  reflects trust.

### Cumulative earnings as the ranking signal

Earnings are a **market-validated quality signal** — they aggregate every
customer's revealed preference into one truthful number. Same primitive
as App Store top charts, Spotify top artists, npm download counts. Way
better than star ratings (gameable) or review counts.

### Refinements worth stealing from existing systems

- **Don't rank by a single number.** Offer multiple views:
  - *All-time earnings* → most battle-tested
  - *Last 30 days earnings* → currently hot
  - *Earnings per invocation* → premium / high-quality niche skills
  - *Attestation pass rate* → reliability dimension
  - *Slash rate (inverse)* → safety dimension
- **Watch the Matthew effect.** Early movers get a compounding advantage
  — a mediocre first-mover can outrank a better latecomer. Mitigate with
  time-decayed rankings and a "rising" / "new" discovery feed.
- **Rank per category.** A great Anchor codegen skill shouldn't compete
  with a great test-harness skill. Leaderboards by skill category
  preserve diversity and reduce winner-take-all dynamics.

## 4. NFT integration — where they fit and where they don't

### Pushback on the DRM framing

The initial NFT idea was "NFT unlocks the compressed agent package." This
is DRM with extra steps, and has three problems:

1. **DRM doesn't work without trusted execution.** Once the file is
   decrypted on a user's machine, they can copy it. Lit Protocol and
   similar tools can gate *decryption* behind wallet ownership, but can't
   stop leaks after plaintext is exposed. Real enforcement needs SGX/TEE.
2. **Culture clash with open-source AI tooling.** Most skill authors want
   their stuff inspectable — it's how trust is built ("I can read this
   skill before I let it run on my task"). Hiding code behind an NFT
   paywall fights that current.
3. **Redundant with the runtime.** If agents only run inside CTO's K8s
   cluster, the runtime already controls access. The registry just needs
   to know which author wallet gets paid when the skill runs — no crypto
   gate required.

### The better framing — flip it

**The code is open; the NFT represents title to the earnings stream.**

The agent package sits unencrypted on Arweave/IPFS. Anyone can inspect it,
learn from it, fork it. When it runs inside a CTO task, a slice of the
customer's payment flows to whoever currently holds the **authorship NFT**.
Transfer the NFT → transfer the royalty rights.

This is how music royalty platforms (Royal, Anotherblock) work. The music
isn't gated — it's on Spotify, free to listen to. The NFT is the title
deed to a future revenue stream.

Why it's strictly better than the DRM version:

- Authors can **sell their skill as a cash-flowing asset.** "This skill
  earned 400 SOL last quarter, I'm listing the NFT for 2000 SOL." Like
  selling a YouTube channel or a Substack.
- Secondary markets emerge naturally (Magic Eden, Tensor). Buyers are
  really buying a royalty stream. The leaderboard becomes a trading screen.
- Doesn't fight open source — enhances it. Open skills with sellable title
  are more valuable than closed skills nobody can audit.

### Where NFTs genuinely add value

**a. Authorship NFT (one per registered skill or agent)**

Regular NFT, minted when the skill is registered. Holder wallet is the
payout address. Transferable = sellable royalty stream. Becomes the
canonical on-chain identity for the skill. Replaces the `author_wallet`
field in the registry design — the NFT *is* the field.

**b. Invocation Receipt cNFTs (Solana-specific)**

**Compressed NFTs** are a Solana primitive that lets you mint millions of
NFTs for fractions of a cent (~$0.00001 each). Perfect for high-volume
receipts.

Every time a skill is invoked in a task, mint a cNFT to the customer as a
receipt: "Skill X was used in Task Y, settled Z USDC on date D." The cNFT
is the on-chain audit trail. Billions of these become feasible. Other
programs read them — so the leaderboard, analytics, insurance pool, and
reputation scoring all query cNFT mint history. The whole attestation
layer becomes a cNFT stream.

This is the **single most Solana-native primitive** in the design and
where to lean in hardest for the hackathon — judges love seeing cNFTs
used for something real.

**c. Achievement badge cNFTs**

Skills that hit milestones earn badges: "Battle-Tested" (1000 clean
invocations), "First Merge", "Zero Slashes", "Audit Approved". Customers
earn badges too: "100 Tasks Completed", "Early Adopter". Cheap to mint,
gamifies participation, creates a portable reputation trail that travels
with the wallet.

**d. Curator collection NFTs**

Power users mint curated "shelves" — bundles of skills vetted for a
specific domain ("Best DeFi skills", "Audited Anchor toolkit", "Rust
backend starter pack"). When a customer uses a skill via a curator's
shelf, a small curation fee flows to the collection NFT holder. This
bootstraps discovery without 5dlabs having to build algorithmic recs.

**e. Bundled agent NFTs (composite)**

An *agent* (the package: `AGENT.md` + skills + tools) is represented as
a parent NFT that references the child skill NFTs it depends on. When the
agent runs, settlement walks the bundle and splits royalties: X% to the
agent NFT holder (the packager), Y% split across the child skill NFT
holders (the authors). Metaplex has composable NFT patterns for this.

This matches the observation that agents are packages of skills, and lets
you monetize *both* layers — the author of each skill and the curator who
packaged them into a working agent.

### Architecture refactor: everything becomes NFT-shaped

Swap the plain-account design for NFT-backed accounts:

- `Skill` account → **Authorship NFT mint** (metadata points to author
  profile)
- `TaskReceipt` account → **cNFT mint tree** (each invocation is a leaf)
- Attestation record → **cNFT** minted by Tess / Cipher / Stitch wallets

Cleaner than the plain-account design because every Solana wallet, explorer,
and marketplace can display and trade the primitives without custom tooling.
UX win and a judge-pleasing visual story.

### Narrowed NFT scope for the hackathon demo

Keep the demo legible by using only **two NFT types**:

1. **Authorship NFT** per skill (regular NFT, transferable, title to
   royalties)
2. **cNFT invocation receipts** minted per task settlement

Demo loop:

- Mint an Authorship NFT for a new skill → show it in Phantom
- Run a task that invokes the skill → watch a cNFT receipt appear in the
  customer's wallet in real time
- Settlement splits USDC across Authorship NFT holder wallets → show SOL
  flowing live
- Pull up Magic Eden/Tensor → the Authorship NFT is listable for sale →
  someone else buys it → *they* start receiving royalties on the next task

That last step is the kill shot for the pitch: **you just sold a working,
revenue-generating AI skill as an NFT**. Nobody else at the hackathon is
showing that.

## Summary of the refined lead idea

Pulling all four follow-up threads back together:

> **A Solana-native open registry where AI agent skills are minted as
> Authorship NFTs (transferable title to the royalty stream), customers
> pay per task via pre-paid balance or session key, settlement is binary
> (pass or slash) via attestations from CTO's existing review agents,
> every invocation mints a cNFT receipt, and the leaderboard ranks skills
> by cumulative earnings per category with Matthew-effect protections.**

The open customer-side question is still: **devnet or mainnet-beta for the
demo, and which payment pattern (pre-paid balance vs. session key vs.
per-task signature) gets filmed for the judges.**
