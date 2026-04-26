# BirdEye gRPC Parity Matrix (Latitude)

## Scope

Target parity for the `dex_feed` gRPC service in `crates/dex-indexer/proto/dex_feed.proto` against BirdEye-style consumer expectations:

- price lookup endpoints
- market history endpoints
- streaming endpoints
- suggestion endpoints (quote/signal/source-quality)

## Endpoint Matrix

| Domain | BirdEye-style capability | Local gRPC endpoint | Status | Notes |
|---|---|---|---|---|
| Price | Single token price | `DexQuery.GetPrice` | Implemented | USD normalization from stable/SOL pairs. |
| Price | Multi-token price | `DexQuery.GetMultiPrice` | Implemented | Up to 100 tokens/request. |
| Candles | OHLCV | `DexQuery.GetOhlcv` | Implemented | `Interval` enum mapped to QuestDB sample windows. |
| Trades | Token trades | `DexQuery.GetTrades` | Implemented | Time-based pagination via `before`. |
| Trades | Pair trades | `DexQuery.GetPairTrades` | Implemented | Bidirectional pair matching. |
| Overview | 24h token overview | `DexQuery.GetTokenOverview` | Implemented | Price, change, volume, high/low, trade count. |
| History | Price history | `DexQuery.GetPriceHistory` | Implemented | Close-per-bucket series. |
| Stream | Trades stream | `DexStream.StreamTrades` | Implemented | Broadcast channel fanout from poller. |
| Stream | Price stream | `DexStream.StreamPrice` | Implemented | Event-triggered refresh for token. |
| Suggestions | Quote + signal + quality (single) | `DexQuery.GetPriceSuggestion` | Implemented | Bid/ask spread, signal action, source quality. |
| Suggestions | Quote + signal + quality (multi) | `DexQuery.GetMultiPriceSuggestion` | Implemented | Batch suggestion retrieval. |
| Suggestions | Suggestion stream | `DexStream.StreamSuggestion` | Implemented | Event-driven token suggestion updates. |

## Suggestion Model Contract

`PriceSuggestion` now carries three parity dimensions:

- **Quote-style**
  - `QuoteSuggestion.bid_usd`
  - `QuoteSuggestion.ask_usd`
  - `QuoteSuggestion.spread_bps`
  - `QuoteSuggestion.confidence`
  - `QuoteSuggestion.staleness_seconds`
- **Signal-style**
  - `SignalSuggestion.action` (`SIGNAL_BUY` / `SIGNAL_SELL` / `SIGNAL_HOLD`)
  - `SignalSuggestion.score`
  - component scores (`trend_score`, `volatility_score`, `liquidity_score`, `slippage_score`)
- **Multi-source quality**
  - `PriceQualitySuggestion.blended_price_usd`
  - `PriceQualitySuggestion.sources_total` / `sources_used`
  - `PriceQualitySuggestion.outlier_threshold`
  - fallback indicator/source fields
  - per-source detail in repeated `PriceSource`

## Data Semantics

- `updated_at` fields are emitted as nanoseconds since epoch.
- Source outliers are excluded from blend when deviation exceeds MAD-based threshold.
- Fallback is marked active when primary-quality filtering excludes source candidates.
- Suggestion persistence writes to QuestDB tables:
  - `price_suggestions`
  - `price_source_quality`

## Latency/SLO Targets (promotion gates)

- Unary methods p95 <= 400ms inside cluster.
- Stream update lag p95 <= 2s from swap event ingestion.
- Price median relative error against BirdEye reference <= configured threshold in parity harness.
