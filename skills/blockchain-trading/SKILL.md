---
name: blockchain-trading
description: >-
  Blockchain trading operations — DEX swaps, token analysis, market data,
  and DeFi protocol interactions across Solana and EVM chains.
  Read-only / quote-only by default; live execution requires explicit
  user confirmation and proper wallet configuration.
metadata:
  version: "1.0.0"
  security:
    default_mode: read-only
    requires_confirmation: true
    no_raw_private_keys: true
---

# Blockchain Trading Operations

Multi-chain trading skill for Block — covers Solana and EVM DeFi ecosystems.

> **Security policy:** This skill operates in **read-only / quote-only mode** by
> default. Any transaction that moves funds requires **explicit user confirmation**
> before signing. Never store, log, or echo private keys. Prefer wallet file paths
> or hardware signer references over raw key material.

## Solana DEX — Jupiter Aggregator

### Get a Swap Quote (read-only)

```bash
curl -s "https://quote-api.jup.ag/v6/quote?\
inputMint=So11111111111111111111111111111111111111112&\
outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&\
amount=1000000000&slippageBps=50" | jq .
```

### Token Search

```bash
curl -s "https://api.jup.ag/ultra/v1/search?query=SOL" | jq '.[] | {symbol, address, decimals}'
```

### Execute a Swap (requires confirmation)

1. Fetch quote (above)
2. Build swap transaction:
   ```bash
   curl -s -X POST "https://quote-api.jup.ag/v6/swap" \
     -H "Content-Type: application/json" \
     -d '{"quoteResponse": <QUOTE>, "userPublicKey": "<WALLET_ADDRESS>"}' | jq .
   ```
3. **⚠️ Confirm with user before signing and sending.**

### Jupiter Ultra API (authenticated)

Requires `JUP_API_KEY` from [portal.jup.ag](https://portal.jup.ag).

```bash
# Order (includes routing + execution)
curl -s -X POST "https://api.jup.ag/ultra/v1/order" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $JUP_API_KEY" \
  -d '{"inputMint": "So1...", "outputMint": "EPj...", "amount": 1000000000}'
```

## Solana Wallet Operations

### Check Balances

```bash
# SOL balance
solana balance --keypair "$SOLANA_KEYPAIR_PATH"

# All token accounts
spl-token accounts --owner $(solana address --keypair "$SOLANA_KEYPAIR_PATH")

# Specific token
spl-token balance <MINT_ADDRESS> --owner $(solana address --keypair "$SOLANA_KEYPAIR_PATH")
```

### Transfer SOL

```bash
# ⚠️ Requires user confirmation
solana transfer <RECIPIENT> <AMOUNT> --keypair "$SOLANA_KEYPAIR_PATH" --allow-unfunded-recipient
```

### Transfer SPL Token

```bash
# ⚠️ Requires user confirmation
spl-token transfer <MINT> <AMOUNT> <RECIPIENT_ATA> --owner "$SOLANA_KEYPAIR_PATH"
```

## EVM DEX Operations (Base, Ethereum, Arbitrum)

### Price Quotes via DEX Aggregators

```typescript
// 1inch API (read-only quote)
const quote = await fetch(
  `https://api.1inch.dev/swap/v6.0/8453/quote?src=${tokenIn}&dst=${tokenOut}&amount=${amount}`,
  { headers: { Authorization: `Bearer ${ONEINCH_API_KEY}` } }
).then(r => r.json());
```

### Uniswap v3 Quoting

```typescript
import { ethers } from 'ethers';

const quoterV2 = new ethers.Contract(QUOTER_V2_ADDRESS, QUOTER_ABI, provider);
const [amountOut] = await quoterV2.callStatic.quoteExactInputSingle({
  tokenIn, tokenOut, fee: 3000, amountIn, sqrtPriceLimitX96: 0
});
```

## Market Data (read-only, no auth required)

### CoinGecko

```bash
# Token price
curl -s "https://api.coingecko.com/api/v3/simple/price?ids=solana,ethereum,bitcoin&vs_currencies=usd" | jq .

# OHLC data
curl -s "https://api.coingecko.com/api/v3/coins/solana/ohlc?vs_currency=usd&days=7" | jq .
```

### Binance Public API

```bash
# 24h ticker
curl -s "https://api.binance.com/api/v3/ticker/24hr?symbol=SOLUSDT" | jq '{symbol,lastPrice,priceChangePercent,volume}'

# Klines (candlesticks)
curl -s "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=4h&limit=50" | jq .
```

### DeFiLlama (TVL, yields)

```bash
# Protocol TVL
curl -s "https://api.llama.fi/protocol/jupiter" | jq '{name,tvl:.tvl[-1]}'

# Top yields
curl -s "https://yields.llama.fi/pools" | jq '[.data[] | select(.chain=="Solana") | {pool,tvlUsd,apy}] | sort_by(-.tvlUsd) | .[:10]'
```

## Technical Analysis Patterns

### RSI Calculation (14-period)

```python
import numpy as np

def rsi(prices, period=14):
    deltas = np.diff(prices)
    gains = np.where(deltas > 0, deltas, 0)
    losses = np.where(deltas < 0, -deltas, 0)
    avg_gain = np.convolve(gains, np.ones(period)/period, mode='valid')
    avg_loss = np.convolve(losses, np.ones(period)/period, mode='valid')
    rs = avg_gain / (avg_loss + 1e-10)
    return 100 - (100 / (1 + rs))
```

### Support / Resistance from Klines

```python
def find_levels(highs, lows, window=5):
    resistance = [h for i, h in enumerate(highs[window:-window], window)
                  if h == max(highs[i-window:i+window+1])]
    support = [l for i, l in enumerate(lows[window:-window], window)
               if l == min(lows[i-window:i+window+1])]
    return sorted(set(support)), sorted(set(resistance))
```

## DeFi Protocol Interactions

### Solana Staking (Marinade / Jito)

```bash
# Check mSOL/JitoSOL rates
curl -s "https://api.marinade.finance/msol/price_sol" | jq .
```

### Lending Protocols (Kamino, Aave)

```bash
# Kamino markets
curl -s "https://api.kamino.finance/v2/markets" | jq '.[].reserves[] | {symbol: .symbol, depositApy: .metrics.depositApy, borrowApy: .metrics.borrowApy}'
```

## Risk Management Rules

1. **Position sizing** — Never exceed 10% of portfolio in a single position
2. **Stop-loss** — Always define exit criteria before entering a trade
3. **Slippage** — Default to 50 bps; increase only for low-liquidity tokens
4. **Contract verification** — Always verify contract source on explorer before interaction
5. **Honeypot check** — Confirm token is sellable before buying
6. **Cool-down** — If daily losses exceed 20%, halt trading for 24 hours

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SOLANA_KEYPAIR_PATH` | For Solana txns | Path to wallet keypair JSON |
| `SOLANA_RPC_URL` | Optional | Custom RPC (default: public mainnet) |
| `JUP_API_KEY` | For Jupiter Ultra | Jupiter portal API key |
| `HELIUS_API_KEY` | Optional | Helius RPC for better rate limits |
| `BIRDEYE_API_KEY` | Optional | Birdeye token analytics |

> **⚠️ Security:** All keys must be provided via environment variables or
> secure file paths. Never hardcode private keys. Never echo or log key material.
> Prefer hardware wallets or KMS-backed signers for production use.
