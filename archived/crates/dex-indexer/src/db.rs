use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{NaiveDateTime, Utc};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

use crate::error::Error;
use crate::proto;
use crate::SwapEvent;

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

const SOL_CACHE_TTL: Duration = Duration::from_secs(5);

fn is_stablecoin(mint: &str) -> bool {
    mint == USDC_MINT || mint == USDT_MINT
}

pub struct DbClient {
    client: tokio_postgres::Client,
    sol_usd_cache: Arc<Mutex<(f64, Instant)>>,
}

impl DbClient {
    pub async fn connect(host: &str, port: u16, user: &str, password: &str) -> Result<Self, Error> {
        let conn_str =
            format!("host={host} port={port} user={user} password={password} dbname=qdb");
        let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
            .await
            .map_err(Error::Postgres)?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!(error = %e, "QuestDB PG wire connection lost");
            }
        });

        tracing::info!(host, port, "connected to QuestDB PG wire");

        let db = Self {
            client,
            sol_usd_cache: Arc::new(Mutex::new((0.0, Instant::now() - SOL_CACHE_TTL))),
        };
        db.init_suggestion_tables().await;
        Ok(db)
    }

    // ── Price ────────────────────────────────────────────────────────────

    pub async fn get_price(&self, token: &str) -> Result<proto::TokenPrice, Error> {
        let row = self
            .client
            .query_opt(
                "SELECT token_in, token_out, price, timestamp \
                 FROM dex_swaps \
                 WHERE token_in = $1 OR token_out = $1 \
                 ORDER BY timestamp DESC \
                 LIMIT 1",
                &[&token],
            )
            .await
            .map_err(Error::Postgres)?
            .ok_or_else(|| Error::Query(format!("no swaps found for {token}")))?;

        let token_in: &str = row.get(0);
        let token_out: &str = row.get(1);
        let stored_price: f64 = row_f64(&row, 2)?;
        let ts_micros = row_ts_micros(&row, 3)?;

        let price_usd = self
            .resolve_usd_price(token, token_in, token_out, stored_price)
            .await;

        let price_24h_ago = self.get_price_at_offset(token, 24 * 3600).await;
        let change_24h = if price_24h_ago > f64::EPSILON {
            (price_usd - price_24h_ago) / price_24h_ago * 100.0
        } else {
            0.0
        };

        Ok(proto::TokenPrice {
            token: token.to_string(),
            price_usd,
            change_24h,
            updated_at: ts_micros * 1000, // micros → nanos
        })
    }

    pub async fn get_multi_price(
        &self,
        tokens: &[String],
    ) -> Result<std::collections::HashMap<String, proto::TokenPrice>, Error> {
        let mut out = std::collections::HashMap::new();
        for t in tokens.iter().take(100) {
            match self.get_price(t).await {
                Ok(p) => {
                    out.insert(t.clone(), p);
                }
                Err(Error::Query(_)) => {} // token not found, skip
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    pub async fn get_price_suggestion(&self, token: &str) -> Result<proto::PriceSuggestion, Error> {
        let price = self.get_price(token).await?;
        let samples = self.get_source_samples(token).await?;
        if samples.is_empty() {
            return Err(Error::Query(format!(
                "no source samples found for suggestion token={token}"
            )));
        }

        let source_count = samples.len();
        let mut source_prices: Vec<f64> = samples.iter().map(|s| s.price_usd).collect();
        source_prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = source_prices[source_prices.len() / 2];
        let abs_dev = source_prices
            .iter()
            .map(|p| (p - median).abs())
            .collect::<Vec<_>>();
        let mut abs_dev_sorted = abs_dev.clone();
        abs_dev_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = abs_dev_sorted[abs_dev_sorted.len() / 2].max(0.0001);
        let outlier_threshold = 3.5 * mad;

        let mut weighted_sum = 0.0;
        let mut weighted_denom = 0.0;
        let mut primary_source = String::new();
        let mut fallback_source = String::new();
        let mut fallback_active = false;
        let mut proto_sources = Vec::with_capacity(source_count);
        let now_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        for sample in &samples {
            let outlier_filtered = (sample.price_usd - median).abs() > outlier_threshold;
            let confidence = if outlier_filtered { 0.15 } else { 0.85 };
            if !outlier_filtered {
                weighted_sum += sample.price_usd * sample.weight;
                weighted_denom += sample.weight;
                if primary_source.is_empty() {
                    primary_source = sample.source.clone();
                }
            } else if fallback_source.is_empty() {
                fallback_source = sample.source.clone();
                fallback_active = true;
            }

            proto_sources.push(proto::PriceSource {
                source: sample.source.clone(),
                weight: sample.weight,
                price_usd: sample.price_usd,
                confidence,
                updated_at: sample.updated_at,
                fallback_used: outlier_filtered,
                outlier_filtered,
            });
        }

        let blended_price_usd = if weighted_denom > f64::EPSILON {
            weighted_sum / weighted_denom
        } else {
            price.price_usd
        };
        let spread_bps = self.estimate_spread_bps(&samples);
        let mid = blended_price_usd;
        let bid = mid * (1.0 - spread_bps / 20_000.0);
        let ask = mid * (1.0 + spread_bps / 20_000.0);

        let signal = self.compute_signal_suggestion(token).await?;
        let staleness_seconds = samples
            .iter()
            .map(|s| ((now_ns - s.updated_at).max(0) as f64) / 1_000_000_000.0)
            .fold(0.0_f64, f64::max);
        let confidence = (0.45 + (proto_sources.len() as f64 / 10.0))
            .min(0.95)
            .max(0.35);

        let suggestion = proto::PriceSuggestion {
            token: token.to_string(),
            quote: Some(proto::QuoteSuggestion {
                bid_usd: bid,
                ask_usd: ask,
                mid_usd: mid,
                spread_bps,
                confidence,
                staleness_seconds,
            }),
            signal: Some(signal),
            quality: Some(proto::PriceQualitySuggestion {
                blended_price_usd,
                outlier_threshold,
                sources_total: source_count as u32,
                sources_used: proto_sources.iter().filter(|s| !s.outlier_filtered).count() as u32,
                primary_source,
                fallback_source,
                fallback_active,
            }),
            sources: proto_sources,
            updated_at: now_ns,
        };
        self.persist_price_suggestion(&suggestion).await;
        Ok(suggestion)
    }

    pub async fn get_multi_price_suggestion(
        &self,
        tokens: &[String],
    ) -> Result<HashMap<String, proto::PriceSuggestion>, Error> {
        let mut out = HashMap::new();
        for token in tokens.iter().take(100) {
            match self.get_price_suggestion(token).await {
                Ok(s) => {
                    out.insert(token.clone(), s);
                }
                Err(Error::Query(_)) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    async fn get_price_at_offset(&self, token: &str, seconds_ago: i64) -> f64 {
        let result = self
            .client
            .query_opt(
                "SELECT token_in, token_out, price \
                 FROM dex_swaps \
                 WHERE (token_in = $1 OR token_out = $1) \
                   AND timestamp < dateadd('s', -$2, now()) \
                 ORDER BY timestamp DESC \
                 LIMIT 1",
                &[&token, &seconds_ago],
            )
            .await;

        match result {
            Ok(Some(row)) => {
                let token_in: &str = row.get(0);
                let token_out: &str = row.get(1);
                let stored_price: f64 = row_f64(&row, 2).unwrap_or(0.0);
                self.resolve_usd_price(token, token_in, token_out, stored_price)
                    .await
            }
            _ => 0.0,
        }
    }

    // ── OHLCV ────────────────────────────────────────────────────────────

    pub async fn get_ohlcv(
        &self,
        token: &str,
        interval: &str,
        time_from: i64,
        time_to: i64,
    ) -> Result<Vec<proto::OhlcvCandle>, Error> {
        // Use stablecoin-paired swaps for direct USD price candles.
        // Two directions: token sold for stablecoin, or stablecoin sold for token.
        let query = format!(
            "SELECT timestamp, \
               first(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as open, \
               max(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as high, \
               min(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as low, \
               last(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as close, \
               sum(CASE WHEN token_in = $1 THEN amount_in ELSE amount_out END) as volume, \
               count() as trades \
             FROM dex_swaps \
             WHERE ((token_in = $1 AND token_out IN ('{USDC_MINT}', '{USDT_MINT}')) \
                OR (token_out = $1 AND token_in IN ('{USDC_MINT}', '{USDT_MINT}'))) \
               AND timestamp >= $2 AND timestamp < $3 \
             SAMPLE BY {interval}"
        );

        let from_ts = format!("{}000000", time_from); // seconds → micros string
        let to_ts = format!("{}000000", time_to);

        let rows = self
            .client
            .query(&query, &[&token, &from_ts, &to_ts])
            .await
            .map_err(Error::Postgres)?;

        let candles = rows
            .iter()
            .map(|r| {
                let ts_micros = row_ts_micros(r, 0).unwrap_or_default();
                proto::OhlcvCandle {
                    timestamp: ts_micros / 1_000_000, // micros → seconds
                    open: row_f64(r, 1).unwrap_or(0.0),
                    high: row_f64(r, 2).unwrap_or(0.0),
                    low: row_f64(r, 3).unwrap_or(0.0),
                    close: row_f64(r, 4).unwrap_or(0.0),
                    volume: row_f64(r, 5).unwrap_or(0.0),
                    trades: row_i64(r, 6).unwrap_or(0) as u64,
                }
            })
            .collect();

        Ok(candles)
    }

    // ── Trades ───────────────────────────────────────────────────────────

    pub async fn get_trades(
        &self,
        token: &str,
        limit: i32,
        before_nanos: i64,
    ) -> Result<Vec<proto::Trade>, Error> {
        let limit = limit.clamp(1, 200) as i64;

        let rows = if before_nanos > 0 {
            let before_micros = before_nanos / 1000;
            self.client
                .query(
                    "SELECT signature, dex, pool, token_in, token_out, \
                            amount_in, amount_out, price, signer, timestamp, slot \
                     FROM dex_swaps \
                     WHERE (token_in = $1 OR token_out = $1) \
                       AND timestamp < $2 \
                     ORDER BY timestamp DESC \
                     LIMIT $3",
                    &[&token, &before_micros, &limit],
                )
                .await
        } else {
            self.client
                .query(
                    "SELECT signature, dex, pool, token_in, token_out, \
                            amount_in, amount_out, price, signer, timestamp, slot \
                     FROM dex_swaps \
                     WHERE token_in = $1 OR token_out = $1 \
                     ORDER BY timestamp DESC \
                     LIMIT $2",
                    &[&token, &limit],
                )
                .await
        }
        .map_err(Error::Postgres)?;

        Ok(rows.iter().map(row_to_trade).collect())
    }

    pub async fn get_pair_trades(
        &self,
        token_a: &str,
        token_b: &str,
        limit: i32,
        before_nanos: i64,
    ) -> Result<Vec<proto::Trade>, Error> {
        let limit = limit.clamp(1, 200) as i64;

        let rows = if before_nanos > 0 {
            let before_micros = before_nanos / 1000;
            self.client
                .query(
                    "SELECT signature, dex, pool, token_in, token_out, \
                            amount_in, amount_out, price, signer, timestamp, slot \
                     FROM dex_swaps \
                     WHERE ((token_in = $1 AND token_out = $2) \
                         OR (token_in = $2 AND token_out = $1)) \
                       AND timestamp < $3 \
                     ORDER BY timestamp DESC \
                     LIMIT $4",
                    &[&token_a, &token_b, &before_micros, &limit],
                )
                .await
        } else {
            self.client
                .query(
                    "SELECT signature, dex, pool, token_in, token_out, \
                            amount_in, amount_out, price, signer, timestamp, slot \
                     FROM dex_swaps \
                     WHERE (token_in = $1 AND token_out = $2) \
                        OR (token_in = $2 AND token_out = $1) \
                     ORDER BY timestamp DESC \
                     LIMIT $3",
                    &[&token_a, &token_b, &limit],
                )
                .await
        }
        .map_err(Error::Postgres)?;

        Ok(rows.iter().map(row_to_trade).collect())
    }

    // ── Overview ─────────────────────────────────────────────────────────

    pub async fn get_token_overview(
        &self,
        token: &str,
    ) -> Result<proto::GetTokenOverviewResponse, Error> {
        let price = self.get_price(token).await?;

        let stats_row = self
            .client
            .query_opt(
                "SELECT count() as trades, \
                        sum(CASE WHEN token_in = $1 THEN amount_in ELSE amount_out END) as volume, \
                        max(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as high, \
                        min(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as low \
                 FROM dex_swaps \
                 WHERE ((token_in = $1 AND token_out IN ($2, $3)) \
                    OR (token_out = $1 AND token_in IN ($2, $3))) \
                   AND timestamp >= dateadd('d', -1, now())",
                &[&token, &USDC_MINT, &USDT_MINT],
            )
            .await
            .map_err(Error::Postgres)?;

        let (trades_24h, volume_24h, high_24h, low_24h) = match stats_row {
            Some(row) => (
                row_i64(&row, 0).unwrap_or(0) as u64,
                row_f64_loose(&row, 1),
                row_f64_loose(&row, 2),
                row_f64_loose(&row, 3),
            ),
            None => (0, 0.0, 0.0, 0.0),
        };

        Ok(proto::GetTokenOverviewResponse {
            token: token.to_string(),
            price_usd: price.price_usd,
            change_24h: price.change_24h,
            volume_24h,
            trades_24h,
            high_24h,
            low_24h,
            updated_at: price.updated_at,
        })
    }

    // ── Price History ────────────────────────────────────────────────────

    pub async fn get_price_history(
        &self,
        token: &str,
        interval: &str,
        time_from: i64,
        time_to: i64,
    ) -> Result<Vec<proto::PricePoint>, Error> {
        let query = format!(
            "SELECT timestamp, \
               last(CASE WHEN token_in = $1 THEN price ELSE 1.0/price END) as price_usd \
             FROM dex_swaps \
             WHERE ((token_in = $1 AND token_out IN ('{USDC_MINT}', '{USDT_MINT}')) \
                OR (token_out = $1 AND token_in IN ('{USDC_MINT}', '{USDT_MINT}'))) \
               AND timestamp >= $2 AND timestamp < $3 \
             SAMPLE BY {interval}"
        );

        let from_ts = format!("{}000000", time_from);
        let to_ts = format!("{}000000", time_to);

        let rows = self
            .client
            .query(&query, &[&token, &from_ts, &to_ts])
            .await
            .map_err(Error::Postgres)?;

        Ok(rows
            .iter()
            .map(|r| {
                let ts_micros = row_ts_micros(r, 0).unwrap_or_default();
                proto::PricePoint {
                    timestamp: ts_micros / 1_000_000,
                    price_usd: row_f64(r, 1).unwrap_or(0.0),
                }
            })
            .collect())
    }

    // ── Streaming support ────────────────────────────────────────────────

    pub async fn get_recent_swaps(&self, since_nanos: i64) -> Result<Vec<SwapEvent>, Error> {
        let since_micros = since_nanos / 1000;
        let rows = self
            .client
            .query(
                "SELECT timestamp, slot, signature, dex, pool, \
                        token_in, token_out, amount_in, amount_out, price, signer \
                 FROM dex_swaps \
                 WHERE timestamp > $1 \
                 ORDER BY timestamp ASC \
                 LIMIT 1000",
                &[&since_micros],
            )
            .await
            .map_err(Error::Postgres)?;

        Ok(rows
            .iter()
            .map(|r| {
                let ts_micros: i64 = r.get(0);
                SwapEvent {
                    timestamp: ts_micros * 1000, // micros → nanos
                    slot: r.get::<_, i64>(1) as u64,
                    signature: r.get(2),
                    dex: r.get(3),
                    pool: r.get(4),
                    token_in: r.get(5),
                    token_out: r.get(6),
                    amount_in: r.get(7),
                    amount_out: r.get(8),
                    price: r.get(9),
                    signer: r.get(10),
                }
            })
            .collect())
    }

    // ── Backfill support ─────────────────────────────────────────────────

    pub async fn earliest_timestamp(&self) -> Result<Option<i64>, Error> {
        let row = self
            .client
            .query_opt("SELECT min(timestamp) FROM dex_swaps", &[])
            .await
            .map_err(Error::Postgres)?;

        Ok(row.and_then(|r| {
            let ts: Option<i64> = r.get(0);
            ts
        }))
    }

    async fn init_suggestion_tables(&self) {
        let statements = [
            "CREATE TABLE IF NOT EXISTS price_suggestions (\
                timestamp TIMESTAMP,\
                token SYMBOL,\
                mid_usd DOUBLE,\
                bid_usd DOUBLE,\
                ask_usd DOUBLE,\
                spread_bps DOUBLE,\
                confidence DOUBLE,\
                signal_action SYMBOL,\
                signal_score DOUBLE,\
                fallback_active BOOLEAN\
            ) timestamp(timestamp) PARTITION BY DAY WAL",
            "CREATE TABLE IF NOT EXISTS price_source_quality (\
                timestamp TIMESTAMP,\
                token SYMBOL,\
                source SYMBOL,\
                weight DOUBLE,\
                price_usd DOUBLE,\
                confidence DOUBLE,\
                fallback_used BOOLEAN,\
                outlier_filtered BOOLEAN\
            ) timestamp(timestamp) PARTITION BY DAY WAL",
        ];
        for sql in statements {
            if let Err(e) = self.client.execute(sql, &[]).await {
                tracing::warn!(error = %e, "price suggestion table init skipped");
            }
        }
    }

    // ── Internal ─────────────────────────────────────────────────────────

    async fn get_source_samples(&self, token: &str) -> Result<Vec<SourceSample>, Error> {
        let rows = self
            .client
            .query(
                "SELECT dex, token_in, token_out, price, timestamp \
                 FROM dex_swaps \
                 WHERE token_in = $1 OR token_out = $1 \
                 ORDER BY timestamp DESC \
                 LIMIT 500",
                &[&token],
            )
            .await
            .map_err(Error::Postgres)?;

        let mut by_source = HashMap::<String, SourceSample>::new();
        for row in rows {
            let source: String = row.get(0);
            if by_source.contains_key(&source) {
                continue;
            }
            let token_in: String = row.get(1);
            let token_out: String = row.get(2);
            let stored_price: f64 = row_f64(&row, 3)?;
            let ts_micros = row_ts_micros(&row, 4)?;
            let price_usd = self
                .resolve_usd_price(token, &token_in, &token_out, stored_price)
                .await;
            if price_usd <= f64::EPSILON {
                continue;
            }
            by_source.insert(
                source.clone(),
                SourceSample {
                    source,
                    weight: source_weight(&token_in, &token_out),
                    price_usd,
                    updated_at: ts_micros * 1000,
                },
            );
        }
        Ok(by_source.into_values().collect())
    }

    fn estimate_spread_bps(&self, samples: &[SourceSample]) -> f64 {
        if samples.len() < 2 {
            return 12.0;
        }
        let min = samples
            .iter()
            .map(|s| s.price_usd)
            .fold(f64::INFINITY, f64::min);
        let max = samples.iter().map(|s| s.price_usd).fold(0.0, f64::max);
        if min <= f64::EPSILON {
            return 12.0;
        }
        (((max - min) / min) * 10_000.0).clamp(3.0, 80.0)
    }

    async fn compute_signal_suggestion(
        &self,
        token: &str,
    ) -> Result<proto::SignalSuggestion, Error> {
        let rows = self
            .client
            .query(
                "SELECT token_in, token_out, price, amount_in, amount_out, timestamp \
                 FROM dex_swaps \
                 WHERE token_in = $1 OR token_out = $1 \
                 ORDER BY timestamp DESC \
                 LIMIT 250",
                &[&token],
            )
            .await
            .map_err(Error::Postgres)?;
        if rows.is_empty() {
            return Err(Error::Query(format!(
                "no swaps found for signal token={token}"
            )));
        }

        let mut prices = Vec::with_capacity(rows.len());
        let mut volume = 0.0;
        for row in rows {
            let token_in: String = row.get(0);
            let token_out: String = row.get(1);
            let stored_price: f64 = row_f64(&row, 2)?;
            let amount_in: f64 = row_f64(&row, 3)?;
            let amount_out: f64 = row_f64(&row, 4)?;
            let p = self
                .resolve_usd_price(token, &token_in, &token_out, stored_price)
                .await;
            if p > f64::EPSILON {
                prices.push(p);
                volume += if token == token_in {
                    amount_in
                } else {
                    amount_out
                };
            }
        }
        if prices.len() < 2 {
            return Ok(proto::SignalSuggestion {
                action: proto::SignalAction::SignalHold as i32,
                score: 0.0,
                trend_score: 0.0,
                volatility_score: 0.0,
                liquidity_score: 0.0,
                slippage_score: 0.0,
                rationale: "insufficient history".to_string(),
            });
        }

        let short = avg(&prices[..prices.len().min(20)]);
        let long = avg(&prices[..prices.len().min(120)]);
        let trend_score = if long > f64::EPSILON {
            ((short - long) / long).clamp(-0.2, 0.2) * 5.0
        } else {
            0.0
        };
        let mean = avg(&prices);
        let variance = prices
            .iter()
            .map(|p| {
                let d = p - mean;
                d * d
            })
            .sum::<f64>()
            / prices.len() as f64;
        let volatility = variance.sqrt() / mean.max(1e-9);
        let volatility_score = (0.12 - volatility).clamp(-0.12, 0.12) * 4.0;
        let liquidity_score = (volume.log10() / 10.0).clamp(-1.0, 1.0);
        let slippage_score = (0.08 - volatility * 1.2).clamp(-0.2, 0.2) * 4.0;

        let score = trend_score * 0.45
            + volatility_score * 0.2
            + liquidity_score * 0.2
            + slippage_score * 0.15;
        let action = if score > 0.35 {
            proto::SignalAction::SignalBuy
        } else if score < -0.35 {
            proto::SignalAction::SignalSell
        } else {
            proto::SignalAction::SignalHold
        };
        let rationale = format!(
            "trend={trend_score:.3}, vol={volatility:.4}, liq={liquidity_score:.3}, slip={slippage_score:.3}"
        );

        Ok(proto::SignalSuggestion {
            action: action as i32,
            score,
            trend_score,
            volatility_score,
            liquidity_score,
            slippage_score,
            rationale,
        })
    }

    async fn persist_price_suggestion(&self, suggestion: &proto::PriceSuggestion) {
        let quote = match &suggestion.quote {
            Some(q) => q,
            None => return,
        };
        let signal = match &suggestion.signal {
            Some(s) => s,
            None => return,
        };
        let ts_micros = suggestion.updated_at / 1000;
        let signal_action = proto::SignalAction::try_from(signal.action)
            .unwrap_or(proto::SignalAction::SignalHold)
            .as_str_name()
            .to_string();
        let fallback_active = suggestion
            .quality
            .as_ref()
            .map(|q| q.fallback_active)
            .unwrap_or(false);

        if let Err(e) = self
            .client
            .execute(
                "INSERT INTO price_suggestions \
                 (timestamp, token, mid_usd, bid_usd, ask_usd, spread_bps, confidence, signal_action, signal_score, fallback_active) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &ts_micros,
                    &suggestion.token,
                    &quote.mid_usd,
                    &quote.bid_usd,
                    &quote.ask_usd,
                    &quote.spread_bps,
                    &quote.confidence,
                    &signal_action,
                    &signal.score,
                    &fallback_active,
                ],
            )
            .await
        {
            tracing::debug!(error = %e, "failed to persist price suggestion");
        }

        for source in &suggestion.sources {
            if let Err(e) = self
                .client
                .execute(
                    "INSERT INTO price_source_quality \
                     (timestamp, token, source, weight, price_usd, confidence, fallback_used, outlier_filtered) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    &[
                        &ts_micros,
                        &suggestion.token,
                        &source.source,
                        &source.weight,
                        &source.price_usd,
                        &source.confidence,
                        &source.fallback_used,
                        &source.outlier_filtered,
                    ],
                )
                .await
            {
                tracing::debug!(error = %e, source = %source.source, "failed to persist source quality");
            }
        }
    }

    async fn resolve_usd_price(
        &self,
        queried_token: &str,
        token_in: &str,
        token_out: &str,
        stored_price: f64,
    ) -> f64 {
        if stored_price.abs() < f64::EPSILON {
            return 0.0;
        }

        // Base price: how many counter tokens per 1 queried token.
        let (base_price, counter) = if queried_token == token_in {
            (stored_price, token_out)
        } else {
            (1.0 / stored_price, token_in)
        };

        if is_stablecoin(counter) {
            base_price
        } else if counter == SOL_MINT {
            base_price * self.get_sol_usd_price().await
        } else {
            // Non-standard counter — return raw ratio.
            base_price
        }
    }

    async fn get_sol_usd_price(&self) -> f64 {
        let mut cache = self.sol_usd_cache.lock().await;
        if cache.1.elapsed() < SOL_CACHE_TTL {
            return cache.0;
        }

        // Direct query: SOL sold for USDC/USDT.
        let result = self
            .client
            .query_opt(
                &format!(
                    "SELECT price FROM dex_swaps \
                     WHERE token_in = '{SOL_MINT}' \
                       AND token_out IN ('{USDC_MINT}', '{USDT_MINT}') \
                     ORDER BY timestamp DESC LIMIT 1"
                ),
                &[],
            )
            .await;

        let price = match result {
            Ok(Some(row)) => row_f64(&row, 0).unwrap_or(0.0),
            _ => {
                // Try inverse: USDC/USDT sold for SOL.
                let inv = self
                    .client
                    .query_opt(
                        &format!(
                            "SELECT price FROM dex_swaps \
                             WHERE token_out = '{SOL_MINT}' \
                               AND token_in IN ('{USDC_MINT}', '{USDT_MINT}') \
                             ORDER BY timestamp DESC LIMIT 1"
                        ),
                        &[],
                    )
                    .await;
                match inv {
                    Ok(Some(row)) => {
                        let p = row_f64(&row, 0).unwrap_or(0.0);
                        if p.abs() > f64::EPSILON {
                            1.0 / p
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                }
            }
        };

        *cache = (price, Instant::now());
        price
    }
}

fn row_to_trade(r: &tokio_postgres::Row) -> proto::Trade {
    let ts_micros = row_ts_micros(r, 9).unwrap_or_default();
    proto::Trade {
        signature: r.get(0),
        dex: r.get(1),
        pool: r.get(2),
        token_in: r.get(3),
        token_out: r.get(4),
        amount_in: row_f64(r, 5).unwrap_or(0.0),
        amount_out: row_f64(r, 6).unwrap_or(0.0),
        price: row_f64(r, 7).unwrap_or(0.0),
        signer: r.get(8),
        timestamp: ts_micros * 1000, // micros → nanos
        slot: row_i64(r, 10).unwrap_or(0) as u64,
    }
}

#[derive(Debug, Clone)]
struct SourceSample {
    source: String,
    weight: f64,
    price_usd: f64,
    updated_at: i64,
}

fn source_weight(token_in: &str, token_out: &str) -> f64 {
    let base: f64 = if token_in == USDC_MINT || token_out == USDC_MINT {
        1.1
    } else if token_in == USDT_MINT || token_out == USDT_MINT {
        1.0
    } else {
        0.9
    };
    base.clamp(0.5, 1.5)
}

fn avg(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// QuestDB PG wire can return FLOAT8, INT8, or text for numeric columns depending on query/plan.
fn row_f64(row: &tokio_postgres::Row, idx: usize) -> Result<f64, Error> {
    if let Ok(v) = row.try_get::<usize, f64>(idx) {
        return Ok(v);
    }
    if let Ok(v) = row.try_get::<usize, f32>(idx) {
        return Ok(f64::from(v));
    }
    if let Ok(v) = row.try_get::<usize, i64>(idx) {
        return Ok(v as f64);
    }
    if let Ok(v) = row.try_get::<usize, String>(idx) {
        return v.parse::<f64>().map_err(|_| {
            Error::Query(format!(
                "unable to parse float column idx={idx} from string"
            ))
        });
    }
    Err(Error::Query(format!(
        "unable to parse float column idx={idx}"
    )))
}

/// Handles NULL aggregates and loose numeric types from QuestDB.
fn row_f64_loose(row: &tokio_postgres::Row, idx: usize) -> f64 {
    match row.try_get::<usize, Option<f64>>(idx) {
        Ok(Some(v)) => v,
        Ok(None) => 0.0,
        Err(_) => row_f64(row, idx).unwrap_or(0.0),
    }
}

fn row_i64(row: &tokio_postgres::Row, idx: usize) -> Result<i64, Error> {
    if let Ok(v) = row.try_get::<usize, i64>(idx) {
        return Ok(v);
    }
    if let Ok(v) = row.try_get::<usize, f32>(idx) {
        return Ok(v as i64);
    }
    if let Ok(v) = row.try_get::<usize, f64>(idx) {
        return Ok(v as i64);
    }
    if let Ok(v) = row.try_get::<usize, String>(idx) {
        return v.parse::<i64>().map_err(|_| {
            Error::Query(format!("unable to parse int column idx={idx} from string"))
        });
    }
    Err(Error::Query(format!(
        "unable to parse int column idx={idx}"
    )))
}

fn row_ts_micros(row: &tokio_postgres::Row, idx: usize) -> Result<i64, Error> {
    if let Ok(v) = row.try_get::<usize, i64>(idx) {
        return Ok(v);
    }
    if let Ok(v) = row.try_get::<usize, NaiveDateTime>(idx) {
        return Ok(v.and_utc().timestamp_micros());
    }
    if let Ok(v) = row.try_get::<usize, String>(idx) {
        if let Ok(parsed) = v.parse::<i64>() {
            return Ok(parsed);
        }
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&v) {
            return Ok(parsed.timestamp_micros());
        }
        if let Ok(parsed) = NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S%.f") {
            return Ok(parsed.and_utc().timestamp_micros());
        }
    }
    Err(Error::Query(format!(
        "unable to parse timestamp column idx={idx}"
    )))
}

/// Map a proto Interval enum to a QuestDB SAMPLE BY string.
pub fn interval_to_sample_by(interval: i32) -> Result<&'static str, Error> {
    match interval {
        0 => Ok("1m"),
        1 => Ok("5m"),
        2 => Ok("15m"),
        3 => Ok("30m"),
        4 => Ok("1h"),
        5 => Ok("4h"),
        6 => Ok("1d"),
        7 => Ok("7d"),
        _ => Err(Error::InvalidRequest(format!(
            "unknown interval: {interval}"
        ))),
    }
}
