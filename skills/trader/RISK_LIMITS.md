# Trading Risk Limits

Total capital: $100
Last updated: 2026-03-15

## Allocation

| Strategy             | Allocation | Notes                        |
|----------------------|------------|------------------------------|
| Solana copy trading  | $30 (0.22 SOL) |                         |
| Solana sniping       | $25 (0.18 SOL) | Moltiumv2 autostrategy |
| Polymarket           | $25 USDC   | Via Simmer SDK               |
| Base chain           | $10        | Reserved, not yet deployed   |
| Gas reserve          | $10 (0.07 SOL) | Never trade into this   |

## Per-Strategy Hard Limits

### Moltiumv2 Autostrategy (Solana sniping)

Config: `moltiumv2/tools/moltium/local/autostrategy/strategies/paper-safe/strategy.json`

| Parameter          | Value  | Rationale                                   |
|--------------------|--------|---------------------------------------------|
| dryRun             | true   | Paper mode until strategy is validated       |
| buySolPerTrade     | 0.03   | ~$4 per trade                                |
| maxOpenPositions   | 1      | One position at a time                       |
| minSolBalance      | 0.05   | Gas reserve floor                            |
| maxBuysPerHour     | 5      | Prevent overtrading                          |
| maxLossSolPerDay   | 0.05   | ~$4.40 daily loss cap (~20% of allocation)   |
| feeBps             | 0      | No additional fee                            |
| exitAfterSec       | 3600   | 1-hour max hold time                         |

### Solana Copy Trader

Config: `solana-copy-trader/scripts/.env`

| Parameter      | Value | Rationale        |
|----------------|-------|------------------|
| MAX_TRADE_SOL  | 0.03  | ~$4 per trade    |

Default mode is dry-run (no --live flag). The .env does not contain a
live-mode toggle; the script must be invoked with an explicit flag to
execute real trades.

### Polymarket Copytrading

Config: `polymarket-copytrading/config.json`

| Parameter          | Value | Rationale                             |
|--------------------|-------|---------------------------------------|
| max_usd            | 5     | $5 max per position                   |
| top_n              | 5     | Mirror only top 5 whale positions     |
| max_trades_per_run | 5     | Cap trades per execution cycle        |

Default mode is dry-run. Pass `--live` to execute real trades.
Total max exposure: 5 positions x $5 = $25 (matches allocation).

### Base Chain

Not yet deployed. $10 reserved.

## Rules

1. Never disable dryRun on any strategy without explicit approval.
2. Never raise per-trade size above 0.05 SOL / $7 without re-evaluating allocation.
3. Gas reserve (0.07 SOL) must remain untouched by trading strategies.
4. Review and update this file whenever allocations change.
