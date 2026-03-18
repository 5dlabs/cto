# Verified Trader Research — March 15, 2026

Research conducted via Grok X search (x_search API), Polymarket live leaderboard, Tavily/Exa web search, and on-chain analyst tracking. All wallet addresses are publicly sourced from on-chain data and analyst posts on X.

---

## Moon Dev Assessment (Why We're Not Following Him)

**Handle**: @MoonDevOnYT | **Location**: San Juan, Puerto Rico | **Site**: moondev.com / algotradecamp.com

Moon Dev is a **course seller, not a trader**. Key findings:

- Sells: Algo Trade Camp ($69/mo or $420 lifetime), Solana Sniper Course ($97), Polymarket Bot Course ($1,950), $100/Day AI Challenge ($7/mo), "Get Funded" program ($250 on Hyperliquid)
- **No public wallet address** — never shared one despite building multiple trading bot repos
- YouTube titles like "71,057% ROI" are backtest numbers, not live results
- His own GitHub disclaimers say: "NO guarantees of profitability" and "This is experimental research, not a profitable trading solution"
- "Get Funded" program caps US traders at $599/year — not real capital
- Classic info-product funnel: free starter pack → cheap challenge → monthly sub → lifetime → high-ticket courses
- His repos (moon-dev-ai-agents, Trading-Algos) are real code and educational value is legit — but no evidence of personal trading profits
- **Verdict**: Good teacher, unverified trader. The money comes from courses, not trading.

---

## Polymarket — Current Monthly Leaders (Live March 2026)

All wallet addresses verified on-chain via Polymarket leaderboard.

### Top Leaderboard Traders

| Rank | Handle | Monthly Profit | Volume | Wallet | Strategy Notes |
|------|--------|---------------|--------|--------|----------------|
| 1 | HorizonSplendidView | +$4,016,108 | $12.4M | `0x02227b8f5a9636e895607edd3185ed6ee5598ff7` | High conviction directional |
| 2 | beachboy4 | +$3,061,230 | $9.6M | `0xc2e7800b5af46e6093872b177b7a5e7f0563be51` | Wild swings: -$7.56M drawdown then +$6.4M recovery. Confirmed via Grok X search. |
| 3 | majorexploiter | +$2,416,975 | $6.9M | `0x019782cab5d844f02bafb71f512758be78579f3c` | Unknown |
| 4 | CemeterySun | +$2,175,661 | $23.7M | `0x37c1874a60d348903594a96703e0507c518fc53a` | High volume — likely arb/MM |
| 5 | Countryside | +$1,877,564 | $17.5M | `0xbddf61af533ff524d27154e589d2d7a81510c684` | Unknown |
| 6 | 0p0jogggg | +$1,300,247 | $41.9M | `0x6ac5bb06a9eb05641fd5e82640268b92f3ab4b6e` | Very high volume:profit ratio — likely arb/MM |
| 7 | gatorr | +$1,190,506 | $9.75M | `0x93abbc022ce98d6f45d4444b594791cc4b7a9723` | Unknown |
| 8 | swisstony | +$780,590 | $90.5M | `0x204f72f35326db932158cba6adff0b9a1da95e14` | Almost certainly arb/MM — $90M volume on $780K profit = spread capture |
| 9 | GamblingIsAllYouNeed | +$586,868 | $28.7M | `0x507e52ef684ca2dd91f90a9d26d149dd3288beae` | Unknown |
| 10 | WoofMaster | +$571,305 | $1.55M | `0x916f7165c2c836aba22edb6453cdbb5f3ea253ba` | Low volume, high profit — directional |
| 11 | HedgeMaster88 | +$558,605 | $5.95M | `0x036c159d5a348058a81066a76b89f35926d4178d` | Unknown |
| 12 | anoin123 | +$527,158 | $11.1M | `0x96489abcb9f583d6835c8ef95ffc923d05a86825` | Unknown |
| 13 | huhaoli | +$514,770 | $8.9M | `0xf19d7d88cf362110027dcd64750fdd209a04276f` | Unknown |
| 14 | SecondWindCapital | +$501,524 | $1.28M | `0x8c80d213c0cbad777d06ee3f58f6ca4bc03102c3` | Very low volume, high profit — strong directional edge |

### gabagool22 — Deep Dive (Most Studied Polymarket Trader)

- **Wallet**: `0x6031b6eed1c97e853c6e0f03ad3ce3529351f96d`
- **PnL**: $868,863 | **Volume**: $144.6M | **Win Rate**: 99.52% | **Markets Traded**: 24,525
- **Alias**: Also trades as `distinct-baguette`
- **Website**: gabagool22.com (sells the bot — 44K lines of Rust, $500K+ verified profit)

**Strategy (fully documented via 0xInsider case study and Reddit reverse-engineering)**:
1. **Spread Capture (Arb)**: Buys both UP + DOWN tokens when combined bid < $1.00 (e.g., UP $0.48 + DOWN $0.49 = $0.97 → guaranteed $0.03 profit at resolution)
2. **Momentum**: Uses real-time Binance aggTrade WebSocket to detect price moves, enters Polymarket position before the market reprices (latency arb)
3. **Market Making**: Posts two-sided limit orders, uses Binance feed for preemptive cancels to avoid toxic fills

**Operational Details**:
- Trades BTC and ETH UP/DOWN binary markets exclusively (5min, 15min, 60min, 4hr windows)
- 365 trades per market average, 64 minutes average hold time
- 24/7 automated, near-uniform hourly distribution
- ~1,892 Yes shares and ~1,878 No shares per market (near-equal = market making)
- Automatic on-chain merging via ProxyWallet Factory to recover capital

**Relevance**: This strategy is directly applicable to our Polymarket info-arb work. The spread capture approach is complementary to our news-driven signal engine.

### Anonymous Polymarket Whales (from @0x_Discover on X)

| Wallet | PnL | Win Rate | Notes |
|--------|-----|----------|-------|
| `0x9b979a065641e8cfde3022a30ed2d9415cf55e12` | $296K | 96% | High frequency bonding |
| `0x12d6cccfc7470a3f4bafc53599a4779cbf2cf2a8` | $213K | 84% | High limit-order volume |
| `0x8278252ebbf354eca8ce316e680a0eaf02859464` | $23K | 73% | Solid positioning |
| `0xd8f8c13644ea84d62e1ec88c5d1215e436eb0f11` | $64K | 34% | Longshot weather trader — low win rate but profitable |

---

## Hyperliquid — Verified Whale Traders (Active March 2026)

All addresses sourced from @lookonchain posts on X via Grok x_search API. Hyperliquid is fully on-chain — all PnL is publicly verifiable.

### Confirmed Profitable (Realized PnL)

| Wallet | Confirmed PnL | Positions | Style |
|--------|--------------|-----------|-------|
| `0x15a4F009BB324A3fb9E36137136B201E3Fe0DFDb` | **+$1.88M** (closed BTC long) | Was: 20x long $42.5M BTC + 20x long $41.2M ETH. Then withdrew all, switched to spot ETH. | Disciplined — takes profit, rotates. **Best copy target.** |
| `0xefe263da9c803d449a572e8d126cbdab306cc147` | **+$1.5M est** | Long $4.06M xyz:CL + $3.67M xyz:BRENTOIL | Commodity perps |
| `0x6f90...336a` | **+$1.5M** (from $6,800 start) | Non-directional — no leverage-based directional bets | Legendary risk-adjusted returns |

### Active Whales This Week (March 8-15, 2026)

| Wallet | Positions | PnL | Notes |
|--------|-----------|-----|-------|
| `0x54d796f92c566372ca09d8d22868d675935b4b98` | Short 10,641 ETH ($51.2M), short WLFI, XPL | Unknown | Massive directional bear |
| `0x8d0e342e0524392d035fb37461c6f5813ff59244` | 4,022 ETH spot ($11.19M), 20x long ETH ($5.66M), 10x long BCH ($908K) | Active | Spot + leverage mix |
| `0x985f02b19dbc062e565c981aac5614baf2cf501f` | $35M oil shorts (CL + BRENTOIL), short HYPE/PUMP/XPL/APT | -$1.87M unrealized | "Oil Bear" — keeps adding capital, conviction bear. Deposited another $4M USDC March 14. |
| `0x3ed4033676d0bdb3938728ca4ac673d00e74bd06` | 20x long 113,080 xyz:CL ($11.52M), liq price $88.4 | Unknown | Returned after 2 months inactivity |
| `0xF780ADB6CA4A737D2A8dEF6DD445b0C0276d70e1` | Short 90,000 xyz:CL ($8.55M), liq price $147.94 | Unknown | Deposited 5.6M USDC for position |
| `0x75088332da14c7c729d79af11436b01268513035` | 40x BTC, 20x SOL, 10x FARTCOIN, 5x PUMP, 10x PEPE, 3x LAUNCHCOIN | Unknown | $5M deposited. Extremely aggressive multi-position. |
| `0x7e4e766d0ae5ea9cded0c694669194db92800107` | Bought 86,322 HYPE ($4.88M) + 466.68M PUMP ($2.74M) | Unknown | HYPE accumulator |
| `0x1abd8168ea60b37a6a8cc149fa058910d89b0767` | Bought HYPE with 4.58M USDC | Unknown | HYPE accumulator |
| `0x0833de1e42b93ec94a6ec31670bada94a9039f2c` | 10x long LINK ($2.27M) | Unknown | Newly created wallet — fresh money |
| `0xab961d7c42bbcd454a54b342bd191a8f090219e6` | 5x short xyz:CL ($10.2M) | Unknown | Newly created wallet |

### Named Traders (Public Figures, Verified by Nansen)

| Name | Twitter | Peak PnL | Strategy | Wallet |
|------|---------|----------|----------|--------|
| TheWhiteWhaleHL | @TheWhiteWhaleHL | $45.8M (30d, Aug 2025) | Conviction longs — holds through crashes. Long ETH/SOL/HYPE. Uses 4 wallets. Advocates for DEX over CEX. | Not publicly shared — trackable via Nansen or Dexly |
| Wanye Kest | — | $13.68M (90d) | Long-term position sizing, 2 wallets. Nansen analysis: "sustained profits come from long-term, stable position management rather than one-off trades." | Not publicly shared — trackable via Nansen |
| Laurent Zeimes | — | $4.7M | Unknown | Unknown |
| CBB | — | $2.3M | Unknown | Unknown |
| CL | @CL207 | $1.1M | Leveraged longs (10x NVDA stock on HL) | Unknown |
| Eric Chen | — | $153.5K (259% ROI) | Unknown | Unknown |

### Critical Warning

LookOnChain documented that **8 top Hyperliquid traders who posted enormous profits ALL eventually got wiped out** due to high leverage. A separate report showed a follower of a "100% winning whale" lost $1.061M in under 24 hours copying trades. Survivorship bias is extreme. The oil whale `0xd38809` got liquidated for $3.2M on March 9 with a 20x position.

---

## Solana — Smart Money Wallets

### From Grok X Search (Active March 2026)

| Wallet | Activity | Source |
|--------|----------|--------|
| `H2oNAX1bc7pc5fJxpM3Ej9VUGLnbDy5B4njKA2NvuLh3` | Withdrew 200K SOL ($17.17M) from Binance and staked | @lookonchain March 2026 |

### From Nansen Published List (Top 10 Memecoin Wallets)

| Label | Wallet | Style |
|-------|--------|-------|
| Trump Memecoin Whale | `4ETAJ...ARUj6` | Large positions |
| WIF Mega Holder | `cifwifhatday.sol` | Early mover |
| Active Smart Money | `traderpow` | Consistent |
| Profitable Sniper | `naseem` | Speed-based |
| Smart Money Trader | `shatter.sol` | Mid-term |
| Short-Term Trader | `tonka.sol` | Fast rotation |
| Multiple Memecoin Holder | `HWdeC...6T7R` | Diversified |
| Sigil Fund | Sigil Fund | High-risk, high-reward fund |

Full addresses available on nansen.ai wallet profiles.

---

## Tracking Tools & Data Sources

### For Monitoring These Wallets

| Platform | Tool | What It Does |
|----------|------|--------------|
| Hyperliquid | dexly.trade/hyperliquid/explorer | Paste any 0x address — see positions, PnL, trade history, equity curve |
| Hyperliquid | hyperdash.info | PnL charts per wallet |
| Hyperliquid | coinglass.com | Whale position aggregates, long/short ratios |
| Polymarket | polymarket.com/leaderboard | Live monthly/all-time PnL with wallet addresses |
| Polymarket | 0xinsider.com ($40/mo) | Deep whale analytics, SHAP analysis, composite scoring, insider radar |
| Polymarket | predicts.guru/checker/{wallet} | Free wallet analysis |
| Solana | birdeye.so | Token analytics, wallet P&L, win rates |
| Solana | gmgn.ai | Smart money tracker, new token scanner, copy trading |
| Cross-chain | nansen.ai | AI-powered whale analytics (paid) |

### X Accounts to Follow for Wallet Drops

| Handle | Focus |
|--------|-------|
| @lookonchain | Primary source — posts whale wallets daily with positions and PnL |
| @ai_9684xtpa | On-chain analyst, whale tracking |
| @EmberCN | Chinese crypto analyst, whale movements |
| @spotonchain | On-chain analytics |
| @0x_Discover | Polymarket whale tracking with win rates |

---

## Recommendations for Copy Trading Integration

### Tier 1 — Highest Conviction (Study & Potentially Copy)

1. **gabagool22** (Polymarket) — Fully documented arb/MM strategy. 99.5% win rate. Directly relevant to our info-arb engine. His spread-capture approach is complementary.
2. **0x15a4...DFDb** (Hyperliquid) — Confirmed +$1.88M. Disciplined: takes profit, rotates to spot. Best risk-adjusted Hyperliquid target.
3. **swisstony** (Polymarket) — $90M volume, $780K profit. This is systematic arb/MM at scale.
4. **96% win rate anon** `0x9b97...f12` (Polymarket) — $296K PnL with 96% win rate. Worth deep analysis.

### Tier 2 — Monitor Before Acting

5. **HorizonSplendidView** (Polymarket) — #1 this month at +$4M but unknown strategy.
6. **SecondWindCapital** (Polymarket) — $501K on only $1.28M volume = strong directional edge.
7. **0x8d0e...9244** (Hyperliquid) — Mixed spot+leverage approach, diversified.
8. **HYPE accumulators** `0x7e4e...` and `0x1abd...` — If bullish on HYPE ecosystem.

### Tier 3 — Watch But Don't Copy

9. **Oil whales** — Both sides are getting destroyed. One liquidated $3.2M on March 9. Too much geopolitical risk.
10. **Multi-asset degen** `0x7508...` — 40x BTC is gambling, not trading.
11. **beachboy4** — +$3M this month but swings wildly (-$7.56M drawdown). Not suitable for copy trading.

### Do NOT Copy

- Any 20x+ leveraged position without understanding the thesis
- Oil/commodity positions on Hyperliquid (too thin liquidity, geopolitical)
- TheWhiteWhaleHL without his actual wallet address (can't verify current state)

---

## Data Files

- **Watchlist JSON**: `skills/trader/watchlist-verified-traders.json` — structured data for programmatic use
- **This document**: `skills/trader/research-verified-traders-2026-03-15.md`
