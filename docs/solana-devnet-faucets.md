# Solana Devnet Faucet Sources

> Compiled 2026-04-14. Goal: accumulate maximum devnet SOL across all available sources.

## Strategy Summary

Hit all no-auth faucets first, then auth-gated ones, then CLI/RPC methods with private endpoints. Theoretical max from a single pass through all sources: **~35+ SOL** in one session, with more available daily.

---

## Tier 1: Best Bang for Buck (High SOL, Low Friction)

### 1. DevnetFaucet.org — **20 SOL/request**
- **URL:** https://devnetfaucet.org
- **Auth:** GitHub login
- **Rate limit:** Separate pool from public RPC (independently operated)
- **Balance:** ~50,812 SOL as of April 2026
- **Notes:** Most generous faucet. Operated by @Ferric, actively maintained (last commit March 2026)

### 2. Solana Foundation Official — **2-5 SOL/request**
- **URL:** https://faucet.solana.com
- **Auth:** Optional GitHub (30+ day old account, 1+ public repo). GitHub unlocks 5 SOL; anonymous gets 2 SOL
- **Rate limit:** 2 requests per 8 hours
- **Networks:** Devnet + Testnet
- **Notes:** Blocks AI agents explicitly. Human intervention required.

### 3. Jumpbit — **0.5-2 SOL/request**
- **URL:** https://jumpbit.io/en/solana/devnet-faucet
- **Auth:** None
- **Rate limit:** Per-wallet
- **Notes:** No signup, no GitHub. Options for 0.5, 1, or 2 SOL.

---

## Tier 2: Solid Sources (1 SOL each, various auth)

### 4. SolanaHub Dev Faucet — **1 SOL**
- **URL:** https://dev-faucet.solanahub.app
- **Auth:** None
- **Rate limit:** Per-wallet

### 5. Solfate — **1 SOL**
- **URL:** https://solfate.com/faucet
- **Auth:** None
- **Rate limit:** 1 per day
- **Networks:** Devnet + Testnet

### 6. Triangle Platform — **~1 SOL**
- **URL:** https://faucet.triangleplatform.com/solana/devnet
- **Auth:** None
- **Rate limit:** 1 per day
- **Notes:** Being acquired by Bridge/Stripe — may not last

### 7. Chainstack — **1 SOL**
- **URL:** https://faucet.chainstack.com
- **Auth:** Chainstack account (free tier) + API key + small mainnet SOL
- **Rate limit:** 1 per 24 hours

### 8. ZAN Faucet — **1-2 SOL**
- **URL:** https://zan.top/faucet/solana
- **Auth:** Free ZAN account + reCAPTCHA
- **Rate limit:** 1 SOL/day; +1 SOL bonus if you join their Discord and verify
- **Notes:** Discord verification for bonus SOL

### 9. Google Cloud Web3 Faucet — **small amount**
- **URL:** https://cloud.google.com/application/web3/faucet/solana/devnet
- **Auth:** Google Account + reCAPTCHA
- **Rate limit:** Per-address and per-account
- **Bonus:** Also provides PYUSD devnet tokens at `/faucet/solana/devnet/pyusd`

### 10. Helius Dashboard Faucet — **1 SOL**
- **URL:** https://dashboard.helius.dev/faucet
- **Auth:** Helius account (paid plan)
- **Rate limit:** 1 SOL per 24 hours
- **Notes:** We already have Helius keys for our RPC node

### 11. Alchemy Faucet — **varies**
- **URL:** https://www.alchemy.com/faucets/solana-devnet
- **Auth:** Alchemy account
- **Rate limit:** Tier-based

### 12. SPL Token Faucet — **1 SOL + SPL tokens**
- **URL:** https://www.spl-token-faucet.com
- **Auth:** None
- **Rate limit:** 1 per day
- **Notes:** Also dispenses test SPL tokens

---

## Tier 3: Lower Amounts / More Friction

### 13. QuickNode — **~0.1 SOL**
- **URL:** https://faucet.quicknode.com/solana/devnet
- **Auth:** Wallet with 0.05+ mainnet SOL (Phantom, Coinbase, MetaMask)
- **Rate limit:** 1 drip per 12 hours

### 14. SolFaucet — **2 SOL**
- **URL:** https://solfaucet.com
- **Auth:** None
- **Rate limit:** Per-IP, but wraps public RPC (shares rate limits)
- **Notes:** May fail during peak hours due to shared RPC pressure

---

## Tier 4: CLI / Programmatic Methods

### 15. Solana CLI via Private RPC (recommended)
```bash
# Use our Helius devnet key for separate rate limit pool
solana config set --url https://devnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY
solana airdrop 2 <WALLET_ADDRESS>
```
- **Amount:** Up to 2 SOL per request
- **Rate limit:** Based on Helius plan (separate from public RPC)

### 16. Solana CLI via Public RPC
```bash
solana config set --url devnet
solana airdrop 2 <WALLET_ADDRESS>
```
- **Amount:** Up to 2 SOL per request
- **Rate limit:** Heavily rate-limited during peak hours

### 17. web3.js Programmatic
```typescript
import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
const connection = new Connection("https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
await connection.requestAirdrop(new PublicKey("ADDRESS"), 2 * LAMPORTS_PER_SOL);
```

### 18. Proof-of-Work Faucet (devnet-pow) — **unlimited, CPU-bound**
```bash
cargo install devnet-pow
devnet-pow mine
# Advanced:
devnet-pow mine -d 3 --reward 0.02 --no-infer -t 5000000000
```
- **Amount:** ~0.02 SOL per solved proof
- **Rate limit:** None — limited only by CPU
- **Notes:** Recommended by faucet.solana.com for AI agents and automation. Run overnight for passive accumulation.

---

## Tier 5: Discord Bots

### 19. The 76 Devs
- **Discord:** https://discord.gg/RrChGyDeRv
- **Command:** `!gibsol` in bot-commands channel

### 20. LamportDAO
- **Discord:** https://discord.gg/JBVrJgtFkq
- **Command:** `/drop <address> <amount>` in bot-commands channel

---

## Tier 6: Local Validator (Unlimited, Offline)
```bash
solana-test-validator              # run in separate terminal
solana config set --url localhost
solana airdrop 1000                # no limits at all
```
- Good for development; resets on restart; not on actual devnet

---

## Tier 7: Paid/Swap (Last Resort)

### solana-devnet-faucet.com
- **URL:** https://solana-devnet-faucet.com
- **Model:** Pay 0.2 mainnet SOL -> receive 10 devnet SOL
- **Amounts:** 10, 20, 50, 100, 1000 devnet SOL available
- **Warning:** Verify legitimacy before sending mainnet SOL

---

## Dead / Inactive Sources (Skip These)
- **omnifaucet.com** — Domain for sale
- **stakely.io/faucet/solana-sol** — Inactive, zero balance, no activity for 2 years
- **Blockdaemon** (solana-faucet.blockdaemon.com) — Testnet only, not devnet

---

## Recommended Execution Order

1. **DevnetFaucet.org** (20 SOL) — GitHub login, biggest single payout
2. **faucet.solana.com** (5 SOL with GitHub) — official, 2 requests
3. **Jumpbit** (2 SOL) — no auth needed
4. **SolFaucet** (2 SOL) — no auth, if RPC not congested
5. **SolanaHub** (1 SOL) — no auth
6. **Solfate** (1 SOL) — no auth
7. **Triangle** (1 SOL) — no auth
8. **SPL Token Faucet** (1 SOL) — no auth
9. **Google Cloud** (small amount) — Google account
10. **ZAN** (1-2 SOL) — free account
11. **Chainstack** (1 SOL) — account needed
12. **Helius dashboard** (1 SOL) — we have keys
13. **CLI via Helius RPC** (2 SOL per request) — script and loop
14. **devnet-pow** — run in background for passive accumulation
15. **Discord bots** — if more needed

**Estimated first-pass total: ~35-40 SOL**
**Daily recurring: ~15-20 SOL/day** from sources with 24h cooldowns
