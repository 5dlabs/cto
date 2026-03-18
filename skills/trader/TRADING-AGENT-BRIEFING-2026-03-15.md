# Trading Agent Briefing — March 15, 2026

Research-backed findings for the Polymarket trading stack. All data is current as of March 15, 2026.

Reference files:
- `watchlist-verified-traders.json` — structured wallet watchlist (30+ wallets)
- `research-verified-traders-2026-03-15.md` — full research with sources

---

## 1. Missing Strategy: Binary Spread Capture (Risk-Free Arb)

The current stack has three strategies:
- **grok-info-arb** — news-driven directional signals
- **polymarket-mm** — two-sided market making with CEX leaning
- **polymarket-copytrading** — mirroring whale directional bets

**Missing**: Risk-free binary arbitrage. When the combined bid for YES + NO tokens on the same market is < $1.00, buying both sides guarantees profit at resolution.

Example: YES bid $0.48 + NO bid $0.49 = $0.97 cost → $1.00 at resolution = $0.03 guaranteed profit (3.1% per trade).

This is gabagool22's primary strategy. He has $868K PnL, 99.5% win rate, $144.6M volume across 24,525 markets. It works because:
- No directional risk — you profit regardless of outcome
- Works on every BTC/ETH UP/DOWN binary (5min, 15min, 60min, 4hr windows)
- Scales linearly with capital
- Complementary to info-arb and MM — doesn't compete for the same edge

**Implementation**: Monitor all active binary markets. When `best_bid_yes + best_bid_no < 1.00 - fees`, buy both sides. Wait for resolution. Merge shares on-chain if needed (gabagool22 uses ProxyWallet Factory for this).

---

## 2. MM Engine Upgrades (from gabagool22 reverse engineering)

The `polymarket-mm/mm_engine.js` already uses CEX momentum for quote leaning — this is correct and mirrors gabagool22's approach. Specific improvements:

### Use Binance aggTrade WebSocket (not REST polling)
gabagool22 connects to `wss://stream.binance.com:9443/ws/btcusdt@aggTrade` for real-time tick data. This gives sub-second price signals for preemptive quote cancellation (avoiding toxic fills). If the MM engine polls REST, latency is the difference between profit and adverse selection.

### Restrict to crypto binary markets only
gabagool22 trades ONLY BTC and ETH UP/DOWN binaries. These markets have a ground truth (CEX price at expiry), making fair value computable. Event-based markets (politics, weather) don't have a CEX-derived fair value — MM on those requires a fundamentally different approach.

### Operational parameters (from on-chain analysis)
- ~365 trades per market average
- 64-minute average hold time
- Near-equal YES/NO share counts (~1,892 YES, ~1,878 NO per market) = textbook MM
- 24/7 automated, near-uniform hourly distribution
- 44,000 lines of Rust (we're in JS — fine for now, but latency matters if competing with him)

---

## 3. Copytrading Wallet Quality Notes

### Tier 1 — High confidence directional copiers
| Wallet | Handle | Why |
|--------|--------|-----|
| `0x02227b8f5a9636e895607edd3185ed6ee5598ff7` | HorizonSplendidView | #1 monthly, +$4M, high conviction |
| `0x019782cab5d844f02bafb71f512758be78579f3c` | majorexploiter | #3 monthly, +$2.4M |
| `0x8c80d213c0cbad777d06ee3f58f6ca4bc03102c3` | SecondWindCapital | Best profit/volume ratio: $501K on $1.28M |
| `0x916f7165c2c836aba22edb6453cdbb5f3ea253ba` | WoofMaster | $571K on $1.55M volume, directional |

### Tier 1 — High confidence anon whales
| Wallet | PnL | Win Rate | Why |
|--------|-----|----------|-----|
| `0x9b979a065641e8cfde3022a30ed2d9415cf55e12` | $296K | 96% | Exceptional win rate |
| `0x12d6cccfc7470a3f4bafc53599a4779cbf2cf2a8` | $213K | 84% | Limit-order based, systematic |

### Do NOT copy trade
| Wallet | Handle | Why NOT |
|--------|--------|---------|
| `0x6031b6eed1c97e853c6e0f03ad3ce3529351f96d` | gabagool22 | Arb/MM strategy — copying positions won't capture spread edge |
| `0x204f72f35326db932158cba6adff0b9a1da95e14` | swisstony | Same — $90M vol arb/MM, no directional signal |
| `0xc2e7800b5af46e6093872b177b7a5e7f0563be51` | beachboy4 | -$7.56M drawdown. Too volatile for copy trading |
| `0x6ac5bb06a9eb05641fd5e82640268b92f3ab4b6e` | 0p0jogggg | Likely arb/MM — $41.9M vol, no directional signal |

---

## 4. Hyperliquid: Watch Only, Do Not Copy

### Best target (if ever building HL copy trading)
`0x15a4F009BB324A3fb9E36137136B201E3Fe0DFDb` — confirmed +$1.88M closed PnL. Disciplined: takes profit on leveraged longs, rotates to spot. Best risk-adjusted profile on HL.

### Critical warning
LookOnChain documented that **8 out of 8 top Hyperliquid traders who posted enormous profits ALL eventually got liquidated** due to high leverage. A separate case: a follower of a "100% win rate whale" lost $1.061M in under 24 hours copying trades. If HL copy trading is ever built, enforce:
- Max 3x leverage on copied positions
- Max 5% of bankroll per position
- Hard stop-loss at -20% (don't trust whale's risk management)

### Oil positions — stay away
Both sides of the oil trade are getting destroyed on HL. The oil bear (`0x985f...`) is down -$1.87M and keeps depositing more capital. Another oil whale (`0xd388...`) got liquidated for $3.2M on March 9 at 20x leverage. Too thin liquidity, too much geopolitical risk.

---

## 5. Signal Sources for grok-info-arb

### X accounts to add to scanner topics/watchlist
These accounts post wallet addresses, whale movements, and breaking on-chain data before it hits news:

| Handle | Focus | Value |
|--------|-------|-------|
| @lookonchain | Whale wallet drops with positions and PnL | Primary source — daily posts |
| @ai_9684xtpa | On-chain analyst, whale tracking | Chinese crypto market intel |
| @EmberCN | Chinese crypto analyst | Whale movements, early signals |
| @spotonchain | On-chain analytics | Structured alerts |
| @0x_Discover | Polymarket whale tracking with win rates | Directly relevant |

### Tracking tools for position verification
| Tool | Use |
|------|-----|
| `predicts.guru/checker/{wallet}` | Free Polymarket wallet analysis |
| `polymarket.com/leaderboard` | Live monthly PnL with wallets |
| `dexly.trade/hyperliquid/explorer` | HL wallet positions, PnL, equity curve |
| `0xinsider.com` ($40/mo) | Deep Polymarket whale analytics, SHAP scoring |

---

## 6. Polymarket Fee Model Reminder

Since Jan 2026, Polymarket charges dynamic taker fees: `fee_pct = 0.03 * (1 - (2p-1)^2)`. Peaks at 3% at 50c, tapers to 0% at extremes. Key implications:
- **Latency arb is dead** — taker fees eat the edge on fast in-and-out trades
- **Market making is favored** — makers pay ZERO fees and receive taker fee rebates
- **Info-arb still works** — if signal is strong enough to overcome 1-3% taker fee
- **Spread capture still works** — gabagool22 proves it at scale ($868K/mo)
- **Event markets are fee-free** — political/event markets have no fees (only crypto binaries)
