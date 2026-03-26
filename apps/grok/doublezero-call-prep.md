# DoubleZero Edge Beta Call Prep — 2026-03-19

## TL;DR for the Call

You've been invited to the **DZ Edge beta programme**. DoubleZero is a dedicated fiber-optic "N1" (Network Layer 1) sitting beneath Solana at OSI layers 1-3. DZ Edge is their new multicast shred feed for traders — the same tech NYSE/CME use. **You still need Jito ShredStream** — they're complementary at different layers. Cherry Servers (your host) is already a DZ bandwidth contributor, so you may be one cross-connect away.

**Your angle**: You're running bare metal Agave RPC on Cherry Servers for a trading operation, you understand the whitepaper architecture, and you want to be an early Edge subscriber because multicast shred delivery eliminates the stake-weighted Turbine advantage that currently favors large validators like Helius (14M SOL).

---

## What You Need to Know Cold (Whitepaper Deep Cuts)

### The Two-Ring Architecture (from the whitepaper)

This is the core innovation — show you understand this:

- **Outer Ring (Filter Ring)**: FPGA appliances at network edge ingress points. They do three things at line rate with zero added latency:
  1. Signature verification
  2. Deduplication
  3. Spam/DDoS filtering
  - Key insight from the paper: "One sample deployment alone can handle deduplication, filtering, and signature verification for multiple Gbps of inbound data"
  - This is **cooperative infrastructure sharing** — instead of every validator provisioning enough resources to handle global spam, the DZ filter ring does it once for all downstream validators

- **Inner Ring (Data Flow Ring)**: Dedicated fiber links between DZX (DoubleZero Exchange) points. These are the metro interconnects that stitch data centers together within a city, plus long-haul dedicated fiber between metros. Supports **multicast** for one-to-many block/shred propagation.

### The x86 Analogy (Great for Conversation)

The whitepaper ends with a killer analogy: a modern x86 processor is actually a distributed system — 24 cores connected by an on-chip network. The bottleneck is never the cores, it's the data movement between them. **DoubleZero is that on-chip interconnect, but for the global validator set.**

> "Doubling the cores does not yield double the performance unless the on-chip interconnect, memory controllers, and other data flow systems can make similar gains in efficiency."

### Why This Matters for RPC Nodes Specifically (Whitepaper Section)

The whitepaper has a dedicated RPC section. Three challenges DZ addresses:
1. **Inbound surge protection**: RPCs are directly exposed to traffic surges (airdrops, mints). FPGA filter ring shields them.
2. **Transaction deliverability**: Your outbound trades reach leaders faster over dedicated fiber. "Transaction deliverability matters immensely for an RPC, especially since the highest-value transactions tend to be both competitive and time-sensitive."
3. **State freshness**: Faster shred/block delivery from validators back to your RPC = fresher state for your trading bots.

---

## DoubleZero Edge — The Product You're Joining

### What It Is
A **multicast market data feed** for Solana shreds. Launched beta late February 2026. Traders subscribe to receive raw, unmodified Solana shreds delivered simultaneously via multicast — the exact same technology traditional exchanges (NYSE, CME) have used for decades.

### How It Works
1. **Validators publish** shreds into a multicast group on the DZ inner ring
2. **DZ network replicates** packets in-flight at points of path divergence (not at the sender)
3. **All subscribers receive** the consolidated feed from multiple validators simultaneously

### Why This Is Revolutionary
**Turbine (current Solana propagation)** is unicast and stake-weighted:
- Leader sends to root node, which propagates layer by layer
- Top ~200 validators by stake receive shreds first
- Low-stake nodes (like your RPC) are at the bottom of the tree
- The tree "assumes negligible network latency" and ignores geography

**DZ Edge multicast** breaks this:
- Single transmission reaches ALL subscribers at the same time
- **Delivery independent of stake weight** — your Cherry Servers node gets shreds at the same time as Helius (14M SOL)
- Reduced variance, flatter tail latency
- Network handles replication, not the sender

### Current Scale
- 100+ validators publishing shreds (since late February beta)
- 359 validators connected to DZ = 22.5% of Solana stake ($22B)
- 70+ dedicated fiber links across 25+ global locations

### Revenue Model
- Traders pay in **2Z token** (pricing TBD — ask about this)
- Revenue split: 10% burned / 32.5% to validators / 17.5% to client teams (Agave, Jito, Firedancer, Harmonic) / 50% to DZ network contributors
- Validators earn proportional share of subscriber fees for publishing shreds

---

## The Key Question: DZ Edge vs Jito ShredStream

**You need BOTH. They are complementary, not competitive.**

| Aspect | DoubleZero (N1) | Jito ShredStream (L7) |
|--------|----------------|----------------------|
| **OSI Layer** | L1-L3 (Physical/Network) | L7 (Application) |
| **What it does** | Dedicated fiber transport + FPGA spam filtering + multicast | Fast shred distribution via Block Engine |
| **Delivery method** | Multicast over private fiber | Unicast UDP from Block Engines |
| **Scope** | ALL network traffic (consensus, txns, shreds, votes) | Specifically shreds from leaders |
| **Stake dependency** | None — multicast is simultaneous | Bypasses Turbine tree (stake-independent) |
| **How it helps trading** | Faster pipe for everything + spam filtering + multicast shreds | 50-200ms earlier shreds vs standard Turbine |
| **Integration** | One CLI command + config flag + restart | Separate sidecar proxy container |
| **Cost** | Zero for validators (Epoch 939+); Edge uses 2Z token | Free (extended beta) |

### Why you want both:
1. **DZ Edge** gives you multicast shred delivery over dedicated fiber — simultaneous delivery to all subscribers regardless of stake
2. **ShredStream** gives you a separate fast-path specifically optimized for shreds from the current leader via Jito Block Engines
3. **Combined**: Two independent fast paths for shreds = redundancy + whichever is faster for a given slot wins
4. Jito's Block Engine itself is integrated into DZ — so ShredStream already benefits from DZ transport
5. **DZ improves outbound too**: Your trade transactions reach leaders faster over dedicated fiber — ShredStream is inbound only

### What DZ does that ShredStream doesn't:
- FPGA spam filtering at edge (~70% spam reduction before it hits your node)
- Improves consensus/vote latency (whole network benefits)
- Improves outbound transaction submission (your trades land faster)
- Deterministic latency — no public internet jitter
- Geographic expansion makes running validators in non-EU locations viable

### What ShredStream does that DZ Edge doesn't:
- Dedicated application-level fast-path optimized specifically for leader shreds
- gRPC interface for decoded transactions from shreds (for trading bots)
- Works without any hardware/network changes
- Proven track record (longer in production)

---

## Geographic Strategy — Where to Host Nodes

### Your Current Setup
- Agave RPC on Cherry Servers bare metal (solana-rpc-01, EU)
- Cherry Servers is a **DZ bandwidth contributor** with DZ hardware in their EU data centers

### DZDP Three-Ring Model (Phase II, started March 9, 2026)

| Ring | Regions | Delegation Incentive | Your Relevance |
|------|---------|---------------------|----------------|
| **Ring 1** (Core) | Frankfurt, Amsterdam, London, etc. (EU high-stake zone) | Baseline — Phase I continues | **You are here** (Cherry EU) |
| **Ring 2** (Expansion) | United States, **Canada**, India | Moderate bonus | Canada is your home base |
| **Ring 3** (Priority Edge) | Hong Kong, Sao Paulo, Singapore, Tokyo | Maximum bonus (600K SOL per region) | Biggest delegation incentive |

### 10 Priority Expansion Locations
Abu Dhabi, Buenos Aires, Dubai, Hong Kong, Johannesburg, Mumbai, Sao Paulo, Shanghai, Singapore, Tokyo

### What This Means for You
- **EU (Ring 1)**: Best for trading latency because most Solana validators are in EU. Your Cherry Servers box is already in the densest part of the network. Keep your primary RPC here.
- **Ring 2/3 opportunity**: If you run a validator (not just RPC), Ring 3 cities get 600K SOL delegation per region. Singapore or Tokyo could be interesting for Asia-hours trading + delegation income.
- **Canada (Ring 2)**: Gets moderate delegation bonus. Could be relevant if you run infrastructure from home base.

### Best Geographic Strategy for Trading
1. **Primary RPC**: Keep in EU (Cherry Servers) — closest to validator density, DZ hardware already present
2. **Edge consideration**: If DZ expands to have strong NA presence, a US East Coast node could be valuable for faster transaction submission to US-based validators
3. **Validator play**: If you want delegation income, a Ring 3 city (Singapore/Tokyo) gets the fattest delegation bonus

---

## Pricing & Economics

### For Validators (DZDP Requirements):
- Network fees **dropped to zero** as of Epoch 939
- Commission rate below 10%
- Minimum 10,000 delegated SOL for past 20 epochs
- No sandwiching of user trades
- Must maintain strong vote and performance metrics
- 13M SOL delegation pool (expanded from 3M in Oct 2025)

### For Traders (Edge):
- Paid in **2Z token**
- 2Z currently ~$0.07-0.08, FDV ~$717M
- **No published Edge pricing yet — THIS IS YOUR #1 QUESTION**
- Revenue split: 10% burn / 32.5% validators / 17.5% client teams / 50% DZ contributors

### 2Z Token:
- Listed on Coinbase, Binance, OKX, Bybit, Kraken
- Grayscale "Assets Under Consideration" (Jan 2026)
- Total supply: 10B, raised ~$39M ($28M private @ $400M valuation + $11.25M public)

---

## Talking Points — Showing You Know Your Stuff

### Open with your context:
> "We're running an Agave RPC node on Cherry Servers bare metal for our trading operation. I understand Cherry is already a DZ bandwidth contributor. We've been invited to the Edge beta and I'm particularly interested in the multicast shred feed — the stake-independent delivery model is exactly what levels the playing field for smaller operations like ours against the Helius-scale validators that sit at the top of the Turbine tree."

### Technical questions that show depth:

1. **"The whitepaper describes DZX exchange points for metro interconnects. Is Cherry Servers connected via a direct cross-connect to a DZX, or does our traffic still traverse a GRE tunnel?"**
   - (Shows you read the whitepaper, understand the DZX concept, and know about the GRE overhead issue Chorus One flagged)

2. **"For Edge, the multicast replication happens at path divergence points on the inner ring. What's the measured latency delta between a DZ-connected subscriber and a non-connected one for the same leader slot?"**
   - (Shows you understand how multicast differs from unicast and want real numbers)

3. **"The whitepaper's filter ring section mentions FPGA appliances doing sig verification + dedup at line rate. What's the current throughput capacity per appliance, and does the filter ring benefit my RPC's inbound transaction flow or only validator gossip traffic?"**
   - (Shows you understand the two-ring model and want to know if your RPC benefits from the filter ring)

4. **"With 22.5% of stake on DZ, roughly 77.5% of leader slots don't have a DZ-connected leader. For Edge multicast, does the feed only carry shreds from DZ-connected leaders, or do you aggregate from non-connected leaders too?"**
   - (Critical business question — if most leaders aren't on DZ, the multicast feed has gaps)

5. **"We're stacking DZ Edge with Jito ShredStream. I see Jito's Block Engine is integrated into DZ. Does that mean ShredStream data already flows over DZ fiber, or are they separate data paths?"**
   - (Shows you understand the layering and are building a proper stack)

6. **"What's the Edge pricing model? Per-stream, per-byte, flat rate in 2Z? And is there a beta discount or free tier for early participants?"**
   - (Direct business question — you need to know this)

### Strategic questions:

7. **"The DZDP Phase II Ring 3 locations (Singapore, Tokyo, HK, Sao Paulo) each get 600K SOL delegation. If we ran a validator in Singapore alongside our EU RPC, what are the hardware requirements and is there a waitlist?"**

8. **"For our trading operation, the outbound side matters as much as inbound. Does DZ improve our transaction submission latency to the current leader, or is Edge purely a read/subscribe product?"**

9. **"What's the adoption roadmap to 50%+ of stake? At what threshold does DZ become the de facto transport layer rather than an optimization?"**

### Power question (whitepaper callback):

10. **"The whitepaper makes the x86 interconnect analogy — the bottleneck is data movement, not compute. With Firedancer pushing towards 1M TPS, does DZ's current inner ring bandwidth support that throughput, or will the N1 need to scale too?"**

---

## What to Listen For

### Green Flags:
- Specific latency benchmarks for Cherry Servers cross-connect
- Clear Edge pricing with competitive unit economics
- "Yes, the filter ring benefits RPCs too, not just validators"
- Roadmap to 50%+ stake coverage with concrete milestones
- Acknowledgment that DZ + ShredStream is the optimal combo
- Technical team available for integration support

### Red Flags:
- Vague latency claims without data
- Trying to position Edge as a ShredStream replacement
- High upfront 2Z token commitment
- No clear SLA or public internet failover story
- "We're working on it" for filter ring / outbound transaction benefits

---

## Quick Reference: Your Full Low-Latency Stack

| Layer | Solution | Status | Priority |
|-------|----------|--------|----------|
| **Bare metal** | Cherry Servers, 64 cores, 755 GiB, NVMe | Done | -- |
| **Network (N1)** | DoubleZero dedicated fiber | **Today's call** | HIGH |
| **Shred feed (multicast)** | DZ Edge (beta) | **Today's call** | HIGH |
| **Shred feed (unicast)** | Jito ShredStream proxy | Key submitted, awaiting approval | HIGH |
| **State streaming** | Yellowstone gRPC geyser | Building (ABI fix in progress) | HIGH |
| **Tx submission** | Jito Bundles + staked connections | Partially set up | MEDIUM |
| **MEV protection** | Jito bundles (atomic execution) | Partially set up | MEDIUM |
| **Priority access** | Staked connections (SWQoS) | Need stake delegation | MEDIUM |

---

## Key People

- **Austin Federa** — President, DoubleZero Foundation (ex-Solana Foundation Head of Strategy)
- **Andrew McConnell** — CTO, Malbec Labs (core contributor, ex-Jump Trading fiber networks)
- **Mateo Ward** — co-author of whitepaper, Malbec Labs
- **Tom Warner** — Head of Blockchain BD (likely your call contact)
- **Backers**: Dragonfly, Multicoin, Wintermute, Jump Crypto, Anatoly Yakovenko, Raj Gokal, Mert Mumtaz, Lucas Bruder (Jito CEO)

---

## Appendix: Jito ShredStream Setup (Parallel Track)

Your Agave identity public key: `6d85L6Qbbh95v6PJu6FB1mVmdxRz6qmKPM6oR9hXJf1A`

Submit at: https://web.miniextensions.com/WV3gZjFwqNqITsMufIBs
- Use case: "RPC node for trading operation, Cherry Servers EU bare metal"
- Best regions: amsterdam, frankfurt
- Bandwidth: ~4 MiB/s (32 Mbit/s) — negligible on 25 Gbps NIC
- Once approved, runs as a sidecar container on your Agave pod

---

## Coles Notes: Terminology & Concepts You Need to Know

### What's Actually in a Shred?

A **shred** is a fragment of a Solana block. Solana doesn't wait until the entire block is built before sending it out — the leader starts streaming pieces (shreds) as it's producing the block. Think of it like watching a video as it buffers rather than waiting for the whole file to download.

- A block is split into **data shreds** (the actual transaction data) and **coding shreds** (error-correction redundancy, like parity bits)
- Each shred is about **1228 bytes** — small enough to fit in a single UDP packet
- A typical slot (block) produces **~64 data shreds + ~32 coding shreds**
- You only need ~64 out of those ~96 total shreds to reconstruct the full block (thanks to Reed-Solomon erasure coding)
- Shreds contain: the slot number, a shred index, the leader's signature, and a chunk of serialized transaction data
- **They are NOT transactions** — they're raw byte fragments. You need to collect enough shreds, reassemble them, then deserialize to get actual transactions

**Why shreds matter for trading**: Getting shreds 50-200ms before your competitor means you see new transactions (swaps, liquidations, oracle updates) before they do. You can react before the block is even fully confirmed.

### Shreds vs RPC Calls

| | Shred Stream | RPC Call (getBlock, getTransaction) |
|--|-------------|-------------------------------------|
| **When you get data** | As the block is being produced (real-time) | After the block is confirmed (~400ms+ later) |
| **Format** | Raw binary fragments — you reassemble | Fully structured JSON response |
| **Latency** | Lowest possible (leader streams as it builds) | Adds network round-trip + RPC processing |
| **Completeness** | Partial — you may miss some shreds | Complete — full block/tx data |
| **Use case** | Ultra-low-latency trading signals | General-purpose queries, historical data |
| **Effort** | You need deshredding code | Just call the API |

### Stake-Weighted Propagation (Turbine)

This is the core thing DZ Edge disrupts. Here's how Solana normally distributes shreds:

**Turbine** is Solana's block propagation protocol. When the leader produces shreds, it doesn't blast them to all ~3,000 validators at once (that would require insane bandwidth). Instead, it uses a **tree structure**:

1. Leader sends shreds to a small set of **root nodes** (first layer)
2. Those root nodes forward to the next layer
3. Those forward to the next, and so on
4. ~6 layers deep, everyone has the shreds

**The catch**: Your position in the tree is determined by your **stake weight**. Validators with more SOL staked get placed closer to the root (layer 1-2). Low-stake validators and RPC nodes (like yours) end up at layer 5-6.

**What this means in practice**:
- Helius (14M SOL staked) → Layer 1 → gets shreds in ~5ms
- Random 1,000 SOL validator → Layer 4-5 → gets shreds in ~150-300ms
- Your RPC node (no stake) → Bottom of tree → gets shreds last

**This is a structural speed advantage you cannot buy your way out of** without massive capital for stake. Unless you use DZ Edge multicast, which delivers to everyone simultaneously regardless of stake.

### Multicast vs Unicast

**Unicast** (how the internet normally works): One sender, one receiver. If you want to send to 100 people, you send 100 separate copies. Each copy goes over its own path. This is how Turbine works — each node forwards to a few specific peers.

**Multicast** (what DZ Edge uses): One sender, many receivers simultaneously. The sender transmits **once**, and the network infrastructure (switches/routers) replicates the packet at branch points. All subscribers receive at effectively the same time.

```
UNICAST (Turbine):                    MULTICAST (DZ Edge):

Leader → Node A → Node C             Leader → [DZ Network] → All subscribers
       → Node B → Node D                       simultaneously
              → Node E
                                      Everyone gets it at ~same time
Nodes C,D,E are 3-4 hops delayed     regardless of stake
```

**Why traditional finance uses multicast**: NYSE, CME, NASDAQ all distribute market data via multicast. When a trade happens, ALL subscribers see it at the same time. Nobody gets a structural head start from their "position in the tree." DZ Edge brings this same fairness model to Solana shred delivery.

### UDP (User Datagram Protocol)

Shreds are sent over **UDP**, not TCP. Key differences:

- **UDP**: Fire-and-forget. No handshake, no acknowledgment, no retransmission if a packet is lost. Fastest possible delivery. If a shred packet gets dropped, it's just gone.
- **TCP**: Reliable delivery with handshakes, acknowledgments, retransmissions. Slower but guaranteed delivery. Used for RPC calls, HTTP, etc.

**Why UDP for shreds**: Speed > reliability. If you miss a few shreds, the erasure coding (those extra coding shreds) lets you reconstruct the block anyway. Waiting for TCP retransmission would defeat the purpose of getting data as fast as possible.

**How UDP multicast works**: The DZ inner ring network is a single network domain (like a giant LAN). Multicast works by having switches replicate UDP packets at junction points. The sender puts one packet on the wire, and each switch that has subscribers downstream copies the packet to the right ports. No extra work for the sender.

### FPGA (Field-Programmable Gate Array)

FPGAs are special chips that sit between general-purpose CPUs and custom ASICs:
- **CPU**: Flexible, can run any software. Relatively slow at specific repetitive tasks.
- **FPGA**: Programmable hardware. You "burn" specific logic into the chip. Much faster than CPU for specific tasks (like signature verification), but only does what you programmed.
- **ASIC**: Hardcoded silicon. Fastest possible, but can't be changed.

DZ uses FPGAs in the outer (filter) ring to verify Ed25519 signatures on transactions at **line rate** — meaning they process as fast as data arrives on the wire. A single FPGA appliance can handle multiple Gbps of transaction verification, dedup, and spam filtering. A CPU would choke on the same volume.

**Why you care**: The filter ring removes ~70% of spam before it reaches your Agave node, freeing up CPU cycles that would otherwise be wasted on verifying garbage transactions.

### DZX (DoubleZero Exchange Points)

Think of these like internet exchange points (IXPs) but for the DZ network. They're metro-level interconnection hubs where:
- Multiple data centers within a city connect
- Long-haul fiber links terminate
- Local validators/RPCs connect to the DZ inner ring

If Cherry Servers has a DZX in their data center (or a cross-connect to one), your Agave node can plug directly into the DZ inner ring without traversing the public internet at all.

### GRE Tunnel

**GRE (Generic Routing Encapsulation)**: A way to create a virtual point-to-point link over an existing network. DZ uses GRE tunnels to connect validators to the DZ network when a direct cross-connect isn't available.

**The downside**: GRE adds overhead (~24 bytes per packet) and can slightly increase latency on very short routes. Chorus One's research showed that for validators already very close to each other, the GRE tunnel overhead could actually make DZ slightly slower than direct public internet. For longer routes, DZ's dedicated fiber easily wins.

**Your question for the call**: "Is our Cherry Servers node connected via direct cross-connect or GRE tunnel?" Direct = best case. GRE = still good but not optimal.

### IBRL Mode

**IBRL (Internet-Based Routing Layer)**: The baseline DZ connection mode. Traffic routes over DZ fiber when available, with automatic failover to public internet when it's not. This means you never lose connectivity — if a DZ link goes down, you seamlessly fall back to regular internet routing.

### SWQoS (Stake-Weighted Quality of Service)

A Solana mechanism where validators preferentially accept transactions from peers who have staked SOL. In practice:
- Validators reserve ~80% of their transaction processing capacity for staked connections
- Only ~20% is available for unstaked/general traffic
- This means your transactions land faster if you have stake

**Relevance**: Even with DZ making your pipe faster, your transaction submission still competes for validator attention. SWQoS + DZ together = fast pipe AND priority access.

### N1 (Network Layer 1)

DZ's branding for what they are. Just like blockchains are "L1s" (Layer 1 application protocols), DZ calls itself "N1" — the base network transport layer that sits beneath all the L1s, L2s, and applications. It's the physical fiber + routing + filtering infrastructure everything else runs on top of.

### Turbine Tree Position

When someone says "you're low in the Turbine tree," they mean your node receives shreds late because of low/no stake. Tree position = priority in the shred delivery chain. DZ Edge's whole pitch is making tree position irrelevant by using multicast instead.

### Erasure Coding (Reed-Solomon)

The reason you don't need ALL shreds to reconstruct a block. The leader generates extra "coding shreds" alongside the data shreds. If you receive any 64 out of 96 total shreds (data + coding), you can mathematically reconstruct the complete block. This is the same math used in RAID storage and QR codes.

**Practical impact**: Even if your network drops 30% of shred packets, you still get the complete block. This makes UDP delivery viable — you don't need TCP's reliability guarantees because the erasure coding provides its own redundancy.

### Jitter

The **variation** in latency over time. Not the same as latency itself.
- Public internet: latency might average 15ms but swing between 8ms and 45ms (high jitter)
- DZ dedicated fiber: latency might average 12ms and stay between 11ms and 13ms (low jitter)

**Why jitter matters more than raw latency for trading**: If your latency is consistently 12ms, you can model and account for it. If it swings wildly between 8ms and 45ms, your trading signals become unreliable. DZ's dedicated fiber eliminates the jitter from shared public internet congestion.

### Block Leader / Leader Schedule

Solana rotates which validator produces blocks. Each validator gets assigned 4 consecutive slots (~1.6 seconds) when they're the "leader." The schedule is deterministic and known in advance (derived from the stake distribution at the start of each epoch).

**Why this matters**: If you know the next leader is a DZ-connected validator, your shreds from that leader will come over DZ fiber. If the leader isn't on DZ (~77.5% of slots currently), those shreds come over regular internet. This is the coverage gap you should ask about on the call.

### Geyser Plugin

An Agave validator plugin interface that streams real-time data (accounts, transactions, slots, blocks) to external consumers. Yellowstone gRPC is the most popular geyser plugin — it's what you're setting up on your node. Geyser gives you structured, programmatic access to state changes as they happen, whereas raw shreds give you unstructured byte fragments.

**The stack**: Shreds (fastest, raw) → Geyser/Yellowstone gRPC (fast, structured) → RPC calls (slowest, fully processed)

---

## Sources

- [DoubleZero Whitepaper](https://doublezero.xyz/whitepaper.pdf) (Dec 2024)
- [DZ Edge Product Page](https://doublezero.xyz/dz-edge)
- [DZDP Phase II Announcement](https://doublezero.xyz/journal/phase-2-of-dzdp)
- [Extending Solana's Network Performance to the Edge](https://doublezero.xyz/journal/extending-solanas-network-performance-to-the-edge)
- [What Does DoubleZero Edge Mean for Solana Validators?](https://solanafloor.com/news/what-does-double-zero-edge-mean-for-solana-validators)
- [What Is DoubleZero? Solana's New Fiber Backbone](https://blocksize.info/blog/what-is-doublezero-jito/)
- [Jito ShredStream Docs](https://docs.jito.wtf/lowlatencytxnfeed/)
- [Solana Trading Infrastructure 2026](https://chainstack.com/solana-trading-infrastructure-2026/)
